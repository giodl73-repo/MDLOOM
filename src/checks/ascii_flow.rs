/// ASCII art flowchart and cell padding validator.
///
/// Checks:
///   1. Arrow alignment — arrows (──▶ --> → ──>) should form straight lines
///      with no unexpected gaps or diagonal drift
///   2. Cell padding — text inside box cells should have consistent whitespace
///      buffers: "| text |" not "|text |" or "| text|"
///   3. Arrow connector integrity — connecting lines (│ ─) between boxes
///      should not have gaps or abrupt endings

use crate::checks::Check;
use crate::config::AsciiFlowConfig;
use crate::diagnostic::Diagnostic;
use std::path::Path;
use unicode_width::UnicodeWidthChar;

pub struct AsciiFlowCheck {
    pub config: AsciiFlowConfig,
}

impl Check for AsciiFlowCheck {
    fn name(&self) -> &'static str { "ascii_flow" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut diags = Vec::new();

        // Find code blocks to check
        let regions = code_block_regions(&lines);
        for (start, end) in regions {
            let region = &lines[start..end];
            let offset = start + 1; // 1-based line number of first line in region

            if self.config.check_cell_padding {
                let mut padding_diags = check_cell_padding(path, region, offset, &self.config);
                diags.append(&mut padding_diags);
            }

            if self.config.check_arrow_alignment {
                let mut arrow_diags = check_arrow_alignment(path, region, offset);
                diags.append(&mut arrow_diags);
            }
        }

        diags
    }
}

fn code_block_regions(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut in_block = false;
    let mut block_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_block = true;
                block_start = i + 1;
            }
        } else if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            regions.push((block_start, i));
            in_block = false;
        }
    }
    if in_block {
        regions.push((block_start, lines.len()));
    }
    regions
}

/// Check that content inside `| ... |` cells has consistent padding.
fn check_cell_padding(
    path: &Path,
    lines: &[&str],
    line_offset: usize,
    config: &AsciiFlowConfig,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let min_pad = config.min_cell_padding;

    for (i, line) in lines.iter().enumerate() {
        let abs_line = i + line_offset;

        // Only check lines that look like box content rows (start/end with | or │)
        let trimmed = line.trim();
        if !is_content_line(trimmed) {
            continue;
        }

        let cells = split_cells(trimmed);
        for (cell_idx, (cell, start_col)) in cells.iter().enumerate() {
            if cell.trim().is_empty() { continue; }

            // Measure leading and trailing whitespace
            let leading = cell.len() - cell.trim_start().len();
            let trailing = cell.len() - cell.trim_end().len();

            if leading < min_pad {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    abs_line,
                    start_col + 1,
                    "ascii_cell_padding",
                    format!(
                        "cell {} missing left padding: {:?} (found {} space{}, need {})",
                        cell_idx + 1, cell.trim(), leading,
                        if leading == 1 { "" } else { "s" },
                        min_pad
                    ),
                ));
            }
            if trailing < min_pad {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    abs_line,
                    start_col + cell.len(),
                    "ascii_cell_padding",
                    format!(
                        "cell {} missing right padding: {:?} (found {} space{}, need {})",
                        cell_idx + 1, cell.trim(), trailing,
                        if trailing == 1 { "" } else { "s" },
                        min_pad
                    ),
                ));
            }
        }
    }

    diags
}

/// Returns true if this line is a box content row (starts/ends with | or │).
/// Requires at least 3 chars (delimiter + some content + delimiter) to avoid
/// false-positives on a lone │ character.
fn is_content_line(s: &str) -> bool {
    // Use char count (fast exit on short strings)
    let mut chars = s.chars();
    let first = chars.next();
    if chars.next().is_none() { return false; } // single-char line → not a content row
    let first_ok = matches!(first, Some('|') | Some('│'));
    let last_ok = matches!(s.chars().last(), Some('|') | Some('│'));
    first_ok && last_ok
}

