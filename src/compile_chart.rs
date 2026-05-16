use std::path::Path;

use crate::chart::{ChartAttrs, ChartData, ChartPoint};
use crate::compile_output;
use crate::compile_source::resolve_source_for_compile;
use crate::compile_types::{CompileViolation, ViolationSeverity};

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_chart(
    attrs: &ChartAttrs,
    source: Option<&String>,
    label_field: Option<&String>,
    value_field: Option<&String>,
    inline_body: &str,
    root: &Path,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match resolve_chart_data(
        source.map(|s| s.as_str()),
        label_field.map(|s| s.as_str()),
        value_field.map(|s| s.as_str()),
        inline_body,
        root,
    ) {
        Ok(data) => match crate::chart::render_chart(&data, attrs) {
            Ok(lines) => {
                *resolved_count += 1;
                let rendered = lines.join("\n");
                if attrs.no_chrome {
                    format!("```\n{}\n```", rendered)
                } else {
                    format!(
                        "<!-- proof:compiled from=\"proof:chart\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
                        rendered
                    )
                }
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: e.code,
                    severity: ViolationSeverity::Error,
                    uri: source.cloned().unwrap_or_default(),
                    figure_id: None,
                    invariant: String::new(),
                    message: e.message,
                    source_line: line_start + 1 + source_line_offset,
                });
                compile_output::source_fallback(source_lines, line_start, line_end)
            }
        },
        Err(msg) => {
            violations.push(CompileViolation {
                code: "CHART-002",
                severity: ViolationSeverity::Error,
                uri: source.cloned().unwrap_or_default(),
                figure_id: None,
                invariant: String::new(),
                message: msg,
                source_line: line_start + 1 + source_line_offset,
            });
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

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
