//! Area chart renderer — line chart with the region under the curve filled.
//!
//! Reuses the line chart's geometry (point placement, segment drawing, axes)
//! but additionally fills every cell at or below the curve with `░` so the
//! plot reads as a "mountain" shape instead of a thin line.

use super::render::{ChartAttrs, ChartData};

const POINT: char       = '\u{25CF}';  // ●
const FILL:  char       = '\u{2591}';  // ░
const HORIZ: char       = '\u{2500}';  // ─
const CORNER: char      = '\u{2514}';  // └
const TICK_TOP: char    = '\u{2524}';  // ┤
const TICK_BOTTOM: char = '\u{252C}';  // ┬

pub fn render_area_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    let height = attrs.height.max(2);

    let max = attrs.max.unwrap_or_else(|| {
        data.0.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max)
    });
    let min = data.0.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
    let range = (max - min).abs();

    let y_max_str = format_value(max);
    let y_min_str = format_value(min);
    let y_axis_w = y_max_str.chars().count().max(y_min_str.chars().count());
    let plot_w = attrs.width.saturating_sub(y_axis_w + 2).max(n.max(1));

    let xs: Vec<usize> = if n == 1 {
        vec![0]
    } else {
        (0..n).map(|i| (i * (plot_w - 1)) / (n - 1)).collect()
    };
    let ys: Vec<usize> = data.0.iter().map(|p| {
        if range == 0.0 { height / 2 }
        else {
            let norm = (p.value - min) / range;
            ((1.0 - norm) * (height - 1) as f64).round() as usize
        }
    }).collect();

    let mut canvas: Vec<Vec<char>> = vec![vec![' '; plot_w]; height];

    // Fill below the curve: at each column x in [0, plot_w), interpolate the
    // y position from the surrounding data points and fill rows from that y
    // down to height-1 with ░.
    for x in 0..plot_w {
        let y_at_x = interpolate_y(x, &xs, &ys);
        for row in y_at_x..height {
            if canvas[row][x] == ' ' { canvas[row][x] = FILL; }
        }
    }
    // Plot point markers on top.
    for i in 0..n {
        if ys[i] < height && xs[i] < plot_w { canvas[ys[i]][xs[i]] = POINT; }
    }

    if let Some(t) = &attrs.title { out.push(center_in_width(t, attrs.width)); }
    if let Some(y) = &attrs.y_label { out.push(y.clone()); }

    for (row_idx, row) in canvas.iter().enumerate() {
        let label = if row_idx == 0 { pad_right(&y_max_str, y_axis_w) }
                    else if row_idx == height - 1 { pad_right(&y_min_str, y_axis_w) }
                    else { " ".repeat(y_axis_w) };
        let row_str: String = row.iter().collect();
        out.push(format!("{} {} {}", label, TICK_TOP, row_str));
    }

    let mut baseline: Vec<char> = vec![HORIZ; plot_w];
    for &x in &xs { if x < plot_w { baseline[x] = TICK_BOTTOM; } }
    let baseline_str: String = baseline.iter().collect();
    out.push(format!("{} {}{}", " ".repeat(y_axis_w), CORNER, baseline_str));

    if let Some(x) = &attrs.x_label {
        out.push(format!("{}  {}", " ".repeat(y_axis_w), x));
    }

    out
}

/// Linear interpolation: y at column x given anchor xs[]/ys[].
fn interpolate_y(x: usize, xs: &[usize], ys: &[usize]) -> usize {
    if xs.is_empty() { return 0; }
    if x <= xs[0] { return ys[0]; }
    if x >= xs[xs.len() - 1] { return ys[ys.len() - 1]; }
    for i in 0..xs.len() - 1 {
        if x >= xs[i] && x <= xs[i + 1] {
            let dx = (xs[i + 1] - xs[i]) as f64;
            if dx == 0.0 { return ys[i]; }
            let t = (x - xs[i]) as f64 / dx;
            let y = ys[i] as f64 + t * (ys[i + 1] as f64 - ys[i] as f64);
            return y.round() as usize;
        }
    }
    ys[ys.len() - 1]
}

fn format_value(v: f64) -> String {
    if v.fract().abs() < 1e-9 { format!("{}", v as i64) } else { format!("{:.2}", v) }
}
fn pad_right(s: &str, w: usize) -> String {
    let dw = s.chars().count();
    if dw >= w { s.to_string() } else { format!("{}{}", " ".repeat(w - dw), s) }
}
fn center_in_width(s: &str, w: usize) -> String {
    let sw = s.chars().count();
    if sw >= w { return s.to_string(); }
    let pad = (w - sw) / 2;
    format!("{}{}", " ".repeat(pad), s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::render::{ChartAttrs, ChartData, ChartKind, ChartPoint};

    fn cfg(w: usize, h: usize) -> ChartAttrs {
        ChartAttrs { kind: ChartKind::Area, width: w, height: h, ..Default::default() }
    }
    fn pts(pairs: &[(&str, f64)]) -> ChartData {
        ChartData(pairs.iter().map(|(l, v)| ChartPoint { label: l.to_string(), value: *v, extras: Vec::new() }).collect())
    }

    #[test]
    fn area_fills_below_curve() {
        let data = pts(&[("A", 10.0), ("B", 0.0), ("C", 10.0)]);
        let lines = render_area_chart(&data, &cfg(40, 6));
        // Expect at least one ░ glyph in the plot area.
        let has_fill = lines.iter().any(|l| l.contains(FILL));
        assert!(has_fill, "expected fill ░ in area chart:\n{:?}", lines);
    }

    #[test]
    fn area_renders_points() {
        let data = pts(&[("A", 1.0), ("B", 5.0), ("C", 3.0)]);
        let lines = render_area_chart(&data, &cfg(40, 6));
        let total_points: usize = lines.iter().map(|l| l.chars().filter(|&c| c == POINT).count()).sum();
        assert_eq!(total_points, 3);
    }

    #[test]
    fn area_title_centered() {
        let data = pts(&[("A", 1.0), ("B", 2.0)]);
        let mut attrs = cfg(40, 5);
        attrs.title = Some("Sales".into());
        let lines = render_area_chart(&data, &attrs);
        assert!(lines[0].contains("Sales"));
    }
}
