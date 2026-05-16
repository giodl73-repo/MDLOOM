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
    fn name(&self) -> &'static str {
        "ascii_flow"
    }

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
    let mut active_border_cols: Option<Vec<usize>> = None;

    for (i, line) in lines.iter().enumerate() {
        let abs_line = i + line_offset;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            active_border_cols = None;
            continue;
        }
        if let Some(cols) = border_junction_columns(trimmed) {
            active_border_cols = Some(cols);
            continue;
        }

        // Only check lines that look like box content rows (start/end with | or │)
        if !is_content_line(trimmed) {
            continue;
        }
        let Some(border_cols) = active_border_cols.as_deref() else {
            continue;
        };

        let delimiters = vertical_delimiters(trimmed, border_cols);
        let cells = split_cells(trimmed, &delimiters);
        if is_separator_row(&cells) {
            continue;
        }
        for (cell_idx, (cell, start_col)) in cells.iter().enumerate() {
            if cell.trim().is_empty() {
                continue;
            }
            if !has_room_for_padding(cell, min_pad) {
                continue;
            }

            // Measure leading and trailing whitespace
            let leading = leading_whitespace_count(cell);
            let trailing = trailing_whitespace_count(cell);

            if leading < min_pad {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    abs_line,
                    start_col + 1,
                    "ascii_cell_padding",
                    format!(
                        "cell {} missing left padding: {:?} (found {} space{}, need {})",
                        cell_idx + 1,
                        cell.trim(),
                        leading,
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
                        cell_idx + 1,
                        cell.trim(),
                        trailing,
                        if trailing == 1 { "" } else { "s" },
                        min_pad
                    ),
                ));
            }
        }
    }

    diags
}

fn is_separator_row(cells: &[(&str, usize)]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .filter(|(cell, _)| !cell.trim().is_empty())
            .all(|(cell, _)| {
                cell.chars()
                    .all(|c| c.is_whitespace() || is_border_fill(c) || is_vertical(c))
            })
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

fn char_advance(c: char, col_0based: usize) -> usize {
    match c {
        '\t' => {
            let next_stop = ((col_0based / 4) + 1) * 4;
            (next_stop - col_0based).max(1)
        }
        _ => c.width().unwrap_or(0),
    }
}

fn visual_width(s: &str) -> usize {
    let mut col = 0usize;
    for c in s.chars() {
        col += char_advance(c, col);
    }
    col
}

fn is_border_fill(c: char) -> bool {
    matches!(c, '-' | '─' | '=' | '━')
}

fn is_border_junction(c: char) -> bool {
    matches!(
        c,
        '+' | '┌'
            | '┐'
            | '└'
            | '┘'
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | '┼'
            | '╔'
            | '╗'
            | '╚'
            | '╝'
            | '╠'
            | '╣'
            | '╦'
            | '╩'
            | '╬'
            | '╭'
            | '╮'
            | '╯'
            | '╰'
    )
}

fn is_vertical(c: char) -> bool {
    matches!(c, '|' | '│' | '║' | '╎' | '┆' | '┊')
}

fn border_junction_columns(line: &str) -> Option<Vec<usize>> {
    let first = line.chars().next()?;
    if !is_border_junction(first) {
        return None;
    }

    let mut junction_count = 0usize;
    let mut fill_count = 0usize;
    let mut other_count = 0usize;
    let mut cols = Vec::new();
    let mut col_0 = 0usize;

    for c in line.chars() {
        if is_border_junction(c) {
            junction_count += 1;
            cols.push(col_0 + 1);
        } else if is_border_fill(c) {
            fill_count += 1;
        } else if c != ' ' {
            other_count += 1;
        }
        col_0 += char_advance(c, col_0);
    }

    if junction_count >= 2 && (fill_count + junction_count) > other_count {
        Some(cols)
    } else {
        None
    }
}

/// Returns true if this line is a box content row (starts/ends with | or │).
/// Requires at least 3 chars (delimiter + some content + delimiter) to avoid
/// false-positives on a lone │ character.
fn is_content_line(s: &str) -> bool {
    // Use char count (fast exit on short strings)
    let mut chars = s.chars();
    let first = chars.next();
    if chars.next().is_none() {
        return false;
    } // single-char line → not a content row
    let first_ok = matches!(first, Some('|') | Some('│'));
    let last_ok = matches!(s.chars().last(), Some('|') | Some('│'));
    first_ok && last_ok
}

fn vertical_delimiters(line: &str, allowed_cols: &[usize]) -> Vec<(usize, usize, char)> {
    let mut positions = Vec::new();
    let mut col_0 = 0usize;
    for (byte_pos, c) in line.char_indices() {
        let col = col_0 + 1;
        if is_vertical(c) && allowed_cols.contains(&col) {
            positions.push((byte_pos, col, c));
        }
        col_0 += char_advance(c, col_0);
    }
    positions
}

/// Split a content line into cells at known box delimiter columns.
///
/// Interior `|` characters in prose or math notation, such as `|G|`, are not
/// cell delimiters unless they line up with a junction in the active border.
fn split_cells<'a>(line: &'a str, delimiters: &[(usize, usize, char)]) -> Vec<(&'a str, usize)> {
    let mut cells = Vec::new();
    if delimiters.len() < 2 {
        return cells;
    }

    for window in delimiters.windows(2) {
        let (left_byte, left_col, left_char) = window[0];
        let (right_byte, _, _) = window[1];
        let start = left_byte + left_char.len_utf8();
        if start <= right_byte && line.is_char_boundary(start) && line.is_char_boundary(right_byte)
        {
            cells.push((&line[start..right_byte], left_col));
        }
    }
    cells
}

