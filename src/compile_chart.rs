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

/// Parse a markdown table and extract `(label_col, value_col)` as a `ChartData`.
/// Delegates to `tree::schema::parse_md_table` so chart directives accept the
/// same lenient table forms as every other md:// table consumer.
fn chart_data_from_table(
    content: &str,
    label_col: &str,
    value_col: &str,
) -> std::result::Result<ChartData, String> {
    let (headers, table_rows) =
        crate::tree::schema::parse_md_table(content).map_err(|e| format!("{}", e))?;
    if !headers.iter().any(|h| h == label_col) {
        return Err(format!("label column {:?} not found in header", label_col));
    }
    if !headers.iter().any(|h| h == value_col) {
        return Err(format!("value column {:?} not found in header", value_col));
    }

    let mut points = Vec::new();
    for (i, row) in table_rows.iter().enumerate() {
        let label = row.get(label_col).cloned().unwrap_or_default();
        let value_str = row.get(value_col).cloned().unwrap_or_default();
        let value: f64 = value_str
            .parse()
            .map_err(|_| format!("row {}: invalid number {:?}", i + 1, value_str))?;
        points.push(ChartPoint {
            label,
            value,
            extras: Vec::new(),
        });
    }
    Ok(ChartData(points))
}
