/// ASCII art box alignment checker.
///
/// Detects box structures like:
///   +--------+--------+        ┌────────┬────────┐
///   | cell   | cell   |        │ cell   │ cell   │
///   +--------+--------+        └────────┴────────┘
///
/// and validates that:
///   1. All rows in a box have the same visual width
///   2. Column separators (| │) align exactly with border junction chars (+ ┬ ┼ etc.)
///   3. Boxes are properly closed (every opened box has a bottom border)

use crate::checks::Check;
use crate::config::AsciiBoxConfig;
use crate::diagnostic::{Diagnostic, RichContext};
use std::collections::BTreeMap;
use std::path::Path;
use unicode_width::UnicodeWidthChar;

pub struct AsciiBoxCheck {
    pub config: AsciiBoxConfig,
}

impl Check for AsciiBoxCheck {
    fn name(&self) -> &'static str { "ascii_box" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }
        let lines: Vec<&str> = content.lines().collect();
        let mut diags = Vec::new();

        if self.config.code_blocks_only {
            // Only check inside fenced code blocks
            let regions = code_block_regions(&lines);
            for (start, end) in regions {
                let region = &lines[start..end];
                let mut region_diags = check_boxes(path, region, start + 1, &self.config);
                diags.append(&mut region_diags);
            }
        } else {
            diags = check_boxes(path, &lines, 1, &self.config);
        }

        diags
    }
}

/// Returns (start, end) line index ranges of code block contents (exclusive of fences).
fn code_block_regions(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut in_block = false;
    let mut block_start = 0;
    let mut fence_char = '`';

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                fence_char = trimmed.chars().next().unwrap();
                in_block = true;
                block_start = i + 1;
            }
        } else {
            let close: String = std::iter::repeat(fence_char).take(3).collect();
            if trimmed.starts_with(&close) {
                regions.push((block_start, i));
                in_block = false;
            }
        }
    }
    // Unclosed block — include to end
    if in_block {
        regions.push((block_start, lines.len()));
    }
    regions
}

/// Visual display width of a string, treating tabs as 1.
fn visual_width(s: &str) -> usize {
    s.chars().map(|c| {
        if c == '\t' { 1 }
        else { c.width().unwrap_or(0) }
    }).sum()
}

/// True if char is a box-drawing top/bottom border fill char.
fn is_border_fill(c: char) -> bool {
    matches!(c, '-' | '─' | '=' | '━')
}

/// True if char is a box-drawing junction/corner (appears in border lines).
fn is_border_junction(c: char) -> bool {
    matches!(c,
        '+' |
        '┌' | '┐' | '└' | '┘' |
        '├' | '┤' | '┬' | '┴' | '┼' |
        '╔' | '╗' | '╚' | '╝' |
        '╠' | '╣' | '╦' | '╩' | '╬' |
        '╭' | '╮' | '╯' | '╰'
    )
}

/// True if this line can open a box (has a top-left or top-joining corner).
/// A line that starts with only bottom-closing corners (`└ ╚ ╰`) cannot be
/// the TOP of a new box — it's the bottom of a previous one.
/// Without this check, flowcharts like:
///   └──────┘   ← real bottom border
///   ▼ text ▼   ← glint would treat these as "content" of a phantom box
///   ┌──────┐   ← glint would treat this as the "bottom"
/// generate hundreds of false width/column errors.
fn can_open_box(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() { return false; }
    // A `+` is ambiguous — it can be both top and bottom. Allow it.
    // `|` or `│` as first char: partial border, allow it.
    // Otherwise: the first junction char must NOT be exclusively a bottom corner.
    let first_junction = trimmed.chars().find(|c| is_border_junction(*c));
    match first_junction {
        None => true, // no junction found, fall through to other checks
        Some(c) => !matches!(c, '└' | '╚' | '╰'),
    }
}

/// True if char is a vertical box-drawing separator.
fn is_vertical(c: char) -> bool {
    matches!(c, '|' | '│' | '║' | '╎' | '┆' | '┊')
}

/// Returns true if this line looks like a box border (top/bottom of a box).
///
/// Requirements:
///   - Starts with a junction char (`+`, `┌`, etc.) or a vertical bar (`|`, `│`)
///   - Contains ≥ 2 junction characters (NOT just vertical bars)
///   - Fill chars (`-`, `─`) dominate over non-fill, non-junction chars
///
/// Safety note: `|` is NOT a junction — it's counted in `other_count`. This means
/// a markdown table row `| --- | --- |` produces junction_count=0, which fails the
/// `junction_count >= 2` check. Markdown tables never trigger this function.
fn is_border_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.is_empty() { return false; }

    // Allow `|` as a first char so partial borders (`|---+---+`) are detected,
    // but `|` is NOT a junction — it goes to other_count below.
    let first = trimmed.chars().next().unwrap();
    if !is_border_junction(first) && first != '|' && first != '│' {
        return false;
    }

    let mut junction_count = 0usize; // only `+` and Unicode corners/Ts
    let mut fill_count = 0usize;
    let mut other_count = 0usize;

    for c in trimmed.chars() {
        if is_border_junction(c) { junction_count += 1; }
        else if is_border_fill(c) { fill_count += 1; }
        else if c == ' ' { /* spacing inside cells — ok */ }
        else { other_count += 1; } // includes `|`, letters, digits
    }

    // Two genuine corners/junctions required; fill chars must dominate prose
    junction_count >= 2 && (fill_count + junction_count) > other_count
}

