use anyhow::Result;
use std::path::Path;

use crate::compile_directive::{collect_directives, Directive};
use crate::compile_output::split_frontmatter;
use crate::compile_region::render_region_body;
use crate::compile_types::{CompileResult, CompileViolation, ViolationSeverity};
use crate::config::GlintConfig;
use crate::dashboard::region::{
    compile_dashboard, parse_dashboard_frontmatter, DashboardError, RegionGeometry,
};
use crate::runner::Runner;

pub(crate) fn compile_dashboard_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &GlintConfig,
) -> Result<CompileResult> {
    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let mut violations: Vec<CompileViolation> = Vec::new();
    let mut resolved_count = 0usize;

    let (frontmatter, body, body_offset) = split_frontmatter(&source_text);
    let (meta, regions) = parse_dashboard_frontmatter(&frontmatter);

    const CANVAS_WARN_WIDTH: usize = 220;
    if meta.width > CANVAS_WARN_WIDTH {
        violations.push(CompileViolation {
            code: "DASHBOARD-006",
            severity: ViolationSeverity::Warning,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "Canvas width {} exceeds terminal threshold {} — reduce or set a --width flag",
                meta.width, CANVAS_WARN_WIDTH
            ),
            source_line: 1,
        });
    }

    let directives = collect_directives(body);
    let runner = Runner::new(root, config.clone())?;

    let mut region_by_name: std::collections::HashMap<String, &RegionGeometry> =
        std::collections::HashMap::new();
    for r in &regions {
        region_by_name.insert(r.name.clone(), r);
    }

    let mut region_contents: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for directive in &directives {
        if let Directive::Region {
            name,
            body,
            line_start,
            ..
        } = directive
        {
            let abs_line = body_offset + line_start;

            if !region_by_name.contains_key(name) {
                violations.push(CompileViolation {
                    code: "DASHBOARD-004",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!(
                        "proof:region {:?} has no matching front-matter declaration",
                        name
                    ),
                    source_line: abs_line + 1,
                });
                continue;
            }

            let rendered = render_region_body(
                body,
                root,
                config,
                &runner,
                abs_line,
                &mut violations,
                &mut resolved_count,
            );
            region_contents.insert(name.clone(), rendered);
        }
    }

    if violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    let (canvas_text, dashboard_errors) = compile_dashboard(&meta, &regions, &region_contents);

    for de in dashboard_errors {
        let DashboardError { code, message } = de;
        let severity = match code {
            "DASHBOARD-005" => ViolationSeverity::Warning,
            _ => ViolationSeverity::Error,
        };
        violations.push(CompileViolation {
            code,
            severity,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message,
            source_line: 1,
        });
    }

    if violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    let title_attr = if meta.title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", meta.title)
    };
    let output_text = format!(
        "<!-- proof:compiled from=\"proof:dashboard\"{} -->\n```dashboard\n{}```\n<!-- /proof:compiled -->\n",
        title_attr, canvas_text
    );

    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing temp output {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path)
        .map_err(|e| anyhow::anyhow!("renaming output {}: {}", output_path.display(), e))?;

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: resolved_count,
        violations,
        from_cache: false,
        resolved_files: vec![],
        written: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dashboard_paths(prefix: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let pid = std::process::id();
        let source_path =
            std::env::temp_dir().join(format!("proof-{prefix}-{pid}.dashboard.source.md"));
        let output_path = std::env::temp_dir().join(format!("proof-{prefix}-{pid}.dashboard.md"));
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&output_path);
        (source_path, output_path)
    }

    #[test]
    fn dashboard_compile_two_regions_e2e() {
        let (source_path, output_path) = temp_dashboard_paths("dash");
        let source = "---\ndashboard:\n  width: 20\n  height: 4\n  title: \"Test\"\n  regions:\n    top: { x: 0, y: 0, width: 20, height: 2 }\n    bot: { x: 0, y: 2, width: 20, height: 2 }\n---\n\n```proof:region name=top\nHEADER LINE\n```\n\n```proof:region name=bot\nFOOTER LINE\n```\n";
        std::fs::File::create(&source_path)
            .expect("create tmp")
            .write_all(source.as_bytes())
            .expect("write tmp");

        let cfg = GlintConfig::default();
        let result =
            compile_dashboard_file(&source_path, &output_path, &std::env::temp_dir(), &cfg)
                .expect("compile_dashboard_file ok");

        let _ = std::fs::remove_file(&source_path);

        assert!(
            result
                .violations
                .iter()
                .all(|v| v.severity != ViolationSeverity::Error),
            "unexpected errors: {:?}",
            result
                .violations
                .iter()
                .map(|v| (v.code, &v.message))
                .collect::<Vec<_>>()
        );
        assert!(result.written, "should have written output");

        let written = std::fs::read_to_string(&output_path).expect("read output");
        let _ = std::fs::remove_file(&output_path);

        assert!(
            written.contains("```dashboard"),
            "should have dashboard fence: {}",
            written
        );
        assert!(written.contains("HEADER LINE"), "top region not rendered");
        assert!(written.contains("FOOTER LINE"), "bot region not rendered");

        let inner: Vec<&str> = written
            .lines()
            .skip_while(|l| !l.starts_with("```dashboard"))
            .skip(1)
            .take_while(|l| *l != "```")
            .collect();
        assert_eq!(
            inner.len(),
            4,
            "canvas should be height=4 lines, got {}: {:?}",
            inner.len(),
            inner
        );
        for line in &inner {
            assert_eq!(line.chars().count(), 20, "row width != 20: {:?}", line);
        }
    }

    #[test]
    fn dashboard_unknown_region_emits_dashboard_004() {
        let (source_path, output_path) = temp_dashboard_paths("dash-bad");
        let source = "---\ndashboard:\n  width: 20\n  height: 2\n  regions:\n    header: { x: 0, y: 0, width: 20, height: 2 }\n---\n\n```proof:region name=ghost\nNo such region\n```\n";
        std::fs::File::create(&source_path)
            .expect("create tmp")
            .write_all(source.as_bytes())
            .expect("write tmp");

        let cfg = GlintConfig::default();
        let result =
            compile_dashboard_file(&source_path, &output_path, &std::env::temp_dir(), &cfg)
                .expect("compile_dashboard_file ok");

        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&output_path);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-004"),
            "expected DASHBOARD-004, got: {:?}",
            codes
        );
    }

    #[test]
    fn dashboard_overlap_emits_dashboard_003() {
        let (source_path, output_path) = temp_dashboard_paths("dash-ovl");
        let source = "---\ndashboard:\n  width: 40\n  height: 10\n  regions:\n    a: { x: 0, y: 0, width: 30, height: 5 }\n    b: { x: 20, y: 0, width: 20, height: 5 }\n---\n";
        std::fs::File::create(&source_path)
            .expect("create tmp")
            .write_all(source.as_bytes())
            .expect("write tmp");

        let cfg = GlintConfig::default();
        let result =
            compile_dashboard_file(&source_path, &output_path, &std::env::temp_dir(), &cfg)
                .expect("compile_dashboard_file ok");

        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&output_path);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-003"),
            "expected DASHBOARD-003 (overlap), got: {:?}",
            codes
        );
    }

    #[test]
    fn dashboard_wide_canvas_emits_dashboard_006() {
        let (source_path, output_path) = temp_dashboard_paths("dash-wide");
        let source =
            "---\ndashboard:\n  width: 300\n  height: 10\n---\n\n```proof:region name=r1\nhello\n```\n";
        std::fs::File::create(&source_path)
            .expect("create tmp")
            .write_all(source.as_bytes())
            .expect("write tmp");

        let cfg = GlintConfig::default();
        let result =
            compile_dashboard_file(&source_path, &output_path, &std::env::temp_dir(), &cfg)
                .expect("compile_dashboard_file ok");

        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&output_path);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-006"),
            "expected DASHBOARD-006 for canvas width 300 > 220, got: {:?}",
            codes
        );
    }
}
