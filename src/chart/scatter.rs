//! Scatter chart renderer — 2D point cloud.
//!
//! Each ChartPoint carries x in `value` and y in `extras[0]`. Body syntax:
//! `pt: 1.5, 4.2` means x=1.5, y=4.2. The label is purely descriptive (used
//! in tooltips conceptually; not currently rendered alongside the marker).

use super::render::{ChartAttrs, ChartData};

const MARKER: char = '\u{25CF}'; // ●
const HORIZ: char = '\u{2500}'; // ─
const CORNER: char = '\u{2514}'; // └
const TICK_TOP: char = '\u{2524}'; // ┤

pub fn render_scatter_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let height = attrs.height.max(2);

    // Pull x/y from value/extras[0]. Points missing y get y=0.
    let pts: Vec<(f64, f64)> = data
        .0
        .iter()
        .map(|p| (p.value, p.extras.first().copied().unwrap_or(0.0)))
        .collect();

    let x_min = pts.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
    let x_max = pts
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let y_min = pts.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
    let y_max = pts
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    let x_range = (x_max - x_min).max(1e-9);
    let y_range = (y_max - y_min).max(1e-9);

    let y_max_str = format_value(y_max);
    let y_min_str = format_value(y_min);
    let y_axis_w = y_max_str.chars().count().max(y_min_str.chars().count());
    let plot_w = attrs.width.saturating_sub(y_axis_w + 2).max(2);

    let mut canvas: Vec<Vec<char>> = vec![vec![' '; plot_w]; height];
    for (x, y) in &pts {
        let col = ((*x - x_min) / x_range * (plot_w - 1) as f64).round() as usize;
        let row = ((1.0 - (*y - y_min) / y_range) * (height - 1) as f64).round() as usize;
        if col < plot_w && row < height {
            canvas[row][col] = MARKER;
        }
    }

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }
    if let Some(y) = &attrs.y_label {
        out.push(y.clone());
    }

    for (row_idx, row) in canvas.iter().enumerate() {
        let label = if row_idx == 0 {
            pad_right(&y_max_str, y_axis_w)
        } else if row_idx == height - 1 {
            pad_right(&y_min_str, y_axis_w)
        } else {
            " ".repeat(y_axis_w)
        };
        let row_str: String = row.iter().collect();
        out.push(format!("{} {} {}", label, TICK_TOP, row_str));
    }
    let baseline_str: String = std::iter::repeat(HORIZ).take(plot_w).collect();
    out.push(format!(
        "{} {}{}",
        " ".repeat(y_axis_w),
        CORNER,
        baseline_str
    ));

    if let Some(x) = &attrs.x_label {
        out.push(format!("{}  {}", " ".repeat(y_axis_w), x));
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
fn pad_right(s: &str, w: usize) -> String {
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

    fn cfg(w: usize, h: usize) -> ChartAttrs {
        ChartAttrs {
            kind: ChartKind::Scatter,
            width: w,
            height: h,
            ..Default::default()
        }
    }

    #[test]
    fn scatter_renders_markers() {
        let data = ChartData(vec![
            ChartPoint {
                label: "p1".into(),
                value: 1.0,
                extras: vec![1.0],
            },
            ChartPoint {
                label: "p2".into(),
                value: 5.0,
                extras: vec![5.0],
            },
            ChartPoint {
                label: "p3".into(),
                value: 10.0,
                extras: vec![2.0],
            },
        ]);
        let lines = render_scatter_chart(&data, &cfg(40, 8));
        let total: usize = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == MARKER).count())
            .sum();
        assert_eq!(total, 3, "3 markers expected:\n{:?}", lines);
    }

    #[test]
    fn scatter_baseline_present() {
        let data = ChartData(vec![
            ChartPoint {
                label: "p".into(),
                value: 1.0,
                extras: vec![1.0],
            },
            ChartPoint {
                label: "q".into(),
                value: 2.0,
                extras: vec![2.0],
            },
        ]);
        let lines = render_scatter_chart(&data, &cfg(40, 6));
        assert!(
            lines.iter().any(|l| l.contains(CORNER)),
            "└ baseline expected:\n{:?}",
            lines
        );
    }

    #[test]
    fn scatter_axis_labels_present() {
        let data = ChartData(vec![
            ChartPoint {
                label: "p".into(),
                value: 0.0,
                extras: vec![0.0],
            },
            ChartPoint {
                label: "q".into(),
                value: 10.0,
                extras: vec![100.0],
            },
        ]);
        let lines = render_scatter_chart(&data, &cfg(40, 6));
        assert!(lines[0].contains("100"));
        assert!(lines[lines.len() - 2].contains('0') || lines[lines.len() - 1].contains('0'));
    }
}
