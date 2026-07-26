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
use std::path::Path;

pub struct MarkdownTableCheck {
    pub config: MarkdownTableConfig,
}

impl Check for MarkdownTableCheck {
    fn name(&self) -> &'static str {
        "markdown_table"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        let in_code = code_block_mask(&lines);

        let tables = parse_tables(&lines, &in_code);
        let mut diags = Vec::new();

        if self.config.flag_inline_source_tables && is_source_document(path) {
            for table in &tables {
                diags.push(
                    Diagnostic::warning(
                        path.to_path_buf(),
                        table.line,
                        1,
                        "source_inline_table",
                        "inline pipe table in .source.md; move durable row data to a sidecar table and reference it from MDLOOM",
                    )
                    .with_note(
                        "source documents may render tables, but canonical data should live in JSON/CSV/sidecar tables for mdloom, mdport, and mdcrop pipelines",
                    )
                    .with_group(format!("source-table-l{}", table.line)),
                );
            }
        }

        // Structural validation for all tables
        for table in &tables {
            diags.extend(validate_structure(path, table, &self.config));
        }

        // Table quality checks — empty headers, max columns
        for table in &tables {
            if self.config.check_empty_headers {
                for (ci, header) in table.headers.iter().enumerate() {
                    if ci == 0 && is_row_label_corner(table) {
                        continue;
                    }
                    if header.trim().is_empty() {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            table.line,
                            ci + 1,
                            "md_table_empty_header",
                            format!(
                                "column {} has an empty header — all columns should be named",
                                ci + 1
                            ),
                        ));
                    }
                }
            }
            if self.config.max_columns > 0 && table.col_count() > self.config.max_columns {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    table.line,
                    1,
                    "md_table_too_wide",
                    format!(
                        "table has {} columns, exceeds max of {} — consider splitting or rotating",
                        table.col_count(),
                        self.config.max_columns
                    ),
                ));
            }
        }

        // Count tables per heading for required_tables check
        if let Some(min) = self.config.required_tables {
            if tables.len() < min {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    1,
                    1,
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
        self.body_rows
            .iter()
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
                    // Keep raw cells (with whitespace) for padding validation.
                    // Schema matching trims at comparison time via .trim().
                    headers: header_cells,
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
    trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed.chars().filter(|&c| c == '|').count() >= 2
}

fn is_source_document(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".source.md"))
}

/// True if this row looks like a GFM separator row — used for DETECTION only.
/// Accepts any number of dashes (≥1) to ensure we find the table even if
/// the separator is malformed. Validation of minimum dash count happens separately.
fn is_separator_row(line: &str) -> bool {
    let cells = parse_row(line);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|cell| is_separator_cell_lenient(cell))
}

/// Lenient check for detection: a separator cell must have ≥1 dash and only
/// dashes, colons, and spaces — but does not enforce the ≥3 dash minimum.
fn is_separator_cell_lenient(cell: &str) -> bool {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return false;
    }
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

