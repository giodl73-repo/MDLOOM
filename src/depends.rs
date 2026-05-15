//! Reverse dependency lookup for `md://` URIs.
//!
//! Given an `md://` URI (figure, heading, table, etc.), find every `.source.md`
//! file in the corpus that references it. Used by `proof depends` to answer
//! "what will break if I rename this heading or move this figure?"
//!
//! ## Matching semantics
//!
//! A reference matches the query URI if any of:
//!   - The reference equals the query exactly
//!   - The query is a *file-level* URI (no heading, type, or selector) and the
//!     reference points to the same file
//!   - The query has a heading path and the reference points to the same file
//!     and starts with the same heading path (deeper selectors still match)
//!
//! This means `proof depends md://doc.md` finds every reference to that file,
//! and `proof depends md://doc.md#section` finds every reference at or below
//! that heading. Asking for an exact selector returns only exact matches.
//!
//! ## How references are extracted
//!
//! The scanner uses the same URI-extraction logic as `SourceLinkCheck`:
//! it walks each `.source.md` file looking for fenced `proof:` directives,
//! and within them collects `md://` URIs from the info string, standalone
//! body lines, and `source=md://...` attributes. Prose `md://` mentions
//! outside `proof:` fences are intentionally skipped — they may be examples,
//! not real references.

use mdpath::uri::Selector;
use mdpath::{ElementType, MdUri};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// One reference to a target URI inside a source file.
#[derive(Debug, Clone, PartialEq)]
pub struct Dependency {
    /// Absolute path of the `.source.md` file containing the reference.
    pub source_file: PathBuf,
    /// 1-based line number of the reference.
    pub line: usize,
    /// Exact URI string as written in the source (preserves selectors etc.).
    pub uri: String,
}

/// Find every `.source.md` file under `root` that references `target_uri`.
///
/// Results are sorted by `(source_file, line)` for stable output.
pub fn find_dependents(target_uri: &str, root: &Path) -> Vec<Dependency> {
    let target = match mdpath::parse(target_uri) {
        Ok(u) => u,
        Err(_) => return Vec::new(),
    };

    let mut deps: Vec<Dependency> = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
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

        scan_source_file(path, &content, &target, &mut deps);
    }

    deps.sort_by(|a, b| a.source_file.cmp(&b.source_file).then(a.line.cmp(&b.line)));
    deps
}

/// Scan one `.source.md` file for references that match `target`.
fn scan_source_file(path: &Path, content: &str, target: &MdUri, deps: &mut Vec<Dependency>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_proof_fence = false;
    let mut in_other_fence = false;

    for (i, &line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            let info = trimmed[3..].trim();
            if !in_proof_fence && !in_other_fence {
                if info.starts_with("proof:") {
                    in_proof_fence = true;
                    collect_matches_from_text(info, i + 1, path, target, deps);
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
            let has_source_attr = trimmed.contains("source=md://") || trimmed.contains("uri=md://");
            if is_standalone_uri || has_source_attr {
                collect_matches_from_text(trimmed, i + 1, path, target, deps);
            }
        }
    }
}

/// Pull every `md://` token out of `text` and record the ones that match `target`.
fn collect_matches_from_text(
    text: &str,
    line_no: usize,
    path: &Path,
    target: &MdUri,
    deps: &mut Vec<Dependency>,
) {
    let mut remaining = text;
    while let Some(pos) = remaining.find("md://") {
        let uri_start = &remaining[pos..];
        let uri_end = uri_start
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ']'))
            .unwrap_or(uri_start.len());
        let uri_str = &uri_start[..uri_end];

        let has_path = uri_str.len() > 5 && {
            let after_scheme = &uri_str[5..];
            after_scheme.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '.')
        };

        if has_path {
            if let Ok(parsed) = mdpath::parse(uri_str) {
                if matches(target, &parsed) {
                    deps.push(Dependency {
                        source_file: path.to_path_buf(),
                        line: line_no,
                        uri: uri_str.to_string(),
                    });
                }
            }
        }

        remaining = &remaining[pos + uri_end..];
        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..];
    }
}

/// Does `candidate` reference the same logical target as `target`?
///
/// File path must always match. Beyond that, broader queries match more
/// narrowly-scoped references (file-only matches anything in the file;
/// heading-only matches references at or under that heading).
fn matches(target: &MdUri, candidate: &MdUri) -> bool {
    if !paths_equivalent(&target.path, &candidate.path) {
        return false;
    }

    if !heading_path_prefix_matches(&target.heading_path, &candidate.heading_path) {
        return false;
    }

    // If target specifies a type, candidate's type (or absence) must be compatible.
    // A file-or-heading-only target (no type) matches any type.
    if target.element_type.is_some() && target.element_type != candidate.element_type {
        return false;
    }

    // Likewise for kind: only enforce when target specifies it.
    if target.kind.is_some() && target.kind != candidate.kind {
        return false;
    }

    // Selector: None on target means "any"; otherwise selectors must equal.
    if !selector_matches(&target.selector, &candidate.selector) {
        return false;
    }

    true
}

