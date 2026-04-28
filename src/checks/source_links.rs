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
    fn name(&self) -> &'static str { "source_links" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        // Only scan source documents, not compiled output
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".source.md") {
            return vec![];
        }

        let mut diags = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_proof_fence = false;  // inside a ```proof:... fence
        let mut in_other_fence = false;  // inside a non-proof fence (skip its content)

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
        let uri_end = uri_start.find(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ']'))
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
        if remaining.is_empty() { break; }
        remaining = &remaining[1..]; // step past the delimiter
    }
}

/// Find the proof root by walking up from `start` until we find `proof.toml`.
/// Falls back to `start` if not found.
fn find_proof_root(start: &Path) -> std::path::PathBuf {
    let mut dir = if start.is_dir() { start.to_path_buf() } else {
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
    let effective_root = if proof_root.join("proof.toml").exists() { proof_root } else { root.to_path_buf() };

    // Check that the file exists
    let file_path = effective_root.join(&parsed.path);
    if !file_path.exists() {
        diags.push(Diagnostic::error(
            path.to_path_buf(),
            line_no,
            1,
            "md_broken_uri",
            format!("md:// URI references missing file: {:?}", parsed.path),
        ));
        return;
    }

    // If the URI has a heading path, check it resolves
    // (skip full element resolution — file existence is the main check for `proof check`)
    // Full element resolution happens at compile time.
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_source(content: &str, root: &Path) -> Vec<Diagnostic> {
        let path = root.join("test.source.md");
        std::fs::write(&path, content).unwrap();
        let check = SourceLinkCheck { root: root.to_path_buf() };
        check.check(&path, content)
    }

    #[test]
    fn no_diags_for_non_source_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        std::fs::write(&path, "```proof:tree kind=org\n```\n").unwrap();
        let check = SourceLinkCheck { root: dir.path().to_path_buf() };
        let diags = check.check(&path, "```proof:tree kind=org\n```\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn missing_file_in_tree_source_produces_error() {
        let dir = tempfile::tempdir().unwrap();
        let content = "# Test\n\n```proof:tree kind=taxonomy source=md://missing.md\n```\n";
        let diags = check_source(content, dir.path());
        assert!(!diags.is_empty(), "should detect broken md:// URI, got empty");
        assert_eq!(diags[0].code, "md_broken_uri");
    }

    #[test]
    fn existing_file_produces_no_error() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("data.md"), "# Data\n| a |\n|---|\n| 1 |\n").unwrap();
        let content = "# Test\n\n```proof:tree kind=taxonomy source=md://data.md\n```\n";
        let diags = check_source(content, dir.path());
        assert!(diags.is_empty(), "valid URI should produce no error, got: {:?}", diags);
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
        let check = SourceLinkCheck { root: dir.path().to_path_buf() };
        let diags = check.check(&path, content);
        assert!(diags.is_empty(), "compiled .md files should not be checked");
    }
}