/// Split a table row into cell strings (strips outer pipes, splits on unescaped `|`).
///
/// Handles GFM table escaping rules:
///   - `\|` inside a cell is an escaped pipe — NOT a column separator
///   - `|` inside backtick code spans is content — NOT a separator
///   - `||` (SQL concat) is two pipe chars but context determines if they're separators
///
/// See: https://github.github.com/gfm/#tables (§4.10)
fn parse_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // Identify the inner content (between outer pipe delimiters if present)
    let inner = if trimmed.starts_with('|') {
        &trimmed[1..]
    } else {
        trimmed
    };
    let inner = if inner.ends_with('|') {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    // Walk character-by-character, respecting:
    //   \| — escaped pipe, kept as content
    //   backtick spans — | inside `` ` `` ... `` ` `` is content
    let mut cells: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    let mut in_code_span = false;
    let mut code_span_char = '`';

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                // Escaped pipe — keep in cell content
                current.push('\\');
                current.push('|');
                chars.next();
            }
            '`' if !in_code_span => {
                // Opening code span
                in_code_span = true;
                code_span_char = '`';
                current.push(c);
            }
            c if c == code_span_char && in_code_span => {
                // Closing code span
                in_code_span = false;
                current.push(c);
            }
            '|' if !in_code_span => {
                // Unescaped, not-in-code-span pipe = column separator
                cells.push(current.clone());
                current = String::new();
            }
            other => {
                current.push(other);
            }
        }
    }
    cells.push(current);
    cells
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
            if ci == 0 && is_row_label_corner(table) && dashes >= 2 {
                continue;
            }
            diags.push(Diagnostic::warning(
                path.to_path_buf(),
                table.line + 1,
                1,
                "md_table_separator_invalid",
                format!(
                    "separator column {} has {} dash{} — need at least {}",
                    ci + 1,
                    dashes,
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
            path.to_path_buf(),
            table.line + 1,
            1,
            "md_table_col_mismatch",
            format!(
                "separator has {} column{} but header has {} (line {})",
                sep_cols,
                if sep_cols == 1 { "" } else { "s" },
                expected_cols,
                table.line
            ),
        ));
    }

    for (ri, row) in table.body_rows.iter().enumerate() {
        if row.len() != expected_cols {
            // If body has MORE cols than header and ignore_extra_body_cols is set,
            // skip — this is likely a false positive from | in math/code content.
            // Body rows with FEWER cols are always flagged (genuine missing columns).
            if config.ignore_extra_body_cols && row.len() > expected_cols {
                continue;
            }
            diags.push(Diagnostic::error(
                path.to_path_buf(),
                table.line + 2 + ri,
                1,
                "md_table_col_mismatch",
                format!(
                    "body row {} has {} column{} but header has {} — all rows must match",
                    ri + 1,
                    row.len(),
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
        let all_content_rows: Vec<(usize, &Vec<String>)> =
            std::iter::once((table.line, &table.headers))
                .chain(table.body_rows.iter().enumerate().filter_map(|(i, row)| {
                    if row.len() == expected_cols {
                        Some((table.line + 2 + i, row))
                    } else {
                        None
                    }
                }))
                .collect();

        for (line_no, cells) in all_content_rows {
            for (ci, cell) in cells.iter().enumerate() {
                if !has_room_for_padding(cell, min) {
                    continue;
                }
                let leading = leading_whitespace_count(cell);
                let trailing = trailing_whitespace_count(cell);
                if leading < min {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        line_no,
                        1,
                        "md_table_cell_padding",
                        format!(
                            "column {} missing left padding (found {} space{}, need {}): {:?}",
                            ci + 1,
                            leading,
                            if leading == 1 { "" } else { "s" },
                            min,
                            cell.trim()
                        ),
                    ));
                }
                if trailing < min {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        line_no,
                        1,
                        "md_table_cell_padding",
                        format!(
                            "column {} missing right padding (found {} space{}, need {}): {:?}",
                            ci + 1,
                            trailing,
                            if trailing == 1 { "" } else { "s" },
                            min,
                            cell.trim()
                        ),
                    ));
                }
            }
        }
    }

    diags
}

fn is_row_label_corner(table: &ParsedTable) -> bool {
    table.headers.first().is_some_and(|h| h.trim().is_empty())
        && table.headers.iter().skip(1).any(|h| !h.trim().is_empty())
        && table
            .body_rows
            .iter()
            .any(|row| row.first().is_some_and(|cell| !cell.trim().is_empty()))
}

fn has_room_for_padding(cell: &str, min_pad: usize) -> bool {
    let available = visual_width(cell);
    let content = visual_width(cell.trim());
    content + (min_pad * 2) <= available
}

fn leading_whitespace_count(s: &str) -> usize {
    s.chars().take_while(|c| c.is_whitespace()).count()
}

fn trailing_whitespace_count(s: &str) -> usize {
    s.chars().rev().take_while(|c| c.is_whitespace()).count()
}

