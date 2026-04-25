/// glint draft — generates a pre-populated fix plan from a diagnostic scan.
///
/// Unlike `glint check --format rich` (which the AI reads to generate a plan),
/// `glint draft` does both steps in one:
///   1. Runs all checks to collect diagnostics
///   2. Groups diagnostics by source object (box, table, chart, heading)
///   3. Pre-computes fixes for deterministic cases (barchart scale, separator dashes)
///   4. Pre-templates old_string for judgment calls (AI fills new_string + decision)
///
/// Output: a draft-plan.json the AI can read and annotate inline, then
/// `glint fix --plan draft-plan.json` applies it.

use crate::diagnostic::{Diagnostic, Severity};
use crate::fix::{Confidence, DiagnosticRef, Edit, Fix, FixPlan, PlanSummary};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    pub auto_fixable: usize,    // groups where all fixes are deterministic
    pub needs_review: usize,    // groups where AI must make a decision
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
                if draft_fix.new_string.is_empty() { continue; }
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
                        code: group.diagnostics.first()
                            .map(|d| d.code.clone()).unwrap_or_default(),
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
            generated_by: "glint-draft".to_string(),
            source_report: "draft-plan.json".to_string(),
            summary: PlanSummary {
                total_fixes: total,
                high_confidence: fixes.iter().filter(|f| f.confidence == Confidence::High).count(),
                medium_confidence: fixes.iter().filter(|f| f.confidence == Confidence::Medium).count(),
                low_confidence: fixes.iter().filter(|f| f.confidence == Confidence::Low).count(),
                files_affected: fixes.iter().map(|f| &f.file).collect::<std::collections::HashSet<_>>().len(),
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
        let key_id = diag.group_id.clone()
            .unwrap_or_else(|| format!("{}-l{}", diag.code, diag.span.line));
        groups.entry((diag.file.clone(), key_id)).or_default().push(diag);
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

    let auto_fixable = fix_groups.iter().filter(|g| g.fixes.iter().all(|f| f.auto)).count();
    let needs_review = fix_groups.len() - auto_fixable;
    let files_affected = fix_groups.iter().map(|g| &g.file).collect::<std::collections::HashSet<_>>().len();

    Ok(DraftPlan {
        schema_version: "1".to_string(),
        generated_by: "glint draft".to_string(),
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
    let group_id = first.group_id.clone()
        .unwrap_or_else(|| format!("{}-l{}", first.code, first.span.line));

    // Collect unique lines that need fixing
    let mut unique_lines: Vec<usize> = diags.iter().map(|d| d.span.line).collect();
    unique_lines.sort();
    unique_lines.dedup();

    // Build draft fixes for each unique line
    let mut fixes = Vec::new();
    for &line_no in &unique_lines {
        let old_string = file_lines.get(line_no.saturating_sub(1))
            .cloned()
            .unwrap_or_default();

        // Try to compute a deterministic fix
        let (new_string, auto) = compute_auto_fix(diags, line_no, &old_string);

        fixes.push(DraftFix { line: line_no, old_string, new_string, auto });
    }

    // Build description from diagnostics
    let codes: Vec<&str> = diags.iter().map(|d| d.code).collect::<std::collections::HashSet<_>>()
        .into_iter().collect();
    let first_msg = &first.message;
    let description = if diags.len() == 1 {
        first_msg.clone()
    } else {
        format!("{} errors ({}) starting at line {}", diags.len(), codes.join(", "), first.span.line)
    };

    // Pre-fill confidence for fully-auto groups
    let all_auto = fixes.iter().all(|f| f.auto);
    let (decision, confidence) = if all_auto {
        ("AUTO: fix computed deterministically".to_string(), Some(Confidence::High))
    } else {
        (String::new(), None) // AI fills these in
    };

    let diag_summaries: Vec<DiagSummary> = diags.iter().map(|d| DiagSummary {
        code: d.code.to_string(),
        line: d.span.line,
        col: d.span.col,
        severity: d.severity.to_string(),
        message: d.message.clone(),
    }).collect();

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
    let codes_on_line: Vec<&str> = diags.iter()
        .filter(|d| d.span.line == line_no)
        .map(|d| d.code)
        .collect();

    // --- Deterministic: table separator too short ---
    // "separator column N has M dashes — need at least 3"
    if codes_on_line.iter().any(|&c| c == "md_table_separator_invalid") {
        if let Some(fixed) = fix_table_separator(old_string) {
            return (fixed, true);
        }
    }

    // --- Deterministic: bar chart scale (proportionality) ---
    // Parse "expected ~N chars" from the message
    if codes_on_line.iter().any(|&c| c == "ascii_barchart_scale") {
        for diag in diags.iter().filter(|d| d.span.line == line_no && d.code == "ascii_barchart_scale") {
            if let Some(fixed) = fix_barchart_scale(old_string, &diag.message) {
                return (fixed, true);
            }
        }
    }

    // All other errors: AI judgment needed
    ("".to_string(), false)
}

/// Fix table separator cells to meet minimum dash requirement.
/// `|--|--|` → `|---|---|`
fn fix_table_separator(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') { return None; }

    let inner = &trimmed[1..trimmed.len()-1];
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
            fixed_cells.push(format!("{}{}{}", " ".repeat(leading), normalized, " ".repeat(trailing)));
        } else {
            fixed_cells.push(cell.to_string());
        }
    }

    // Reconstruct the leading indentation
    let leading_spaces = line.len() - line.trim_start().len();
    Some(format!("{}|{}|", " ".repeat(leading_spaces), fixed_cells.join("|")))
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
    let new_bar: String = std::iter::repeat(bar_char).take(expected_n).collect();

    // Adjust whitespace gap to keep value at same visual column
    let after_trimmed = after.trim_start();
    let gap_chars = after.chars().take_while(|c| c.is_whitespace()).count();
    let new_gap = (gap_chars as isize + old_bar_char_count as isize - expected_n as isize).max(1) as usize;
    let new_after = format!("{}{}", " ".repeat(new_gap), after_trimmed);

    Some(format!("{}{}{}", before, new_bar, new_after))
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
        let bar_len = fixed.chars().take_while(|_| true)
            .skip_while(|&c| c != '█').take_while(|&c| c == '█').count();
        // bar length in the fixed version might not be exact due to char boundary,
        // but the bar should be longer than original 13
        assert!(fixed.contains("█████████████████"), "bar should be extended");
    }

    #[test]
    fn fix_table_separator_preserves_indentation() {
        let result = fix_table_separator("  |--|--|");
        assert_eq!(result, Some("  |---|---|".to_string()));
    }
}
