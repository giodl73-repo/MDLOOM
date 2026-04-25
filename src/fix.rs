/// Fix plan: the data format shared between AI-generated plans and `glint fix`.
///
/// Workflow:
///   1. `glint check --format rich` → rich.json
///   2. AI (fix-guide skill) reads rich.json → writes plan.json
///   3. `glint fix --plan plan.json` applies edits to files

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────
// Fix plan types (AI writes these; glint fix reads them)
// ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct FixPlan {
    pub schema_version: String,
    #[serde(default)]
    pub generated_by: String,
    #[serde(default)]
    pub source_report: String,
    #[serde(default)]
    pub summary: PlanSummary,
    pub fixes: Vec<Fix>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlanSummary {
    pub total_fixes: usize,
    pub high_confidence: usize,
    pub medium_confidence: usize,
    pub low_confidence: usize,
    pub files_affected: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Fix {
    pub id: String,
    pub file: PathBuf,
    pub description: String,
    pub confidence: Confidence,
    #[serde(default)]
    pub reasoning: String,
    pub edit: Edit,
    /// Diagnostic that triggered this fix (for traceability)
    #[serde(default)]
    pub diagnostic: DiagnosticRef,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct DiagnosticRef {
    pub code: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Edit {
    /// 1-based line number in the file (informational — old_string matching is authoritative)
    pub line: usize,
    /// The exact current content of the line. If this doesn't match, the fix is skipped.
    pub old_string: String,
    /// The replacement content.
    pub new_string: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "high"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::Low => write!(f, "low"),
        }
    }
}

// ─────────────────────────────────────────────────────────
// Fix application
// ─────────────────────────────────────────────────────────

pub struct FixOptions {
    pub dry_run: bool,
    pub min_confidence: Confidence,
}

#[derive(Debug)]
pub struct FixResult {
    pub applied: Vec<String>,   // fix IDs applied
    pub skipped: Vec<SkipReason>,
    pub files_modified: usize,
}

#[derive(Debug)]
pub struct SkipReason {
    pub id: String,
    pub reason: String,
}

impl FixPlan {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading fix plan: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("parsing fix plan: {}", path.display()))
    }

    pub fn apply(&self, opts: &FixOptions, root: &Path) -> Result<FixResult> {
        // Group fixes by file, filtering by confidence
        let mut by_file: HashMap<PathBuf, Vec<&Fix>> = HashMap::new();
        let mut skipped = Vec::new();

        for fix in &self.fixes {
            if fix.confidence > opts.min_confidence {
                skipped.push(SkipReason {
                    id: fix.id.clone(),
                    reason: format!(
                        "confidence {} below minimum {}",
                        fix.confidence, opts.min_confidence
                    ),
                });
                continue;
            }

            let abs_path = if fix.file.is_absolute() {
                fix.file.clone()
            } else {
                root.join(&fix.file)
            };
            by_file.entry(abs_path).or_default().push(fix);
        }

        let mut applied = Vec::new();
        let mut files_modified = 0;

        for (file_path, mut fixes) in by_file {
            // Apply in reverse line order so earlier line numbers stay valid
            fixes.sort_by(|a, b| b.edit.line.cmp(&a.edit.line));

            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("reading {}", file_path.display()))?;

            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            // Preserve trailing newline flag
            let had_trailing_newline = content.ends_with('\n');

            let mut file_modified = false;

            for fix in fixes {
                let line_idx = fix.edit.line.saturating_sub(1); // 0-based

                // Safety: check line exists and old_string matches
                if line_idx >= lines.len() {
                    skipped.push(SkipReason {
                        id: fix.id.clone(),
                        reason: format!(
                            "line {} out of range (file has {} lines)",
                            fix.edit.line,
                            lines.len()
                        ),
                    });
                    continue;
                }

                if lines[line_idx] != fix.edit.old_string {
                    skipped.push(SkipReason {
                        id: fix.id.clone(),
                        reason: format!(
                            "old_string mismatch at line {} — expected {:?}, found {:?}",
                            fix.edit.line,
                            fix.edit.old_string,
                            lines[line_idx]
                        ),
                    });
                    continue;
                }

                if opts.dry_run {
                    // Show the diff for this fix
                    print_fix_diff(&file_path, fix, &lines[line_idx]);
                } else {
                    lines[line_idx] = fix.edit.new_string.clone();
                    file_modified = true;
                }
                applied.push(fix.id.clone());
            }

            if file_modified && !opts.dry_run {
                let mut new_content = lines.join("\n");
                if had_trailing_newline {
                    new_content.push('\n');
                }
                std::fs::write(&file_path, new_content)
                    .with_context(|| format!("writing {}", file_path.display()))?;
                files_modified += 1;
            }
        }

        Ok(FixResult { applied, skipped, files_modified })
    }
}

fn print_fix_diff(file: &Path, fix: &Fix, old_line: &str) {
    println!(
        "\n--- {} (fix {}: {})",
        file.display(),
        fix.id,
        fix.description
    );
    let old = format!("{}\n", old_line);
    let new = format!("{}\n", fix.edit.new_string);
    let diff = TextDiff::from_lines(&old, &new);
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}:{} {}{}", file.display(), fix.edit.line, prefix, change);
    }
}

