/// Source document link checker.
///
/// Scans `.source.md` files for `md://` URIs inside `proof:` directives and
/// reports broken references as diagnostics — so `proof check` catches them
/// without requiring a full compile.
use crate::checks::Check;
use crate::diagnostic::Diagnostic;
use std::path::Path;

pub struct SourceLinkCheck {
    pub root: std::path::PathBuf,
}

impl Check for SourceLinkCheck {
    fn name(&self) -> &'static str {
        "source_links"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        // Only scan source documents, not compiled output
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".source.md") {
            return vec![];
        }

        let mut diags = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_proof_fence = false; // inside a ```proof:... fence
        let mut in_other_fence = false; // inside a non-proof fence (skip its content)

        for (i, &line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("```") {
                let info = trimmed[3..].trim();
                if !in_proof_fence && !in_other_fence {
                    // Opening a fence
                    if info.starts_with("proof:") {
                        in_proof_fence = true;
                        // Check the info string for md:// URIs
                        check_uris_in_text(info, i + 1, path, &self.root, &mut diags);
                        // F42b: proof:row requires source=md://... — catch at lint time
                        if info.starts_with("proof:row") && !info.contains("source=md://") {
                            diags.push(crate::diagnostic::Diagnostic::error(
                                path.to_path_buf(),
                                i + 1,
                                1,
                                "md_missing_source",
                                "proof:row requires a source=md://... attribute",
                            ));
                        }
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

            // Only scan specific body lines inside proof: directive fences:
            // - Standalone md:// lines (for proof:include)
            // - Lines containing source=md:// attribute
            // Skip tree node labels, prose descriptions, and other body text
            // that may contain example URIs that aren't real references.
            if in_proof_fence {
                let is_standalone_uri = trimmed.starts_with("md://");
                let has_source_attr = trimmed.contains("source=md://");
                if is_standalone_uri || has_source_attr {
                    check_uris_in_text(trimmed, i + 1, path, &self.root, &mut diags);
                }
            }
        }

        diags
    }
}

/// Extract and validate all md:// URIs from a line of text.
fn check_uris_in_text(
    text: &str,
    line_no: usize,
    path: &Path,
    root: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    // Find all md:// tokens in the line
    let mut remaining = text;
    while let Some(pos) = remaining.find("md://") {
        let uri_start = &remaining[pos..];
        // URI ends at whitespace, quote, or end of string
        let uri_end = uri_start
            .find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ']'))
            .unwrap_or(uri_start.len());
        let uri_str = &uri_start[..uri_end];

        // Skip bare "md://" with no path component (e.g. in prose descriptions)
        let has_path = uri_str.len() > 5 && {
            let after_scheme = &uri_str[5..];
            after_scheme.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '.')
        };

        if has_path {
            validate_uri(uri_str, line_no, path, root, diags);
        }

        remaining = &remaining[pos + uri_end..];
        if remaining.is_empty() {
            break;
        }
        remaining = &remaining[1..]; // step past the delimiter
    }
}

/// Find the proof root by walking up from `start` until we find `proof.toml`.
/// Falls back to `start` if not found.
fn find_proof_root(start: &Path) -> std::path::PathBuf {
    let mut dir = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };
    loop {
        if dir.join("proof.toml").exists() {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return start.to_path_buf(),
        }
    }
}

