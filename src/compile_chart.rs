use std::path::Path;

use crate::chart::{ChartData, ChartPoint};
use crate::compile_source::resolve_source_for_compile;

/// Resolve a proof:chart directive's data from either an md:// table source or
/// the inline `label: value` directive body.
pub(crate) fn resolve_chart_data(
    source: Option<&str>,
    label_field: Option<&str>,
    value_field: Option<&str>,
    inline_body: &str,
    root: &Path,
) -> std::result::Result<ChartData, String> {
    if let Some(uri) = source {
        let label_col = label_field
            .ok_or_else(|| "proof:chart with source= requires label-field=".to_string())?;
        let value_col = value_field
            .ok_or_else(|| "proof:chart with source= requires value-field=".to_string())?;
        let content = resolve_source_for_compile(uri, root)
            .map_err(|e| format!("chart source error: {}", e))?;
        chart_data_from_table(&content, label_col, value_col)
            .map_err(|e| format!("chart table error: {}", e))
    } else {
        crate::chart::render::parse_inline_body(inline_body)
            .map_err(|(line, msg)| format!("chart body line {}: {}", line + 1, msg))
    }
}

/// Parse a markdown pipe table and extract `(label_col, value_col)` as
/// `ChartData`. Header row determines column order; values must parse as f64.
fn chart_data_from_table(
    content: &str,
    label_col: &str,
    value_col: &str,
) -> std::result::Result<ChartData, String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        rows.push(cells);
    }
    if rows.len() < 2 {
        return Err("expected pipe table with header + separator + body rows".to_string());
    }
    let header = &rows[0];
    let label_idx = header
        .iter()
        .position(|h| h == label_col)
        .ok_or_else(|| format!("label column {:?} not found in header", label_col))?;
    let value_idx = header
        .iter()
        .position(|h| h == value_col)
        .ok_or_else(|| format!("value column {:?} not found in header", value_col))?;

    let mut points = Vec::new();
    for (i, row) in rows.iter().enumerate().skip(1) {
        if row.iter().all(|c| {
            c.chars()
                .all(|ch| ch == '-' || ch == ':' || ch.is_whitespace())
        }) {
            continue;
        }
        if row.len() <= label_idx.max(value_idx) {
            continue;
        }
        let label = row[label_idx].clone();
        let value: f64 = row[value_idx]
            .parse()
            .map_err(|_| format!("row {}: invalid number {:?}", i, row[value_idx]))?;
        points.push(ChartPoint {
            label,
            value,
            extras: Vec::new(),
        });
    }
    Ok(ChartData(points))
}