// ─────────────────────────────────────────────────────────
// Rich output serialization helper
// ─────────────────────────────────────────────────────────

/// Serialize a slice of diagnostics as a rich JSON array.
/// Includes the `context` field for each diagnostic.
pub fn serialize_rich(diags: &[crate::diagnostic::Diagnostic]) -> Result<String> {
    serde_json::to_string_pretty(diags).map_err(Into::into)
}

/// Serialize a slice of diagnostics as a compact JSON array (no context).
pub fn serialize_json(diags: &[crate::diagnostic::Diagnostic]) -> Result<String> {
    // Temporarily zero out the rich fields for compact output
    let compact: Vec<_> = diags
        .iter()
        .map(|d| CompactDiagnostic {
            file: d.file.display().to_string().replace('\\', "/"),
            line: d.span.line,
            col: d.span.col,
            severity: d.severity.to_string(),
            code: d.code,
            message: d.message.clone(),
        })
        .collect();
    serde_json::to_string_pretty(&compact).map_err(Into::into)
}

#[derive(Serialize)]
struct CompactDiagnostic {
    file: String,
    line: usize,
    col: usize,
    severity: String,
    code: &'static str,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_plan(old: &str, new: &str) -> FixPlan {
        FixPlan {
            schema_version: "1".to_string(),
            generated_by: "test".to_string(),
            source_report: "test.json".to_string(),
            summary: PlanSummary::default(),
            fixes: vec![Fix {
                id: "fix-001".to_string(),
                file: PathBuf::from("test.md"),
                description: "test fix".to_string(),
                confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: old.to_string(),
                    new_string: new.to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            }],
        }
    }

    #[test]
    fn apply_fix_modifies_matching_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "hello world\n").unwrap();

        let plan = FixPlan {
            fixes: vec![Fix {
                id: "fix-001".to_string(),
                file: path.clone(),
                description: "replace".to_string(),
                confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "hello world".to_string(),
                    new_string: "hello earth".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            }],
            ..sample_plan("hello world", "hello earth")
        };

        let result = plan.apply(
            &FixOptions { dry_run: false, min_confidence: Confidence::Low },
            dir.path(),
        ).unwrap();

        assert_eq!(result.applied, vec!["fix-001"]);
        assert!(result.skipped.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello earth\n");
    }

    #[test]
    fn apply_fix_skips_stale_old_string() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "actual content\n").unwrap();

        let plan = FixPlan {
            fixes: vec![Fix {
                id: "fix-001".to_string(),
                file: path.clone(),
                confidence: Confidence::High,
                description: "stale".to_string(),
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "expected content".to_string(), // won't match
                    new_string: "new content".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            }],
            ..sample_plan("x", "y")
        };

        let result = plan.apply(
            &FixOptions { dry_run: false, min_confidence: Confidence::Low },
            dir.path(),
        ).unwrap();

        assert!(result.applied.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].reason.contains("mismatch"));
        // File unchanged
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "actual content\n");
    }

    #[test]
    fn dry_run_makes_no_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "original\n").unwrap();

        let plan = FixPlan {
            fixes: vec![Fix {
                id: "fix-001".to_string(),
                file: path.clone(),
                confidence: Confidence::High,
                description: "dry run test".to_string(),
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "original".to_string(),
                    new_string: "modified".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            }],
            ..sample_plan("original", "modified")
        };

        let _result = plan.apply(
            &FixOptions { dry_run: true, min_confidence: Confidence::Low },
            dir.path(),
        ).unwrap();

        // File must be unchanged (invariant I-12)
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original\n");
    }

    #[test]
    fn fixes_applied_reverse_line_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "line 1\nline 2\nline 3\n").unwrap();

        let plan = FixPlan {
            schema_version: "1".to_string(),
            generated_by: "test".to_string(),
            source_report: String::new(),
            summary: PlanSummary::default(),
            fixes: vec![
                Fix {
                    id: "fix-001".to_string(),
                    file: path.clone(),
                    confidence: Confidence::High,
                    description: "fix line 1".to_string(),
                    reasoning: String::new(),
                    edit: Edit { line: 1, old_string: "line 1".to_string(), new_string: "LINE 1".to_string() },
                    diagnostic: DiagnosticRef::default(),
                },
                Fix {
                    id: "fix-002".to_string(),
                    file: path.clone(),
                    confidence: Confidence::High,
                    description: "fix line 3".to_string(),
                    reasoning: String::new(),
                    edit: Edit { line: 3, old_string: "line 3".to_string(), new_string: "LINE 3".to_string() },
                    diagnostic: DiagnosticRef::default(),
                },
            ],
        };

        let result = plan.apply(
            &FixOptions { dry_run: false, min_confidence: Confidence::Low },
            dir.path(),
        ).unwrap();

        assert_eq!(result.applied.len(), 2);
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "LINE 1\nline 2\nLINE 3\n");
    }
}