fn visual_width(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

// ─────────────────────────────────────────────────────────
// Schema validation
// ─────────────────────────────────────────────────────────

/// Returns true if a cell contains a markdown link [text](url).
fn has_markdown_link(cell: &str) -> bool {
    cell.contains("](") && cell.contains('[')
}

fn validate_schema(path: &Path, tables: &[ParsedTable], schema: &TableSchema) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Find tables matching this schema's heading context
    let matching: Vec<&ParsedTable> = tables
        .iter()
        .filter(|t| schema_matches_table(schema, t))
        .collect();

    // If schema requires a table under a specific heading and none found
    if matching.is_empty() {
        let location = schema.heading.as_deref().unwrap_or("(any heading)");
        diags.push(Diagnostic::warning(
            path.to_path_buf(),
            1,
            1,
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
        Some(required_heading) => table
            .heading_context
            .as_deref()
            .map(|h| {
                h.trim_start_matches('#').trim() == required_heading.trim_start_matches('#').trim()
            })
            .unwrap_or(false),
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
                path.to_path_buf(),
                table.line,
                1,
                "md_table_schema",
                format!(
                    "table under \"{}\" missing required column \"{}\"\n  \
                     headers found: [{}]",
                    heading,
                    req_col,
                    table
                        .headers
                        .iter()
                        .map(|h| format!("{:?}", h.trim()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }

    // required_columns_any — at least one must be present
    if !schema.required_columns_any.is_empty() {
        let found_any = schema
            .required_columns_any
            .iter()
            .any(|req| table.headers.iter().any(|h| h.trim() == req.as_str()));
        if !found_any {
            diags.push(Diagnostic::warning(
                path.to_path_buf(),
                table.line,
                1,
                "md_table_schema",
                format!(
                    "table under \"{}\" must have at least one of: [{}]\n  headers: [{}]",
                    heading,
                    schema
                        .required_columns_any
                        .iter()
                        .map(|s| format!("{:?}", s))
                        .collect::<Vec<_>>()
                        .join(", "),
                    table
                        .headers
                        .iter()
                        .map(|h| format!("{:?}", h.trim()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }

    // Min body rows
    if let Some(min_rows) = schema.min_body_rows {
        if table.body_rows.len() < min_rows {
            diags.push(Diagnostic::warning(
                path.to_path_buf(),
                table.line,
                1,
                "md_table_schema",
                format!(
                    "table under \"{}\" has {} body row{}, requires at least {}",
                    heading,
                    table.body_rows.len(),
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
                    path.to_path_buf(),
                    table.line,
                    1,
                    "md_table_schema",
                    format!(
                        "table under \"{}\" missing required row with key \"{}\"\n  \
                         first-column values: [{}]",
                        heading,
                        req_key,
                        key_vals
                            .iter()
                            .map(|v| format!("{:?}", v.trim()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }
        }
    }

    // Column allowed values
    for (col_name, allowed) in &schema.column_allowed_values {
        // Find the column index
        let col_idx = table
            .headers
            .iter()
            .position(|h| h.trim() == col_name.as_str());
        if let Some(idx) = col_idx {
            for (ri, row) in table.body_rows.iter().enumerate() {
                if let Some(cell) = row.get(idx) {
                    let val = cell.trim();
                    if !allowed.iter().any(|a| a.as_str() == val) {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            table.line + 2 + ri,
                            1,
                            "md_table_schema",
                            format!(
                                "column \"{}\" value {:?} not in allowed set: [{}]",
                                col_name,
                                val,
                                allowed
                                    .iter()
                                    .map(|s| format!("{:?}", s))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Link column validation
    for link_col_name in &schema.link_columns {
        let col_idx = table
            .headers
            .iter()
            .position(|h| h.trim() == link_col_name.as_str());
        if let Some(idx) = col_idx {
            for (ri, row) in table.body_rows.iter().enumerate() {
                if let Some(cell) = row.get(idx) {
                    let content = cell.trim();
                    if !content.is_empty() && !has_markdown_link(content) {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            table.line + 2 + ri, // body row line number
                            idx + 1,
                            "md_table_missing_link",
                            format!(
                                "column {:?} cell {:?} should contain a markdown link [text](url)\n  \
                                 add a link or set link_auto_fix to auto-generate it",
                                link_col_name, content
                            ),
                        ));
                    }
                }
            }
        }
    }

    // verify_link_targets: check that link paths exist on disk
    if schema.verify_link_targets {
        for link_col_name in &schema.link_columns {
            let col_idx = table
                .headers
                .iter()
                .position(|h| h.trim() == link_col_name.as_str());
            if let Some(idx) = col_idx {
                for (ri, row) in table.body_rows.iter().enumerate() {
                    if let Some(cell) = row.get(idx) {
                        // Extract all (url) from [text](url) in the cell
                        let mut rest = cell.trim();
                        while let Some(open) = rest.find("](") {
                            rest = &rest[open + 2..];
                            if let Some(close) = rest.find(')') {
                                let url = &rest[..close];
                                // Only check relative file paths (not http:// or #anchors)
                                if !url.starts_with("http") && !url.starts_with('#') {
                                    let target = path.parent().unwrap_or(path).join(url);
                                    if !target.exists() {
                                        diags.push(Diagnostic::warning(
                                            path.to_path_buf(),
                                            table.line + 2 + ri,
                                            idx + 1,
                                            "md_broken_link",
                                            format!(
                                                "link target {:?} does not exist (column {:?})",
                                                url, link_col_name
                                            ),
                                        ));
                                    }
                                }
                                rest = &rest[close + 1..];
                            } else {
                                break;
                            }
                        }
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
                if run >= 3 {
                    in_block = true;
                    fence_char = c;
                    fence_len = run;
                }
            }
        } else {
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len {
                    in_block = false;
                    continue;
                }
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
    use std::collections::HashMap;

    fn default_check() -> MarkdownTableCheck {
        MarkdownTableCheck {
            config: MarkdownTableConfig::default(),
        }
    }

    // ─── Structural ───

    #[test]
    fn perfect_table_zero_errors() {
        let content = "# Guide\n\n| Axis | Value |\n|------|-------|\n| Binding | Late |\n| Typing | Static |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        let errs: Vec<_> = diags
            .iter()
            .filter(|d| matches!(d.severity, crate::diagnostic::Severity::Error))
            .collect();
        assert!(
            errs.is_empty(),
            "perfect table must have zero errors: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn col_mismatch_in_body_detected() {
        // Body row has 3 cols, header has 2
        let content = "| A | B |\n|---|---|\n| x | y | extra |\n";
        let tables = parse_tables(&content.lines().collect::<Vec<_>>(), &[false, false, false]);
        assert_eq!(tables.len(), 1);
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "md_table_col_mismatch"),
            "body col mismatch must be detected"
        );
    }

    #[test]
    fn separator_too_short_detected() {
        // Only 2 dashes — need 3
        let content = "| A | B |\n|--|--|\n| x | y |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "md_table_separator_invalid"),
            "short separator must be detected"
        );
    }

    #[test]
    fn blank_corner_header_comparison_matrix_is_allowed() {
        let content =
            "| | PPO | SAC |\n|--|-----|-----|\n| On/off-policy | On-policy | Off-policy |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().all(|d| {
                d.code != "md_table_empty_header" && d.code != "md_table_separator_invalid"
            }),
            "blank top-left corner labels body rows in comparison matrices: {:?}",
            diags
        );
    }

    #[test]
    fn missing_separator_not_detected_as_table() {
        // Two pipe rows but no separator — NOT a GFM table
        let content = "| A | B |\n| x | y |\n";
        let lines: Vec<&str> = content.lines().collect();
        let mask = vec![false; lines.len()];
        let tables = parse_tables(&lines, &mask);
        assert!(
            tables.is_empty(),
            "two pipe rows without separator must not be detected as a table"
        );
    }

    #[test]
    fn table_inside_code_block_ignored() {
        let content = "```\n| A | B |\n|---|---|\n| x | y |\n```\n";
        let lines: Vec<&str> = content.lines().collect();
        let mask = code_block_mask(&lines);
        let tables = parse_tables(&lines, &mask);
        assert!(
            tables.is_empty(),
            "table inside code block must not be detected"
        );
    }

    #[test]
    fn alignment_colons_accepted() {
        // Left, center, right aligned separators
        let content = "| A | B | C |\n|:---|:---:|---:|\n| a | b | c |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            !diags.iter().any(|d| d.code == "md_table_separator_invalid"),
            "alignment colons must be valid"
        );
    }

    #[test]
    fn cell_padding_missing_detected() {
        let content = "| A | B |\n|---|---|\n|no-space  |no-space  |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "md_table_cell_padding"),
            "missing cell padding must be warned"
        );
    }

    #[test]
    fn padding_skips_over_split_math_rows_ignored_by_structure() {
        let content =
            "| Task | Tool |\n|------|------|\n| Count orbits | Burnside: (1/|G|)Σ|Fix(g)| |\n";
        let check = MarkdownTableCheck {
            config: MarkdownTableConfig {
                ignore_extra_body_cols: true,
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_table_cell_padding"),
            "ignored extra body columns should not still emit padding warnings: {:?}",
            diags
        );
    }

    #[test]
    fn padding_allows_full_cells_with_no_room() {
        let content = "| Short | Long |\n|-------|------|\n|tight|ok    |\n|bad  |no pad |\n";
        let diags = default_check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().all(|d| {
                !(d.code == "md_table_cell_padding" && d.message.contains("\"tight\""))
            }),
            "full cells cannot add padding without widening table: {:?}",
            diags
        );
        assert!(
            diags
                .iter()
                .any(|d| { d.code == "md_table_cell_padding" && d.message.contains("\"bad\"") }),
            "cells with spare width should still warn: {:?}",
            diags
        );
    }

    #[test]
    fn parse_row_handles_escaped_pipe_and_code_span() {
        let row = r"| Type | `A|B` or A\|B | Notes |";
        let cells = parse_row(row);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[1].trim(), r"`A|B` or A\|B");
    }

    #[test]
    fn parse_row_handles_sql_concat_as_content_when_escaped_or_in_code() {
        let row = r"| SQL | `first || last` | concat operator |";
        let cells = parse_row(row);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[1].trim(), "`first || last`");
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
        assert!(
            diags.iter().any(|d| d.code == "md_missing_table"),
            "must warn when required table count not met"
        );
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
        assert!(
            !diags.iter().any(|d| d.code == "md_missing_table"),
            "must not warn when required table is present"
        );
    }

    // ─── Schema validation ───

    #[test]
    fn schema_required_column_detected() {
        let content =
            "## Type System Snapshot\n\n| Wrong | Value |\n|-------|-------|\n| Binding | Late |\n";
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
        assert!(
            diags.iter().any(|d| d.code == "md_table_schema"),
            "missing required column 'Axis' must be flagged"
        );
    }

    #[test]
    fn schema_required_row_key_detected() {
        let content =
            "## Type System Snapshot\n\n| Axis | Value |\n|------|-------|\n| Binding | Late |\n";
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
        assert!(
            diags.iter().any(|d| d.message.contains("Typing")),
            "missing row key 'Typing' must be flagged"
        );
        // "Binding" appears in context listing of the "Typing" error — check specifically
        // that there's no diagnostic saying Binding itself is the MISSING key
        assert!(
            !diags.iter().any(|d| d.message.contains("key \"Binding\"")),
            "present row key 'Binding' must not be the missing key"
        );
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
        assert!(
            diags.iter().any(|d| d.code == "md_missing_table"),
            "must flag when no table found under required heading"
        );
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == "md_table_schema" && d.message.contains("3")),
            "must flag when body row count below minimum"
        );
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
                        m.insert(
                            "Status".to_string(),
                            vec![
                                "DONE".to_string(),
                                "IN_PROGRESS".to_string(),
                                "TODO".to_string(),
                            ],
                        );
                        m
                    },
                    ..Default::default()
                }],
                ..Default::default()
            },
        };
        let diags = check.check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.message.contains("INVALID_STATUS")),
            "invalid column value must be flagged"
        );
        // DONE appears in the "not in allowed set" listing — check specifically
        // that no diagnostic says DONE is the invalid value itself
        assert!(
            !diags.iter().any(|d| d.message.contains("value \"DONE\"")),
            "valid value DONE must not itself be flagged as invalid"
        );
    }
}
