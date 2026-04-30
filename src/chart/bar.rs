//! Horizontal bar chart renderer.
//!
//! Layout: title (if any) above; one row per data point with left-padded label,
//! a vertical separator, the bar fill, then the right-padded numeric value;
//! optional x-axis baseline + caption below.
//!
//! The bar area width is `attrs.width - label_col_width - 2 (separator) - value_width`.

use super::render::{ChartAttrs, ChartData};

const BAR_FILL: char = '\u{2588}';   // █
const BAR_EMPTY: char = ' ';

pub fn render_bar_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();

    // ── Determine column widths
    let label_w = data.0.iter().map(|p| display_width(&p.label)).max().unwrap_or(0);
    let max_value = attrs.max.unwrap_or_else(|| {
        data.0.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max).max(0.0)
    });
    let value_strs: Vec<String> = data.0.iter().map(|p| format_value(p.value)).collect();
    let value_w = value_strs.iter().map(|s| s.chars().count()).max().unwrap_or(0);

    // bar area = total - label - " │ " (3) - " " (1) - value
    let chrome = label_w + 3 + 1 + value_w;
    let bar_area = attrs.width.saturating_sub(chrome).max(1);

    // ── Title
    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    // ── y-label (printed once, above the bars)
    if let Some(y) = &attrs.y_label {
        out.push(y.clone());
    }

    // ── Bars
    for (point, value_str) in data.0.iter().zip(value_strs.iter()) {
        let filled = if max_value > 0.0 {
            ((point.value / max_value) * bar_area as f64).round() as usize
        } else {
            0
        };
        let filled = filled.min(bar_area);
        let label = pad_left(&point.label, label_w);
        let bar: String = std::iter::repeat(BAR_FILL).take(filled)
            .chain(std::iter::repeat(BAR_EMPTY).take(bar_area - filled))
            .collect();
        let value = pad_left_str(value_str, value_w);
        out.push(format!("{}  \u{2502} {} {}", label, bar, value));
    }

    // ── x-label baseline
    if let Some(x) = &attrs.x_label {
        let baseline_indent = label_w + 3; // align with start of bars
        let dashes = "\u{2500}".repeat(bar_area);
        out.push(format!("{}\u{2514}{}", " ".repeat(baseline_indent.saturating_sub(1)), dashes));
        out.push(format!("{}{}", " ".repeat(baseline_indent), x));
    }

    out
}

fn format_value(v: f64) -> String {
    if v.fract().abs() < 1e-9 {
        format!("{}", v as i64)
    } else {
        format!("{:.2}", v)
    }
}

fn pad_left(s: &str, w: usize) -> String {
    let dw = display_width(s);
    if dw >= w { s.to_string() } else { format!("{}{}", s, " ".repeat(w - dw)) }
}

fn pad_left_str(s: &str, w: usize) -> String {
    let dw = s.chars().count();
    if dw >= w { s.to_string() } else { format!("{}{}", " ".repeat(w - dw), s) }
}

fn center_in_width(s: &str, w: usize) -> String {
    let sw = display_width(s);
    if sw >= w { return s.to_string(); }
    let pad = (w - sw) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

fn display_width(s: &str) -> usize {
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::render::{ChartAttrs, ChartData, ChartKind, ChartPoint};

    fn cfg(width: usize) -> ChartAttrs {
        ChartAttrs { kind: ChartKind::Bar, width, ..Default::default() }
    }

    fn pts(pairs: &[(&str, f64)]) -> ChartData {
        ChartData(pairs.iter().map(|(l, v)| ChartPoint { label: l.to_string(), value: *v, extras: Vec::new() }).collect())
    }

    #[test]
    fn bar_basic_two_rows() {
        let data = pts(&[("Alpha", 10.0), ("Beta", 5.0)]);
        let lines = render_bar_chart(&data, &cfg(40));
        assert_eq!(lines.len(), 2);
        // Both lines should contain the separator
        assert!(lines[0].contains('\u{2502}'));
        // Alpha bar should be longer than Beta's
        let alpha_filled = lines[0].chars().filter(|&c| c == BAR_FILL).count();
        let beta_filled = lines[1].chars().filter(|&c| c == BAR_FILL).count();
        assert!(alpha_filled > beta_filled, "Alpha={} Beta={}", alpha_filled, beta_filled);
    }

    #[test]
    fn bar_max_overrides_data() {
        let data = pts(&[("A", 10.0), ("B", 20.0)]);
        let mut attrs = cfg(40);
        attrs.max = Some(100.0);
        let lines = render_bar_chart(&data, &attrs);
        // With max=100, bars should be much shorter than full width
        for l in &lines {
            let filled = l.chars().filter(|&c| c == BAR_FILL).count();
            assert!(filled < 10, "expected shrunk bar, got {} fills in {:?}", filled, l);
        }
    }

    #[test]
    fn bar_value_appears_in_output() {
        let data = pts(&[("X", 42.0)]);
        let lines = render_bar_chart(&data, &cfg(40));
        assert!(lines[0].contains("42"), "{:?}", lines);
    }

    #[test]
    fn bar_label_left_padded() {
        let data = pts(&[("Short", 1.0), ("MuchLonger", 1.0)]);
        let lines = render_bar_chart(&data, &cfg(50));
        // Labels padded so the separator is at the same column on every row
        let pos0 = lines[0].find('\u{2502}').unwrap();
        let pos1 = lines[1].find('\u{2502}').unwrap();
        assert_eq!(pos0, pos1);
    }

    #[test]
    fn bar_title_centered() {
        let data = pts(&[("A", 1.0)]);
        let mut attrs = cfg(40);
        attrs.title = Some("Sales".to_string());
        let lines = render_bar_chart(&data, &attrs);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Sales"));
        // Title should have leading whitespace (center-padded)
        assert!(lines[0].starts_with(' '));
    }

    #[test]
    fn bar_x_label_adds_baseline_and_caption() {
        let data = pts(&[("A", 1.0)]);
        let mut attrs = cfg(40);
        attrs.x_label = Some("Category".to_string());
        let lines = render_bar_chart(&data, &attrs);
        // 1 row + baseline + caption
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains('\u{2514}'));  // └ corner
        assert!(lines[1].contains('\u{2500}'));  // ─ baseline
        assert!(lines[2].contains("Category"));
    }

    #[test]
    fn bar_zero_max_does_not_panic() {
        let data = pts(&[("A", 0.0), ("B", 0.0)]);
        let lines = render_bar_chart(&data, &cfg(40));
        assert_eq!(lines.len(), 2);
        // No fill chars when max is 0
        for l in &lines {
            assert!(!l.contains(BAR_FILL));
        }
    }

    #[test]
    fn bar_narrow_width_still_renders() {
        // Even at very narrow widths, render should not panic and produce one line per point
        let data = pts(&[("AA", 5.0), ("BB", 10.0)]);
        let lines = render_bar_chart(&data, &cfg(15));
        assert_eq!(lines.len(), 2);
    }
}
