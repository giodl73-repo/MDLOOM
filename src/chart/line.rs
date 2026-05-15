//! Line chart renderer — points connected with ASCII line segments.
//!
//! Layout: title (if any), y_max label at top of plot, plot rows with vertical
//! axis on the left (height rows total), points joined by `/`, `\`, `-`, or `|`
//! glyphs, baseline with tick marks at each data x-position, then x-tick labels
//! and an optional x_label caption.

use super::render::{ChartAttrs, ChartData};

const POINT: char = '\u{25CF}'; // ●
const VERT: char = '\u{2502}'; // │
const HORIZ: char = '\u{2500}'; // ─
const CORNER: char = '\u{2514}'; // └
const TICK_TOP: char = '\u{2524}'; // ┤
const TICK_BOTTOM: char = '\u{252C}'; // ┬
const SLASH_UP: char = '/';
const SLASH_DOWN: char = '\\';

pub fn render_line_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    let height = attrs.height.max(2);

    let max = attrs.max.unwrap_or_else(|| {
        data.0
            .iter()
            .map(|p| p.value)
            .fold(f64::NEG_INFINITY, f64::max)
    });
    let min = data.0.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
    let range = (max - min).abs();

    // ── Axis labels (left margin width)
    let y_max_str = format_value(max);
    let y_min_str = format_value(min);
    let y_axis_w = y_max_str.chars().count().max(y_min_str.chars().count());

    // Plot area width (after y-axis label + " ┤ " separator = y_axis_w + 2)
    let plot_w = attrs.width.saturating_sub(y_axis_w + 2).max(n.max(1));

    // X positions for each data point — evenly spaced across plot_w
    let xs: Vec<usize> = if n == 1 {
        vec![0]
    } else {
        (0..n).map(|i| (i * (plot_w - 1)) / (n - 1)).collect()
    };

    // Y positions (row index, 0 = top of plot area, height-1 = bottom)
    let ys: Vec<usize> = data
        .0
        .iter()
        .map(|p| {
            if range == 0.0 {
                height / 2
            } else {
                let norm = (p.value - min) / range;
                ((1.0 - norm) * (height - 1) as f64).round() as usize
            }
        })
        .collect();

    // Build the canvas: height rows, plot_w columns, ' ' fill.
    let mut canvas: Vec<Vec<char>> = vec![vec![' '; plot_w]; height];

    // Draw connecting segments between consecutive points
    for i in 0..n.saturating_sub(1) {
        draw_segment(&mut canvas, xs[i], ys[i], xs[i + 1], ys[i + 1]);
    }
    // Plot points last so they sit on top of segments
    for i in 0..n {
        if ys[i] < height && xs[i] < plot_w {
            canvas[ys[i]][xs[i]] = POINT;
        }
    }

    // ── Title
    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    // ── y-label
    if let Some(y) = &attrs.y_label {
        out.push(y.clone());
    }

    // ── Plot rows with y-axis labels at top and bottom
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

    // ── x-axis baseline with ticks at each data x position
    let mut baseline: Vec<char> = vec![HORIZ; plot_w];
    for &x in &xs {
        if x < plot_w {
            baseline[x] = TICK_BOTTOM;
        }
    }
    let baseline_str: String = baseline.iter().collect();
    out.push(format!(
        "{} {}{}",
        " ".repeat(y_axis_w),
        CORNER,
        baseline_str
    ));

    // ── x-tick labels — abbreviate if labels would collide
    let labels = render_x_tick_labels(
        &data.0.iter().map(|p| p.label.clone()).collect::<Vec<_>>(),
        &xs,
        plot_w,
    );
    if !labels.is_empty() {
        out.push(format!("{}  {}", " ".repeat(y_axis_w), labels));
    }

    // ── x_label caption (free-text axis title)
    if let Some(x) = &attrs.x_label {
        out.push(format!("{}  {}", " ".repeat(y_axis_w), x));
    }

    out
}

/// Bresenham-style line drawing between two grid points.
fn draw_segment(canvas: &mut [Vec<char>], x0: usize, y0: usize, x1: usize, y1: usize) {
    if x0 == x1 && y0 == y1 {
        return;
    }
    let height = canvas.len();
    let width = canvas[0].len();

    let dx = x1 as i32 - x0 as i32;
    let dy = y1 as i32 - y0 as i32;
    let steps = dx.abs().max(dy.abs());
    if steps == 0 {
        return;
    }

    for s in 1..steps {
        // skip endpoints — points draw them
        let t = s as f64 / steps as f64;
        let x = (x0 as f64 + dx as f64 * t).round() as i32;
        let y = (y0 as f64 + dy as f64 * t).round() as i32;
        if x < 0 || y < 0 {
            continue;
        }
        let (xu, yu) = (x as usize, y as usize);
        if xu >= width || yu >= height {
            continue;
        }
        if canvas[yu][xu] == ' ' {
            // Pick connector based on slope direction
            let glyph = if dy == 0 {
                HORIZ
            } else if dx == 0 {
                VERT
            } else if (dy < 0) == (dx > 0) {
                // up-right or down-left: '/'
                SLASH_UP
            } else {
                SLASH_DOWN
            };
            canvas[yu][xu] = glyph;
        }
    }
}

