//! Gantt chart renderer — tasks as horizontal bars across a time axis.
//!
//! Each ChartPoint is one task: `label` is the task name, `value` is the
//! start position, `extras[0]` is the end position. Optional `extras[1]`
//! encodes a status (0=Done, 1=InProgress, 2=Planned, 3=Optional) which
//! selects the bar's shading glyph.
//!
//! Body syntax: `Build core: 0, 4, 0` → start=0, end=4, status=Done.
//! Time positions are abstract numeric units; the renderer scales to the
//! plot width.

use super::render::{ChartAttrs, ChartData};

const STATUS_GLYPHS: [char; 4] = ['\u{2588}', '\u{2592}', '\u{2591}', '\u{2502}'];
//                                Done(█)    InProgress(▒)  Planned(░)  Optional(│)

pub fn render_gantt_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    if n == 0 {
        return out;
    }

    // Compute time range.
    let starts: Vec<f64> = data.0.iter().map(|p| p.value).collect();
    let ends: Vec<f64> = data
        .0
        .iter()
        .map(|p| p.extras.first().copied().unwrap_or(p.value))
        .collect();
    let t_min = starts.iter().cloned().fold(f64::INFINITY, f64::min);
    let t_max = ends.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let t_range = (t_max - t_min).max(1e-9);

    let label_w = data
        .0
        .iter()
        .map(|p| p.label.chars().count())
        .max()
        .unwrap_or(0);
    let plot_w = attrs.width.saturating_sub(label_w + 3).max(4);

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    // Optional time axis row at top (showing min..max scale).
    let mut axis_row = String::new();
    axis_row.push_str(&" ".repeat(label_w + 3));
    let t_min_str = format_value(t_min);
    let t_max_str = format_value(t_max);
    axis_row.push_str(&t_min_str);
    let mid_pad = plot_w.saturating_sub(t_min_str.chars().count() + t_max_str.chars().count());
    axis_row.push_str(&" ".repeat(mid_pad));
    axis_row.push_str(&t_max_str);
    out.push(axis_row);

    for (i, point) in data.0.iter().enumerate() {
        let s = ((starts[i] - t_min) / t_range * (plot_w - 1) as f64).round() as usize;
        let e = ((ends[i] - t_min) / t_range * (plot_w - 1) as f64).round() as usize;
        let (lo, hi) = if s <= e { (s, e) } else { (e, s) };
        let status = point.extras.get(1).copied().unwrap_or(0.0) as usize;
        let glyph = STATUS_GLYPHS[status.min(STATUS_GLYPHS.len() - 1)];

        let mut bar = String::with_capacity(plot_w);
        for c in 0..plot_w {
            if c >= lo && c <= hi {
                bar.push(glyph);
            } else {
                bar.push(' ');
            }
        }
        let label = pad_left(&point.label, label_w);
        out.push(format!("{}  \u{2502} {}", label, bar));
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
            kind: ChartKind::Gantt,
            width: w,
            ..Default::default()
        }
    }
    fn task(name: &str, start: f64, end: f64, status: f64) -> ChartPoint {
        ChartPoint {
            label: name.into(),
            value: start,
            extras: vec![end, status],
        }
    }

    #[test]
    fn gantt_renders_one_row_per_task() {
        let data = ChartData(vec![
            task("Spec", 0.0, 2.0, 0.0),
            task("Build", 2.0, 6.0, 1.0),
            task("Test", 6.0, 8.0, 2.0),
        ]);
        let lines = render_gantt_chart(&data, &cfg(60));
        // 1 axis row + 3 task rows.
        assert_eq!(lines.len(), 4, "axis + 3 tasks: {:?}", lines);
        for name in ["Spec", "Build", "Test"] {
            assert!(
                lines.iter().any(|l| l.contains(name)),
                "{} present: {:?}",
                name,
                lines
            );
        }
    }

    #[test]
    fn gantt_status_glyphs_distinct() {
        let data = ChartData(vec![
            task("Done", 0.0, 4.0, 0.0),
            task("WIP", 0.0, 4.0, 1.0),
            task("Plan", 0.0, 4.0, 2.0),
        ]);
        let lines = render_gantt_chart(&data, &cfg(60));
        // Each status glyph appears in its task row.
        assert!(lines
            .iter()
            .any(|l| l.contains("Done") && l.contains(STATUS_GLYPHS[0])));
        assert!(lines
            .iter()
            .any(|l| l.contains("WIP") && l.contains(STATUS_GLYPHS[1])));
        assert!(lines
            .iter()
            .any(|l| l.contains("Plan") && l.contains(STATUS_GLYPHS[2])));
    }

    #[test]
    fn gantt_bar_position_reflects_time() {
        // Two tasks: one at the start, one at the end.
        let data = ChartData(vec![
            task("Early", 0.0, 1.0, 0.0),
            task("Late", 9.0, 10.0, 0.0),
        ]);
        let lines = render_gantt_chart(&data, &cfg(60));
        let early_row = lines.iter().find(|l| l.contains("Early")).unwrap();
        let late_row = lines.iter().find(|l| l.contains("Late")).unwrap();
        let early_pos = early_row.find(STATUS_GLYPHS[0]).unwrap();
        let late_pos = late_row.find(STATUS_GLYPHS[0]).unwrap();
        assert!(
            early_pos < late_pos,
            "early task left of late task: {} vs {}",
            early_pos,
            late_pos
        );
    }
}