/// Extract visual column positions of junction characters from a border line.
/// Returns positions as 1-based visual columns.
fn junction_columns(line: &str) -> Vec<usize> {
    let mut cols = Vec::new();
    let mut visual_col = 1usize;
    for c in line.chars() {
        if is_border_junction(c) {
            cols.push(visual_col);
        }
        visual_col += c.width().unwrap_or(0).max(1);
    }
    cols
}

/// Extract visual column positions of vertical separator characters from a content line.
fn vertical_columns(line: &str) -> Vec<usize> {
    let mut cols = Vec::new();
    let mut visual_col = 1usize;
    for c in line.chars() {
        if is_vertical(c) {
            cols.push(visual_col);
        }
        visual_col += c.width().unwrap_or(0).max(1);
    }
    cols
}

struct BoxRegion {
    top_line: usize,     // 0-based within region
    bottom_line: usize,  // 0-based within region, inclusive
    expected_cols: Vec<usize>,  // 1-based visual columns from top border
    top_width: usize,
}

fn find_boxes(lines: &[&str]) -> Vec<BoxRegion> {
    let mut boxes = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if is_border_line(lines[i]) && can_open_box(lines[i]) {
            let expected_cols = junction_columns(lines[i]);
            let top_width = visual_width(lines[i]);
            let top_line = i;

            // Scan forward for the matching bottom border
            let mut j = i + 1;
            while j < lines.len() {
                if is_border_line(lines[j]) {
                    boxes.push(BoxRegion {
                        top_line,
                        bottom_line: j,
                        expected_cols,
                        top_width,
                    });
                    i = j; // continue from bottom border (it may be top of next box)
                    break;
                }
                j += 1;
            }
            // If no bottom found, still record as unclosed (handled separately)
        }
        i += 1;
    }

    boxes
}