/// Split a content line into cells, returning (cell_content, start_byte_offset).
/// The line must start and end with | or │. Excludes the outer delimiters.
fn split_cells(line: &str) -> Vec<(&str, usize)> {
    let mut cells = Vec::new();
    let first_len = line.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    let last_len = line.chars().last().map(|c| c.len_utf8()).unwrap_or(1);
    let inner_end = line.len().saturating_sub(last_len);
    // Guard: if first and last chars overlap (very short string), bail out
    if inner_end <= first_len {
        return cells;
    }
    let inner = &line[first_len..inner_end];
    let base_offset = first_len;

    let mut start = 0usize;
    for (byte_pos, c) in inner.char_indices() {
        if c == '|' || c == '│' {
            cells.push((&inner[start..byte_pos], base_offset + start));
            start = byte_pos + c.len_utf8();
        }
    }
    cells.push((&inner[start..], base_offset + start));
    cells
}

/// Check arrow alignment: arrows on consecutive lines should be vertically aligned
/// when they're meant to connect (│ connectors) or horizontally clean (── lines).
fn check_arrow_alignment(
    path: &Path,
    lines: &[&str],
    line_offset: usize,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Find all lines containing horizontal arrows and check they're unbroken
    for (i, line) in lines.iter().enumerate() {
        let abs_line = i + line_offset;
        diags.extend(check_horizontal_arrow(path, line, abs_line));
    }

    // Find vertical connector columns (│) and check they're uninterrupted
    diags.extend(check_vertical_connectors(path, lines, line_offset));

    diags
}

/// Check a horizontal arrow line for gaps: "──▶" should not have spaces in the middle.
fn check_horizontal_arrow(path: &Path, line: &str, abs_line: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Find runs of arrow/dash chars that include an arrow head
    let arrow_heads = ['▶', '►', '▷', '→', '>'];
    let arrow_fills = ['─', '-', '═', '─', ' '];

    // Scan for positions of arrow heads
    let chars: Vec<char> = line.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if arrow_heads.contains(&c) {
            // Look back for the arrow body — should be all fill chars with no unexpected gaps
            let mut j = i;
            let mut gap_found = false;
            let mut gap_col = 0usize;
            while j > 0 {
                j -= 1;
                let prev = chars[j];
                if arrow_fills.contains(&prev) {
                    if prev == ' ' && j > 0 && (chars[j - 1] == '─' || chars[j - 1] == '-') {
                        gap_found = true;
                        gap_col = j + 1; // 1-based
                    }
                } else {
                    break;
                }
            }
            if gap_found {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    abs_line,
                    gap_col,
                    "ascii_arrow_gap",
                    format!("gap in horizontal arrow at column {} (space inside arrow body)", gap_col),
                ));
            }
        }
    }

    diags
}

/// Check that vertical connectors (│) on consecutive lines stay in the same column.
fn check_vertical_connectors(
    path: &Path,
    lines: &[&str],
    line_offset: usize,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Collect │ positions per line (visual column, 1-based)
    let connector_positions: Vec<Vec<usize>> = lines.iter().map(|line| {
        let mut positions = Vec::new();
        let mut col = 1usize;
        for c in line.chars() {
            if c == '│' || c == '|' {
                if !is_content_line(line.trim()) {
                    positions.push(col);
                }
            }
            col += c.width().unwrap_or(1);
        }
        positions
    }).collect();

    // Find consecutive pairs of lines both containing │ and check alignment
    for i in 1..lines.len() {
        let prev = &connector_positions[i - 1];
        let curr = &connector_positions[i];

        if prev.is_empty() || curr.is_empty() { continue; }

        // For each connector in the current line, check it aligns with one in previous
        for &curr_col in curr {
            let aligned = prev.iter().any(|&p| p == curr_col);
            if !aligned && !prev.is_empty() {
                // Only flag if we're clearly in a connector section (both lines have connectors)
                let closest = prev.iter().min_by_key(|&&p| p.abs_diff(curr_col));
                if let Some(&closest_col) = closest {
                    let drift = curr_col.abs_diff(closest_col);
                    if drift > 0 && drift <= 3 {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            i + line_offset,
                            curr_col,
                            "ascii_connector_drift",
                            format!(
                                "vertical connector │ at col {} drifted {} from col {} above",
                                curr_col, drift, closest_col
                            ),
                        ));
                    }
                }
            }
        }
    }

    diags
}
