//! Stacked-bar chart renderer.
//!
//! Each ChartPoint represents one category (label) with multiple series
//! values: `value` is series 1, `extras[0]` series 2, `extras[1]` series 3,
//! etc. Body syntax: `Q1: 100, 50, 25` → series widths 100/50/25 stacked into
//! a single bar.
//!
//! Bars are drawn by alternating shading characters per series so segments
//! are visually distinguishable: full-block, dark-shade, medium-shade,
//! light-shade, then cycle. Total bar length is proportional to the sum of
//! all series for that row.

use super::render::{ChartAttrs, ChartData};

const SERIES_GLYPHS: &[char] = &[
    '\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}', '#', '*', '+',
];

pub fn render_stacked_bar_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();

    let label_w = data
        .0
        .iter()
        .map(|p| p.label.chars().count())
        .max()
        .unwrap_or(0);

    let totals: Vec<f64> = data
        .0
        .iter()
        .map(|p| p.value + p.extras.iter().sum::<f64>())
        .collect();
    let max_total = attrs.max.unwrap_or_else(|| {
        totals
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0)
    });

    let total_strs: Vec<String> = totals.iter().map(|t| format_value(*t)).collect();
    let value_w = total_strs
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    let chrome = label_w + 3 + 1 + value_w;
    let bar_area = attrs.width.saturating_sub(chrome).max(1);

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    for (i, point) in data.0.iter().enumerate() {
        let mut bar = String::new();
        let mut filled = 0usize;
        // Build segments in order: value, then each extras entry.
        let mut series_values = Vec::with_capacity(1 + point.extras.len());
        series_values.push(point.value);
        series_values.extend_from_slice(&point.extras);
        for (s_idx, sv) in series_values.iter().enumerate() {
            let segment = if max_total > 0.0 {
                ((*sv / max_total) * bar_area as f64).round() as usize
            } else {
                0
            };
            let segment = segment.min(bar_area.saturating_sub(filled));
            let glyph = SERIES_GLYPHS[s_idx % SERIES_GLYPHS.len()];
            for _ in 0..segment {
                bar.push(glyph);
            }
            filled += segment;
        }
        // Pad remaining bar area with spaces.
        for _ in filled..bar_area {
            bar.push(' ');
        }
        let label = pad_left(&point.label, label_w);
        let value = pad_left_str(&total_strs[i], value_w);
        out.push(format!("{}  \u{2502} {} {}", label, bar, value));
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
    let dw = s.chars().count();
    if dw >= w {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(w - dw))
    }
}
fn pad_left_str(s: &str, w: usize) -> String {
    let dw = s.chars().count();
    if dw >= w {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(w - dw), s)
    }
}
fn center_in_width(s: &str, w: usize) -> String {
    let sw = s.chars().count();
    if sw >= w {
        return s.to_string();
    }
    let pad = (w - sw) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

#[cfg(test)]
mod tests {
    use super::super::render::{ChartAttrs, ChartData, ChartKind, ChartPoint};
    use super::*;

    fn cfg(w: usize) -> ChartAttrs {
        ChartAttrs {
            kind: ChartKind::StackedBar,
            width: w,
            ..Default::default()
        }
    }

    #[test]
    fn stacked_segments_use_distinct_glyphs() {
        let data = ChartData(vec![ChartPoint {
            label: "Q1".into(),
            value: 50.0,
            extras: vec![25.0, 10.0],
        }]);
        let lines = render_stacked_bar_chart(&data, &cfg(60));
        let blob = lines.join("\n");
        // First three series glyphs should appear in the bar.
        assert!(
            blob.contains('\u{2588}'),
            "series 1 glyph present: {:?}",
            blob
        );
        assert!(
            blob.contains('\u{2593}'),
            "series 2 glyph present: {:?}",
            blob
        );
        assert!(
            blob.contains('\u{2592}'),
            "series 3 glyph present: {:?}",
            blob
        );
    }

    #[test]
    fn stacked_total_value_displayed() {
        let data = ChartData(vec![ChartPoint {
            label: "Q1".into(),
            value: 30.0,
            extras: vec![20.0],
        }]);
        let lines = render_stacked_bar_chart(&data, &cfg(60));
        // Total = 50; should appear at the row's right side.
        assert!(
            lines[0].contains("50"),
            "total value rendered: {:?}",
            lines[0]
        );
    }

    #[test]
    fn stacked_segment_widths_sum_to_total_bar() {
        let data = ChartData(vec![ChartPoint {
            label: "Q1".into(),
            value: 80.0,
            extras: vec![10.0, 10.0],
        }]);
        let mut attrs = cfg(60);
        attrs.max = Some(100.0);
        let lines = render_stacked_bar_chart(&data, &attrs);
        // Sum of all glyph chars in the bar must roughly equal bar_area.
        let glyphs_total: usize = lines[0]
            .chars()
            .filter(|c| SERIES_GLYPHS.contains(c))
            .count();
        // Allow rounding slack — segment widths are computed independently and
        // each can round up by 0.5, so the sum can exceed bar_area by a few.
        assert!(
            glyphs_total >= 40 && glyphs_total <= 55,
            "expected ~bar_area filled, got {}: {:?}",
            glyphs_total,
            lines[0]
        );
    }
}
