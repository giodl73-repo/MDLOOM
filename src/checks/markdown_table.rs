/// GFM pipe table validator.
///
/// Validates markdown pipe tables for:
///   1. Structural correctness — separator row, consistent column counts
///   2. Cell padding — at least 1 space on each side
///   3. Schema conformance — required headings, column names, row keys
///
/// GFM pipe table syntax (§4.10):
///   | Header A | Header B |   ← header row (row 0)
///   |----------|----------|   ← separator row (row 1, required)
///   | data     | data     |   ← body rows (rows 2+)
///
/// Tables are detected OUTSIDE code blocks only.
/// Separator cells must match: optional spaces + optional `:` + 3+ dashes + optional `:` + optional spaces

use crate::checks::Check;
use crate::config::{MarkdownTableConfig, TableSchema};
use crate::diagnostic::Diagnostic;
use std::collections::HashMap;
use std::path::Path;

pub struct MarkdownTableCheck {
    pub config: MarkdownTableConfig,
}

impl Check for MarkdownTableCheck {
    fn name(&self) -> &'static str { "markdown_table" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        let in_code = code_block_mask(&lines);

        let tables = parse_tables(&lines, &in_code);
        let mut diags = Vec::new();

        // Structural validation for all tables
        for table in &tables {
            diags.extend(validate_structure(path, table, &self.config));
        }

        // Count tables per heading for required_tables check
        if let Some(min) = self.config.required_tables {
            if tables.len() < min {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(), 1, 1,
                    "md_missing_table",
                    format!(
                        "file has {} pipe table{}, requires at least {}",
                        tables.len(),
                        if tables.len() == 1 { "" } else { "s" },
                        min
                    ),
                ));
            }
        }

        // Schema validation for named/headed tables
        for schema in &self.config.table_schemas {
            diags.extend(validate_schema(path, &tables, schema));
        }

        diags
    }
}

// ─────────────────────────────────────────────────────────
// Table data model
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParsedTable {
    /// Column header names (trimmed)
    pub headers: Vec<String>,
    /// Raw separator cells (for format checking)
    pub separator_cells: Vec<String>,
    /// Body rows — each is a vec of trimmed cell values
    pub body_rows: Vec<Vec<String>>,
    /// 1-based line number of the header row
    pub line: usize,
    /// The nearest `## heading` above this table, if any
    pub heading_context: Option<String>,
}

impl ParsedTable {
    pub fn col_count(&self) -> usize {
        self.headers.len()
    }

    /// Return cell value at (row_idx, col_idx) in body, or None
    pub fn body_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.body_rows.get(row)?.get(col).map(|s| s.as_str())
    }

    /// All values in the first (key) column of body rows
    pub fn key_column_values(&self) -> Vec<&str> {
        self.body_rows.iter()
            .filter_map(|row| row.first().map(|s| s.as_str()))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────
// Table parser
// ─────────────────────────────────────────────────────────

/// Parse all pipe tables in `lines`, skipping lines inside code blocks.
/// Returns tables with their heading context.
pub fn parse_tables(lines: &[&str], in_code: &[bool]) -> Vec<ParsedTable> {
    let mut tables = Vec::new();
    let mut heading_context: Option<String> = None;
    let mut i = 0;

    while i < lines.len() {
        // Track heading context
        if !in_code[i] && lines[i].starts_with("## ") {
            heading_context = Some(lines[i].to_string());
        }

        // Look for a table header row (non-code, has pipes, enough cols)
        if !in_code[i] && is_table_row(lines[i]) {
            // Check if next line is a separator
            let next = i + 1;
            if next < lines.len() && !in_code[next] && is_separator_row(lines[next]) {
                // Parse the table
                let header_cells = parse_row(lines[i]);
                let sep_cells = parse_row(lines[next]);

                let mut body_rows = Vec::new();
                let mut j = next + 1;
                while j < lines.len() && !in_code[j] && is_table_row(lines[j]) {
                    body_rows.push(parse_row(lines[j]));
                    j += 1;
                }

                tables.push(ParsedTable {
                    headers: header_cells.iter().map(|c| c.trim().to_string()).collect(),
                    separator_cells: sep_cells,
                    body_rows,
                    line: i + 1, // 1-based
                    heading_context: heading_context.clone(),
                });
                i = j; // skip to after table
                continue;
            }
        }
        i += 1;
    }

    tables
}

/// True if this line looks like a table row (has at least 2 pipe chars).
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.chars().filter(|&c| c == '|').count() >= 2
}