/// Lay out x-tick labels under their tick positions, dropping any that would overlap.
fn render_x_tick_labels(labels: &[String], xs: &[usize], plot_w: usize) -> String {
    let mut out: Vec<char> = vec![' '; plot_w];
    let mut last_end = 0usize;
    for (label, &x) in labels.iter().zip(xs.iter()) {
        let lw = label.chars().count();
        if x < last_end {
            continue;
        } // overlap with prior label, skip
          // Place label centered on tick, but clamp into [0, plot_w-lw]
        let start = x.saturating_sub(lw / 2);
        let start = start.min(plot_w.saturating_sub(lw));
        for (i, ch) in label.chars().enumerate() {
            if start + i < plot_w {
                out[start + i] = ch;
            }
        }
        last_end = start + lw + 1; // +1 for spacing
    }
    out.iter().collect::<String>().trim_end().to_string()
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
            kind: ChartKind::Line,
            width: w,
            height: h,
            ..Default::default()
        }
    }

    fn pts(pairs: &[(&str, f64)]) -> ChartData {
        ChartData(
            pairs
                .iter()
                .map(|(l, v)| ChartPoint {
                    label: l.to_string(),
                    value: *v,
                })
                .collect(),
        )
    }

    #[test]
    fn line_basic_renders() {
        let data = pts(&[("A", 1.0), ("B", 5.0), ("C", 3.0)]);
        let lines = render_line_chart(&data, &cfg(40, 6));
        // height (6) + baseline (1) + tick-labels (1) = 8
        assert_eq!(lines.len(), 8);
        // Should contain at least 3 point glyphs
        let total_points: usize = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == POINT).count())
            .sum();
        assert_eq!(total_points, 3);
    }

    #[test]
    fn line_baseline_present() {
        let data = pts(&[("A", 1.0), ("B", 2.0)]);
        let lines = render_line_chart(&data, &cfg(30, 5));
        assert!(
            lines.iter().any(|l| l.contains(CORNER)),
            "expected └ in {:?}",
            lines
        );
    }

    #[test]
    fn line_y_axis_labels_present() {
        let data = pts(&[("A", 0.0), ("B", 100.0)]);
        let lines = render_line_chart(&data, &cfg(40, 6));
        // top row should mention 100 (max), bottom row should mention 0 (min)
        assert!(lines[0].contains("100"), "top: {:?}", lines[0]);
        assert!(lines[5].contains('0'), "bot: {:?}", lines[5]);
    }

    #[test]
    fn line_x_tick_labels() {
        let data = pts(&[("Jan", 1.0), ("Feb", 2.0), ("Mar", 3.0)]);
        let lines = render_line_chart(&data, &cfg(40, 6));
        let last = &lines[lines.len() - 1];
        assert!(last.contains("Jan"), "x-ticks: {:?}", last);
        assert!(last.contains("Mar"), "x-ticks: {:?}", last);
    }

    #[test]
    fn line_constant_series_no_panic() {
        let data = pts(&[("A", 5.0), ("B", 5.0), ("C", 5.0)]);
        let lines = render_line_chart(&data, &cfg(30, 5));
        assert!(!lines.is_empty());
    }

    #[test]
    fn line_single_point() {
        let data = pts(&[("Only", 7.0)]);
        let lines = render_line_chart(&data, &cfg(30, 5));
        let total_points: usize = lines
            .iter()
            .map(|l| l.chars().filter(|&c| c == POINT).count())
            .sum();
        assert_eq!(total_points, 1);
    }

    #[test]
    fn line_title_on_first_line() {
        let data = pts(&[("A", 1.0), ("B", 2.0)]);
        let mut attrs = cfg(40, 5);
        attrs.title = Some("Trend".to_string());
        let lines = render_line_chart(&data, &attrs);
        assert!(lines[0].contains("Trend"));
    }

    #[test]
    fn line_x_label_caption() {
        let data = pts(&[("A", 1.0), ("B", 2.0)]);
        let mut attrs = cfg(40, 5);
        attrs.x_label = Some("Months".to_string());
        let lines = render_line_chart(&data, &attrs);
        assert!(lines.last().unwrap().contains("Months"));
    }

    #[test]
    fn line_segments_drawn_between_points() {
        let data = pts(&[("A", 0.0), ("B", 10.0)]);
        let lines = render_line_chart(&data, &cfg(40, 6));
        // Ascending line should produce some / glyphs in plot area
        let has_slash = lines.iter().any(|l| l.contains(SLASH_UP));
        assert!(has_slash, "expected slash in ascending line: {:?}", lines);
    }
}
