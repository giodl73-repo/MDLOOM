//! Detect figure `.md` files that are never referenced by any `.source.md`.
//!
//! Authors create figures (typically as standalone `.md` files in a `figures/`
//! directory) and pull them into prose via `proof:include` or `proof:layout`.
//! Over time, drafts get reorganized and figures can be orphaned — they live
//! in the repo but no source document references them. This module walks the
//! corpus and reports those orphans as `unused_figure` warnings so the BOOK
//! role can prune them.
//!
//! ## What counts as a figure candidate
//!
//! Any `.md` file that:
//!   - Is **not** a `.source.md` file (those are entry points, not figures)
//!   - Is **not** a structural file: README.md, CHANGELOG.md, LICENSE.md,
//!     CONTRIBUTING.md, CLAUDE.md, MEMORY.md, TRACKER.md, BILL-OF-MATERIALS.md,
//!     VOLUMES.md, EXPANSION.md, REVIEW.md, FOREWORD.md, COLOPHON.md,
//!     PROJECTS.md, DEDICATION.md, PUZZLE-HUNT.md, TO-SIGNAL.md
//!   - Is **not** under a `node_modules`, `.git`, or `target` directory
//!
//! ## What counts as a reference
//!
//! A `.source.md` references a figure if any `proof:` directive in it carries
//! an `md://` URI whose path resolves to that figure file. We honour the same
//! extraction rules as `depends.rs` (info string, standalone body lines,
//! `source=md://...` / `uri=md://...` attributes inside `proof:` fences).

use crate::diagnostic::Diagnostic;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// One unused figure result — used by tests and the CLI renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct UnusedFigure {
    pub path: PathBuf,
}

/// Walk `root`, find every figure-candidate `.md` file, and return the ones
/// that no `.source.md` references via `proof:include` / `proof:layout` /
/// `source=md://...`. Results are sorted by path.
pub fn find_unused_figures(root: &Path) -> Vec<UnusedFigure> {
    let candidates = collect_figure_candidates(root);
    let referenced = collect_referenced_paths(root);

    let mut unused: Vec<UnusedFigure> = candidates
        .into_iter()
        .filter(|p| !referenced.contains(&canonical_key(p, root)))
        .map(|path| UnusedFigure { path })
        .collect();

    unused.sort_by(|a, b| a.path.cmp(&b.path));
    unused
}

/// Convert each unused figure into a `Diagnostic` (warning, code = "unused_figure").
/// Diagnostic is anchored at line 1 col 1 since the file as a whole is the issue.
pub fn unused_diagnostics(root: &Path) -> Vec<Diagnostic> {
    find_unused_figures(root)
        .into_iter()
        .map(|u| {
            let display = display_relative(&u.path, root);
            Diagnostic::warning(
                u.path.clone(),
                1,
                1,
                "unused_figure",
                format!(
                    "Figure '{}' is not referenced by any source document.",
                    display
                ),
            )
        })
        .collect()
}

/// All `.md` files under `root` that are eligible to be "figures".
fn collect_figure_candidates(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !is_excluded_dir(e.path()))
        .map(|e| e.into_path())
        .filter(|p| is_figure_candidate(p))
        .collect()
}

/// Canonical lookup key — the path relative to `root` with forward slashes,
/// so we can compare a discovered figure against `md://` paths from sources.
fn canonical_key(path: &Path, root: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn display_relative(path: &Path, root: &Path) -> String {
    canonical_key(path, root)
}

/// Walk the corpus a second time, scan every `.source.md` for referenced
/// `md://` paths, and return the set of canonical relative paths.
fn collect_referenced_paths(root: &Path) -> HashSet<String> {
    let mut set: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| !is_excluded_dir(e.path()))
    {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with(".source.md") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        scan_source(&content, &mut set);
    }

    set
}

/// Pull every `md://` URI out of a single source file and record its path
/// component as a normalized lookup key.
fn scan_source(content: &str, set: &mut HashSet<String>) {
    let mut in_proof_fence = false;
    let mut in_other_fence = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            let info = trimmed[3..].trim();
            if !in_proof_fence && !in_other_fence {
                if info.starts_with("proof:") {
                    in_proof_fence = true;
                    extract_uri_paths(info, set);
                } else {
                    in_other_fence = true;
                }
            } else if in_proof_fence {
                in_proof_fence = false;
            } else if in_other_fence {
                in_other_fence = false;
            }
            continue;
        }

        if in_proof_fence {
            let is_standalone_uri = trimmed.starts_with("md://");
            let has_attr = trimmed.contains("source=md://") || trimmed.contains("uri=md://");
            if is_standalone_uri || has_attr {
                extract_uri_paths(trimmed, set);
            }
        }
    }
}

/// Extract the path portion of every `md://...` token in `text`. The path
/// is the substring between `md://` and the first `#` / whitespace / quote.
fn extract_uri_paths(text: &str, set: &mut HashSet<String>) {
    let mut remaining = text;
    while let Some(pos) = remaining.find("md://") {
        let after = &remaining[pos + 5..];
        let end = after
            .find(|c: char| c == '#' || c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ']'))
            .unwrap_or(after.len());
        let path = &after[..end];
        if !path.is_empty() {
            set.insert(path.replace('\\', "/"));
        }
        let advance = pos + 5 + end;
        if advance >= remaining.len() {
            break;
        }
        remaining = &remaining[advance..];
    }
}

