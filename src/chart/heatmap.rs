//! Heatmap renderer — 2D grid of shading glyphs proportional to cell value.
//!
//! Body syntax: `row|col: value`. Rows and columns are inferred from the data;
//! cells absent from the input render as the lowest shading. value is bucketed
//! into 5 levels mapped to ` ░▒▓█`.

use super::render::{ChartAttrs, ChartData};
use std::collections::BTreeMap;

const SHADING: [char; 5] = [' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];

pub fn render_heatmap_chart(data: &ChartData, attrs: &ChartAttrs) -> Vec<String> {
    let mut out = Vec::new();

    // Parse `row|col` from each label; collect unique rows and cols in
    // first-seen order.
    let mut row_order: Vec<String> = Vec::new();
    let mut col_order: Vec<String> = Vec::new();
    let mut cells: BTreeMap<(String, String), f64> = BTreeMap::new();
    for p in &data.0 {
        let (row, col) = match p.label.split_once('|') {
            Some((r, c)) => (r.trim().to_string(), c.trim().to_string()),
            None => (p.label.clone(), "".to_string()),
        };
        if !row_order.contains(&row) {
            row_order.push(row.clone());
        }
        if !col_order.contains(&col) {
            col_order.push(col.clone());
        }
        cells.insert((row, col), p.value);
    }
    if row_order.is_empty() || col_order.is_empty() {
        return out;
    }

    let max = attrs.max.unwrap_or_else(|| {
        cells
            .values()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0)
    });
    let min = cells
        .values()
        .cloned()
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let range = (max - min).max(1e-9);

    let row_label_w = row_order
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0);
    // Each col header gets at least its own label width or 3, whichever bigger.
    let cell_w = col_order
        .iter()
        .map(|s| s.chars().count().max(3))
        .max()
        .unwrap_or(3);

    if let Some(t) = &attrs.title {
        out.push(center_in_width(t, attrs.width));
    }

    // Column header row.
    let mut header = String::new();
    for _ in 0..row_label_w + 1 {
        header.push(' ');
    }
    for c in &col_order {
        let padded = pad_left(c, cell_w);
        header.push_str(&padded);
        header.push(' ');
    }
    out.push(header.trim_end().to_string());

    // Data rows.
    for r in &row_order {
        let mut row = String::new();
        let label = pad_left(r, row_label_w);
        row.push_str(&label);
        row.push(' ');
        for c in &col_order {
            let v = cells.get(&(r.clone(), c.clone())).copied().unwrap_or(min);
            let bucket = (((v - min) / range * (SHADING.len() - 1) as f64).round() as usize)
                .min(SHADING.len() - 1);
            let glyph = SHADING[bucket];
            // Each cell renders as a block of cell_w glyphs (uniform shading).
            for _ in 0..cell_w {
                row.push(glyph);
            }
            row.push(' ');
        }
        out.push(row.trim_end().to_string());
    }

    out
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
            kind: ChartKind::Heatmap,
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
    fn heatmap_uses_full_block_for_max() {
        let data = ChartData(vec![pt("Mon|9am", 0.0), pt("Mon|10am", 100.0)]);
        let lines = render_heatmap_chart(&data, &cfg(40));
        let blob = lines.join("\n");
        assert!(
            blob.contains('\u{2588}'),
            "max cell uses full block: {}",
            blob
        );
    }

    #[test]
    fn heatmap_renders_row_and_col_headers() {
        let data = ChartData(vec![pt("Mon|9am", 1.0), pt("Tue|9am", 2.0)]);
        let lines = render_heatmap_chart(&data, &cfg(40));
        let blob = lines.join("\n");
        assert!(blob.contains("9am"), "col header: {}", blob);
        assert!(blob.contains("Mon"), "row header: {}", blob);
        assert!(blob.contains("Tue"), "row header: {}", blob);
    }

    #[test]
    fn heatmap_shading_distinguishes_buckets() {
        let data = ChartData(vec![
            pt("a|x", 0.0),
            pt("a|y", 25.0),
            pt("a|z", 50.0),
            pt("a|w", 100.0),
        ]);
        let lines = render_heatmap_chart(&data, &cfg(60));
        let blob = lines.join("\n");
        // At least 3 distinct shading glyphs should appear.
        let mut seen = 0;
        for &g in &SHADING {
            if g != ' ' && blob.contains(g) {
                seen += 1;
            }
        }
        assert!(seen >= 3, "expected ≥3 distinct shading levels:\n{}", blob);
    }
}