/// Check arrow alignment: arrows on consecutive lines should be vertically aligned
/// when they're meant to connect (│ connectors) or horizontally clean (── lines).
fn check_arrow_alignment(path: &Path, lines: &[&str], line_offset: usize) -> Vec<Diagnostic> {
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
            if c == '→' && chars[..i].contains(&'←') {
                continue;
            }
            // Look back for the arrow body — should be all fill chars with no unexpected gaps
            let mut j = i;
            let mut gap_found = false;
            let mut gap_col = 0usize;
            let mut body_len = 0usize;
            let mut gap_count = 0usize;
            let mut consecutive_spaces = 0usize;
            while j > 0 {
                j -= 1;
                let prev = chars[j];
                if arrow_fills.contains(&prev) {
                    if prev == ' ' {
                        consecutive_spaces += 1;
                        if consecutive_spaces >= 2 {
                            break;
                        }
                    } else {
                        consecutive_spaces = 0;
                    }
                    if prev == '─' || prev == '-' || prev == '═' {
                        body_len += 1;
                    }
                    if prev == ' ' && j > 0 && chars[j - 1] == '─' {
                        gap_found = true;
                        gap_count += 1;
                        gap_col = j + 1; // 1-based
                    }
                } else {
                    break;
                }
            }
            if gap_found && c != '>' && body_len >= 3 && gap_count * 2 <= body_len {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    abs_line,
                    gap_col,
                    "ascii_arrow_gap",
                    format!(
                        "gap in horizontal arrow at column {} (space inside arrow body)",
                        gap_col
                    ),
                ));
            }
        }
    }

    diags
}

/// Check that vertical connectors (│) on consecutive lines stay in the same column.
fn check_vertical_connectors(path: &Path, lines: &[&str], line_offset: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Collect │ positions per line (visual column, 1-based)
    let connector_positions: Vec<Vec<usize>> = lines
        .iter()
        .map(|line| {
            let mut positions = Vec::new();
            if !is_connector_only_line(line) {
                return positions;
            }
            let mut col = 1usize;
            for c in line.chars() {
                if c == '│' || c == '|' {
                    positions.push(col);
                }
                col += char_advance(c, col - 1);
            }
            positions
        })
        .collect();

    // Find consecutive pairs of lines both containing │ and check alignment
    for i in 1..lines.len() {
        let prev = &connector_positions[i - 1];
        let curr = &connector_positions[i];

        if prev.is_empty() || curr.is_empty() {
            continue;
        }

        // For each connector in the current line, check it aligns with one in previous
        for &curr_col in curr {
            let aligned = prev.contains(&curr_col);
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

fn is_connector_only_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && !is_content_line(trimmed)
        && trimmed.chars().any(|c| c == '│' || c == '|')
        && trimmed.chars().all(|c| {
            c.is_whitespace()
                || matches!(
                    c,
                    '│' | '|' | '║' | '╎' | '┆' | '┊' | '▼' | '▲' | 'v' | '^' | '↓' | '↑'
                )
        })
}