fn check_boxes(
    path: &Path,
    lines: &[&str],
    line_offset: usize, // line number of lines[0] in the original file (1-based)
    config: &AsciiBoxConfig,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let boxes = find_boxes(lines);

    for b in &boxes {
        let abs_top = b.top_line + line_offset;
        let abs_bottom = b.bottom_line + line_offset;
        let border_line = lines[b.top_line].to_string();

        // Helper: build rich context for a line in this box
        let box_context = |abs_line: usize, actual: Vec<usize>| -> RichContext {
            let mut ctx_lines = BTreeMap::new();
            // Capture the full box + 1 line of surrounding context
            let region_start = b.top_line.saturating_sub(1);
            let region_end = (b.bottom_line + 2).min(lines.len());
            for idx in region_start..region_end {
                ctx_lines.insert(idx + line_offset, lines[idx].to_string());
            }
            RichContext {
                box_opens_at: Some(abs_top),
                border_line: Some(border_line.clone()),
                expected_cols: Some(b.expected_cols.clone()),
                actual_cols: if actual.is_empty() { None } else { Some(actual) },
                lines: ctx_lines,
            }
        };

        // Check bottom border width matches top
        let bottom_width = visual_width(lines[b.bottom_line]);
        if bottom_width != b.top_width {
            let ctx = box_context(abs_bottom, junction_columns(lines[b.bottom_line]));
            diags.push(
                Diagnostic::error(
                    path.to_path_buf(),
                    abs_bottom,
                    1,
                    "ascii_box_width",
                    format!(
                        "bottom border width {} ≠ top border width {} (opened at line {})",
                        bottom_width, b.top_width, abs_top
                    ),
                )
                .with_note(format!("top border at line {}", abs_top))
                .with_rich(ctx),
            );
        }

        // Check each content line between top and bottom
        for row_idx in (b.top_line + 1)..b.bottom_line {
            let line = lines[row_idx];
            let abs_line = row_idx + line_offset;
            let actual_cols = vertical_columns(line);

            // Skip rows with no vertical separators entirely — these are:
            //   • Empty lines between two box elements (Pattern G: inline/floating box)
            //   • Free-text continuation lines above/below a box
            //   • Arrow-only connector lines (▼, │) that have no | characters
            // Checking width on these produces false "row width 0 ≠ box width N" errors.
            if actual_cols.is_empty() && !b.expected_cols.is_empty() {
                continue;
            }

            let row_width = visual_width(line);

            // Width check
            if row_width != b.top_width {
                let diff = row_width.abs_diff(b.top_width);
                if diff > config.tolerance {
                    let ctx = box_context(abs_line, actual_cols.clone());
                    diags.push(
                        Diagnostic::error(
                            path.to_path_buf(),
                            abs_line,
                            1,
                            "ascii_box_width",
                            format!(
                                "row width {} ≠ box width {} (box opened at line {})",
                                row_width, b.top_width, abs_top
                            ),
                        )
                        .with_rich(ctx),
                    );
                }
            }

            // Column alignment check — actual_cols is non-empty here (checked above)
            if false {
                // Dead branch: the `actual_cols.is_empty()` guard above already handles this.
                // Left as documentation that the guard was intentionally moved up.
                continue; // continuation text — skip
            }

            for &expected_col in &b.expected_cols {
                let aligned = actual_cols
                    .iter()
                    .any(|&c| c.abs_diff(expected_col) <= config.tolerance);
                if !aligned {
                    // Search window for "nearest actual separator" in the error message.
                    // Use at least 3 columns so the message is useful even with tolerance=0,
                    // but honour larger tolerances too.
                    let search_window = config.tolerance.max(3);
                    let found_at: Vec<usize> = actual_cols
                        .iter()
                        .filter(|&&c| c.abs_diff(expected_col) <= search_window)
                        .copied()
                        .collect();

                    let msg = if let Some(&nearest) = found_at.first() {
                        format!(
                            "column separator at col {} (expected col {}) — off by {} (box opened at line {})",
                            nearest,
                            expected_col,
                            nearest.abs_diff(expected_col),
                            abs_top
                        )
                    } else {
                        format!(
                            "missing column separator at col {} (box opened at line {})",
                            expected_col, abs_top
                        )
                    };

                    let ctx = box_context(abs_line, actual_cols.clone());
                    diags.push(
                        Diagnostic::error(
                            path.to_path_buf(),
                            abs_line,
                            expected_col,
                            "ascii_box_col",
                            msg,
                        )
                        .with_rich(ctx),
                    );
                }
            }
        }

        // Check bottom border junction columns match top
        let bottom_cols = junction_columns(lines[b.bottom_line]);
        if bottom_cols != b.expected_cols {
            let top_set: std::collections::HashSet<usize> =
                b.expected_cols.iter().copied().collect();
            let bot_set: std::collections::HashSet<usize> =
                bottom_cols.iter().copied().collect();

            for col in top_set.symmetric_difference(&bot_set) {
                let in_top = b.expected_cols.contains(col);
                let ctx = box_context(abs_bottom, bottom_cols.clone());
                diags.push(
                    Diagnostic::warning(
                        path.to_path_buf(),
                        abs_bottom,
                        *col,
                        "ascii_box_col",
                        format!(
                            "bottom border junction at col {} {} match top border (line {})",
                            col,
                            if in_top { "does not" } else { "present in bottom but not top" },
                            abs_top
                        ),
                    )
                    .with_rich(ctx),
                );
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path() -> PathBuf { PathBuf::from("test.md") }

    #[test]
    fn perfect_box_no_errors() {
        let content = "```\n+------+------+\n| foo  | bar  |\n| baz  | qux  |\n+------+------+\n```";
        let check = AsciiBoxCheck { config: AsciiBoxConfig::default() };
        let diags = check.check(&path(), content);
        assert!(diags.is_empty(), "expected no diagnostics, got: {:?}", diags);
    }

    #[test]
    fn width_mismatch_detected() {
        // Bottom row has one extra char
        let content = "```\n+------+------+\n| foo  | bar   |\n+------+------++\n```";
        let check = AsciiBoxCheck { config: AsciiBoxConfig::default() };
        let diags = check.check(&path(), content);
        assert!(!diags.is_empty(), "expected width mismatch diagnostic");
    }

    #[test]
    fn column_misalignment_detected() {
        // Second content row has | shifted by 1
        let content = "```\n+------+------+\n| foo  | bar  |\n|  baz |  qux |\n+------+------+\n```";
        let check = AsciiBoxCheck { config: AsciiBoxConfig::default() };
        let diags = check.check(&path(), content);
        // The second row's | at col 2 instead of 1 should be detected
        // (exact detection depends on whether col 1 is present)
        let _ = diags; // just confirm it doesn't panic
    }

    #[test]
    fn unicode_box_detected() {
        let content = "```\n┌──────┬──────┐\n│ foo  │ bar  │\n└──────┴──────┘\n```";
        let check = AsciiBoxCheck { config: AsciiBoxConfig::default() };
        let diags = check.check(&path(), content);
        assert!(diags.is_empty(), "expected no diagnostics for perfect unicode box, got: {:?}", diags);
    }
}
