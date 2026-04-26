/// proof baseline — snapshot known issues so CI only fails on NEW regressions.
///
/// Workflow:
///   proof baseline save .          → writes .proof-baseline.json (commit this)
///   proof check --baseline .       → fails only on errors NOT in the baseline
///
/// Matching strategy: (file, code, line) with ±LINE_TOLERANCE lines of drift
/// tolerance, so baseline entries survive small line-number shifts from edits.

use crate::diagnostic::{Diagnostic, Severity};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const DEFAULT_BASELINE_FILE: &str = ".proof-baseline.json";
const LINE_TOLERANCE: usize = 3; // match baseline entries within ±3 lines

// ─────────────────────────────────────────────────────────
// Baseline types
// ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaselineEntry {
    pub file: String,   // relative path, forward slashes
    pub code: String,   // diagnostic code e.g. "ascii_box_col"
    pub line: usize,    // 1-based line
    pub col: usize,     // 1-based col (used as secondary tiebreak)
    pub message: String, // first 60 chars of message (for human readability)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Baseline {
    pub schema_version: u32,
    pub created_at: String,    // ISO 8601 — informational only
    pub entry_count: usize,
    pub entries: Vec<BaselineEntry>,
}

impl Baseline {
    pub fn from_diagnostics(diags: &[Diagnostic], root: &Path) -> Self {
        let mut entries: Vec<BaselineEntry> = diags.iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| {
                let rel = d.file.strip_prefix(root).unwrap_or(&d.file);
                let file = rel.to_string_lossy().replace('\\', "/");
                let message = if d.message.len() > 60 {
                    format!("{}…", &d.message[..60])
                } else {
                    d.message.clone()
                };
                BaselineEntry {
                    file: file.to_string(),
                    code: d.code.to_string(),
                    line: d.span.line,
                    col: d.span.col,
                    message,
                }
            })
            .collect();

        // Sort for deterministic output
        entries.sort_by(|a, b| {
            a.file.cmp(&b.file)
                .then(a.line.cmp(&b.line))
                .then(a.code.cmp(&b.code))
        });

        let count = entries.len();
        Baseline {
            schema_version: 1,
            created_at: chrono_now(),
            entry_count: count,
            entries,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading baseline {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("parsing baseline {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        // Atomic write
        let tmp = path.with_extension("proof_tmp");
        std::fs::write(&tmp, &json)
            .with_context(|| format!("writing baseline {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("finalizing baseline {}", path.display()))?;
        Ok(())
    }

    /// Return diagnostics that are NOT present in this baseline (new regressions).
    pub fn new_since(&self, diags: &[Diagnostic], root: &Path) -> Vec<Diagnostic> {
        diags.iter()
            .filter(|d| d.severity == Severity::Error)
            .filter(|d| !self.is_known(d, root))
            .cloned()
            .collect()
    }

    /// Check if a diagnostic is covered by a baseline entry.
    /// Matches on (file, code) with LINE_TOLERANCE lines of drift.
    fn is_known(&self, diag: &Diagnostic, root: &Path) -> bool {
        let rel = diag.file.strip_prefix(root).unwrap_or(&diag.file);
        let file = rel.to_string_lossy().replace('\\', "/");
        let line = diag.span.line;
        let code = diag.code;

        self.entries.iter().any(|e| {
            e.file == file
                && e.code == code
                && e.line.abs_diff(line) <= LINE_TOLERANCE
        })
    }
}

fn chrono_now() -> String {
    // Simple ISO 8601 without chrono dependency
    // Returns something like "2026-04-26T20:00:00Z" — good enough for metadata
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Rough decode: good enough for display
    let day_secs = secs % 86400;
    let days = secs / 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    // Epoch = Jan 1 1970
    let (y, mo, d) = epoch_to_ymd(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

fn epoch_to_ymd(days: u64) -> (u64, u64, u64) {
    // Approximate calendar calculation — only used for human-readable metadata
    let mut remaining = days;
    let mut year = 1970u64;
    loop {
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        month += 1;
    }
    (year, month, remaining + 1)
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, Severity, Span};

    fn make_diag(file: &str, code: &str, line: usize, sev: Severity) -> Diagnostic {
        Diagnostic {
            file: PathBuf::from(file),
            span: Span { line, col: 1 },
            severity: sev,
            code,
            message: "test message".to_string(),
            note: None,
            group_id: None,
            rich: None,
        }
    }

    #[test]
    fn test_baseline_exact_match() {
        let diags = vec![
            make_diag("foo.md", "ascii_box_col", 10, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        let new_diag = make_diag("foo.md", "ascii_box_col", 10, Severity::Error);
        assert!(baseline.is_known(&new_diag, root));
    }

    #[test]
    fn test_baseline_line_drift() {
        let diags = vec![
            make_diag("foo.md", "ascii_box_col", 10, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        // Within ±3 lines — still known
        let shifted = make_diag("foo.md", "ascii_box_col", 12, Severity::Error);
        assert!(baseline.is_known(&shifted, root));
        // Beyond tolerance — new
        let far = make_diag("foo.md", "ascii_box_col", 20, Severity::Error);
        assert!(!baseline.is_known(&far, root));
    }

    #[test]
    fn test_baseline_different_code_is_new() {
        let diags = vec![
            make_diag("foo.md", "ascii_box_col", 10, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        let new_code = make_diag("foo.md", "ascii_box_width", 10, Severity::Error);
        assert!(!baseline.is_known(&new_code, root));
    }

    #[test]
    fn test_baseline_different_file_is_new() {
        let diags = vec![
            make_diag("foo.md", "ascii_box_col", 10, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        let new_file = make_diag("bar.md", "ascii_box_col", 10, Severity::Error);
        assert!(!baseline.is_known(&new_file, root));
    }

    #[test]
    fn test_new_since_filters_correctly() {
        let diags = vec![
            make_diag("foo.md", "ascii_box_col", 10, Severity::Error),
            make_diag("foo.md", "ascii_box_width", 20, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags[..1], root); // only first
        let new = baseline.new_since(&diags, root);
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].code, "ascii_box_width");
    }

    #[test]
    fn test_warnings_excluded_from_baseline() {
        let diags = vec![
            make_diag("foo.md", "ascii_cell_padding", 10, Severity::Warning),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        assert_eq!(baseline.entries.len(), 0);
    }

    #[test]
    fn test_roundtrip_via_json() {
        let diags = vec![
            make_diag("languages/10-GO.md", "ascii_box_col", 42, Severity::Error),
        ];
        let root = Path::new(".");
        let baseline = Baseline::from_diagnostics(&diags, root);
        let json = serde_json::to_string(&baseline).unwrap();
        let loaded: Baseline = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].code, "ascii_box_col");
    }
}