/// True if this row looks like a GFM separator row — used for DETECTION only.
/// Accepts any number of dashes (≥1) to ensure we find the table even if
/// the separator is malformed. Validation of minimum dash count happens separately.
fn is_separator_row(line: &str) -> bool {
    let cells = parse_row(line);
    if cells.is_empty() { return false; }
    cells.iter().all(|cell| is_separator_cell_lenient(cell))
}

/// Lenient check for detection: a separator cell must have ≥1 dash and only
/// dashes, colons, and spaces — but does not enforce the ≥3 dash minimum.
fn is_separator_cell_lenient(cell: &str) -> bool {
    let trimmed = cell.trim();
    if trimmed.is_empty() { return false; }
    let core = trimmed.trim_start_matches(':').trim_end_matches(':');
    let dashes = core.chars().filter(|&c| c == '-').count();
    dashes >= 1 && core.chars().all(|c| c == '-' || c == ' ')
}

/// Strict check for validation: a separator cell must have ≥ min_dashes.
fn is_separator_cell_strict(cell: &str, min_dashes: usize) -> bool {
    let trimmed = cell.trim();
    let core = trimmed.trim_start_matches(':').trim_end_matches(':');
    let dashes = core.chars().filter(|&c| c == '-').count();
    dashes >= min_dashes && core.chars().all(|c| c == '-' || c == ' ')
}

/// Split a table row into cell strings (strips outer pipes, splits on `|`).
fn parse_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    // Strip leading and trailing |
    let inner = if trimmed.starts_with('|') { &trimmed[1..] } else { trimmed };
    let inner = if inner.ends_with('|') { &inner[..inner.len()-1] } else { inner };
    inner.split('|').map(|s| s.to_string()).collect()
}

// ─────────────────────────────────────────────────────────
// Structural validation
// ─────────────────────────────────────────────────────────

fn validate_structure(
    path: &Path,
    table: &ParsedTable,
    config: &MarkdownTableConfig,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let expected_cols = table.col_count();

    // Separator format — each cell must have ≥ min_separator_dashes
    for (ci, cell) in table.separator_cells.iter().enumerate() {
        if !is_separator_cell_strict(cell, config.min_separator_dashes) {
            let trimmed = cell.trim();
            let core = trimmed.trim_start_matches(':').trim_end_matches(':');
            let dashes = core.chars().filter(|&c| c == '-').count();
            diags.push(Diagnostic::warning(
                path.to_path_buf(), table.line + 1, 1,
                "md_table_separator_invalid",
                format!(
                    "separator column {} has {} dash{} — need at least {}",
                    ci + 1, dashes,
                    if dashes == 1 { "" } else { "es" },
                    config.min_separator_dashes
                ),
            ));
        }
    }

    // Column count consistency across all rows
    let sep_cols = table.separator_cells.len();
    if sep_cols != expected_cols {
        diags.push(Diagnostic::error(
            path.to_path_buf(), table.line + 1, 1,
            "md_table_col_mismatch",
            format!(
                "separator has {} column{} but header has {} (line {})",
                sep_cols, if sep_cols == 1 { "" } else { "s" },
                expected_cols, table.line
            ),
        ));
    }

    for (ri, row) in table.body_rows.iter().enumerate() {
        if row.len() != expected_cols {
            diags.push(Diagnostic::error(
                path.to_path_buf(), table.line + 2 + ri, 1,
                "md_table_col_mismatch",
                format!(
                    "body row {} has {} column{} but header has {} — all rows must match",
                    ri + 1, row.len(),
                    if row.len() == 1 { "" } else { "s" },
                    expected_cols
                ),
            ));
        }
    }

    // Cell padding check
    if config.check_cell_padding {
        let min = config.min_cell_padding;
        // Check header and body rows (separator is exempt — dashes, not prose)
        let all_content_rows: Vec<(usize, &Vec<String>)> = std::iter::once((table.line, &table.headers))
            .chain(table.body_rows.iter().enumerate().map(|(i, r)| (table.line + 2 + i, r)))
            .collect();

        for (line_no, cells) in all_content_rows {
            for (ci, cell) in cells.iter().enumerate() {
                let leading = cell.len() - cell.trim_start().len();
                let trailing = cell.len() - cell.trim_end().len();
                if leading < min {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(), line_no, 1,
                        "md_table_cell_padding",
                        format!(
                            "column {} missing left padding (found {} space{}, need {}): {:?}",
                            ci + 1, leading, if leading == 1 { "" } else { "s" }, min,
                            cell.trim()
                        ),
                    ));
                }
                if trailing < min {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(), line_no, 1,
                        "md_table_cell_padding",
                        format!(
                            "column {} missing right padding (found {} space{}, need {}): {:?}",
                            ci + 1, trailing, if trailing == 1 { "" } else { "s" }, min,
                            cell.trim()
                        ),
                    ));
                }
            }
        }
    }

    diags
}

