use crate::layout::visual_width;

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ShapeAttrs {
    pub name: String,
    pub title: Option<String>,
    pub label: Option<String>,
    pub text: Option<String>,
    pub style: String,
    pub direction: String,
    pub size: usize,
    pub width: Option<usize>,
}

impl ShapeAttrs {
    pub fn parse(attrs_str: &str) -> Self {
        let mut out = ShapeAttrs {
            style: "double".to_string(),
            direction: "right".to_string(),
            size: 1,
            ..Default::default()
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            if let Some(eq) = rest.find('=') {
                let key = rest[..eq].trim();
                rest = &rest[eq + 1..];
                let (val, next) = if rest.starts_with('"') {
                    if let Some(close) = rest[1..].find('"') {
                        (&rest[1..close + 1], &rest[close + 2..])
                    } else {
                        ("", "")
                    }
                } else {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (&rest[..end], &rest[end..])
                };
                match key {
                    "name" => out.name = val.to_string(),
                    "title" => out.title = Some(val.to_string()),
                    "label" => out.label = Some(val.to_string()),
                    "text" => out.text = Some(val.to_string()),
                    "style" => out.style = val.to_string(),
                    "direction" => out.direction = val.to_string(),
                    "size" => out.size = val.parse().unwrap_or(1),
                    "width" => out.width = val.parse().ok(),
                    _ => {}
                }
                rest = next.trim_start();
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                rest = rest[end..].trim_start();
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct ShapeError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

// ─────────────────────────────────────────────────────────
// Banner border character sets
// ─────────────────────────────────────────────────────────

struct BorderSet {
    tl: char,
    tr: char,
    bl: char,
    br: char,
    h: char,
    v: char,
}

fn border_set(style: &str) -> BorderSet {
    match style {
        "single" => BorderSet {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '─',
            v: '│',
        },
        "rounded" => BorderSet {
            tl: '╭',
            tr: '╮',
            bl: '╰',
            br: '╯',
            h: '─',
            v: '│',
        },
        "heavy" => BorderSet {
            tl: '┏',
            tr: '┓',
            bl: '┗',
            br: '┛',
            h: '━',
            v: '┃',
        },
        "ascii" => BorderSet {
            tl: '+',
            tr: '+',
            bl: '+',
            br: '+',
            h: '-',
            v: '|',
        },
        _ => BorderSet {
            tl: '╔',
            tr: '╗',
            bl: '╚',
            br: '╝',
            h: '═',
            v: '║',
        }, // double (default)
    }
}

// ─────────────────────────────────────────────────────────
// Shape renderers
// ─────────────────────────────────────────────────────────

/// Render a banner with the given title, style, and inner width.
/// Output: 3 lines — top border, centered title, bottom border.
/// `inner_width` is the number of characters between the vertical bars (minimum: title len + 2 for padding).
pub fn render_banner(title: &str, style: &str, inner_width: usize) -> String {
    let b = border_set(style);
    let title_w = visual_width(title);
    let w = inner_width.max(title_w + 2); // at least 1 space pad on each side

    let top = format!("{}{}{}", b.tl, b.h.to_string().repeat(w), b.tr);
    let bottom = format!("{}{}{}", b.bl, b.h.to_string().repeat(w), b.br);

    // Center the title within w columns, tie-break extra space on right
    let pad_total = w.saturating_sub(title_w);
    let pad_left = pad_total / 2;
    let pad_right = pad_total - pad_left;
    let mid = format!(
        "{}{}{}{}{}",
        b.v,
        " ".repeat(pad_left),
        title,
        " ".repeat(pad_right),
        b.v
    );

    format!("{}\n{}\n{}", top, mid, bottom)
}

/// Render a badge with the given label and style.
/// Output: 3 lines — top frame, padded label, bottom frame.
/// Frame width = label visual width + 4 (2 spaces padding + 2 border chars).
pub fn render_badge(label: &str, style: &str) -> String {
    let (tl, tr, bl, br, h) = match style {
        "square" => ('┌', '┐', '└', '┘', '─'),
        "sharp" => ('+', '+', '+', '+', '-'),
        _ => ('╭', '╮', '╰', '╯', '─'), // rounded (default)
    };
    let v = match style {
        "square" => '│',
        "sharp" => '|',
        _ => '│',
    };

    let label_w = visual_width(label);
    let inner = label_w + 2; // 1 space each side

    let top = format!(" {}{}{}", tl, h.to_string().repeat(inner), tr);
    let mid = format!(" {} {} {}", v, label, v);
    let bottom = format!(" {}{}{}", bl, h.to_string().repeat(inner), br);

    format!("{}\n{}\n{}", top, mid, bottom)
}

/// Render a ribbon with the given text.
/// Output: 3 lines — a simple angled ribbon frame.
pub fn render_ribbon(text: &str) -> String {
    let text_w = visual_width(text);
    let inner = text_w + 6; // padding for the slanted sides
    let bar: String = "_".repeat(inner);
    let mid_pad = " ".repeat(3);
    let top = format!("   ╱{}╲", "‾".repeat(inner));
    let middle = format!("  ╱{}{}{}╲", mid_pad, text, mid_pad);
    let bottom = format!(" ╱{}╲", bar);
    format!("{}\n{}\n{}", top, middle, bottom)
}

// ─────────────────────────────────────────────────────────
// Main dispatch
// ─────────────────────────────────────────────────────────

/// Render a named shape to a multi-line String.
/// Returns ShapeError (SYMBOL-003) if shape name is not found.
pub fn render_shape(attrs: &ShapeAttrs) -> Result<String, ShapeError> {
    let default_width = attrs.width.unwrap_or(30);

    match attrs.name.as_str() {
        "banner" => {
            let title = attrs
                .title
                .as_deref()
                .or(attrs.text.as_deref())
                .unwrap_or("");
            Ok(render_banner(title, &attrs.style, default_width))
        }
        "badge" => {
            let label = attrs
                .label
                .as_deref()
                .or(attrs.text.as_deref())
                .unwrap_or("");
            Ok(render_badge(label, &attrs.style))
        }
        "ribbon" => {
            let text = attrs
                .text
                .as_deref()
                .or(attrs.title.as_deref())
                .unwrap_or("");
            Ok(render_ribbon(text))
        }
        other => Err(ShapeError {
            code: "SYMBOL-003",
            message: format!(
                "shape {:?} not found — supported: banner, badge, ribbon",
                other
            ),
        }),
    }
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── banner ────────────────────────────────────────────

    #[test]
    fn banner_double_has_3_lines() {
        let out = render_banner("Test", "double", 30);
        assert_eq!(out.lines().count(), 3, "banner: {:?}", out);
    }

    #[test]
    fn banner_double_top_border_uses_double_chars() {
        let out = render_banner("Test", "double", 30);
        let top = out.lines().next().unwrap();
        assert!(top.contains('╔'), "double TL: {:?}", top);
        assert!(top.contains('╗'), "double TR: {:?}", top);
        assert!(top.contains('═'), "double H: {:?}", top);
    }

    #[test]
    fn banner_double_bottom_border() {
        let out = render_banner("Test", "double", 30);
        let bottom = out.lines().last().unwrap();
        assert!(bottom.contains('╚'), "double BL: {:?}", bottom);
        assert!(bottom.contains('╝'), "double BR: {:?}", bottom);
    }

    #[test]
    fn banner_single_style_uses_single_chars() {
        let out = render_banner("Title", "single", 20);
        let top = out.lines().next().unwrap();
        assert!(top.contains('┌'), "single TL: {:?}", top);
        assert!(top.contains('┐'), "single TR: {:?}", top);
        assert!(top.contains('─'), "single H: {:?}", top);
    }

    #[test]
    fn banner_rounded_style() {
        let out = render_banner("Title", "rounded", 20);
        let top = out.lines().next().unwrap();
        assert!(top.contains('╭'), "rounded TL: {:?}", top);
        assert!(top.contains('╮'), "rounded TR: {:?}", top);
    }

    #[test]
    fn banner_heavy_style() {
        let out = render_banner("Title", "heavy", 20);
        let top = out.lines().next().unwrap();
        assert!(top.contains('┏'), "heavy TL: {:?}", top);
        assert!(top.contains('┓'), "heavy TR: {:?}", top);
        assert!(top.contains('━'), "heavy H: {:?}", top);
    }

    #[test]
    fn banner_ascii_style_only_ascii() {
        let out = render_banner("Test", "ascii", 20);
        for line in out.lines() {
            for ch in line.chars() {
                assert!(
                    ch.is_ascii(),
                    "non-ASCII in ascii banner: {:?} in {:?}",
                    ch,
                    line
                );
            }
        }
    }

    #[test]
    fn banner_title_appears_in_middle_row() {
        let out = render_banner("SECTION", "double", 30);
        let mid = out.lines().nth(1).unwrap();
        assert!(mid.contains("SECTION"), "title in middle row: {:?}", mid);
    }

    #[test]
    fn banner_title_centered() {
        let out = render_banner("HI", "double", 10);
        let mid = out.lines().nth(1).unwrap();
        // "HI" (2 chars) in inner_width 10: pad_left=4, pad_right=4
        // Total: ║ + 4 spaces + HI + 4 spaces + ║ = 12 chars
        assert!(
            mid.contains("    HI    ") || mid.contains("   HI    ") || mid.contains("    HI   "),
            "title should be centered: {:?}",
            mid
        );
    }

    #[test]
    fn banner_width_expands_for_long_title() {
        // Title longer than declared inner_width → auto-expand
        let long_title = "A Very Long Title Here";
        let out = render_banner(long_title, "double", 5);
        let mid = out.lines().nth(1).unwrap();
        assert!(mid.contains(long_title), "long title in banner: {:?}", mid);
    }

    // ── badge ─────────────────────────────────────────────

    #[test]
    fn badge_rounded_has_3_lines() {
        let out = render_badge("MVP", "rounded");
        assert_eq!(out.lines().count(), 3, "badge: {:?}", out);
    }

    #[test]
    fn badge_rounded_top_uses_rounded_corners() {
        let out = render_badge("MVP", "rounded");
        let top = out.lines().next().unwrap();
        assert!(top.contains('╭'), "rounded TL: {:?}", top);
        assert!(top.contains('╮'), "rounded TR: {:?}", top);
    }

    #[test]
    fn badge_rounded_label_in_middle_row() {
        let out = render_badge("UFA", "rounded");
        let mid = out.lines().nth(1).unwrap();
        assert!(mid.contains("UFA"), "label in middle: {:?}", mid);
    }

    #[test]
    fn badge_square_uses_square_corners() {
        let out = render_badge("MVP", "square");
        let top = out.lines().next().unwrap();
        assert!(top.contains('┌'), "square TL: {:?}", top);
        assert!(top.contains('┐'), "square TR: {:?}", top);
    }

    #[test]
    fn badge_sharp_uses_ascii() {
        let out = render_badge("RFA", "sharp");
        for line in out.lines() {
            // Allow vertical bar and horizontal dash
            for ch in line.chars() {
                if !ch.is_ascii() {
                    panic!("non-ASCII in sharp badge: {:?} in {:?}", ch, line);
                }
            }
        }
    }

    // ── ribbon ────────────────────────────────────────────

    #[test]
    fn ribbon_has_3_lines() {
        let out = render_ribbon("WINNER");
        assert_eq!(out.lines().count(), 3, "ribbon: {:?}", out);
    }

    #[test]
    fn ribbon_contains_text() {
        let out = render_ribbon("WINNER");
        assert!(out.contains("WINNER"), "ribbon: {:?}", out);
    }

    // ── render_shape dispatch ─────────────────────────────

    #[test]
    fn shape_banner_dispatches_correctly() {
        let attrs = ShapeAttrs {
            name: "banner".to_string(),
            title: Some("Section 2".to_string()),
            style: "double".to_string(),
            width: Some(30),
            ..Default::default()
        };
        let out = render_shape(&attrs).unwrap();
        assert!(out.contains('╔'));
        assert!(out.contains("Section 2"));
    }

    #[test]
    fn shape_badge_dispatches_correctly() {
        let attrs = ShapeAttrs {
            name: "badge".to_string(),
            label: Some("MVP".to_string()),
            style: "rounded".to_string(),
            ..Default::default()
        };
        let out = render_shape(&attrs).unwrap();
        assert!(out.contains("MVP"));
        assert!(out.contains('╭'));
    }

    #[test]
    fn shape_unknown_returns_symbol003() {
        let attrs = ShapeAttrs {
            name: "nonexistent".to_string(),
            ..Default::default()
        };
        let err = render_shape(&attrs).unwrap_err();
        assert_eq!(err.code, "SYMBOL-003");
        assert!(err.message.contains("nonexistent"));
    }

    // ── ShapeAttrs::parse ─────────────────────────────────

    #[test]
    fn parse_attrs_name_and_style() {
        let attrs = ShapeAttrs::parse("name=banner style=double title=\"Test\"");
        assert_eq!(attrs.name, "banner");
        assert_eq!(attrs.style, "double");
        assert_eq!(attrs.title.as_deref(), Some("Test"));
    }

    #[test]
    fn parse_attrs_defaults() {
        let attrs = ShapeAttrs::parse("name=badge");
        assert_eq!(attrs.style, "double"); // default
        assert_eq!(attrs.size, 1);
        assert_eq!(attrs.direction, "right");
    }

    #[test]
    fn parse_attrs_size() {
        let attrs = ShapeAttrs::parse("name=star size=3");
        assert_eq!(attrs.size, 3);
    }
}
