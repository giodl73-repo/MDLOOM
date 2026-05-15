//! Waterfall chart renderer.
//!
//! Each ChartPoint carries one delta (positive or negative). The first point
//! is the starting baseline, the last is conventionally the running total
//! ("End"). Bars are positioned vertically based on the running total before
//! the delta is applied; positive bars rise from the prior level, negative
//! bars descend.
//!
//! Visual: each row is one data point. Within the bar area, the segment
//! showing the delta runs between the prior-running-total column and the
//! new-running-total column. Positive deltas use `█`, negative deltas use
//! `▒`, and the start/end "totals" use `▓`.

use super::render::{ChartAttrs, ChartData};

const POS: char = '\u{2588}'; // █  positive delta
const NEG: char = '\u{2592}'; // ▒  negative delta
const TOTAL: char = '\u{2593}'; // ▓  start / end totals

pub fn render_waterfall_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    if n == 0 {
        return out;
    }

    // Compute running totals: running[0] = data[0].value, running[i] = running[i-1] + data[i].value
    // for i >= 1. The first point is the baseline, not a delta.
    let mut running: Vec<f64> = Vec::with_capacity(n);
    running.push(data.0[0].value);
    for i in 1..n {
        running.push(running[i - 1] + data.0[i].value);
    }

    // Determine value range for bar scaling.
    let max = attrs
        .max
        .unwrap_or_else(|| running.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let min = running
        .iter()
        .cloned()
        .chain(std::iter::once(0.0))
        .fold(f64::INFINITY, f64::min);
    let range = (max - min).max(1.0);

    let label_w = data
        .0
        .iter()
        .map(|p| p.label.chars().count())
        .max()
        .unwrap_or(0);
    let value_strs: Vec<String> = running.iter().map(|v| format_value(*v)).collect();
    let value_w = value_strs
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    let bar_area = attrs.width.saturating_sub(label_w + 3 + 1 + value_w).max(4);

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    for i in 0..n {
        let prev = if i == 0 { 0.0 } else { running[i - 1] };
        let curr = running[i];
        let prev_col = ((prev - min) / range * (bar_area - 1) as f64).round() as usize;
        let curr_col = ((curr - min) / range * (bar_area - 1) as f64).round() as usize;
        let (lo, hi) = if prev_col <= curr_col {
            (prev_col, curr_col)
        } else {
            (curr_col, prev_col)
        };
        let glyph = if i == 0 || i == n - 1 {
            TOTAL
        } else if data.0[i].value >= 0.0 {
            POS
        } else {
            NEG
        };

        let mut bar = String::with_capacity(bar_area);
        for c in 0..bar_area {
            if c >= lo && c <= hi {
                bar.push(glyph);
            } else {
                bar.push(' ');
            }
        }
        let label = pad_left(&data.0[i].label, label_w);
        let value = pad_left_str(&value_strs[i], value_w);
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
            kind: ChartKind::Waterfall,
            width: w,
            ..Default::default()
        }
    }
    fn pt(label: &str, v: f64) -> ChartPoint {
        ChartPoint {
            label: label.into(),
            value: v,
            extras: Vec::new(),
        }
    }

    #[test]
    fn waterfall_start_uses_total_glyph() {
        let data = ChartData(vec![pt("Start", 100.0), pt("d1", 20.0), pt("End", 0.0)]);
        let lines = render_waterfall_chart(&data, &cfg(60));
        assert!(lines[0].contains(TOTAL), "start row uses ▓: {:?}", lines[0]);
    }

    #[test]
    fn waterfall_positive_delta_uses_pos_glyph() {
        let data = ChartData(vec![pt("Start", 100.0), pt("d1", 20.0), pt("End", 0.0)]);
        let lines = render_waterfall_chart(&data, &cfg(60));
        // Middle row (d1) is +20 → POS glyph.
        assert!(
            lines[1].contains(POS),
            "positive delta uses █: {:?}",
            lines[1]
        );
    }

    #[test]
    fn waterfall_negative_delta_uses_neg_glyph() {
        let data = ChartData(vec![pt("Start", 100.0), pt("d1", -30.0), pt("End", 0.0)]);
        let lines = render_waterfall_chart(&data, &cfg(60));
        assert!(
            lines[1].contains(NEG),
            "negative delta uses ▒: {:?}",
            lines[1]
        );
    }

    #[test]
    fn waterfall_running_total_displayed() {
        let data = ChartData(vec![
            pt("Start", 100.0),
            pt("d1", 20.0),
            pt("d2", -30.0),
            pt("End", 0.0),
        ]);
        let lines = render_waterfall_chart(&data, &cfg(60));
        // Running totals: 100, 120, 90, 90 — verify "120" appears on row 1.
        assert!(
            lines[1].contains("120"),
            "running total at d1 = 120: {:?}",
            lines[1]
        );
        assert!(
            lines[2].contains("90"),
            "running total at d2 = 90: {:?}",
            lines[2]
        );
    }
}