fn paths_equivalent(a: &str, b: &str) -> bool {
    // Normalize backslashes so Windows-style refs match POSIX-style refs.
    a.replace('\\', "/") == b.replace('\\', "/")
}

fn heading_path_prefix_matches(target: &[String], candidate: &[String]) -> bool {
    if target.len() > candidate.len() {
        return false;
    }
    target.iter().zip(candidate.iter()).all(|(a, b)| a == b)
}

fn selector_matches(target: &Selector, candidate: &Selector) -> bool {
    match target {
        Selector::None => true,
        _ => target == candidate,
    }
}

/// Suppress unused-import warnings when `ElementType` isn't directly named.
#[allow(dead_code)]
fn _ensure_element_type_imported(_: ElementType) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(dir: &Path, rel: &str, content: &str) -> PathBuf {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn finds_standalone_include_uri() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "# Heading\n\n```\ncontent\n```\n");
        write(
            dir.path(),
            "guide.source.md",
            "# Guide\n\n```proof:include\nmd://fig.md#heading:0\n```\n",
        );

        let deps = find_dependents("md://fig.md#heading:0", dir.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].line, 4);
        assert_eq!(deps[0].uri, "md://fig.md#heading:0");
    }

    #[test]
    fn file_only_query_matches_any_reference_to_that_file() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "# A\n```\nx\n```\n");
        write(
            dir.path(),
            "g1.source.md",
            "```proof:include\nmd://fig.md#a:0\n```\n",
        );
        write(
            dir.path(),
            "g2.source.md",
            "```proof:include\nmd://fig.md#a:0[box=1]\n```\n",
        );

        let deps = find_dependents("md://fig.md", dir.path());
        assert_eq!(deps.len(), 2, "file-only query should hit both refs");
    }

    #[test]
    fn heading_query_matches_descendants() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "# Top\n## Sub\n```\nx\n```\n");
        write(
            dir.path(),
            "g.source.md",
            "```proof:include\nmd://fig.md#top/sub:0\n```\n",
        );

        let deps = find_dependents("md://fig.md#top", dir.path());
        assert_eq!(deps.len(), 1, "heading query should match deeper ref");
    }

    #[test]
    fn exact_query_does_not_match_different_selector() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "```\nx\n```\n");
        write(
            dir.path(),
            "g.source.md",
            "```proof:include\nmd://fig.md#:1\n```\n",
        );

        let deps = find_dependents("md://fig.md#:0", dir.path());
        assert!(
            deps.is_empty(),
            "exact selector should not match different index"
        );
    }

    #[test]
    fn ignores_non_source_md_files() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "```\nx\n```\n");
        write(
            dir.path(),
            "compiled.md",
            "```proof:include\nmd://fig.md#:0\n```\n",
        );

        let deps = find_dependents("md://fig.md#:0", dir.path());
        assert!(
            deps.is_empty(),
            "compiled .md (no .source.md suffix) must be skipped"
        );
    }

    #[test]
    fn ignores_md_uris_inside_non_proof_code_fences() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "```\nx\n```\n");
        write(
            dir.path(),
            "g.source.md",
            "```rust\n// example: md://fig.md#:0 — not a real ref\n```\n",
        );

        let deps = find_dependents("md://fig.md#:0", dir.path());
        assert!(
            deps.is_empty(),
            "URIs in non-proof fences are examples, not refs"
        );
    }

    #[test]
    fn finds_uri_in_proof_directive_info_string() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "data.md", "| h |\n|---|\n| 1 |\n");
        write(
            dir.path(),
            "g.source.md",
            "```proof:tree kind=org source=md://data.md\nname: x\nparent: y\n```\n",
        );

        let deps = find_dependents("md://data.md", dir.path());
        assert_eq!(deps.len(), 1);
        assert_eq!(
            deps[0].line, 1,
            "info-string URI should report opening fence line"
        );
    }

    #[test]
    fn results_are_sorted_by_file_then_line() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "fig.md", "```\nx\n```\n");
        write(
            dir.path(),
            "b.source.md",
            "```proof:include\nmd://fig.md#:0\n```\n",
        );
        write(
            dir.path(),
            "a.source.md",
            "```proof:include\nmd://fig.md#:0\n```\n\n```proof:include\nmd://fig.md#:0\n```\n",
        );

        let deps = find_dependents("md://fig.md#:0", dir.path());
        assert_eq!(deps.len(), 3, "got {} deps: {:?}", deps.len(), deps);
        assert!(deps[0].source_file.ends_with("a.source.md"));
        assert!(deps[1].source_file.ends_with("a.source.md"));
        assert!(deps[2].source_file.ends_with("b.source.md"));
        assert!(
            deps[0].line < deps[1].line,
            "lines within a file must be ascending"
        );
    }

    #[test]
    fn malformed_target_uri_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let deps = find_dependents("not-a-real-uri", dir.path());
        assert!(deps.is_empty());
    }
}