// ─────────────────────────────────────────────────────────
// Schema validation
// ─────────────────────────────────────────────────────────

fn validate_schema(
    path: &Path,
    tables: &[ParsedTable],
    schema: &TableSchema,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Find tables matching this schema's heading context
    let matching: Vec<&ParsedTable> = tables.iter()
        .filter(|t| schema_matches_table(schema, t))
        .collect();

    // If schema requires a table under a specific heading and none found
    if matching.is_empty() {
        let location = schema.heading.as_deref().unwrap_or("(any heading)");
        diags.push(Diagnostic::warning(
            path.to_path_buf(), 1, 1,
            "md_missing_table",
            format!(
                "no table found under \"{}\"; required by table schema",
                location
            ),
        ));
        return diags;
    }

    // Validate each matching table against the schema
    for table in &matching {
        diags.extend(validate_table_against_schema(path, table, schema));
    }

    diags
}

fn schema_matches_table(schema: &TableSchema, table: &ParsedTable) -> bool {
    match &schema.heading {
        None => true, // schema applies to any table
        Some(required_heading) => {
            table.heading_context.as_deref()
                .map(|h| h.trim_start_matches('#').trim() == required_heading.trim_start_matches('#').trim())
                .unwrap_or(false)
        }
    }
}

fn validate_table_against_schema(
    path: &Path,
    table: &ParsedTable,
    schema: &TableSchema,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let heading = schema.heading.as_deref().unwrap_or("table");

    // Required column names (exact header match)
    for req_col in &schema.required_columns {
        let found = table.headers.iter().any(|h| h.trim() == req_col.as_str());
        if !found {
            diags.push(Diagnostic::warning(
                path.to_path_buf(), table.line, 1,
                "md_table_schema",
                format!(
                    "table under \"{}\" missing required column \"{}\"\n  \
                     headers found: [{}]",
                    heading, req_col,
                    table.headers.iter().map(|h| format!("{:?}", h.trim())).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }

    // required_columns_any — at least one must be present
    if !schema.required_columns_any.is_empty() {
        let found_any = schema.required_columns_any.iter()
            .any(|req| table.headers.iter().any(|h| h.trim() == req.as_str()));
        if !found_any {
            diags.push(Diagnostic::warning(
                path.to_path_buf(), table.line, 1,
                "md_table_schema",
                format!(
                    "table under \"{}\" must have at least one of: [{}]\n  headers: [{}]",
                    heading,
                    schema.required_columns_any.iter().map(|s| format!("{:?}", s)).collect::<Vec<_>>().join(", "),
                    table.headers.iter().map(|h| format!("{:?}", h.trim())).collect::<Vec<_>>().join(", ")
                ),
            ));
        }
    }

    // Min body rows
    if let Some(min_rows) = schema.min_body_rows {
        if table.body_rows.len() < min_rows {
            diags.push(Diagnostic::warning(
                path.to_path_buf(), table.line, 1,
                "md_table_schema",
                format!(
                    "table under \"{}\" has {} body row{}, requires at least {}",
                    heading, table.body_rows.len(),
                    if table.body_rows.len() == 1 { "" } else { "s" },
                    min_rows
                ),
            ));
        }
    }

    // Required row keys — values that must appear in the first column
    if !schema.required_row_keys.is_empty() {
        let key_vals: Vec<&str> = table.key_column_values();
        for req_key in &schema.required_row_keys {
            let found = key_vals.iter().any(|v| v.trim() == req_key.as_str());
            if !found {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(), table.line, 1,
                    "md_table_schema",
                    format!(
                        "table under \"{}\" missing required row with key \"{}\"\n  \
                         first-column values: [{}]",
                        heading, req_key,
                        key_vals.iter().map(|v| format!("{:?}", v.trim())).collect::<Vec<_>>().join(", ")
                    ),
                ));
            }
        }
    }

    // Column allowed values
    for (col_name, allowed) in &schema.column_allowed_values {
        // Find the column index
        let col_idx = table.headers.iter().position(|h| h.trim() == col_name.as_str());
        if let Some(idx) = col_idx {
            for (ri, row) in table.body_rows.iter().enumerate() {
                if let Some(cell) = row.get(idx) {
                    let val = cell.trim();
                    if !allowed.iter().any(|a| a.as_str() == val) {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(), table.line + 2 + ri, 1,
                            "md_table_schema",
                            format!(
                                "column \"{}\" value {:?} not in allowed set: [{}]",
                                col_name, val,
                                allowed.iter().map(|s| format!("{:?}", s)).collect::<Vec<_>>().join(", ")
                            ),
                        ));
                    }
                }
            }
        }
    }

    diags
}

