/// Dashboard region content parsing and rendering.
///
/// A proof:region block contains:
/// - Directive lines (starting with proof:element, proof:tree, proof:chart, proof:row)
/// - Literal content lines (everything else — rendered verbatim)
///
/// No nested fences. The proof:region fence IS the container.
use crate::dashboard::canvas::Canvas;
#[allow(unused_imports)]
use unicode_width::UnicodeWidthChar;

// ─────────────────────────────────────────────────────────
// Region geometry
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RegionGeometry {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Parse the regions: map from YAML front-matter.
/// Format: `name: { x: 0, y: 0, width: 120, height: 3 }`
pub fn parse_regions(yaml_value: &str) -> Vec<RegionGeometry> {
    let mut regions = Vec::new();
    for line in yaml_value.lines() {
        let line = line.trim();
        // Pattern: "name: { x: N, y: N, width: N, height: N }"
        if let Some((name, rest)) = line.split_once(':') {
            let name = name.trim().trim_matches('"').to_string();
            let rest = rest
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            let mut x = 0usize;
            let mut y = 0usize;
            let mut width = 0usize;
            let mut height = 0usize;
            for part in rest.split(',') {
                let part = part.trim();
                if let Some((k, v)) = part.split_once(':') {
                    let k = k.trim();
                    let v = v.trim().parse::<usize>().unwrap_or(0);
                    match k {
                        "x" => x = v,
                        "y" => y = v,
                        "width" => width = v,
                        "height" => height = v,
                        _ => {}
                    }
                }
            }
            if !name.is_empty() && width > 0 && height > 0 {
                regions.push(RegionGeometry {
                    name,
                    x,
                    y,
                    width,
                    height,
                });
            }
        }
    }
    regions
}

// ─────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DashboardError {
    pub code: &'static str,
    pub message: String,
}

/// Validate region geometries against canvas dimensions (D-2, D-3).
pub fn validate_regions(
    regions: &[RegionGeometry],
    canvas_width: usize,
    canvas_height: usize,
) -> Vec<DashboardError> {
    let mut errors = Vec::new();

    for r in regions {
        // D-2: bounds check
        if r.x + r.width > canvas_width {
            errors.push(DashboardError {
                code: "DASHBOARD-001",
                message: format!(
                    "region {:?}: x({}) + width({}) = {} exceeds canvas width {}",
                    r.name,
                    r.x,
                    r.width,
                    r.x + r.width,
                    canvas_width
                ),
            });
        }
        if r.y + r.height > canvas_height {
            errors.push(DashboardError {
                code: "DASHBOARD-002",
                message: format!(
                    "region {:?}: y({}) + height({}) = {} exceeds canvas height {}",
                    r.name,
                    r.y,
                    r.height,
                    r.y + r.height,
                    canvas_height
                ),
            });
        }
    }

    // D-3: overlap check — O(n²) but dashboards have few regions
    for i in 0..regions.len() {
        for j in (i + 1)..regions.len() {
            let a = &regions[i];
            let b = &regions[j];
            let overlap_x = a.x < b.x + b.width && b.x < a.x + a.width;
            let overlap_y = a.y < b.y + b.height && b.y < a.y + a.height;
            if overlap_x && overlap_y {
                errors.push(DashboardError {
                    code: "DASHBOARD-003",
                    message: format!("regions {:?} and {:?} overlap", a.name, b.name),
                });
            }
        }
    }

    errors
}

// ─────────────────────────────────────────────────────────
// Content rendering
// ─────────────────────────────────────────────────────────

/// Classify a line inside a proof:region block.
/// Directive lines start with one of the proof: directive keywords.
#[derive(Debug, Clone, PartialEq)]
pub enum RegionLine<'a> {
    Directive(&'a str), // proof:element, proof:tree, proof:chart, proof:row
    Literal(&'a str),   // plain text — rendered verbatim
}

pub fn classify_region_line(line: &str) -> RegionLine<'_> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("proof:element")
        || trimmed.starts_with("proof:tree")
        || trimmed.starts_with("proof:chart")
        || trimmed.starts_with("proof:row")
        || trimmed.starts_with("proof:symbol")
        || trimmed.starts_with("proof:shape")
        || trimmed.starts_with("proof:bullets")
        || trimmed.starts_with("proof:centered")
        || trimmed.starts_with("proof:stat")
    {
        RegionLine::Directive(trimmed)
    } else {
        RegionLine::Literal(line)
    }
}

