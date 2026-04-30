//! Candlestick (OHLC) chart renderer.
//!
//! Each ChartPoint carries the four OHLC values: `value` is open, then
//! `extras = [high, low, close]`. Body syntax: `2024-01: 100, 110, 95, 105`.
//!
//! Per period the chart draws:
//!   * a vertical wick spanning [low, high] (`│`)
//!   * a body spanning [open, close]:
//!       - if close ≥ open (up): hollow body `┃`-styled, ASCII `O`
//!       - if close < open (down): filled body `█`
//! Periods are arranged left to right in input order; height is shared so all
//! candles share the same y-axis scale.

use super::render::{ChartAttrs, ChartData};

const WICK: char       = '\u{2502}'; // │
const BODY_UP: char    = 'O';
const BODY_DOWN: char  = '\u{2588}'; // █
const HORIZ: char      = '\u{2500}';
const CORNER: char     = '\u{2514}';
const TICK_TOP: char   = '\u{2524}';
const TICK_BOTTOM: char = '\u{252C}';

pub fn render_candlestick_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();
    let n = data.0.len();
    if n == 0 { return out; }
    let height = attrs.height.max(4);

    // Extract OHLC (with safe defaults if extras shorter).
    let ohlc: Vec<(f64, f64, f64, f64)> = data.0.iter().map(|p| {
        let o = p.value;
        let h = p.extras.first().copied().unwrap_or(o);
        let l = p.extras.get(1).copied().unwrap_or(o);
        let c = p.extras.get(2).copied().unwrap_or(o);
        (o, h, l, c)
    }).collect();

    let max = attrs.max.unwrap_or_else(|| ohlc.iter().map(|(_, h, _, _)| *h).fold(f64::NEG_INFINITY, f64::max));
    let min = ohlc.iter().map(|(_, _, l, _)| *l).fold(f64::INFINITY, f64::min);
    let range = (max - min).max(1e-9);

    let y_max_str = format_value(max);
    let y_min_str = format_value(min);
    let y_axis_w = y_max_str.chars().count().max(y_min_str.chars().count());
    let plot_w = attrs.width.saturating_sub(y_axis_w + 2).max(n);

    let xs: Vec<usize> = (0..n).map(|i| (i * (plot_w - 1)) / n.saturating_sub(1).max(1)).collect();

    let mut canvas: Vec<Vec<char>> = vec![vec![' '; plot_w]; height];
    for (i, (o, h, l, c)) in ohlc.iter().enumerate() {
        let col = xs[i].min(plot_w - 1);
        let row_for = |v: f64| -> usize {
            ((1.0 - (v - min) / range) * (height - 1) as f64).round() as usize
        };
        let row_h = row_for(*h);
        let row_l = row_for(*l);
        let row_o = row_for(*o);
        let row_c = row_for(*c);
        // wick from high → low
        for r in row_h..=row_l { if r < height { canvas[r][col] = WICK; } }
        // body between open and close
        let (b_lo, b_hi) = if row_o <= row_c { (row_o, row_c) } else { (row_c, row_o) };
        let body_glyph = if c >= o { BODY_UP } else { BODY_DOWN };
        for r in b_lo..=b_hi { if r < height { canvas[r][col] = body_glyph; } }
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

    out
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
        ChartAttrs { kind: ChartKind::Candlestick, width: w, height: h, ..Default::default() }
    }
    fn ohlc(label: &str, o: f64, h: f64, l: f64, c: f64) -> ChartPoint {
        ChartPoint { label: label.into(), value: o, extras: vec![h, l, c] }
    }

    #[test]
    fn candlestick_up_period_uses_up_body() {
        // close > open → up body.
        let data = ChartData(vec![ohlc("p1", 100.0, 110.0, 95.0, 108.0)]);
        let lines = render_candlestick_chart(&data, &cfg(40, 8));
        let blob = lines.join("\n");
        assert!(blob.contains(BODY_UP), "up period uses {} body: {}", BODY_UP, blob);
    }

    #[test]
    fn candlestick_down_period_uses_down_body() {
        let data = ChartData(vec![ohlc("p1", 110.0, 115.0, 95.0, 100.0)]);
        let lines = render_candlestick_chart(&data, &cfg(40, 8));
        let blob = lines.join("\n");
        assert!(blob.contains(BODY_DOWN), "down period uses {} body: {}", BODY_DOWN, blob);
    }

    #[test]
    fn candlestick_renders_wick_above_and_below_body() {
        // High = 110, body close 105, low 90 → wick above (one row) and below (several).
        let data = ChartData(vec![ohlc("p1", 100.0, 110.0, 90.0, 105.0)]);
        let lines = render_candlestick_chart(&data, &cfg(40, 12));
        let blob = lines.join("\n");
        assert!(blob.contains(WICK), "wick character present: {}", blob);
    }

    #[test]
    fn candlestick_baseline_present() {
        let data = ChartData(vec![ohlc("p1", 100.0, 110.0, 95.0, 105.0)]);
        let lines = render_candlestick_chart(&data, &cfg(40, 6));
        assert!(lines.iter().any(|l| l.contains(CORNER)));
    }
}
