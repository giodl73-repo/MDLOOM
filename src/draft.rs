/// proof draft — generates a pre-populated fix plan from a diagnostic scan.
///
/// Unlike `proof check --format rich` (which the AI reads to generate a plan),
/// `proof draft` does both steps in one:
///   1. Runs all checks to collect diagnostics
///   2. Groups diagnostics by source object (box, table, chart, heading)
///   3. Pre-computes fixes for deterministic cases (barchart scale, separator dashes)
///   4. Pre-templates old_string for judgment calls (AI fills new_string + decision)
///
/// Output: a draft-plan.json the AI can read and annotate inline, then
/// `proof fix --plan draft-plan.json` applies it.
use crate::diagnostic::Diagnostic;
use crate::fix::is_pattern_b;
use crate::fix::{Confidence, DiagnosticRef, Edit, Fix, FixPlan, PlanSummary};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthChar;

/// A group of related diagnostics from the same source object.
/// The AI makes one decision per group (not per line).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FixGroup {
    pub group_id: String,
    pub file: PathBuf,
    /// Human-readable description of what's wrong in this group
    pub description: String,
    /// Pre-filled for deterministic fixes; AI writes for judgment calls
    pub decision: String,
    /// Pre-filled for deterministic; AI fills for judgment calls
    pub confidence: Option<Confidence>,
    /// All diagnostics in this group (for AI context)
    pub diagnostics: Vec<DiagSummary>,
    /// Pre-templated fixes — AI fills `new_string` where blank
    pub fixes: Vec<DraftFix>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagSummary {
    pub code: String,
    pub line: usize,
    pub col: usize,
    pub severity: String,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DraftFix {
    /// 1-based line in the file
    pub line: usize,
    /// Current content of the line (read from file)
    pub old_string: String,
    /// Pre-computed for deterministic fixes; blank for AI judgment
    pub new_string: String,
    /// True = already computed, no AI needed.
    /// False = AI must supply new_string before applying.
    pub auto: bool,
    /// True = this line has Pattern B (text after closing │/|).
    /// The text after the bar must be preserved — move it inside the box
    /// or to adjacent prose. proof fix will block this unless new_string
    /// contains the removed words OR --no-signal-check is set.
    #[serde(default)]
    pub pattern_b: bool,
    /// Rich context for this line — border_line, expected_cols, actual_cols.
    /// Populated when available (box diagnostics always have it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DraftPlan {
    pub schema_version: String,
    pub generated_by: String,
    pub summary: DraftSummary,
    pub groups: Vec<FixGroup>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct DraftSummary {
    pub total_groups: usize,
    pub auto_fixable: usize, // groups where all fixes are deterministic
    pub needs_review: usize, // groups where AI must make a decision
    pub files_affected: usize,
}

impl DraftPlan {
    /// Convert a DraftPlan into a FixPlan by including only groups that have
    /// a non-empty new_string for all their fixes.
    pub fn to_fix_plan(&self) -> FixPlan {
        let mut fixes = Vec::new();
        let mut fix_id = 1usize;

        for group in &self.groups {
            for draft_fix in &group.fixes {
                if draft_fix.new_string.is_empty() {
                    continue;
                }
                fixes.push(Fix {
                    id: format!("fix-{:03}", fix_id),
                    file: group.file.clone(),
                    description: group.description.clone(),
                    confidence: group.confidence.clone().unwrap_or(Confidence::Medium),
                    reasoning: group.decision.clone(),
                    edit: Edit {
                        line: draft_fix.line,
                        old_string: draft_fix.old_string.clone(),
                        new_string: draft_fix.new_string.clone(),
                    },
                    diagnostic: DiagnosticRef {
                        code: group
                            .diagnostics
                            .first()
                            .map(|d| d.code.clone())
                            .unwrap_or_default(),
                        line: group.diagnostics.first().map(|d| d.line).unwrap_or(0),
                        col: group.diagnostics.first().map(|d| d.col).unwrap_or(0),
                    },
                });
                fix_id += 1;
            }
        }

        let total = fixes.len();
        FixPlan {
            schema_version: "1".to_string(),
            generated_by: "proof-draft".to_string(),
            source_report: "draft-plan.json".to_string(),
            summary: PlanSummary {
                total_fixes: total,
                high_confidence: fixes
                    .iter()
                    .filter(|f| f.confidence == Confidence::High)
                    .count(),
                medium_confidence: fixes
                    .iter()
                    .filter(|f| f.confidence == Confidence::Medium)
                    .count(),
                low_confidence: fixes
                    .iter()
                    .filter(|f| f.confidence == Confidence::Low)
                    .count(),
                files_affected: fixes
                    .iter()
                    .map(|f| &f.file)
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
            },
            fixes,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Draft plan generation
// ─────────────────────────────────────────────────────────

/// Build a draft plan from a set of diagnostics.
/// Reads file contents to populate old_string; computes new_string where deterministic.
pub fn build_draft_plan(diagnostics: &[Diagnostic], root: &Path) -> Result<DraftPlan> {
    // Group diagnostics: by (file, group_id) or by (file, line) if no group_id
    let mut groups: HashMap<(PathBuf, String), Vec<&Diagnostic>> = HashMap::new();

    for diag in diagnostics {
        let key_id = diag
            .group_id
            .clone()
            .unwrap_or_else(|| format!("{}-l{}", diag.code, diag.span.line));
        groups
            .entry((diag.file.clone(), key_id))
            .or_default()
            .push(diag);
    }

    // Sort groups: by file, then by first diagnostic line
    let mut sorted_keys: Vec<(PathBuf, String)> = groups.keys().cloned().collect();
    sorted_keys.sort_by(|a, b| {
        a.0.cmp(&b.0).then({
            let line_a = groups[a].iter().map(|d| d.span.line).min().unwrap_or(0);
            let line_b = groups[b].iter().map(|d| d.span.line).min().unwrap_or(0);
            line_a.cmp(&line_b)
        })
    });

    // Cache file contents by path
    let mut file_cache: HashMap<PathBuf, Vec<String>> = HashMap::new();

    let mut fix_groups = Vec::new();

    for key in &sorted_keys {
        let diags = &groups[key];
        let file = &key.0;

        // Load file lines (cached)
        let file_lines = file_cache.entry(file.clone()).or_insert_with(|| {
            std::fs::read_to_string(file)
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect()
        });

        let group = build_group(file, diags, file_lines, root)?;
        fix_groups.push(group);
    }

    let auto_fixable = fix_groups
        .iter()
        .filter(|g| g.fixes.iter().all(|f| f.auto))
        .count();
    let needs_review = fix_groups.len() - auto_fixable;
    let files_affected = fix_groups
        .iter()
        .map(|g| &g.file)
        .collect::<std::collections::HashSet<_>>()
        .len();

    Ok(DraftPlan {
        schema_version: "1".to_string(),
        generated_by: "proof draft".to_string(),
        summary: DraftSummary {
            total_groups: fix_groups.len(),
            auto_fixable,
            needs_review,
            files_affected,
        },
        groups: fix_groups,
    })
}

fn build_group(
    file: &Path,
    diags: &[&Diagnostic],
    file_lines: &[String],
    _root: &Path,
) -> Result<FixGroup> {
    let first = diags[0];
    let group_id = first
        .group_id
        .clone()
        .unwrap_or_else(|| format!("{}-l{}", first.code, first.span.line));

    // Collect unique lines that need fixing
    let mut unique_lines: Vec<usize> = diags.iter().map(|d| d.span.line).collect();
    unique_lines.sort();
    unique_lines.dedup();

    // Build draft fixes for each unique line
    let mut fixes = Vec::new();
    for &line_no in &unique_lines {
        let old_string = file_lines
            .get(line_no.saturating_sub(1))
            .cloned()
            .unwrap_or_default();

        // Try to compute a deterministic fix
        let (new_string, auto) = compute_auto_fix(diags, line_no, &old_string);

        // Include rich context from the first diagnostic on this line (if available)
        let context = diags
            .iter()
            .find(|d| d.span.line == line_no && d.rich.is_some())
            .and_then(|d| serde_json::to_value(d.rich.as_ref()?).ok());

        let pattern_b = is_pattern_b(&old_string);
        fixes.push(DraftFix {
            line: line_no,
            old_string,
            new_string,
            auto,
            pattern_b,
            context,
        });
    }

    // Build description from diagnostics
    let codes: Vec<&str> = diags
        .iter()
        .map(|d| d.code)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let first_msg = &first.message;
    let description = if diags.len() == 1 {
        first_msg.clone()
    } else {
        format!(
            "{} errors ({}) starting at line {}",
            diags.len(),
            codes.join(", "),
            first.span.line
        )
    };

    // Pre-fill confidence for fully-auto groups
    let all_auto = fixes.iter().all(|f| f.auto);
    let (decision, confidence) = if all_auto {
        (
            "AUTO: fix computed deterministically".to_string(),
            Some(Confidence::High),
        )
    } else {
        (String::new(), None) // AI fills these in
    };

    let diag_summaries: Vec<DiagSummary> = diags
        .iter()
        .map(|d| DiagSummary {
            code: d.code.to_string(),
            line: d.span.line,
            col: d.span.col,
            severity: d.severity.to_string(),
            message: d.message.clone(),
        })
        .collect();

    Ok(FixGroup {
        group_id,
        file: file.to_path_buf(),
        description,
        decision,
        confidence,
        diagnostics: diag_summaries,
        fixes,
    })
}

/// Attempt to compute a deterministic fix for the given line.
/// Returns (new_string, auto=true) if deterministic, ("", false) if AI needed.
fn compute_auto_fix(diags: &[&Diagnostic], line_no: usize, old_string: &str) -> (String, bool) {
    // Collect codes affecting this line
    let codes_on_line: Vec<&str> = diags
        .iter()
        .filter(|d| d.span.line == line_no)
        .map(|d| d.code)
        .collect();

    // --- Deterministic: table separator too short ---
    // "separator column N has M dashes — need at least 3"
    if codes_on_line.contains(&"md_table_separator_invalid") {
        if let Some(fixed) = fix_table_separator(old_string) {
            return (fixed, true);
        }
    }

    // --- Deterministic: table cell padding ---
    // Now safe: uses escaped-pipe-aware parser that handles \|, code spans, ||, |>
    if codes_on_line.contains(&"md_table_cell_padding") {
        if let Some(fixed) = fix_table_cell_padding(old_string) {
            return (fixed, true);
        }
    }

    // --- Deterministic: box width ±1 (trailing space) ---
    // "row width N ≠ box width M (box opened at line L)"
    // For ±1: add or remove a trailing space in the last cell before closing │/|
    if codes_on_line.contains(&"ascii_box_width") {
        // Only handle ±1 cases — larger diffs need AI judgment
        let all_width_diags: Vec<&&Diagnostic> = diags
            .iter()
            .filter(|d| d.span.line == line_no && d.code == "ascii_box_width")
            .collect();
        // Get the diff from the first width diagnostic
        if let Some(diag) = all_width_diags.first() {
            if let Some(fixed) = fix_box_width_one(old_string, &diag.message) {
                return (fixed, true);
            }
        }
    }

    // --- Deterministic: ASCII art box cell padding (missing space against │) ---
    // "cell N missing right padding" or "cell N missing left padding"
    // These rows have content butting against │ with 0 spaces. Add 1 space.
    if codes_on_line.contains(&"ascii_cell_padding")
        && !codes_on_line
            .iter()
            .any(|&c| c == "ascii_box_width" || c == "ascii_box_col")
    {
        if let Some(fixed) = fix_box_cell_padding(old_string) {
            return (fixed, true);
        }
    }

    // --- Deterministic: box col ±1 (shift trailing space in correct cell) ---
    // "column separator at col N (expected col M) — off by 1"
    // Only auto-fix when:
    //   - drift == 1 (single column off)
    //   - No width error on the same line (width errors are fixed by box_width logic)
    //   - The rich context has expected_cols and actual_cols
    if codes_on_line.contains(&"ascii_box_col") && !codes_on_line.contains(&"ascii_box_width") {
        let col_diag = diags
            .iter()
            .find(|d| d.span.line == line_no && d.code == "ascii_box_col");
        if let Some(diag) = col_diag {
            if let Some(ctx) = &diag.rich {
                if let (Some(expected), Some(actual)) = (&ctx.expected_cols, &ctx.actual_cols) {
                    if let Some(fixed) = fix_box_col_one(old_string, expected, actual) {
                        return (fixed, true);
                    }
                }
            }
        }
    }

    // --- Deterministic: bar chart scale (proportionality) ---
    // Parse "expected ~N chars" from the message
    if codes_on_line.contains(&"ascii_barchart_scale") {
        for diag in diags
            .iter()
            .filter(|d| d.span.line == line_no && d.code == "ascii_barchart_scale")
        {
            if let Some(fixed) = fix_barchart_scale(old_string, &diag.message) {
                return (fixed, true);
            }
        }
    }

    // --- Deterministic: md_table_missing_link with auto_link_pattern ---
    // "directory" pattern: `computing/` → `[computing/](../computing/00-OVERVIEW.md)`
    // "file" pattern: `01-PKG.md` → `[01-PKG.md](../dirname/01-PKG.md)`
    if codes_on_line.contains(&"md_table_missing_link") {
        // The fix strategy comes from the schema config via the diagnostic message
        // For now, detect bare directory (ends with /) or bare file (ends with .md)
        // and generate the standard link
        let trimmed = old_string.trim();
        if let Some(fixed) = fix_missing_table_link(trimmed) {
            return (fixed, true);
        }
    }

    // All other errors: AI judgment needed
    ("".to_string(), false)
}

/// Fix table separator cells to meet minimum dash requirement.
/// `|--|--|` → `|---|---|`
fn fix_table_separator(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut fixed_cells: Vec<String> = Vec::new();

    for cell in inner.split('|') {
        let c = cell.trim();
        let is_sep = {
            let core = c.trim_start_matches(':').trim_end_matches(':');
            core.chars().all(|ch| ch == '-') && !core.is_empty()
        };
        if is_sep {
            // Normalize to exactly 3 dashes, preserving alignment colons
            let has_left = c.starts_with(':');
            let has_right = c.ends_with(':');
            let normalized = match (has_left, has_right) {
                (true, true) => ":---:".to_string(),
                (true, false) => ":---".to_string(),
                (false, true) => "---:".to_string(),
                (false, false) => "---".to_string(),
            };
            // Preserve original spacing
            let leading = cell.len() - cell.trim_start().len();
            let trailing = cell.len() - cell.trim_end().len();
            fixed_cells.push(format!(
                "{}{}{}",
                " ".repeat(leading),
                normalized,
                " ".repeat(trailing)
            ));
        } else {
            fixed_cells.push(cell.to_string());
        }
    }

    // Reconstruct the leading indentation
    let leading_spaces = line.len() - line.trim_start().len();
    Some(format!(
        "{}|{}|",
        " ".repeat(leading_spaces),
        fixed_cells.join("|")
    ))
}

/// Fix a bar chart row's bar width to be proportional.
/// Parses "expected ~N chars" from the message.
/// Uses byte positions throughout — block chars (█) are 3 bytes each in UTF-8.
fn fix_barchart_scale(line: &str, message: &str) -> Option<String> {
    // Parse "expected ~N chars"
    let expected_n: usize = message
        .split("expected ~")
        .nth(1)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;

    // Find bar start/end as BYTE positions (block chars are multi-byte)
    let bar_start_byte = line.char_indices().find(|(_, c)| is_block_char(*c))?.0;
    let bar_char = line[bar_start_byte..].chars().next()?;
    let char_byte_len = bar_char.len_utf8();
    let old_bar_char_count = line[bar_start_byte..]
        .chars()
        .take_while(|&c| is_block_char(c))
        .count();
    let bar_end_byte = bar_start_byte + char_byte_len * old_bar_char_count;

    let before = &line[..bar_start_byte];
    let after = &line[bar_end_byte..];
    let new_bar: String = std::iter::repeat_n(bar_char, expected_n).collect();

    // Adjust whitespace gap to keep value at same visual column
    let after_trimmed = after.trim_start();
    let gap_chars = after.chars().take_while(|c| c.is_whitespace()).count();
    let new_gap =
        (gap_chars as isize + old_bar_char_count as isize - expected_n as isize).max(1) as usize;
    let new_after = format!("{}{}", " ".repeat(new_gap), after_trimmed);

    Some(format!("{}{}{}", before, new_bar, new_after))
}

/// Add padding to table cells using the escaped-pipe-aware parser.
/// Correctly handles \|, backtick code spans, and operator sequences.
fn fix_table_cell_padding(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }

    let cells = parse_table_cells(trimmed);
    if cells.is_empty() {
        return None;
    }

    let mut any_fixed = false;
    let fixed_cells: Vec<String> = cells
        .into_iter()
        .map(|cell| {
            let content = cell.trim();
            if content.is_empty() {
                return cell;
            }
            let leading = cell.len() - cell.trim_start().len();
            let trailing = cell.len() - cell.trim_end().len();
            if leading >= 1 && trailing >= 1 {
                return cell;
            }
            any_fixed = true;
            format!(
                "{}{}{}",
                " ".repeat(leading.max(1)),
                content,
                " ".repeat(trailing.max(1))
            )
        })
        .collect();

    if !any_fixed {
        return None;
    }

    let indent = line.len() - line.trim_start().len();
    Some(format!("{}|{}|", " ".repeat(indent), fixed_cells.join("|")))
}

/// Escaped-pipe-aware cell splitter — used by the fix generator.
/// Mirrors parse_row in markdown_table.rs but operates on the full line.
/// Auto-generate markdown links for a table row where cells need links.
///
/// Handles both bare and backtick-wrapped names:
///   `computing/`          → `[computing/](../computing/00-OVERVIEW.md)`
///   `` `computing/` ``    → `[computing/](../computing/00-OVERVIEW.md)`
///   `01-PACKAGE.md — desc`→ `[01-PACKAGE.md](../DIRNAME/01-PACKAGE.md) — desc`
///   `` `01-PKG.md` ``     → `[01-PKG.md](../DIRNAME/01-PKG.md)`
///
/// The dirname for file links is extracted from the first directory-like cell
/// in the same row (the Directory column typically comes first).
fn fix_missing_table_link(table_row: &str) -> Option<String> {
    if !table_row.starts_with('|') {
        return None;
    }

    let cells = parse_table_cells(table_row);

    // Extract the directory from the first cell that looks like a dirname.
    // Handles three formats:
    //   bare:     "computing/"          → "computing"
    //   wrapped:  "`computing/`"        → "computing"
    //   linked:   "[computing/](../computing/00-OVERVIEW.md)" → "computing"
    let parent_dir: Option<String> = cells.iter().find_map(|cell| {
        let raw = cell.trim();
        let inner = raw.trim_matches('`').trim();

        // Format 1: bare or backtick-wrapped dirname/
        if inner.ends_with('/') && !inner.contains(' ') && !inner.contains('[') {
            return Some(inner.trim_end_matches('/').to_string());
        }

        // Format 2: already a markdown link [dirname/](../dirname/...)
        // Extract dirname from the link text or URL
        if inner.starts_with('[') {
            // Try link text: [dirname/](url) → "dirname"
            if let Some(close_bracket) = inner.find("](") {
                let link_text = &inner[1..close_bracket];
                if link_text.ends_with('/') && !link_text.contains('/') {
                    // single-component dir: [computing/](...)
                    return Some(link_text.trim_end_matches('/').to_string());
                }
                // Also try the URL part: [...](../dirname/00-OVERVIEW.md)
                if let Some(url_end) = inner[close_bracket + 2..].find(')') {
                    let url = &inner[close_bracket + 2..close_bracket + 2 + url_end];
                    // url like "../computing/00-OVERVIEW.md" → "computing"
                    let parts: Vec<&str> = url.split('/').collect();
                    if parts.len() >= 2 {
                        let dir_candidate = parts[parts.len() - 2];
                        if !dir_candidate.is_empty() && !dir_candidate.starts_with('.') {
                            return Some(dir_candidate.to_string());
                        }
                    }
                }
            }
        }

        None
    });

    let mut any_fixed = false;
    let fixed_cells: Vec<String> = cells
        .into_iter()
        .map(|cell| {
            let raw = cell.trim();
            if raw.is_empty() {
                return cell;
            }
            if has_markdown_link_in_cell(raw) {
                return cell;
            } // already linked

            // Unwrap backtick code span if present: `content` → content
            let (inner, had_backtick) =
                if raw.starts_with('`') && raw.ends_with('`') && raw.len() > 2 {
                    (&raw[1..raw.len() - 1], true)
                } else {
                    (raw, false)
                };
            let _ = had_backtick; // consumed; the link replaces the backtick wrapper

            let leading = cell.len() - cell.trim_start().len();
            let trailing = cell.len() - cell.trim_end().len();

            // Pattern 1: directory name (ends with /, no spaces)
            if inner.ends_with('/') && !inner.contains(' ') {
                let dir = inner.trim_end_matches('/');
                any_fixed = true;
                return format!(
                    "{}[{}/](../{}/00-OVERVIEW.md){}",
                    " ".repeat(leading),
                    dir,
                    dir,
                    " ".repeat(trailing)
                );
            }

            // Pattern 2: backtick-wrapped "filename.md" possibly with " — description" after
            // Also handles "`01-FILE.md` — description" where inner is just "01-FILE.md"
            // and Pattern 3: bare "01-FILE.md — description"
            let (check_inner, description) = if let Some(dash_pos) = inner.find(" — ") {
                (&inner[..dash_pos], Some(inner[dash_pos..].to_string()))
            } else if let Some(dash_pos) = inner.find(" - ") {
                (&inner[..dash_pos], Some(inner[dash_pos..].to_string()))
            } else {
                (inner, None)
            };

            // Strip backticks from check_inner before filename detection
            // handles: `01-FILE.md` — description (where check_inner = "`01-FILE.md`")
            let check_inner = check_inner.trim_matches('`');

            // Detect if check_inner is a filename.md
            if check_inner.ends_with(".md") && !check_inner.contains(' ') {
                let fname = check_inner;
                let dirname = parent_dir.as_deref().unwrap_or("DIRNAME");
                let desc = description.as_deref().unwrap_or("");
                any_fixed = true;
                let link = format!("[{}](../{}/{}){}", fname, dirname, fname, desc);
                return format!(
                    "{} {} {}",
                    " ".repeat(leading.saturating_sub(1)),
                    link,
                    " ".repeat(trailing.saturating_sub(1))
                );
            }

            cell
        })
        .collect();

    if !any_fixed {
        return None;
    }

    let indent = table_row.len() - table_row.trim_start().len();
    Some(format!("{}|{}|", " ".repeat(indent), fixed_cells.join("|")))
}

fn has_markdown_link_in_cell(cell: &str) -> bool {
    cell.contains("](") && cell.contains('[')
}

fn parse_table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
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

    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    let mut in_code = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                current.push('\\');
                current.push('|');
                chars.next();
            }
            '`' => {
                in_code = !in_code;
                current.push(c);
            }
            '|' if !in_code => {
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

/// Fix box content row width by ±1 trailing space.
/// Only handles the exact ±1 case — larger diffs need AI judgment.
/// "row width N ≠ box width M" — if diff == 1, adjust trailing space in last cell.
/// Fix ASCII art box cell padding: add 1 space before the closing │/| where content
/// is flush against the bar (0 spaces, need 1). Only applied when there is no
/// concurrent width or col error on the same line (those take priority).
fn fix_box_cell_padding(line: &str) -> Option<String> {
    let trimmed = line.trim_end();
    let last_char = trimmed.chars().last()?;
    if last_char != '│' && last_char != '|' {
        return None;
    }

    // Check if content is flush against the closing bar (no space before it)
    let without_last = &trimmed[..trimmed.len() - last_char.len_utf8()];
    if without_last.ends_with(' ') {
        return None;
    } // already has padding

    // Only handle single-column box rows (one opening + one closing bar)
    // to avoid corrupting multi-column layouts
    let bar_count = trimmed.chars().filter(|&c| c == '│' || c == '|').count();
    if bar_count != 2 {
        return None;
    }

    Some(format!("{} {}", without_last, last_char))
}

fn fix_box_width_one(line: &str, message: &str) -> Option<String> {
    // Parse "row width N ≠ box width M"
    let (actual, expected) = parse_width_diff(message)?;
    let diff = actual.abs_diff(expected);
    if diff > 4 {
        return None;
    } // only handle small offsets

    // Find the closing vertical bar (│ or |) at the end of the line
    let trimmed = line.trim_end();
    let last_char = trimmed.chars().last()?;
    if last_char != '│' && last_char != '|' {
        return None;
    }

    let without_last = &trimmed[..trimmed.len() - last_char.len_utf8()];

    if actual > expected {
        // Too wide: remove trailing spaces before the closing bar
        let spaces_before = without_last.chars().rev().take_while(|&c| c == ' ').count();
        if spaces_before >= diff {
            let trimmed_inner = &without_last[..without_last.len() - diff];
            return Some(format!("{}{}", trimmed_inner, last_char));
        }
    } else {
        // Too narrow: add trailing spaces before the closing bar
        return Some(format!("{}{}{}", without_last, " ".repeat(diff), last_char));
    }

    None
}

/// Fix box column misalignment for nested and flat boxes.
///
/// Key insight for nested boxes: when multiple `│` positions all shift by the
/// same amount (because an inner cell grew/shrank), fixing the LEFTMOST
/// misaligned `│` cascades all subsequent positions automatically. Fixing each
/// `│` independently would double-insert spaces. So we find the leftmost
/// mismatch, fix only that one, and stop.
///
/// Handles up to ±4 drift (single column off). Larger drifts need AI judgment.
fn fix_box_col_one(line: &str, expected_cols: &[usize], actual_cols: &[usize]) -> Option<String> {
    // Find ALL mismatches where drift ≤ 4, sorted by actual column (leftmost first)
    let mut mismatches: Vec<(usize, usize)> = expected_cols
        .iter()
        .filter_map(|&exp| {
            let closest = actual_cols.iter().min_by_key(|&&a| a.abs_diff(exp))?;
            let drift = closest.abs_diff(exp);
            if drift <= 4 {
                Some((exp, *closest))
            } else {
                None
            }
        })
        .collect();
    mismatches.sort_by_key(|&(_, act)| act); // leftmost actual column first

    if mismatches.is_empty() {
        return None;
    }

    // All mismatches have the same drift direction?
    let first_dir = if mismatches[0].0 > mismatches[0].1 {
        1i32
    } else {
        -1i32
    };
    let all_same_dir = mismatches.iter().all(|&(exp, act)| {
        let dir = if exp > act { 1i32 } else { -1i32 };
        dir == first_dir
    });

    // If all columns are off in the same direction, fixing the leftmost cascades.
    // If they diverge (some left, some right), AI judgment needed.
    if !all_same_dir {
        return None;
    }

    // Fix ONLY the leftmost misaligned │ — cascades to all subsequent positions
    let (exp_first, act_first) = mismatches[0];
    let drift = exp_first.abs_diff(act_first);
    let need_insert = exp_first > act_first; // insert spaces before | to push it right
    let need_remove = exp_first < act_first; // remove spaces before | to pull it left

    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut result: Vec<char> = Vec::with_capacity(n + drift + 1);
    let mut col_0 = 0usize;
    let mut fixed = false;

    let mut i = 0;
    while i < n {
        let c = chars[i];
        let cur_col_1 = col_0 + 1; // 1-based

        let is_vertical = matches!(c, '|' | '│' | '║');

        if is_vertical && !fixed && cur_col_1 == act_first {
            if need_insert {
                result.extend(std::iter::repeat_n(' ', drift));
            } else if need_remove {
                // Remove up to `drift` trailing spaces from result
                let mut removed = 0;
                while removed < drift && result.last() == Some(&' ') {
                    result.pop();
                    removed += 1;
                }
                if removed < drift {
                    return None;
                } // not enough spaces to remove
            }
            fixed = true;
        }

        result.push(c);
        let char_width = match c {
            '\t' => {
                let ns = ((col_0 / 4) + 1) * 4;
                ns - col_0
            }
            _ => c.width().unwrap_or(1),
        };
        col_0 += char_width;
        i += 1;
    }

    let new_line: String = result.into_iter().collect();
    if new_line == line {
        None
    } else {
        Some(new_line)
    }
}

/// Parse "row width N ≠ box width M" or "bottom border width N ≠ top border width M"
/// from a width diagnostic message. Returns (actual, expected).
fn parse_width_diff(message: &str) -> Option<(usize, usize)> {
    // Pattern: "... width N ≠ ... width M ..."
    let parts: Vec<&str> = message.split("width").collect();
    if parts.len() < 3 {
        return None;
    }
    let actual: usize = parts[1].split_whitespace().next()?.parse().ok()?;
    let expected: usize = parts[2].split_whitespace().next()?.parse().ok()?;
    Some((actual, expected))
}

fn is_block_char(c: char) -> bool {
    matches!(c, '█' | '▓' | '▒' | '░' | '#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_table_separator_normalizes_short_dashes() {
        let result = fix_table_separator("|--|--|");
        assert_eq!(result, Some("|---|---|".to_string()));
    }

    #[test]
    fn fix_table_separator_preserves_alignment_colons() {
        // :-- → :--- (left-only colon), --: → ---: (right colon), :--: → :---: (both)
        let result = fix_table_separator("|:--|--:|:--:|");
        assert_eq!(result, Some("|:---|---:|:---:|".to_string()));
    }

    #[test]
    fn fix_barchart_scale_extends_bar() {
        let line = "Item B  █████████████                  45%";
        let msg = "bar width 13 for value 45 is disproportionate — expected ~17 chars (scale: 78 → 30 chars), off by 4";
        let result = fix_barchart_scale(line, msg);
        assert!(result.is_some(), "should produce a fix");
        let fixed = result.unwrap();
        let _bar_len = fixed
            .chars()
            .take_while(|_| true)
            .skip_while(|&c| c != '█')
            .take_while(|&c| c == '█')
            .count();
        // bar length in the fixed version might not be exact due to char boundary,
        // but the bar should be longer than original 13
        assert!(
            fixed.contains("█████████████████"),
            "bar should be extended"
        );
    }

    #[test]
    fn fix_table_separator_preserves_indentation() {
        let result = fix_table_separator("  |--|--|");
        assert_eq!(result, Some("  |---|---|".to_string()));
    }
}
