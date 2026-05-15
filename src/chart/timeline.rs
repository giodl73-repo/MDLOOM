//! Timeline chart renderer — point events on a horizontal time axis.
//!
//! Each ChartPoint is one event: `label` is the event description, `value`
//! is the time position. Body syntax: `Launch: 2024.5`. Events are placed as
//! markers on the axis with non-overlapping label captions below.

use super::render::{ChartAttrs, ChartData};

const MARKER: char = '\u{25C6}'; // ◆
const HORIZ: char = '\u{2500}';
const TICK: char = '\u{252C}';

pub fn render_timeline_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    if n == 0 {
        return out;
    }

    let t_min = data.0.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
    let t_max = data
        .0
        .iter()
        .map(|p| p.value)
        .fold(f64::NEG_INFINITY, f64::max);
    let t_range = (t_max - t_min).max(1e-9);

    let plot_w = attrs.width.max(20);

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, plot_w));
    }

    // Marker row.
    let mut marker_row: Vec<char> = vec![' '; plot_w];
    let mut tick_cols: Vec<usize> = Vec::with_capacity(n);
    for p in &data.0 {
        let col = ((p.value - t_min) / t_range * (plot_w - 1) as f64).round() as usize;
        if col < plot_w {
            marker_row[col] = MARKER;
        }
        tick_cols.push(col);
    }
    out.push(marker_row.iter().collect::<String>());

    // Axis baseline with ticks under each event.
    let mut axis: Vec<char> = vec![HORIZ; plot_w];
    for &c in &tick_cols {
        if c < plot_w {
            axis[c] = TICK;
        }
    }
    out.push(axis.iter().collect::<String>());

    // Label row(s) — labels placed under their tick. Stagger if overlapping
    // by emitting labels on alternating rows.
    let mut row_a: Vec<char> = vec![' '; plot_w];
    let mut row_b: Vec<char> = vec![' '; plot_w];
    let mut last_end_a = 0usize;
    let mut last_end_b = 0usize;
    let mut used_b = false;
    for (i, p) in data.0.iter().enumerate() {
        let lw = p.label.chars().count();
        let center = tick_cols[i];
        let start = center.saturating_sub(lw / 2).min(plot_w.saturating_sub(lw));
        if start >= last_end_a {
            for (j, ch) in p.label.chars().enumerate() {
                if start + j < plot_w {
                    row_a[start + j] = ch;
                }
            }
            last_end_a = start + lw + 1;
        } else if start >= last_end_b {
            used_b = true;
            for (j, ch) in p.label.chars().enumerate() {
                if start + j < plot_w {
                    row_b[start + j] = ch;
                }
            }
            last_end_b = start + lw + 1;
        } // else label dropped (collision) — accepted to keep layout stable
    }
    out.push(row_a.iter().collect::<String>().trim_end().to_string());
    if used_b {
        out.push(row_b.iter().collect::<String>().trim_end().to_string());
    }

    if let Some(x) = &attrs.x_label {
        out.push(format!("{}  {}", " ".repeat(2), x));
    }
    out
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
            kind: ChartKind::Timeline,
            width: w,
            ..Default::default()
        }
    }
    fn ev(name: &str, t: f64) -> ChartPoint {
        ChartPoint {
            label: name.into(),
            value: t,
            extras: Vec::new(),
        }
    }

    #[test]
    fn timeline_renders_markers_for_each_event() {
        let data = ChartData(vec![ev("v1", 0.0), ev("v2", 5.0), ev("v3", 10.0)]);
        let lines = render_timeline_chart(&data, &cfg(40));
        let total_markers: usize = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == MARKER).count())
            .sum();
        assert_eq!(total_markers, 3, "3 markers: {:?}", lines);
    }

    #[test]
    fn timeline_baseline_present() {
        let data = ChartData(vec![ev("a", 1.0), ev("b", 2.0)]);
        let lines = render_timeline_chart(&data, &cfg(30));
        assert!(
            lines.iter().any(|l| l.contains(HORIZ)),
            "baseline: {:?}",
            lines
        );
    }

    #[test]
    fn timeline_labels_appear() {
        let data = ChartData(vec![ev("Launch", 0.0), ev("v2.0", 10.0)]);
        let lines = render_timeline_chart(&data, &cfg(40));
        let blob = lines.join("\n");
        assert!(blob.contains("Launch"));
        assert!(blob.contains("v2.0"));
    }

    #[test]
    fn timeline_first_marker_left_of_last() {
        let data = ChartData(vec![ev("A", 0.0), ev("Z", 100.0)]);
        let lines = render_timeline_chart(&data, &cfg(40));
        let marker_row = lines
            .iter()
            .find(|l| l.chars().filter(|&c| c == MARKER).count() >= 2)
            .unwrap();
        let positions: Vec<usize> = marker_row
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == MARKER)
            .map(|(i, _)| i)
            .collect();
        assert!(
            positions[0] < positions[1],
            "first event left of last: {:?}",
            positions
        );
    }
}