// ─────────────────────────────────────────────────────────
// Code block mask (shared with markdown.rs logic)
// ─────────────────────────────────────────────────────────

fn code_block_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut in_block = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            let ch = trimmed.chars().next();
            if matches!(ch, Some('`') | Some('~')) {
                let c = ch.unwrap();
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if run >= 3 { in_block = true; fence_char = c; fence_len = run; }
            }
        } else {
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len { in_block = false; continue; }
            }
            mask[i] = true;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MarkdownTableConfig, TableSchema};

    fn default_check() -> MarkdownTableCheck {
        MarkdownTableCheck { config: MarkdownTableConfig::default() }
    }

    // ─── Structural ───

    #[test]
    fn perfect_table_zero_errors() {
        let content = "# Guide\n\n| Axis | Value |\n|------|-------|\n| Binding | Late |\n| Typing | Static |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        let errs: Vec<_> = diags.iter().filter(|d| matches!(d.severity, crate::diagnostic::Severity::Error)).collect();
        assert!(errs.is_empty(), "perfect table must have zero errors: {:?}", diags.iter().map(|d| &d.message).collect::<Vec<_>>());
    }

    #[test]
    fn col_mismatch_in_body_detected() {
        // Body row has 3 cols, header has 2
        let content = "| A | B |\n|---|---|\n| x | y | extra |\n";
        let tables = parse_tables(&content.lines().collect::<Vec<_>>(), &[false, false, false]);
        assert_eq!(tables.len(), 1);
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_table_col_mismatch"),
            "body col mismatch must be detected");
    }

    #[test]
    fn separator_too_short_detected() {
        // Only 2 dashes — need 3
        let content = "| A | B |\n|--|--|\n| x | y |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_table_separator_invalid"),
            "short separator must be detected");
    }

    #[test]
    fn missing_separator_not_detected_as_table() {
        // Two pipe rows but no separator — NOT a GFM table
        let content = "| A | B |\n| x | y |\n";
        let lines: Vec<&str> = content.lines().collect();
        let mask = vec![false; lines.len()];
        let tables = parse_tables(&lines, &mask);
        assert!(tables.is_empty(), "two pipe rows without separator must not be detected as a table");
    }

    #[test]
    fn table_inside_code_block_ignored() {
        let content = "```\n| A | B |\n|---|---|\n| x | y |\n```\n";
        let lines: Vec<&str> = content.lines().collect();
        let mask = code_block_mask(&lines);
        let tables = parse_tables(&lines, &mask);
        assert!(tables.is_empty(), "table inside code block must not be detected");
    }

    #[test]
    fn alignment_colons_accepted() {
        // Left, center, right aligned separators
        let content = "| A | B | C |\n|:---|:---:|---:|\n| a | b | c |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(!diags.iter().any(|d| d.code == "md_table_separator_invalid"),
            "alignment colons must be valid");
    }

    #[test]
    fn cell_padding_missing_detected() {
        let content = "| A | B |\n|---|---|\n|no-space|no-space|\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_table_cell_padding"),
            "missing cell padding must be warned");
    }

    // ─── Required tables count ───

    #[test]
    fn required_tables_fires_when_none_present() {
        let content = "# Guide\n\nsome prose\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                required_tables: Some(1),
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_missing_table"),
            "must warn when required table count not met");
    }

    #[test]
    fn required_tables_passes_when_met() {
        let content = "# Guide\n\n| A | B |\n|---|---|\n| x | y |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                required_tables: Some(1),
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(!diags.iter().any(|d| d.code == "md_missing_table"),
            "must not warn when required table is present");
    }

    // ─── Schema validation ───

    #[test]
    fn schema_required_column_detected() {
        let content = "## Type System Snapshot\n\n| Wrong | Value |\n|-------|-------|\n| Binding | Late |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                table_schemas: vec![TableSchema {
                    heading: Some("Type System Snapshot".to_string()),
                    required_columns: vec!["Axis".to_string()],
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_table_schema"),
            "missing required column 'Axis' must be flagged");
    }

    #[test]
    fn schema_required_row_key_detected() {
        let content = "## Type System Snapshot\n\n| Axis | Value |\n|------|-------|\n| Binding | Late |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                table_schemas: vec![TableSchema {
                    heading: Some("Type System Snapshot".to_string()),
                    required_row_keys: vec!["Binding".to_string(), "Typing".to_string()],
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.message.contains("Typing")),
            "missing row key 'Typing' must be flagged");
        // "Binding" appears in context listing of the "Typing" error — check specifically
        // that there's no diagnostic saying Binding itself is the MISSING key
        assert!(!diags.iter().any(|d| d.message.contains("key \"Binding\"")),
            "present row key 'Binding' must not be the missing key");
    }

    #[test]
    fn schema_missing_table_under_heading() {
        // Heading exists but no table under it
        let content = "## Type System Snapshot\n\nSome prose but no table.\n\n## Next Section\n\n| A | B |\n|---|---|\n| x | y |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                table_schemas: vec![TableSchema {
                    heading: Some("Type System Snapshot".to_string()),
                    min_body_rows: Some(1),
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_missing_table"),
            "must flag when no table found under required heading");
    }

    #[test]
    fn schema_min_body_rows_enforced() {
        let content = "## Decision Cheat Sheet\n\n| When | Use |\n|------|-----|\n| x | y |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                table_schemas: vec![TableSchema {
                    heading: Some("Decision Cheat Sheet".to_string()),
                    min_body_rows: Some(3),
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_table_schema" && d.message.contains("3")),
            "must flag when body row count below minimum");
    }

    #[test]
    fn schema_column_allowed_values_enforced() {
        let content = "## Status\n\n| Name | Status |\n|------|--------|\n| Item | DONE |\n| Item | IN_PROGRESS |\n| Item | INVALID_STATUS |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                table_schemas: vec![TableSchema {
                    heading: Some("Status".to_string()),
                    column_allowed_values: {
                        let mut m = HashMap::new();
                        m.insert("Status".to_string(), vec!["DONE".to_string(), "IN_PROGRESS".to_string(), "TODO".to_string()]);
                        m
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(diags.iter().any(|d| d.message.contains("INVALID_STATUS")),
            "invalid column value must be flagged");
        // DONE appears in the "not in allowed set" listing — check specifically
        // that no diagnostic says DONE is the invalid value itself
        assert!(!diags.iter().any(|d| d.message.contains("value \"DONE\"")),
            "valid value DONE must not itself be flagged as invalid");
    }
}