fn validate_uri(
    uri_str: &str,
    line_no: usize,
    path: &Path,
    root: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    // Parse the URI
    let parsed = match mdpath::parse(uri_str) {
        Ok(u) => u,
        Err(e) => {
            diags.push(Diagnostic::error(
                path.to_path_buf(),
                line_no,
                1,
                "md_broken_uri",
                format!("malformed md:// URI {:?}: {}", uri_str, e),
            ));
            return;
        }
    };

    // Resolve against the proof root (where proof.toml lives), not the scan dir
    let proof_root = find_proof_root(path);
    let effective_root = if proof_root.join("proof.toml").exists() {
        proof_root
    } else {
        root.to_path_buf()
    };

    // Check that the file exists
    let file_path = effective_root.join(&parsed.path);
    if !file_path.exists() {
        let hint = suggest_similar_file(&parsed.path, &effective_root);
        let msg = match hint {
            Some(ref candidate) => format!(
                "Reference to '{}' not found — did you mean '{}'?",
                parsed.path, candidate
            ),
            None => format!("Reference to '{}' not found", parsed.path),
        };
        diags.push(Diagnostic::error(
            path.to_path_buf(),
            line_no,
            1,
            "md_broken_uri",
            msg,
        ));
        return;
    }

    // If the URI has a heading path, validate that each heading slug exists in the file.
    // We check the heading_path sequence by walking the document and verifying that the
    // requested heading appears (at any level) after the previous match — a lightweight
    // structural check that catches typos without full element resolution.
    if !parsed.heading_path.is_empty() {
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                if let Some(missing) = find_missing_heading_slug(&content, &parsed.heading_path) {
                    let full_path = parsed.heading_path.join("/");
                    diags.push(Diagnostic::error(
                        path.to_path_buf(),
                        line_no,
                        1,
                        "md_broken_heading",
                        format!(
                            "Heading '{}' not found in '{}' (looking for '#{}' in heading path '#{}') ",
                            missing, parsed.path, missing, full_path
                        ),
                    ));
                }
            }
            Err(_) => {} // file read error — silently skip; the file-existence check already passed
        }
    }
}

/// Walk the heading path and return the first slug that cannot be found
/// sequentially in the document headings. Returns `None` if all slugs resolve.
fn find_missing_heading_slug<'a>(content: &str, heading_path: &'a [String]) -> Option<&'a str> {
    // Collect all heading slugs in document order (skipping code blocks).
    let slugs: Vec<String> = collect_heading_slugs(content);

    // Walk the heading_path: each element must appear in `slugs` at or after
    // the position of the previous match (loose sequence, not strict nesting).
    let mut search_from = 0usize;
    for slug in heading_path {
        let target = slug.as_str();
        let found = slugs[search_from..].iter().position(|s| s == target);
        match found {
            Some(rel_pos) => search_from += rel_pos + 1,
            None => return Some(target),
        }
    }
    None
}

/// Extract heading slugs from a markdown document (skipping code fences).
fn collect_heading_slugs(content: &str) -> Vec<String> {
    let mut slugs = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed[level..].trim();
            if !text.is_empty() {
                slugs.push(heading_slug(text));
            }
        }
    }
    slugs
}

/// GitHub-style heading slug: lowercase, spaces → hyphens, strip non-alnum/non-hyphen.
fn heading_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Find the closest `.md` file name under `root` to the missing `path`.
/// Returns the relative path string if one is within edit distance 3, else None.
fn suggest_similar_file(missing: &str, root: &Path) -> Option<String> {
    let missing_lower = missing.to_lowercase();
    let mut best: Option<(String, usize)> = None;

    for entry in walkdir::WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel = match entry.path().strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if !rel.ends_with(".md") {
            continue;
        }
        let d = edit_distance_str(&missing_lower, &rel.to_lowercase());
        if d <= 3 && best.as_ref().is_none_or(|(_, bd)| d < *bd) {
            best = Some((rel, d));
        }
    }

    best.map(|(name, _)| name)
}