/// Render region content lines into a Canvas.
///
/// By the time content reaches this function, `compile_region::render_region_body`
/// has already resolved every embedded `proof:*` directive and written the
/// rendered glyph rows back into `content_lines`. So at this stage every line
/// is treated as literal: clipped to region width, padded, and pasted.
///
/// Content overflowing the region height emits DASHBOARD-005.
pub fn render_region_into_canvas(
    canvas: &mut Canvas,
    region: &RegionGeometry,
    content_lines: &[&str],
) -> Vec<DashboardError> {
    let mut errors = Vec::new();
    let mut output_lines: Vec<String> = Vec::new();

    for &line in content_lines {
        // Directives have been pre-resolved upstream — paste literal output.
        let clipped = clip_to_width(line, region.width);
        output_lines.push(clipped);
    }

    // Check for overflow
    if output_lines.len() > region.height {
        let clipped_count = output_lines.len() - region.height;
        errors.push(DashboardError {
            code: "DASHBOARD-005",
            message: format!(
                "region {:?}: content overflows by {} line{} — lines {}..{} clipped; use --explain to see clipped content",
                region.name,
                clipped_count,
                if clipped_count == 1 { "" } else { "s" },
                region.height + 1,
                output_lines.len(),
            ),
        });
        output_lines.truncate(region.height);
    }

    // Pad each line to region width using visual_width (not char count),
    // then paste onto canvas. canvas.paste() also uses visual width internally.
    let padded: Vec<String> = output_lines
        .iter()
        .map(|l| {
            let mut s = l.clone();
            let w = crate::layout::visual_width(&s);
            if w < region.width {
                s.push_str(&" ".repeat(region.width - w));
            }
            s
        })
        .collect();

    let as_str: Vec<&str> = padded.iter().map(|s| s.as_str()).collect();
    canvas.paste(region.x, region.y, &as_str);

    errors
}

fn clip_to_width(s: &str, width: usize) -> String {
    // Clip by visual_width, not char count, to handle wide chars correctly.

    let mut result = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + cw > width.saturating_sub(1) {
            result.push('…');
            return result;
        }
        result.push(ch);
        w += cw;
    }
    result
}

// ─────────────────────────────────────────────────────────
// Dashboard document
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DashboardMeta {
    pub width: usize,
    pub height: usize,
    pub title: String,
}

impl Default for DashboardMeta {
    fn default() -> Self {
        DashboardMeta {
            width: 120,
            height: 40,
            title: String::new(),
        }
    }
}

/// Parse dashboard front-matter. Hand-parsed — no YAML dependency.
/// Expects:
/// ```text
/// dashboard:
///   width: 120
///   height: 40
///   title: "..."
///   regions:
///     name: { x: 0, y: 0, width: 40, height: 20 }
/// ```
pub fn parse_dashboard_frontmatter(yaml: &str) -> (DashboardMeta, Vec<RegionGeometry>) {
    let mut meta = DashboardMeta::default();
    let mut regions = Vec::new();
    let mut in_dashboard = false;
    let mut in_regions = false;
    let mut regions_yaml = String::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed == "dashboard:" {
            in_dashboard = true;
            continue;
        }
        if !in_dashboard {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        if trimmed.starts_with("regions:") {
            in_regions = true;
            continue;
        }

        if in_regions {
            // Collect all indented region lines
            if indent >= 4 {
                regions_yaml.push_str(line);
                regions_yaml.push('\n');
            } else {
                in_regions = false;
            }
            continue;
        }

        // Top-level dashboard properties
        if let Some((k, v)) = trimmed.split_once(':') {
            let v = v.trim().trim_matches('"');
            match k.trim() {
                "width" => meta.width = v.parse().unwrap_or(120),
                "height" => meta.height = v.parse().unwrap_or(40),
                "title" => meta.title = v.to_string(),
                _ => {}
            }
        }
    }

    if !regions_yaml.is_empty() {
        regions = parse_regions(&regions_yaml);
    }

    (meta, regions)
}

