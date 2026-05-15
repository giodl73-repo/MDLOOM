/// Inline slide content renderers: quote, centered, stat, callout, divider.
use crate::slide::layout::{center_in_width, fit_to_width};

// ─────────────────────────────────────────────────────────
// proof:quote
// ─────────────────────────────────────────────────────────

/// Render a centered block quote with optional attribution.
/// Text is centered within `width`. Attribution line uses "— " prefix.
pub fn render_quote(text: &str, attribution: Option<&str>, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    // Opening curly quote, centered text, closing curly quote
    let quoted = format!("\u{201C}{}\u{201D}", text.trim()); // " and "
    lines.push(center_in_width(&quoted, width));
    if let Some(attr) = attribution {
        lines.push(center_in_width(&format!("— {}", attr), width));
    }
    lines
}

// ─────────────────────────────────────────────────────────
// proof:centered
// ─────────────────────────────────────────────────────────

/// Center each line of text within `width`.
pub fn render_centered(text: &str, width: usize) -> Vec<String> {
    text.lines().map(|l| center_in_width(l, width)).collect()
}

// ─────────────────────────────────────────────────────────
// proof:stat
// ─────────────────────────────────────────────────────────

/// Render a single statistic: large value + label + optional sublabel, centered.
pub fn render_stat(value: &str, label: &str, sublabel: Option<&str>, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(center_in_width(value, width));
    if !label.is_empty() {
        lines.push(center_in_width(label, width));
    }
    if let Some(sl) = sublabel {
        if !sl.is_empty() {
            lines.push(center_in_width(sl, width));
        }
    }
    lines
}

// ─────────────────────────────────────────────────────────
// proof:callout
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalloutStyle {
    Key,     // ★
    Info,    // ℹ
    Warning, // ⚠
    Tip,     // →
    Note,    // ◆
}

impl CalloutStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "key" => Self::Key,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "tip" => Self::Tip,
            _ => Self::Note,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Key => "★",
            Self::Info => "ℹ",
            Self::Warning => "⚠",
            Self::Tip => "→",
            Self::Note => "◆",
        }
    }
}

/// Render a callout box with icon prefix.
pub fn render_callout(text: &str, style: CalloutStyle, width: usize) -> Vec<String> {
    let icon = style.icon();
    let prefix = format!("{} ", icon);
    let content_width = width.saturating_sub(prefix.len());
    let mut lines = Vec::new();
    for (i, raw_line) in text.lines().enumerate() {
        let pfx = if i == 0 { prefix.as_str() } else { "  " };
        let clipped: String = raw_line.chars().take(content_width).collect();
        let line = format!("{}{}", pfx, clipped);
        lines.push(fit_to_width(&line, width));
    }
    lines
}

// ─────────────────────────────────────────────────────────
// proof:divider
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DividerStyle {
    Thin,   // ─────────
    Double, // ═════════
    Dotted, // ·········
    Dashed, // - - - - -
    Approx, // ≈≈≈≈≈≈≈≈≈ (wave alt — avoids ~ strikethrough risk)
}

impl DividerStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "double" => Self::Double,
            "dotted" => Self::Dotted,
            "dashed" => Self::Dashed,
            "approx" | "wave" => Self::Approx,
            _ => Self::Thin,
        }
    }
}

// ─────────────────────────────────────────────────────────
// proof:right
// ─────────────────────────────────────────────────────────

/// Right-align each line of text within `width` — complement to render_centered.
pub fn render_right(text: &str, width: usize) -> Vec<String> {
    text.lines()
        .map(|l| {
            let len = l.chars().count();
            if len >= width {
                return l.to_string();
            }
            format!("{}{}", " ".repeat(width - len), l)
        })
        .collect()
}

// ─────────────────────────────────────────────────────────
// proof:numbered-list  (alias: proof:ol — ordered / numbered list)
// ─────────────────────────────────────────────────────────

/// Render a numbered (ordered) list.
///
/// Dispatched from both `proof:numbered-list` (primary) and `proof:ol`
/// (short-form alias) — both names render identically.
///
/// Input lines starting with `- ` are items; indented items (2+ spaces) are
/// sub-items and get decimal numbering (1.1, 1.2, ...).
/// Returns rendered lines word-wrapped to `width`.
pub fn render_ol(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut counters: Vec<usize> = Vec::new(); // stack of counters per depth

    for raw in text.lines() {
        if raw.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let leading = raw.len() - raw.trim_start().len();
        let depth = leading / 2; // 2 spaces per level

        // Grow or reset counter stack
        while counters.len() <= depth {
            counters.push(0);
        }
        counters[depth] += 1;
        // Reset all deeper counters when we go back up
        for d in (depth + 1)..counters.len() {
            counters[d] = 0;
        }

        let trimmed = raw.trim();
        let content = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .unwrap_or(trimmed);

        // Build number prefix: "1." / "1.1." / "1.1.1."
        let number: String = counters[..=depth]
            .iter()
            .filter(|&&c| c > 0)
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(".")
            + ".";

        let indent = "  ".repeat(depth);
        let prefix = format!("{}{} ", indent, number);
        let prefix_w = prefix.chars().count();
        let first_line = format!("{}{}", prefix, content);

        // Word-wrap with hanging indent
        let wrapped = if crate::layout::visual_width(&first_line) <= width {
            vec![first_line]
        } else {
            let continuation = " ".repeat(prefix_w);
            let cont_width = width.saturating_sub(prefix_w).max(1);
            let mut result = Vec::new();
            let mut first = true;
            let mut cur = String::new();
            let mut cur_w = 0usize;
            for word in first_line.split(' ') {
                let ww = crate::layout::visual_width(word);
                let max_w = if first { width } else { cont_width };
                if cur.is_empty() {
                    cur = word.to_string();
                    cur_w = ww;
                } else if cur_w + 1 + ww <= max_w {
                    cur.push(' ');
                    cur.push_str(word);
                    cur_w += 1 + ww;
                } else {
                    result.push(if first {
                        cur.clone()
                    } else {
                        format!("{}{}", continuation, cur)
                    });
                    first = false;
                    cur = word.to_string();
                    cur_w = ww;
                }
            }
            if !cur.is_empty() {
                result.push(if first {
                    cur
                } else {
                    format!("{}{}", continuation, cur)
                });
            }
            result
        };
        lines.extend(wrapped);
    }
    lines
}

