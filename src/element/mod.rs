pub mod mini_bar;
pub mod row;
pub mod sparkline;
pub mod value;

use crate::layout::visual_width;

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementKind {
    Value,
    Delta,
    Sparkline,
    MiniBar,
    Label,
    Badge,
}

impl ElementKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "value"    => Some(Self::Value),
            "delta"    => Some(Self::Delta),
            "sparkline"=> Some(Self::Sparkline),
            "mini-bar" => Some(Self::MiniBar),
            "label"    => Some(Self::Label),
            "badge"    => Some(Self::Badge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementAlign {
    Left,
    Right,
    Center,
}

impl ElementAlign {
    pub fn parse(s: &str) -> Self {
        match s {
            "right"  => Self::Right,
            "center" => Self::Center,
            _ => Self::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElementConfig {
    pub kind: ElementKind,
    pub width: usize,
    pub align: ElementAlign,
    pub format: String,
    pub no_chrome: bool,
    pub max: Option<f64>,
    pub fill_char: char,
    pub empty_char: char,
}

impl Default for ElementConfig {
    fn default() -> Self {
        Self {
            kind: ElementKind::Value,
            width: 0,
            align: ElementAlign::Left,
            format: "{}".to_string(),
            no_chrome: false,
            max: None,
            fill_char: '█',
            empty_char: '░',
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementData {
    Scalar(f64),
    Text(String),
    Series(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementError {
    WidthExceeded { actual: usize, budget: usize },
    WrongDataKind { expected: &'static str, got: &'static str },
    ZeroWidth,
}

impl std::fmt::Display for ElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WidthExceeded { actual, budget } =>
                write!(f, "output width {} exceeds budget {}", actual, budget),
            Self::WrongDataKind { expected, got } =>
                write!(f, "expected {} data, got {}", expected, got),
            Self::ZeroWidth =>
                write!(f, "element width must be >= 1"),
        }
    }
}

// ─────────────────────────────────────────────────────────
// Core render dispatch
// ─────────────────────────────────────────────────────────

/// Render an element to exactly `cfg.width` visual characters.
/// Returns raw chars — no fence, no traceability comment.
/// Caller wraps in fenced block when no_chrome=false.
pub fn render_element(data: &ElementData, cfg: &ElementConfig) -> Result<String, ElementError> {
    if cfg.width == 0 {
        return Err(ElementError::ZeroWidth);
    }

    let raw = match cfg.kind {
        ElementKind::Value => {
            match data {
                ElementData::Text(s) => render_label_str(s, cfg),
                _ => {
                    let v = require_scalar(data, "value")?;
                    value::render_value(v, cfg)
                }
            }
        }
        ElementKind::Delta => {
            let v = require_scalar(data, "delta")?;
            value::render_delta(v, cfg)
        }
        ElementKind::Sparkline => {
            let series = require_series(data, "sparkline")?;
            sparkline::render_sparkline(&series, cfg)
        }
        ElementKind::MiniBar => {
            let v = require_scalar(data, "mini-bar")?;
            let max = cfg.max.unwrap_or(v.abs().max(1.0));
            mini_bar::render_mini_bar(v, max, cfg)
        }
        ElementKind::Label => {
            let s = require_text(data, "label")?;
            render_label_str(&s, cfg)
        }
        ElementKind::Badge => {
            let s = require_text(data, "badge")?;
            render_badge_str(&s, cfg)
        }
    };

    // E-1: enforce exact width
    let w = visual_width(&raw);
    if w != cfg.width {
        return Err(ElementError::WidthExceeded { actual: w, budget: cfg.width });
    }
    Ok(raw)
}

// ─────────────────────────────────────────────────────────
// Label / Badge rendering
// ─────────────────────────────────────────────────────────

fn render_label_str(s: &str, cfg: &ElementConfig) -> String {
    let w = cfg.width;
    let sw = visual_width(s);
    if sw > w {
        // Truncate with ellipsis
        truncate_to_width(s, w.saturating_sub(1))
            + "…"
    } else {
        align_in_width(s, sw, w, cfg.align)
    }
}

fn render_badge_str(s: &str, cfg: &ElementConfig) -> String {
    // Badge: always right-pad with spaces (left-align), truncate if too wide
    let w = cfg.width;
    let sw = visual_width(s);
    if sw >= w {
        truncate_to_width(s, w)
    } else {
        format!("{}{}", s, " ".repeat(w - sw))
    }
}

// ─────────────────────────────────────────────────────────
// Alignment helper
// ─────────────────────────────────────────────────────────

/// Pad string `s` (already measured at `sw` visual columns) to exactly `width` columns.
/// Does not truncate — caller must ensure sw <= width.
pub fn align_in_width(s: &str, sw: usize, width: usize, align: ElementAlign) -> String {
    if sw >= width {
        return s.to_string();
    }
    let pad = width - sw;
    match align {
        ElementAlign::Left => format!("{}{}", s, " ".repeat(pad)),
        ElementAlign::Right => format!("{}{}", " ".repeat(pad), s),
        ElementAlign::Center => {
            let left = pad / 2;
            let right = pad - left; // E-6: tie-break extra space on right
            format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
        }
    }
}

/// Truncate string to at most `max_width` visual columns, returning a new String.
/// Does not append ellipsis — caller adds if needed.
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut w = 0usize;
    for c in s.chars() {
        let cw = char_visual_width(c);
        if w + cw > max_width {
            break;
        }
        result.push(c);
        w += cw;
    }
    // If we stopped mid-double-wide we might be 1 short — pad with space
    if w < max_width && w > 0 {
        result.push(' ');
    }
    result
}

fn char_visual_width(c: char) -> usize {
    let cp = c as u32;
    if (0x2190..=0x21FF).contains(&cp)
        || (0x2500..=0x259F).contains(&cp)
        || (0x25A0..=0x25FF).contains(&cp)
    {
        1
    } else {
        unicode_width::UnicodeWidthChar::width(c).unwrap_or(1)
    }
}

// ─────────────────────────────────────────────────────────
// Data kind helpers
// ─────────────────────────────────────────────────────────

fn require_scalar(data: &ElementData, _kind: &'static str) -> Result<f64, ElementError> {
    match data {
        ElementData::Scalar(v) => Ok(*v),
        ElementData::Text(s) => s.parse::<f64>().map_err(|_| ElementError::WrongDataKind {
            expected: "scalar",
            got: "text (non-numeric)",
        }),
        ElementData::Series(_) => Err(ElementError::WrongDataKind {
            expected: "scalar",
            got: "series",
        }),
    }
}

fn require_series(data: &ElementData, _kind: &'static str) -> Result<Vec<f64>, ElementError> {
    match data {
        ElementData::Series(v) => Ok(v.clone()),
        ElementData::Scalar(v) => Ok(vec![*v]),
        ElementData::Text(s) => {
            let vals: Result<Vec<f64>, _> = s.split(',').map(|t| t.trim().parse::<f64>()).collect();
            vals.map_err(|_| ElementError::WrongDataKind {
                expected: "series",
                got: "text (non-numeric)",
            })
        }
    }
}

fn require_text(data: &ElementData, _kind: &'static str) -> Result<String, ElementError> {
    Ok(match data {
        ElementData::Text(s) => s.clone(),
        ElementData::Scalar(v) => v.to_string(),
        ElementData::Series(_) => return Err(ElementError::WrongDataKind {
            expected: "text",
            got: "series",
        }),
    })
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_value(width: usize) -> ElementConfig {
        ElementConfig { kind: ElementKind::Value, width, ..Default::default() }
    }
    fn cfg_delta(width: usize) -> ElementConfig {
        ElementConfig { kind: ElementKind::Delta, width, format: "{:+.2}".to_string(), ..Default::default() }
    }
    fn cfg_sparkline(width: usize) -> ElementConfig {
        ElementConfig { kind: ElementKind::Sparkline, width, ..Default::default() }
    }
    fn cfg_mini_bar(width: usize, max: f64) -> ElementConfig {
        ElementConfig { kind: ElementKind::MiniBar, width, max: Some(max), ..Default::default() }
    }
    fn cfg_label(width: usize, align: ElementAlign) -> ElementConfig {
        ElementConfig { kind: ElementKind::Label, width, align, ..Default::default() }
    }
    fn cfg_badge(width: usize) -> ElementConfig {
        ElementConfig { kind: ElementKind::Badge, width, ..Default::default() }
    }

    // ── E-1: exact width invariant ────────────────────────

    #[test]
    fn e1_value_exact_width() {
        let out = render_element(&ElementData::Scalar(42.0), &cfg_value(6)).unwrap();
        assert_eq!(visual_width(&out), 6, "output: {:?}", out);
    }

    #[test]
    fn e1_delta_exact_width() {
        let out = render_element(&ElementData::Scalar(0.19), &cfg_delta(6)).unwrap();
        assert_eq!(visual_width(&out), 6, "output: {:?}", out);
    }

    #[test]
    fn e1_sparkline_exact_width() {
        let data = ElementData::Series(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let out = render_element(&data, &cfg_sparkline(5)).unwrap();
        assert_eq!(visual_width(&out), 5, "output: {:?}", out);
    }

    #[test]
    fn e1_mini_bar_exact_width() {
        let out = render_element(&ElementData::Scalar(50.0), &cfg_mini_bar(10, 100.0)).unwrap();
        assert_eq!(visual_width(&out), 10, "output: {:?}", out);
    }

    #[test]
    fn e1_label_exact_width() {
        let out = render_element(&ElementData::Text("Hello".to_string()), &cfg_label(10, ElementAlign::Left)).unwrap();
        assert_eq!(visual_width(&out), 10, "output: {:?}", out);
    }

    #[test]
    fn e1_badge_exact_width() {
        let out = render_element(&ElementData::Text("UFA".to_string()), &cfg_badge(5)).unwrap();
        assert_eq!(visual_width(&out), 5, "output: {:?}", out);
    }

    // ── value ─────────────────────────────────────────────

    #[test]
    fn value_integer_format() {
        let cfg = ElementConfig {
            kind: ElementKind::Value, width: 4, format: "{}".to_string(), ..Default::default()
        };
        let out = render_element(&ElementData::Scalar(138.0), &cfg).unwrap();
        assert!(out.contains("138"), "output: {:?}", out);
        assert_eq!(visual_width(&out), 4);
    }

    #[test]
    fn value_one_decimal() {
        let cfg = ElementConfig {
            kind: ElementKind::Value, width: 6,
            format: "{:.1}".to_string(),
            align: ElementAlign::Right,
            ..Default::default()
        };
        let out = render_element(&ElementData::Scalar(138.0), &cfg).unwrap();
        assert!(out.contains("138.0"), "output: {:?}", out);
        assert_eq!(visual_width(&out), 6);
    }

    #[test]
    fn value_right_align() {
        let cfg = ElementConfig {
            kind: ElementKind::Value, width: 8,
            format: "{:.1}".to_string(),
            align: ElementAlign::Right,
            ..Default::default()
        };
        let out = render_element(&ElementData::Scalar(1.5), &cfg).unwrap();
        assert!(out.starts_with(' '), "should be right-aligned: {:?}", out);
        assert_eq!(visual_width(&out), 8);
    }

    #[test]
    fn value_center_align() {
        let cfg = ElementConfig {
            kind: ElementKind::Value, width: 9,
            format: "{}".to_string(),
            align: ElementAlign::Center,
            ..Default::default()
        };
        let out = render_element(&ElementData::Scalar(42.0), &cfg).unwrap();
        assert_eq!(visual_width(&out), 9);
        // "42" is 2 chars, pad=7, left=3, right=4
        assert!(out.starts_with("   "), "output: {:?}", out);
    }

    // ── delta ─────────────────────────────────────────────

    #[test]
    fn delta_positive_sign() {
        let cfg = cfg_delta(6);
        let out = render_element(&ElementData::Scalar(0.19), &cfg).unwrap();
        assert!(out.contains('+'), "positive delta must have + sign: {:?}", out);
        assert_eq!(visual_width(&out), 6);
    }

    #[test]
    fn delta_negative_sign() {
        let cfg = cfg_delta(6);
        let out = render_element(&ElementData::Scalar(-4.1), &cfg).unwrap();
        // negative rendered by {:+.2} format produces '-'
        assert!(out.contains('-'), "negative delta must have - sign: {:?}", out);
        assert_eq!(visual_width(&out), 6);
    }

    #[test]
    fn delta_right_align_in_width() {
        let cfg = ElementConfig {
            kind: ElementKind::Delta, width: 8,
            format: "{:+.2}".to_string(),
            align: ElementAlign::Right,
            ..Default::default()
        };
        let out = render_element(&ElementData::Scalar(0.5), &cfg).unwrap();
        assert!(out.starts_with(' '), "should be right-aligned: {:?}", out);
        assert_eq!(visual_width(&out), 8);
    }

    // ── sparkline ─────────────────────────────────────────

    #[test]
    fn sparkline_min_maps_to_lowest_char() {
        let series = vec![0.0, 5.0, 10.0];
        let cfg = cfg_sparkline(3);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        let first = out.chars().next().unwrap();
        assert_eq!(first, '▁', "min value must map to ▁: {:?}", out);
    }

    #[test]
    fn sparkline_max_maps_to_highest_char() {
        let series = vec![0.0, 5.0, 10.0];
        let cfg = cfg_sparkline(3);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        let last = out.chars().last().unwrap();
        assert_eq!(last, '█', "max value must map to █: {:?}", out);
    }

    #[test]
    fn sparkline_all_equal_maps_to_mid() {
        // E-3 edge: all same value → all ▄ (mid-height, per spec F76)
        let series = vec![5.0, 5.0, 5.0, 5.0];
        let cfg = cfg_sparkline(4);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert!(out.chars().all(|c| c == '▄'), "all-equal series should be ▄: {:?}", out);
    }

    #[test]
    fn sparkline_width_equals_series_length() {
        let series = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        let cfg = cfg_sparkline(5);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert_eq!(visual_width(&out), 5);
        assert_eq!(out.chars().count(), 5);
    }

    #[test]
    fn sparkline_longer_series_bucketed_to_width() {
        // 10 values → width=5: each output char = bucket of 2 values
        let series = (0..10).map(|i| i as f64).collect();
        let cfg = cfg_sparkline(5);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert_eq!(visual_width(&out), 5);
    }

    #[test]
    fn sparkline_shorter_series_repeat_fills_width() {
        let series = vec![1.0, 5.0, 3.0];
        let cfg = cfg_sparkline(9); // width > series_len: repeat
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert_eq!(visual_width(&out), 9);
    }

    #[test]
    fn sparkline_single_value_fills_width() {
        let series = vec![7.0];
        let cfg = cfg_sparkline(4);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert_eq!(visual_width(&out), 4);
        // single value: all same → all ▄ (mid-height per spec F76)
        assert!(out.chars().all(|c| c == '▄'), "single value should be ▄: {:?}", out);
    }

    #[test]
    fn sparkline_width_1() {
        let series = vec![1.0, 5.0, 3.0];
        let cfg = cfg_sparkline(1);
        let out = render_element(&ElementData::Series(series), &cfg).unwrap();
        assert_eq!(visual_width(&out), 1);
        assert_eq!(out.chars().count(), 1);
    }

    // ── mini-bar ─────────────────────────────────────────

    #[test]
    fn mini_bar_half_fill() {
        let out = render_element(&ElementData::Scalar(50.0), &cfg_mini_bar(10, 100.0)).unwrap();
        let filled = out.chars().filter(|&c| c == '█').count();
        let empty = out.chars().filter(|&c| c == '░').count();
        assert_eq!(filled, 5, "50% of 10: {:?}", out);
        assert_eq!(empty, 5);
    }

    #[test]
    fn mini_bar_full_fill() {
        let out = render_element(&ElementData::Scalar(200.0), &cfg_mini_bar(20, 200.0)).unwrap();
        assert!(out.chars().all(|c| c == '█'), "100% fill: {:?}", out);
    }

    #[test]
    fn mini_bar_zero_fill() {
        let out = render_element(&ElementData::Scalar(0.0), &cfg_mini_bar(8, 100.0)).unwrap();
        assert!(out.chars().all(|c| c == '░'), "0% fill: {:?}", out);
    }

    #[test]
    fn mini_bar_overflow_clamped() {
        // val > max → clamp to full
        let out = render_element(&ElementData::Scalar(300.0), &cfg_mini_bar(8, 100.0)).unwrap();
        assert!(out.chars().all(|c| c == '█'), "overflow clamped: {:?}", out);
        assert_eq!(visual_width(&out), 8);
    }

    #[test]
    fn mini_bar_e4_proportion_within_1_char() {
        // E-4: fill proportion = val/max within ±1 char
        let val = 65.0_f64;
        let max = 100.0_f64;
        let width = 20;
        let out = render_element(&ElementData::Scalar(val), &cfg_mini_bar(width, max)).unwrap();
        let filled = out.chars().filter(|&c| c == '█').count();
        let expected = (val / max * width as f64).round() as usize;
        assert!((filled as isize - expected as isize).abs() <= 1, "filled={} expected≈{}: {:?}", filled, expected, out);
    }

    // ── label ────────────────────────────────────────────

    #[test]
    fn label_short_left_aligned_padded() {
        let out = render_element(&ElementData::Text("Hi".to_string()), &cfg_label(8, ElementAlign::Left)).unwrap();
        assert!(out.starts_with("Hi"), "output: {:?}", out);
        assert!(out.ends_with("      "), "should be left-padded: {:?}", out);
        assert_eq!(visual_width(&out), 8);
    }

    #[test]
    fn label_truncated_with_ellipsis() {
        let long = "Connor McDavid".to_string();
        let out = render_element(&ElementData::Text(long), &cfg_label(8, ElementAlign::Left)).unwrap();
        assert_eq!(visual_width(&out), 8, "output: {:?}", out);
        assert!(out.contains('…'), "should contain ellipsis: {:?}", out);
    }

    #[test]
    fn label_right_aligned() {
        let out = render_element(&ElementData::Text("Hi".to_string()), &cfg_label(6, ElementAlign::Right)).unwrap();
        assert!(out.starts_with("    "), "should be right-aligned: {:?}", out);
        assert_eq!(visual_width(&out), 6);
    }

    // ── badge ─────────────────────────────────────────────

    #[test]
    fn badge_right_padded_to_width() {
        let out = render_element(&ElementData::Text("UFA".to_string()), &cfg_badge(5)).unwrap();
        assert!(out.starts_with("UFA"), "output: {:?}", out);
        assert_eq!(visual_width(&out), 5);
        assert_eq!(&out, "UFA  ");
    }

    #[test]
    fn badge_exact_fit() {
        let out = render_element(&ElementData::Text("RFA".to_string()), &cfg_badge(3)).unwrap();
        assert_eq!(&out, "RFA");
    }

    #[test]
    fn badge_truncated_if_too_long() {
        let out = render_element(&ElementData::Text("TOOLONG".to_string()), &cfg_badge(4)).unwrap();
        assert_eq!(visual_width(&out), 4, "output: {:?}", out);
    }

    // ── E-2: wrong data kind for scalar kinds ─────────────

    #[test]
    fn e2_series_rejected_for_value() {
        let cfg = cfg_value(6);
        let err = render_element(&ElementData::Series(vec![1.0, 2.0]), &cfg).unwrap_err();
        assert_eq!(err, ElementError::WrongDataKind { expected: "scalar", got: "series" });
    }

    // ── E-5: no_chrome output has no fence ────────────────

    #[test]
    fn e5_no_chrome_has_no_fence() {
        let cfg = ElementConfig {
            kind: ElementKind::Label, width: 5, no_chrome: true,
            ..Default::default()
        };
        let out = render_element(&ElementData::Text("Hi".to_string()), &cfg).unwrap();
        // render_element always returns raw — no fence regardless of no_chrome
        assert!(!out.contains("```"), "should not contain fence: {:?}", out);
        assert!(!out.contains("<!--"), "should not contain HTML comment: {:?}", out);
    }

    // ── E-6: center alignment tie-break ──────────────────

    #[test]
    fn e6_center_tie_break_extra_right() {
        // "Hi" (2) in width=7: pad=5, left=2, right=3
        let cfg = ElementConfig {
            kind: ElementKind::Label, width: 7, align: ElementAlign::Center, ..Default::default()
        };
        let out = render_element(&ElementData::Text("Hi".to_string()), &cfg).unwrap();
        assert_eq!(visual_width(&out), 7);
        assert!(out.starts_with("  Hi"), "output: {:?}", out);
        assert!(out.ends_with("   "), "output: {:?}", out);
    }

    // ── zero width guard ─────────────────────────────────

    #[test]
    fn zero_width_returns_error() {
        let cfg = ElementConfig { kind: ElementKind::Value, width: 0, ..Default::default() };
        let err = render_element(&ElementData::Scalar(1.0), &cfg).unwrap_err();
        assert_eq!(err, ElementError::ZeroWidth);
    }

    // ── align_in_width ────────────────────────────────────

    #[test]
    fn align_left_pads_right() {
        let out = align_in_width("AB", 2, 6, ElementAlign::Left);
        assert_eq!(out, "AB    ");
    }

    #[test]
    fn align_right_pads_left() {
        let out = align_in_width("AB", 2, 6, ElementAlign::Right);
        assert_eq!(out, "    AB");
    }

    #[test]
    fn align_center_even_split() {
        // "AB" (2) in width 6: pad=4, left=2, right=2
        let out = align_in_width("AB", 2, 6, ElementAlign::Center);
        assert_eq!(out, "  AB  ");
    }

    #[test]
    fn align_center_odd_extra_right() {
        // "AB" (2) in width 5: pad=3, left=1, right=2
        let out = align_in_width("AB", 2, 5, ElementAlign::Center);
        assert_eq!(out, " AB  ");
    }
}