/// Compile a dashboard source into a Canvas string.
/// region_contents: map from region name → content lines
pub fn compile_dashboard(
    meta: &DashboardMeta,
    regions: &[RegionGeometry],
    region_contents: &std::collections::HashMap<String, Vec<String>>,
) -> (String, Vec<DashboardError>) {
    let mut canvas = Canvas::new(meta.width, meta.height);
    let mut all_errors = Vec::new();

    // Validate geometry first
    let geo_errors = validate_regions(regions, meta.width, meta.height);
    all_errors.extend(geo_errors);
    if all_errors
        .iter()
        .any(|e| e.code.starts_with("DASHBOARD-00"))
    {
        // Return empty canvas on geometry error
        return (canvas.render(), all_errors);
    }

    // Render each region
    for region in regions {
        let empty = Vec::new();
        let content = region_contents.get(&region.name).unwrap_or(&empty);
        let content_refs: Vec<&str> = content.iter().map(|s| s.as_str()).collect();
        let errs = render_region_into_canvas(&mut canvas, region, &content_refs);
        all_errors.extend(errs);
    }

    (canvas.render(), all_errors)
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn parse_regions_basic() {
        let yaml = "    header: { x: 0, y: 0, width: 120, height: 3 }\n    body: { x: 0, y: 3, width: 120, height: 37 }";
        let regions = parse_regions(yaml);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].name, "header");
        assert_eq!(regions[0].x, 0);
        assert_eq!(regions[0].width, 120);
        assert_eq!(regions[0].height, 3);
    }

    #[test]
    fn validate_regions_in_bounds() {
        let r = vec![RegionGeometry {
            name: "a".into(),
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        }];
        assert!(validate_regions(&r, 120, 40).is_empty());
    }

    #[test]
    fn validate_regions_out_of_bounds_x() {
        let r = vec![RegionGeometry {
            name: "a".into(),
            x: 100,
            y: 0,
            width: 40,
            height: 10,
        }];
        let errs = validate_regions(&r, 120, 40);
        assert!(errs.iter().any(|e| e.code == "DASHBOARD-001"));
    }

    #[test]
    fn validate_regions_out_of_bounds_y() {
        let r = vec![RegionGeometry {
            name: "a".into(),
            x: 0,
            y: 35,
            width: 40,
            height: 10,
        }];
        let errs = validate_regions(&r, 120, 40);
        assert!(errs.iter().any(|e| e.code == "DASHBOARD-002"));
    }

    #[test]
    fn validate_regions_overlap() {
        let r = vec![
            RegionGeometry {
                name: "a".into(),
                x: 0,
                y: 0,
                width: 60,
                height: 20,
            },
            RegionGeometry {
                name: "b".into(),
                x: 40,
                y: 0,
                width: 60,
                height: 20,
            }, // overlaps a
        ];
        let errs = validate_regions(&r, 120, 40);
        assert!(errs.iter().any(|e| e.code == "DASHBOARD-003"));
    }

    #[test]
    fn validate_adjacent_regions_no_overlap() {
        let r = vec![
            RegionGeometry {
                name: "a".into(),
                x: 0,
                y: 0,
                width: 40,
                height: 20,
            },
            RegionGeometry {
                name: "b".into(),
                x: 40,
                y: 0,
                width: 40,
                height: 20,
            }, // adjacent, not overlapping
        ];
        assert!(validate_regions(&r, 80, 20).is_empty());
    }

    #[test]
    fn classify_directive_line() {
        assert_eq!(
            classify_region_line("proof:element kind=value"),
            RegionLine::Directive("proof:element kind=value")
        );
        assert_eq!(
            classify_region_line("  proof:tree kind=org"),
            RegionLine::Directive("proof:tree kind=org")
        );
    }

    #[test]
    fn classify_literal_line() {
        assert!(matches!(
            classify_region_line("Hello world"),
            RegionLine::Literal("Hello world")
        ));
        assert!(matches!(classify_region_line(""), RegionLine::Literal("")));
    }

    #[test]
    fn compile_dashboard_two_regions() {
        let meta = DashboardMeta {
            width: 20,
            height: 4,
            title: "Test".into(),
        };
        let regions = vec![
            RegionGeometry {
                name: "top".into(),
                x: 0,
                y: 0,
                width: 20,
                height: 2,
            },
            RegionGeometry {
                name: "bottom".into(),
                x: 0,
                y: 2,
                width: 20,
                height: 2,
            },
        ];
        let mut contents = HashMap::new();
        contents.insert("top".into(), vec!["HEADER LINE".to_string()]);
        contents.insert("bottom".into(), vec!["FOOTER LINE".to_string()]);

        let (output, errors) = compile_dashboard(&meta, &regions, &contents);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[0].starts_with("HEADER LINE"));
        assert!(lines[2].starts_with("FOOTER LINE"));
    }

    #[test]
    fn compile_dashboard_overflow_emits_005() {
        let meta = DashboardMeta {
            width: 20,
            height: 2,
            title: String::new(),
        };
        let regions = vec![RegionGeometry {
            name: "r".into(),
            x: 0,
            y: 0,
            width: 20,
            height: 2,
        }];
        let mut contents = HashMap::new();
        contents.insert(
            "r".into(),
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        );
        let (_, errors) = compile_dashboard(&meta, &regions, &contents);
        assert!(errors.iter().any(|e| e.code == "DASHBOARD-005"));
    }

    #[test]
    fn parse_dashboard_frontmatter_basic() {
        let yaml = "dashboard:\n  width: 80\n  height: 24\n  title: \"Test Dashboard\"\n  regions:\n    header: { x: 0, y: 0, width: 80, height: 3 }\n";
        let (meta, regions) = parse_dashboard_frontmatter(yaml);
        assert_eq!(meta.width, 80);
        assert_eq!(meta.height, 24);
        assert_eq!(meta.title, "Test Dashboard");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].name, "header");
    }
}
