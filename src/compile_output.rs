use std::path::{Path, PathBuf};

pub(crate) fn atomic_write(output_path: &Path, text: &str) -> anyhow::Result<()> {
    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, text)
        .map_err(|e| anyhow::anyhow!("writing temp output {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path)
        .map_err(|e| anyhow::anyhow!("renaming output {}: {}", output_path.display(), e))?;
    Ok(())
}

pub(crate) fn apply_replacements(
    source_lines: &[&str],
    replacements: &[(usize, usize, String)],
) -> String {
    if replacements.is_empty() {
        return source_lines.join("\n");
    }

    let mut out: Vec<String> = Vec::new();
    let mut cursor = 0usize;

    for (start, end, replacement) in replacements {
        for line in &source_lines[cursor..*start] {
            out.push(line.to_string());
        }
        out.push(replacement.clone());
        cursor = end + 1;
    }

    for line in &source_lines[cursor..] {
        out.push(line.to_string());
    }

    out.join("\n")
}

/// Safe fallback: return source lines for the directive block, guarded against OOB.
pub(crate) fn source_fallback(
    source_lines: &[&str],
    source_line: usize,
    line_end: usize,
) -> String {
    if source_line <= line_end && line_end < source_lines.len() {
        source_lines[source_line..=line_end].join("\n")
    } else {
        String::new()
    }
}

/// Split a proof source into (frontmatter_yaml, body, body_offset_in_lines).
/// Frontmatter is the block between the opening `---` on line 0 and the next `---`.
/// If no frontmatter is present, returns ("", source, 0).
pub(crate) fn split_frontmatter(source: &str) -> (String, &str, usize) {
    let mut lines = source.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (String::new(), source, 0);
    };
    if first.trim_end_matches(['\r', '\n']).trim() != "---" {
        return (String::new(), source, 0);
    }

    let mut fm_lines: Vec<String> = Vec::new();
    let mut byte_offset = first.len();
    let mut body_offset_lines = 1usize;

    for line in lines {
        byte_offset += line.len();
        body_offset_lines += 1;
        if line.trim_end_matches(['\r', '\n']).trim() == "---" {
            let body = &source[byte_offset.min(source.len())..];
            return (fm_lines.join("\n"), body, body_offset_lines);
        }
        fm_lines.push(line.trim_end_matches(['\r', '\n']).to_string());
    }

    (String::new(), source, 0)
}

/// Derive output path from source path.
/// `foo.source.md` -> `foo.md` (drops `.source.`).
/// Any other `.md` file -> None (require explicit -o).
pub fn derive_output_path(source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?.to_str()?;
    let parent = source.parent().unwrap_or(Path::new("."));
    if let Some(stem) = name.strip_suffix(".slides.source.md") {
        return Some(parent.join(format!("{}.slides.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".dashboard.source.md") {
        return Some(parent.join(format!("{}.dashboard.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".source.md") {
        return Some(parent.join(format!("{}.md", stem)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_output_source_md() {
        let src = Path::new("languages/10-GO.source.md");
        let out = derive_output_path(src).unwrap();
        assert_eq!(out, PathBuf::from("languages/10-GO.md"));
    }

    #[test]
    fn derive_output_plain_md_returns_none() {
        let src = Path::new("languages/10-GO.md");
        assert!(derive_output_path(src).is_none());
    }

    #[test]
    fn derive_output_root_level() {
        let src = Path::new("overview.source.md");
        let out = derive_output_path(src).unwrap();
        assert_eq!(out, PathBuf::from("overview.md"));
    }

    #[test]
    fn apply_replacements_single() {
        let lines = vec!["line0", "```proof:include", "md://x", "```", "line4"];
        let replacements = vec![(1, 3, "REPLACED".to_string())];
        let out = apply_replacements(&lines, &replacements);
        assert_eq!(out, "line0\nREPLACED\nline4");
    }

    #[test]
    fn apply_replacements_none() {
        let lines = vec!["a", "b", "c"];
        let out = apply_replacements(&lines, &[]);
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn apply_replacements_multiple() {
        let lines = vec![
            "before",
            "```proof:include",
            "md://a",
            "```",
            "middle",
            "```proof:include",
            "md://b",
            "```",
            "after",
        ];
        let replacements = vec![
            (1, 3, "A_RESOLVED".to_string()),
            (5, 7, "B_RESOLVED".to_string()),
        ];
        let out = apply_replacements(&lines, &replacements);
        assert_eq!(out, "before\nA_RESOLVED\nmiddle\nB_RESOLVED\nafter");
    }

    #[test]
    fn split_frontmatter_with_yaml() {
        let src = "---\ntitle: Demo\n---\n# Body\n";
        let (fm, body, offset) = split_frontmatter(src);
        assert_eq!(fm, "title: Demo");
        assert_eq!(body, "# Body\n");
        assert_eq!(offset, 3);
    }

    #[test]
    fn split_frontmatter_preserves_crlf_body_offset() {
        let src = "---\r\ntitle: Demo\r\n---\r\n# Body\r\n\r\nText\r\n";
        let (fm, body, offset) = split_frontmatter(src);
        assert_eq!(fm, "title: Demo");
        assert_eq!(body, "# Body\r\n\r\nText\r\n");
        assert_eq!(offset, 3);
    }

    #[test]
    fn split_frontmatter_no_yaml() {
        let src = "# Body\n";
        let (fm, body, offset) = split_frontmatter(src);
        assert_eq!(fm, "");
        assert_eq!(body, src);
        assert_eq!(offset, 0);
    }
}
