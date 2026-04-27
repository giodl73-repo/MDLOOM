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
    text.lines()
        .map(|l| center_in_width(l, width))
        .collect()
}

// ─────────────────────────────────────────────────────────
// proof:stat
// ─────────────────────────────────────────────────────────

/// Render a single statistic: large value + label + optional sublabel, centered.
pub fn render_stat(value: &str, label: &str, sublabel: Option<&str>, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(center_in_width(value, width));
    if !label.is_empty() { lines.push(center_in_width(label, width)); }
    if let Some(sl) = sublabel { if !sl.is_empty() { lines.push(center_in_width(sl, width)); } }
    lines
}

// ─────────────────────────────────────────────────────────
// proof:callout
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalloutStyle {
    Key,      // ★
    Info,     // ℹ
    Warning,  // ⚠
    Tip,      // →
    Note,     // ◆
}

impl CalloutStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "key"     => Self::Key,
            "info"    => Self::Info,
            "warning" => Self::Warning,
            "tip"     => Self::Tip,
            _         => Self::Note,
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Key     => "★",
            Self::Info    => "ℹ",
            Self::Warning => "⚠",
            Self::Tip     => "→",
            Self::Note    => "◆",
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
    Thin,    // ─────────
    Double,  // ═════════
    Dotted,  // ·········
    Dashed,  // - - - - -
    Approx,  // ≈≈≈≈≈≈≈≈≈ (wave alt — avoids ~ strikethrough risk)
}

impl DividerStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "double"  => Self::Double,
            "dotted"  => Self::Dotted,
            "dashed"  => Self::Dashed,
            "approx" | "wave" => Self::Approx,
            _         => Self::Thin,
        }
    }
}

/// Render a horizontal divider of `width` chars.
pub fn render_divider(style: DividerStyle, width: usize) -> String {
    let ch: String = match style {
        DividerStyle::Thin   => "─".repeat(width),
        DividerStyle::Double => "═".repeat(width),
        DividerStyle::Dotted => "·".repeat(width),
        DividerStyle::Dashed => {
            let mut s = String::with_capacity(width);
            for i in 0..width { s.push(if i % 2 == 0 { '-' } else { ' ' }); }
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
        for l in &lines { assert_eq!(l.chars().count(), 20); }
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
        assert!(!d.contains('~'), "wave divider must not use ~ (strikethrough risk)");
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
}