fn edit_distance_str(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut row: Vec<usize> = (0..=n).collect();
    for i in 1..=m {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=n {
            let old = row[j];
            row[j] = if a[i - 1] == b[j - 1] {
                prev
            } else {
                1 + prev.min(row[j]).min(row[j - 1])
            };
            prev = old;
        }
    }
    row[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(content: &str, root: &Path) -> Vec<Diagnostic> {
        let path = root.join("test.source.md");
        std::fs::write(&path, content).unwrap();
        let check = SourceLinkCheck {
            root: root.to_path_buf(),
        };
        check.check(&path, content)
    }

    #[test]
    fn no_diags_for_non_source_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "```proof:tree kind=org\n```\n").unwrap();
        let check = SourceLinkCheck {
            root: dir.path().to_path_buf(),
        };
        let diags = check.check(&path, "```proof:tree kind=org\n```\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_file_in_tree_source_produces_error() {
        let dir = tempfile::tempdir().unwrap();
        let content = "# Test\n\n```proof:tree kind=taxonomy source=md://missing.md\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            !diags.is_empty(),
            "should detect broken md:// URI, got empty"
        );
        assert_eq!(diags[0].code, "md_broken_uri");
    }

    #[test]
    fn existing_file_produces_no_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data.md"), "# Data\n| a |\n|---|\n| 1 |\n").unwrap();
        let content = "# Test\n\n```proof:tree kind=taxonomy source=md://data.md\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.is_empty(),
            "valid URI should produce no error, got: {:?}",
            diags
        );
    }

    #[test]
    fn missing_row_source_produces_error() {
        let dir = tempfile::tempdir().unwrap();
        let content = "# Test\n\n```proof:row source=md://no-such-file.md foreach=row separator=\" | \"\nproof:element kind=label field=name width=10\n```\n";
        let diags = check_source(content, dir.path());
        assert!(!diags.is_empty());
        assert_eq!(diags[0].code, "md_broken_uri");
    }

    #[test]
    fn non_source_md_file_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guide.md");
        // This is a compiled output file — should NOT be checked for source links
        let content = "```proof:tree kind=org source=md://nonexistent.md\n```\n";
        std::fs::write(&path, content).unwrap();
        let check = SourceLinkCheck {
            root: dir.path().to_path_buf(),
        };
        let diags = check.check(&path, content);
        assert!(diags.is_empty(), "compiled .md files should not be checked");
    }

    // ── heading path validation ───────────────────────────────────────────────

    #[test]
    fn valid_heading_in_uri_produces_no_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("api.md"),
            "# API\n\n## Authentication\n\nContent.\n",
        )
        .unwrap();
        let content = "```proof:include\nmd://api.md#authentication\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.is_empty(),
            "existing heading should not produce error, got: {:?}",
            diags
        );
    }

    #[test]
    fn missing_heading_in_uri_produces_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("api.md"),
            "# API\n\n## Overview\n\nContent.\n",
        )
        .unwrap();
        let content = "```proof:include\nmd://api.md#nonexistent-section\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.iter().any(|d| d.code == "md_broken_heading"),
            "missing heading should produce md_broken_heading, got: {:?}",
            diags
        );
    }

    #[test]
    fn heading_check_skipped_for_file_only_uri() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data.md"), "# Data\n\nContent.\n").unwrap();
        let content = "```proof:include\nmd://data.md\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.is_empty(),
            "file-only URI (no heading) must not trigger heading check"
        );
    }

    #[test]
    fn heading_inside_code_fence_does_not_count() {
        let dir = tempfile::tempdir().unwrap();
        // The only ## heading is inside a code fence — should not satisfy the heading check
        std::fs::write(
            dir.path().join("doc.md"),
            "# Title\n\n```\n## code-section\n```\n",
        )
        .unwrap();
        let content = "```proof:include\nmd://doc.md#code-section\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.iter().any(|d| d.code == "md_broken_heading"),
            "heading inside code fence must not satisfy heading path check, got: {:?}",
            diags
        );
    }

    #[test]
    fn nested_heading_path_validated() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("guide.md"),
            "# Guide\n\n## API Reference\n\n### Authentication\n\nContent.\n",
        )
        .unwrap();
        // md://guide.md#api-reference/authentication — both slugs must be found
        let content = "```proof:include\nmd://guide.md#api-reference/authentication\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.is_empty(),
            "nested heading path that exists must not produce error, got: {:?}",
            diags
        );
    }

    #[test]
    fn nested_heading_path_missing_child_warns() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("guide.md"),
            "# Guide\n\n## API Reference\n\nContent only, no sub-headings.\n",
        )
        .unwrap();
        let content = "```proof:include\nmd://guide.md#api-reference/missing-child\n```\n";
        let diags = check_source(content, dir.path());
        assert!(
            diags.iter().any(|d| d.code == "md_broken_heading"),
            "missing child heading should produce md_broken_heading, got: {:?}",
            diags
        );
    }

    #[test]
    fn collect_heading_slugs_basic() {
        let content = "# Title One\n\n## Section A\n\n### Sub Section\n\nProse\n";
        let slugs = collect_heading_slugs(content);
        assert_eq!(slugs, vec!["title-one", "section-a", "sub-section"]);
    }

    #[test]
    fn heading_slug_strips_special_chars() {
        assert_eq!(heading_slug("What's New?"), "whats-new");
        assert_eq!(heading_slug("API Reference"), "api-reference");
    }
}