/// Is this `.md` file a figure candidate (vs. a structural file or a source doc)?
fn is_figure_candidate(path: &Path) -> bool {
    let name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };

    if !name.ends_with(".md") {
        return false;
    }
    if name.ends_with(".source.md") {
        return false;
    }

    let is_structural = matches!(
        name,
        "README.md"
            | "CHANGELOG.md"
            | "LICENSE.md"
            | "CONTRIBUTING.md"
            | "CLAUDE.md"
            | "MEMORY.md"
            | "TRACKER.md"
            | "BILL-OF-MATERIALS.md"
            | "VOLUMES.md"
            | "EXPANSION.md"
            | "REVIEW.md"
            | "FOREWORD.md"
            | "COLOPHON.md"
            | "PROJECTS.md"
            | "DEDICATION.md"
            | "PUZZLE-HUNT.md"
            | "TO-SIGNAL.md"
            | "STATUS.md"
            | "HISTORY.md"
            | "SCORECARD.md"
            | "CONCEPT-INDEX.md"
            | "READING-MAPS.md"
            | "PREREQUISITES.md"
            | "index.md"
            | "book.md"
    );
    !is_structural
}

fn is_excluded_dir(path: &Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        matches!(
            s.as_ref(),
            "node_modules" | ".git" | "target" | ".proof-cache"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn figure_referenced_via_proof_include_is_not_unused() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("figures/used.md"), "# Used figure\n\nbody\n");
        write(
            &root.join("doc.source.md"),
            "# Doc\n\n```proof:include\nmd://figures/used.md#:0\n```\n",
        );

        let unused = find_unused_figures(root);
        assert!(
            unused.is_empty(),
            "used.md should not be flagged: {:?}",
            unused
        );
    }

    #[test]
    fn orphan_figure_is_reported_as_unused() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("figures/orphan.md"), "# Orphan\n");
        write(&root.join("doc.source.md"), "# Doc, no references here\n");

        let unused = find_unused_figures(root);
        assert_eq!(unused.len(), 1);
        assert!(unused[0].path.ends_with("orphan.md"));
    }

    #[test]
    fn proof_layout_reference_counts() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("figures/a.md"), "# A\n");
        write(&root.join("figures/b.md"), "# B\n");
        write(
            &root.join("doc.source.md"),
            "# Doc\n\n```proof:layout gap=4\nmd://figures/a.md#:0\nmd://figures/b.md#:0\n```\n",
        );

        let unused = find_unused_figures(root);
        assert!(
            unused.is_empty(),
            "both figures referenced via proof:layout: {:?}",
            unused
        );
    }

    #[test]
    fn source_attribute_reference_counts() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(
            &root.join("data/table.md"),
            "# Table\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
        );
        write(
            &root.join("doc.source.md"),
            "```proof:row source=md://data/table.md\nproof:element field=a\n```\n",
        );

        let unused = find_unused_figures(root);
        assert!(
            unused.is_empty(),
            "table referenced via source= attribute: {:?}",
            unused
        );
    }

    #[test]
    fn structural_files_are_never_flagged_as_unused() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("README.md"), "# README\n");
        write(&root.join("CHANGELOG.md"), "# Changelog\n");
        write(&root.join("doc.source.md"), "# Doc\n");

        let unused = find_unused_figures(root);
        assert!(
            unused.is_empty(),
            "structural files must be excluded: {:?}",
            unused
        );
    }

    #[test]
    fn mention_outside_proof_fence_does_not_count_as_reference() {
        // Prose mentioning md://figures/x.md in a paragraph (not inside a proof:
        // fence) must not save a figure from being marked unused.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("figures/x.md"), "# X\n");
        write(
            &root.join("doc.source.md"),
            "# Doc\n\nThis prose mentions md://figures/x.md but that is not a real reference.\n",
        );

        let unused = find_unused_figures(root);
        assert_eq!(
            unused.len(),
            1,
            "prose mention must not protect figure: {:?}",
            unused
        );
    }

    #[test]
    fn diagnostic_message_names_the_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        write(&root.join("figures/orphan.md"), "# Orphan\n");
        write(&root.join("doc.source.md"), "# Doc\n");

        let diags = unused_diagnostics(root);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, "unused_figure");
        assert!(
            diags[0].message.contains("figures/orphan.md"),
            "message should name the figure: {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("not referenced"),
            "message should explain the issue: {}",
            diags[0].message
        );
    }

    #[test]
    fn excluded_directories_are_skipped() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // A figure deep inside node_modules — must NOT be reported.
        write(&root.join("node_modules/pkg/figure.md"), "# vendored\n");
        write(&root.join("doc.source.md"), "# Doc\n");

        let unused = find_unused_figures(root);
        assert!(
            unused.is_empty(),
            "node_modules figures must be ignored: {:?}",
            unused
        );
    }
}