/// Render a horizontal divider of `width` chars.
pub fn render_divider(style: DividerStyle, width: usize) -> String {
    let ch: String = match style {
        DividerStyle::Thin => "─".repeat(width),
        DividerStyle::Double => "═".repeat(width),
        DividerStyle::Dotted => "·".repeat(width),
        DividerStyle::Dashed => {
            let mut s = String::with_capacity(width);
            for i in 0..width {
                s.push(if i % 2 == 0 { '-' } else { ' ' });
            }
            s
        }
        DividerStyle::Approx => "≈".repeat(width),
    };
    ch
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_has_curly_quotes() {
        let lines = render_quote("To be or not to be", None, 40);
        assert!(lines[0].contains('\u{201C}'));
        assert!(lines[0].contains('\u{201D}'));
    }

    #[test]
    fn quote_attribution() {
        let lines = render_quote("Quote text", Some("Author"), 40);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("— Author"));
    }

    #[test]
    fn centered_each_line() {
        let lines = render_centered("hello\nworld", 20);
        assert_eq!(lines.len(), 2);
        for l in &lines {
            assert_eq!(l.chars().count(), 20);
        }
    }

    #[test]
    fn stat_renders_value_label_sublabel() {
        let lines = render_stat("138.0", "Pts/82", Some("#1 all-time"), 40);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("138.0"));
        assert!(lines[1].contains("Pts/82"));
        assert!(lines[2].contains("#1 all-time"));
    }

    #[test]
    fn callout_key_has_star() {
        let lines = render_callout("Important note", CalloutStyle::Key, 40);
        assert!(lines[0].starts_with("★ "));
    }

    #[test]
    fn callout_warning_has_icon() {
        let lines = render_callout("Watch out", CalloutStyle::Warning, 40);
        assert!(lines[0].starts_with("⚠ "));
    }

    #[test]
    fn divider_thin_correct_width() {
        let d = render_divider(DividerStyle::Thin, 40);
        assert_eq!(d, "─".repeat(40));
    }

    #[test]
    fn divider_double() {
        let d = render_divider(DividerStyle::Double, 10);
        assert_eq!(d, "══════════");
    }

    #[test]
    fn divider_approx_not_tilde() {
        let d = render_divider(DividerStyle::Approx, 5);
        assert!(
            !d.contains('~'),
            "wave divider must not use ~ (strikethrough risk)"
        );
        assert!(d.contains('≈'));
    }

    #[test]
    fn callout_style_parse() {
        assert_eq!(CalloutStyle::parse("key"), CalloutStyle::Key);
        assert_eq!(CalloutStyle::parse("warning"), CalloutStyle::Warning);
        assert_eq!(CalloutStyle::parse("unknown"), CalloutStyle::Note);
    }

    #[test]
    fn divider_style_wave_maps_to_approx() {
        assert_eq!(DividerStyle::parse("wave"), DividerStyle::Approx);
    }

    // ── proof:right ──────────────────────────────────────

    #[test]
    fn right_align_short_line() {
        let lines = render_right("hello", 10);
        assert_eq!(lines[0], "     hello");
        assert_eq!(lines[0].len(), 10);
    }

    #[test]
    fn right_align_exact_width_no_pad() {
        let lines = render_right("1234567890", 10);
        assert_eq!(lines[0], "1234567890");
    }

    #[test]
    fn right_align_multi_line() {
        let lines = render_right("a\nbb\nccc", 5);
        assert_eq!(lines.len(), 3);
        assert!(lines[0].ends_with('a'));
        assert!(lines[1].ends_with("bb"));
        assert!(lines[2].ends_with("ccc"));
        // Each line right-aligned
        assert_eq!(lines[0], "    a");
        assert_eq!(lines[1], "   bb");
        assert_eq!(lines[2], "  ccc");
    }

    // ── proof:ol ─────────────────────────────────────────

    #[test]
    fn ol_basic_numbered_list() {
        let text = "- First item\n- Second item\n- Third item";
        let lines = render_ol(text, 40);
        assert!(lines[0].contains("1.") && lines[0].contains("First"));
        assert!(lines[1].contains("2.") && lines[1].contains("Second"));
        assert!(lines[2].contains("3.") && lines[2].contains("Third"));
    }

    #[test]
    fn ol_nested_decimal_numbering() {
        let text = "- Top\n  - Sub one\n  - Sub two\n- Second top";
        let lines = render_ol(text, 40);
        assert!(lines[0].contains("1."), "first item should be 1.");
        assert!(lines[1].contains("1.1"), "first sub-item should be 1.1");
        assert!(lines[2].contains("1.2"), "second sub-item should be 1.2");
        assert!(lines[3].contains("2."), "second top-level should be 2.");
    }

    #[test]
    fn ol_empty_lines_preserved() {
        let text = "- A\n\n- B";
        let lines = render_ol(text, 40);
        // Should have a blank line between items
        assert!(lines.iter().any(|l| l.is_empty()));
    }
}
