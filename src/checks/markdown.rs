/// Markdown structure validator.
///
/// Checks heading counts, required sections, and required content patterns.
/// All heading detection skips content inside fenced code blocks to avoid
/// false positives from code comments that start with `#`.
use crate::checks::Check;
use crate::config::{MarkdownConfig, PatternSeverity};
use crate::diagnostic::Diagnostic;
use std::path::{Path, PathBuf};

pub struct MarkdownCheck {
    pub config: MarkdownConfig,
    /// Runner root used to resolve cross-document links. When `None`, links
    /// are resolved only against the file's parent directory.
    pub root: Option<PathBuf>,
}

impl Check for MarkdownCheck {
    fn name(&self) -> &'static str {
        "markdown"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        // Build a boolean mask: true = line is inside a fenced code block.
        // Headings inside code blocks are not headings — they're code (e.g.
        // Python `# comment` or shell `#!/bin/bash`).
        let in_code_block = code_block_mask(&lines);

        let mut diags = Vec::new();

        // H1 count — only count headings outside code blocks
        if let Some(max_h1) = self.config.max_h1 {
            let h1_lines: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter(|(i, l)| !in_code_block[*i] && l.starts_with("# ") && !l.starts_with("## "))
                .map(|(i, _)| i + 1)
                .collect();
            if h1_lines.len() > max_h1 {
                for &ln in &h1_lines[max_h1..] {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_h1_count",
                        format!("extra H1 heading (max {} per file)", max_h1),
                    ));
                }
            }
        }

        // Required H2 sections (any one) — outside code blocks only
        if !self.config.required_h2.is_empty() {
            let h2_headings: Vec<&str> = lines
                .iter()
                .enumerate()
                .filter(|(i, l)| !in_code_block[*i] && l.starts_with("## "))
                .map(|(_, l)| l.trim_start_matches("## ").trim())
                .collect();
            let found_any = self
                .config
                .required_h2
                .iter()
                .any(|req| h2_headings.contains(&req.as_str()));
            if !found_any {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    1,
                    1,
                    "md_missing_section",
                    format!(
                        "missing required section — expected one of: {}",
                        self.config.required_h2.join(", ")
                    ),
                ));
            }
        }

        // Required H2 sections (all) — outside code blocks only
        for required in &self.config.required_h2_all {
            let found = lines.iter().enumerate().any(|(i, l)| {
                !in_code_block[i]
                    && l.starts_with("## ")
                    && l.trim_start_matches("## ").trim() == required.as_str()
            });
            if !found {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    1,
                    1,
                    "md_missing_section",
                    format!("missing required section: \"{}\"", required),
                ));
            }
        }

        // Forbidden H2 sections — outside code blocks only. Complement to
        // required_h2_all: keeps authoring scaffolds out of production guides.
        if !self.config.forbidden_h2.is_empty() {
            for (i, line) in lines.iter().enumerate() {
                if in_code_block[i] {
                    continue;
                }
                if !line.starts_with("## ") {
                    continue;
                }
                let heading = line.trim_start_matches("## ").trim();
                if self.config.forbidden_h2.iter().any(|f| f == heading) {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        i + 1,
                        1,
                        "md_forbidden_section",
                        format!("forbidden section: \"{}\" must not appear", heading),
                    ));
                }
            }
        }

        // H2 allowlist (optional_h2 + required lists): `optional_h2` is the
        // explicit signal to close the schema. Required H2 lists only enforce
        // presence, so broad corpus schemas can require anchors without warning
        // on every local section heading.
        let has_allowlist = !self.config.optional_h2.is_empty();
        if has_allowlist {
            let allowed: std::collections::HashSet<&str> = self
                .config
                .required_h2
                .iter()
                .chain(self.config.required_h2_all.iter())
                .chain(self.config.optional_h2.iter())
                .map(|s| s.as_str())
                .collect();
            // Forbidden headings are flagged separately as md_forbidden_section —
            // skip them here so a single offending H2 doesn't produce two warnings.
            let forbidden: std::collections::HashSet<&str> = self
                .config
                .forbidden_h2
                .iter()
                .map(|s| s.as_str())
                .collect();
            for (i, line) in lines.iter().enumerate() {
                if in_code_block[i] {
                    continue;
                }
                if line.starts_with("## ") {
                    let heading = line.trim_start_matches("## ").trim();
                    if !allowed.contains(heading) && !forbidden.contains(heading) {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            i + 1,
                            1,
                            "md_unexpected_section",
                            format!(
                                "Unexpected H2 section \"{}\" — not in schema's allowed H2 list",
                                heading
                            ),
                        ));
                    }
                }
            }
        }

        // Required content patterns — search full content (patterns may
        // legitimately appear inside or outside code blocks)
        for req in &self.config.required_patterns {
            let found = content.contains(&req.pattern);
            if !found {
                let d = match req.severity {
                    PatternSeverity::Error => Diagnostic::error(
                        path.to_path_buf(),
                        1,
                        1,
                        "md_missing_pattern",
                        format!(
                            "missing required content: {} (pattern: {:?})",
                            req.description, req.pattern
                        ),
                    ),
                    PatternSeverity::Warning => Diagnostic::warning(
                        path.to_path_buf(),
                        1,
                        1,
                        "md_missing_pattern",
                        format!(
                            "missing recommended content: {} (pattern: {:?})",
                            req.description, req.pattern
                        ),
                    ),
                };
                diags.push(d);
            }
        }

        // Max lines
        if let Some(max) = self.config.max_lines {
            if lines.len() > max {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    lines.len(),
                    1,
                    "md_file_length",
                    format!("file has {} lines, exceeds limit of {}", lines.len(), max),
                ));
            }
        }

        // ── Heading quality checks ───────────────────────────────────────────

        // Collect all ATX headings outside code blocks
        let atx_headings: Vec<(usize, usize, &str)> = lines
            .iter()
            .enumerate()
            .filter(|(i, l)| !in_code_block[*i] && l.starts_with('#'))
            .filter_map(|(i, l)| {
                let level = l.chars().take_while(|&c| c == '#').count();
                if level == 0 {
                    return None;
                }
                Some((i + 1, level, *l)) // (1-based line, level, raw line)
            })
            .collect();

        if self.config.check_heading_format {
            for &(ln, _level, raw) in &atx_headings {
                let after_hashes = raw.trim_start_matches('#');
                // Must start with exactly one space (not zero, not two+)
                if !after_hashes.is_empty() && !after_hashes.starts_with(' ') {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_heading_format",
                        "heading missing space after `#` — use `# Title` not `#Title`".to_string(),
                    ));
                } else if after_hashes.starts_with("  ") {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_heading_format",
                        "heading has extra space after `#` — use exactly one space",
                    ));
                }
                // Trailing `#` signs (e.g. `## Title ##`) are valid CommonMark but
                // considered bad style in this library.
                // IMPORTANT: Only flag when the trailing # is preceded by a space.
                // `C#` and `F#` are language names, not markdown decoration —
                // `## Gotchas from C#` must NOT be flagged.
                let content = after_hashes.trim();
                if content.ends_with('#') {
                    let without_trailing = content.trim_end_matches('#');
                    // Trailing # is markdown decoration only when preceded by whitespace
                    if without_trailing.ends_with(' ') || without_trailing.ends_with('\t') {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(), ln, 1,
                            "md_heading_format",
                            "trailing `#` in heading — omit closing hashes (e.g. `## Title` not `## Title ##`)",
                        ));
                    }
                }
            }
        }

        if self.config.check_empty_headings {
            for &(ln, _level, raw) in &atx_headings {
                let after_hashes = raw.trim_start_matches('#');
                if after_hashes.trim().is_empty() {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_empty_heading",
                        "empty heading — must have content after `#`",
                    ));
                }
            }
        }

        if self.config.check_heading_hierarchy {
            let mut prev_level = 0usize;
            for &(ln, level, _raw) in &atx_headings {
                if prev_level > 0 && level > prev_level + 1 {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_heading_hierarchy",
                        format!(
                            "heading level skips from H{} to H{} — expected H{}",
                            prev_level,
                            level,
                            prev_level + 1
                        ),
                    ));
                }
                prev_level = level;
            }
        }

        if self.config.check_duplicate_headings {
            let mut seen: std::collections::HashMap<(usize, String), usize> =
                std::collections::HashMap::new();
            for &(ln, level, raw) in &atx_headings {
                let text = raw.trim_start_matches('#').trim().to_lowercase();
                let key = (level, text.clone());
                if let Some(&first_ln) = seen.get(&key) {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        ln,
                        1,
                        "md_duplicate_heading",
                        format!(
                            "duplicate H{} heading {:?} — first appeared at line {}",
                            level,
                            raw.trim_start_matches('#').trim(),
                            first_ln
                        ),
                    ));
                } else {
                    seen.insert(key, ln);
                }
            }
        }

        // ── Document style checks ─────────────────────────────────────────────

        if let Some(ref required_style) = self.config.thematic_break_style {
            for (i, &line) in lines.iter().enumerate() {
                if in_code_block[i] {
                    continue;
                }
                let trimmed = line.trim();
                // Thematic break: line of only `-`, `*`, or `_` (with optional spaces), ≥3 chars
                if is_thematic_break(trimmed) && !required_style.is_empty() {
                    let char_used = trimmed
                        .chars()
                        .find(|&c| c != ' ')
                        .unwrap_or('-')
                        .to_string();
                    let expected_char = required_style
                        .trim_matches('-')
                        .trim_matches('*')
                        .trim_matches('_');
                    let _ = expected_char; // compare the repeated char vs required style
                    if !trimmed
                        .replace(' ', "")
                        .chars()
                        .all(|c| required_style.contains(c))
                    {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            i + 1,
                            1,
                            "md_break_style",
                            format!(
                                "thematic break uses {:?} — project style requires {:?}",
                                char_used, required_style
                            ),
                        ));
                    }
                }
            }
        }

        if self.config.check_blockquote_spacing {
            for (i, &line) in lines.iter().enumerate() {
                if in_code_block[i] {
                    continue;
                }
                // `>text` without space is valid CommonMark but bad style
                if line.starts_with('>') && line.len() > 1 {
                    let after = &line[1..];
                    if !after.starts_with(' ') && !after.starts_with('>') {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            i + 1,
                            1,
                            "md_blockquote_spacing",
                            "block quote missing space after `>` — use `> text` not `>text`",
                        ));
                    }
                }
            }
        }

        // ── Cross-document link target verification ───────────────────────────
        if self.config.check_links {
            check_link_targets(
                path,
                self.root.as_deref(),
                &lines,
                &in_code_block,
                &mut diags,
            );
        }

        diags
    }
}

/// Extract every `[text](url)` link from the file (outside fenced code blocks
/// and outside backtick code spans) and verify that file-path links resolve
/// to a target that exists on disk. Skips http(s)://, mailto:, md://, and
/// `#anchor` links — those are handled by other checks or are out of scope.
fn check_link_targets(
    path: &Path,
    root: Option<&Path>,
    lines: &[&str],
    in_code_block: &[bool],
    diags: &mut Vec<Diagnostic>,
) {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    for (i, &line) in lines.iter().enumerate() {
        if in_code_block[i] {
            continue;
        }
        let opaque = backtick_spans(line);
        for span in extract_md_links(line, &opaque) {
            // Trim a fragment so that `path.md#section` still resolves on disk
            let url = span.url.trim();
            if url.is_empty() {
                continue;
            }
            if is_inline_math_notation_link(span.text, url) {
                continue;
            }
            if is_external_or_anchor(url) {
                continue;
            }
            // mdpath URIs are validated by SourceLinkCheck on .source.md files
            if url.starts_with("md://") {
                continue;
            }

            let path_part = match url.find('#') {
                Some(h) => &url[..h],
                None => url,
            };
            if path_part.is_empty() {
                continue;
            }

            // Reject explicit empty links and obvious non-paths
            let target = resolve_link_target(parent, root, path_part);
            if !target.exists() {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(),
                    i + 1,
                    span.col + 1,
                    "link_broken_target",
                    format!("link target {:?} does not exist", path_part),
                ));
            }
        }
    }
}

/// Resolve a relative link path against the source file's parent. Absolute-
/// looking paths (leading `/`) are resolved against the runner root if one
/// was supplied; otherwise they fall through to the file's parent.
fn resolve_link_target(parent: &Path, root: Option<&Path>, link: &str) -> PathBuf {
    if let Some(stripped) = link.strip_prefix('/') {
        if let Some(r) = root {
            return r.join(stripped);
        }
    }
    parent.join(link)
}

/// `true` for links the link-target check should ignore.
fn is_external_or_anchor(url: &str) -> bool {
    url.starts_with('#')
        || url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with("ftp://")
        || url.starts_with("ftps://")
        || url.starts_with("tel:")
        || url.starts_with("data:")
}

#[derive(Debug)]
struct LinkSpan<'a> {
    /// 0-based column where the `[` begins.
    col: usize,
    text: &'a str,
    url: &'a str,
}

/// Extract `[text](url)` spans from a single line, skipping spans that fall
/// inside a backtick code span. Image syntax (`![alt](url)`) is also caught,
/// since the leading `!` does not change the URL parsing.
fn extract_md_links<'a>(line: &'a str, opaque: &[(usize, usize)]) -> Vec<LinkSpan<'a>> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        if in_opaque(i, opaque) {
            i += 1;
            continue;
        }

        // Find matching `](` — text portion may contain nested brackets, but
        // we keep this simple: find the next `](` not inside a code span.
        let after_open = i + 1;
        let close_text = match find_unopaque(line, after_open, "](", opaque) {
            Some(p) => p,
            None => {
                i += 1;
                continue;
            }
        };
        let url_start = close_text + 2;
        let close_url = match find_unopaque(line, url_start, ")", opaque) {
            Some(p) => p,
            None => {
                i += 1;
                continue;
            }
        };

        let text = &line[after_open..close_text];
        let url = &line[url_start..close_url];
        out.push(LinkSpan { col: i, text, url });
        i = close_url + 1;
    }
    out
}

fn is_inline_math_notation_link(text: &str, url: &str) -> bool {
    if url.contains(',')
        && !url.contains(['/', '\\', '.', '#'])
        && url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ',' | '_' | ' '))
    {
        return true;
    }

    let math_text = text.len() <= 8
        && !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ',' | '/' | '_' | ' '));
    let function_arg_url = url.len() <= 8
        && !url.is_empty()
        && url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ',' | '_' | ' '));

    math_text
        && function_arg_url
        && (text.contains(',')
            || text.contains('/')
            || text.chars().all(|c| c.is_ascii_uppercase()))
}

fn find_unopaque(
    line: &str,
    from: usize,
    needle: &str,
    opaque: &[(usize, usize)],
) -> Option<usize> {
    let mut search_from = from;
    while let Some(rel) = line[search_from..].find(needle) {
        let abs = search_from + rel;
        if !in_opaque(abs, opaque) {
            return Some(abs);
        }
        search_from = abs + needle.len();
    }
    None
}

fn in_opaque(pos: usize, opaque: &[(usize, usize)]) -> bool {
    opaque.iter().any(|(s, e)| pos >= *s && pos < *e)
}

/// Byte ranges of every backtick code span in the line. A code span runs from
/// an opening backtick run to the next backtick run of the same length.
fn backtick_spans(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run_start = i;
        let mut run = 0usize;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
            run += 1;
        }
        // Find a matching backtick run of the same length
        let mut j = i;
        while j < bytes.len() {
            if bytes[j] == b'`' {
                let mut close_run = 0usize;
                while j < bytes.len() && bytes[j] == b'`' {
                    j += 1;
                    close_run += 1;
                }
                if close_run == run {
                    out.push((run_start, j));
                    i = j; // advance past close backtick so it isn't treated as new open
                    break;
                }
            } else {
                j += 1;
            }
        }
        // If unterminated, treat rest of line as opaque to be safe
        if j == bytes.len() && out.last().map(|(s, _)| *s) != Some(run_start) {
            out.push((run_start, bytes.len()));
            break;
        }
    }
    out
}

fn is_thematic_break(trimmed: &str) -> bool {
    if trimmed.len() < 3 {
        return false;
    }
    let without_spaces: String = trimmed.chars().filter(|&c| c != ' ').collect();
    if without_spaces.len() < 3 {
        return false;
    }
    let first = without_spaces.chars().next().unwrap();
    matches!(first, '-' | '*' | '_') && without_spaces.chars().all(|c| c == first)
}

/// Returns a boolean mask where `mask[i]` is true if line `i` is inside a
/// fenced code block. Used to prevent `# comment` in code from being counted
/// as a Markdown H1 heading.
fn code_block_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut in_block = false;
    let mut fence_char = '`';
    let mut fence_len = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            // Detect opening fence
            let ch = trimmed.chars().next();
            if matches!(ch, Some('`') | Some('~')) {
                let c = ch.unwrap();
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if run >= 3 {
                    in_block = true;
                    fence_char = c;
                    fence_len = run;
                    // The fence line itself is NOT inside the block
                }
            }
        } else {
            // Detect closing fence
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len {
                    // Closing fence line itself is NOT inside the block
                    in_block = false;
                    continue;
                }
            }
            mask[i] = true;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h1_inside_code_block_not_counted() {
        let content = "# Real Title\n\n```python\n# This is a comment\ndef foo(): pass\n```\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                max_h1: Some(1),
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        let h1_warns: Vec<_> = diags.iter().filter(|d| d.code == "md_h1_count").collect();
        assert!(
            h1_warns.is_empty(),
            "# inside code block must not be counted as H1, got: {:?}",
            h1_warns.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn real_extra_h1_still_detected() {
        let content = "# Title One\n\n# Title Two\n\ncontent\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                max_h1: Some(1),
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "md_h1_count"),
            "real extra H1 outside code block must still be detected"
        );
    }

    #[test]
    fn required_section_not_fooled_by_code_block_heading() {
        // ## Required Section inside a code block should not count as satisfying the requirement
        let content = "# Title\n\n```\n## Decision Cheat Sheet\n```\n\nno real section\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                required_h2_all: vec!["Decision Cheat Sheet".to_string()],
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "md_missing_section"),
            "## inside code block must not satisfy required section check"
        );
    }

    #[test]
    fn tilde_fences_also_excluded() {
        let content = "# Title\n\n~~~bash\n# bash comment\n~~~\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                max_h1: Some(1),
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_h1_count"),
            "# inside tilde fence must not be counted as H1"
        );
    }

    #[test]
    fn trailing_hash_style_warns_but_csharp_heading_is_clean() {
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                check_heading_format: true,
                ..Default::default()
            },
            root: None,
        };

        let decorated = check.check(Path::new("decorated.md"), "# Title\n\n## Title ##\n");
        assert!(
            decorated.iter().any(|d| d.code == "md_heading_format"),
            "CommonMark closing hashes should be warned as local style"
        );

        let csharp = check.check(Path::new("csharp.md"), "# Title\n\n## Gotchas from C#\n");
        assert!(
            csharp.iter().all(|d| d.code != "md_heading_format"),
            "C# and F# language names are not trailing hash decoration"
        );
    }

    // ── link_broken_target ────────────────────────────────────────────────────

    fn link_check(root: Option<&Path>) -> MarkdownCheck {
        MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                check_links: true,
                ..Default::default()
            },
            root: root.map(|p| p.to_path_buf()),
        }
    }

    #[test]
    fn link_to_existing_sibling_passes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("other.md"), "# Other\n").unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\nSee [other](other.md) for details.\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "link to existing file must not be flagged: {:?}",
            diags
        );
    }

    #[test]
    fn link_to_missing_file_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\nSee [other](missing.md) for details.\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        let broken: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "link_broken_target")
            .collect();
        assert_eq!(
            broken.len(),
            1,
            "expected exactly one broken link, got {:?}",
            diags
        );
        assert!(broken[0].message.contains("missing.md"));
    }

    #[test]
    fn http_links_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content =
            "# Doc\n\nSee [the spec](https://example.com/spec) and [mailto](mailto:x@y).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(diags.iter().all(|d| d.code != "link_broken_target"));
    }

    #[test]
    fn anchor_only_links_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\nSee [the section below](#decision-cheat-sheet).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(diags.iter().all(|d| d.code != "link_broken_target"));
    }

    #[test]
    fn fragment_after_existing_path_passes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("other.md"), "# Other\n").unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\nSee [section](other.md#some-section).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "path#fragment must resolve against the path: {:?}",
            diags
        );
    }

    #[test]
    fn link_inside_fenced_code_block_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\n```\n[bogus](does-not-exist.md)\n```\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "links inside fenced code blocks must not be checked"
        );
    }

    #[test]
    fn link_inside_backtick_span_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content =
            "# Doc\n\nUse the syntax `[text](url)` to add a link, see [README](README.md).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        // The README.md doesn't exist, so we expect exactly one diagnostic — not two.
        let broken: Vec<_> = diags
            .iter()
            .filter(|d| d.code == "link_broken_target")
            .collect();
        assert_eq!(
            broken.len(),
            1,
            "code-span link must be ignored; only README.md is real: {:?}",
            broken
        );
        assert!(broken[0].message.contains("README.md"));
    }

    #[test]
    fn md_uri_links_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\nSee [figure](md://path/to/file.md#section:0).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "md:// URIs are validated by SourceLinkCheck, not the prose link checker"
        );
    }

    #[test]
    fn inline_math_function_notation_links_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content =
            "# Doc\n\n[X](t), [A,A](X,Y), and [m/n](x) are math notation, not file links.\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "math notation should not be checked as relative links: {:?}",
            diags
        );
    }

    #[test]
    fn check_disabled_via_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let content = "# Doc\n\n[broken](no-such-file.md)\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                check_links: false,
                ..Default::default()
            },
            root: Some(dir.path().to_path_buf()),
        };
        let diags = check.check(&path, content);
        assert!(diags.iter().all(|d| d.code != "link_broken_target"));
    }

    #[test]
    fn absolute_path_resolves_against_root() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(dir.path().join("at-root.md"), "# Root\n").unwrap();
        let path = sub.join("doc.md");
        let content = "# Doc\n\nSee [root](/at-root.md).\n";
        let diags = link_check(Some(dir.path())).check(&path, content);
        assert!(
            diags.iter().all(|d| d.code != "link_broken_target"),
            "leading-slash path must resolve against the runner root: {:?}",
            diags
        );
    }

    // ── optional_h2 allowlist ─────────────────────────────────────────────────

    fn allowlist_check(required_all: Vec<&str>, optional: Vec<&str>) -> MarkdownCheck {
        MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                required_h2_all: required_all.into_iter().map(|s| s.to_string()).collect(),
                optional_h2: optional.into_iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            },
            root: None,
        }
    }

    #[test]
    fn optional_h2_unexpected_section_warns() {
        let content = "# Title\n\n## Overview\n\ntext\n\n## Changelog\n\ntext\n";
        let check = allowlist_check(vec!["Overview"], vec!["Notes"]);
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags
                .iter()
                .any(|d| d.code == "md_unexpected_section" && d.message.contains("Changelog")),
            "H2 not in allowlist should produce md_unexpected_section, got: {:?}",
            diags
        );
    }

    #[test]
    fn required_h2_without_optional_h2_allows_any_other_heading() {
        let content = "# Title\n\n## Overview\n\ntext\n\n## Local Notes\n\ntext\n";
        let check = allowlist_check(vec!["Overview"], vec![]);
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_unexpected_section"),
            "required H2s alone must not close the H2 allowlist, got: {:?}",
            diags
        );
    }

    #[test]
    fn optional_h2_allowed_section_no_warn() {
        let content = "# Title\n\n## Overview\n\ntext\n\n## Notes\n\ntext\n";
        let check = allowlist_check(vec!["Overview"], vec!["Notes"]);
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_unexpected_section"),
            "H2 in optional_h2 must not trigger md_unexpected_section, got: {:?}",
            diags
        );
    }

    #[test]
    fn optional_h2_empty_allows_any_heading() {
        let content = "# Title\n\n## Anything Goes\n\ntext\n\n## Even This\n\ntext\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_unexpected_section"),
            "empty allowlist must not flag any H2, got: {:?}",
            diags
        );
    }

    #[test]
    fn optional_h2_required_heading_also_allowed() {
        // A heading in required_h2_all is also "allowed" — must not produce unexpected warning
        let content = "# Title\n\n## Overview\n\ntext\n";
        let check = allowlist_check(vec!["Overview"], vec![]);
        let diags = check.check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_unexpected_section"),
            "required H2 must also be treated as allowed, got: {:?}",
            diags
        );
    }

    // ── forbidden_h2 ──────────────────────────────────────────────────────────

    fn forbidden_check(forbidden: Vec<&str>) -> MarkdownCheck {
        MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                forbidden_h2: forbidden.into_iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            },
            root: None,
        }
    }

    #[test]
    fn forbidden_h2_warns_with_line_and_name() {
        let content = "# Title\n\n## Overview\n\nbody\n\n## Draft\n\nstub\n";
        let diags =
            forbidden_check(vec!["Draft", "TODO", "WIP"]).check(Path::new("test.md"), content);
        let hit = diags
            .iter()
            .find(|d| d.code == "md_forbidden_section")
            .expect("expected md_forbidden_section, got: {:?}");
        assert!(
            hit.message.contains("Draft"),
            "message should name the heading: {:?}",
            hit.message
        );
        assert_eq!(hit.span.line, 7, "should point at the offending H2 line");
    }

    #[test]
    fn forbidden_h2_empty_no_warn() {
        let content = "# Title\n\n## Draft\n\nbody\n";
        let diags = forbidden_check(vec![]).check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_forbidden_section"),
            "empty forbidden_h2 must not flag anything, got: {:?}",
            diags
        );
    }

    #[test]
    fn forbidden_h2_inside_code_block_ignored() {
        let content = "# Title\n\n```\n## Draft\n```\n\n## Overview\n\nbody\n";
        let diags = forbidden_check(vec!["Draft"]).check(Path::new("test.md"), content);
        assert!(
            diags.iter().all(|d| d.code != "md_forbidden_section"),
            "## inside code block must not trigger md_forbidden_section, got: {:?}",
            diags
        );
    }

    #[test]
    fn forbidden_h2_does_not_double_with_unexpected_section() {
        // When an allowlist is active, a forbidden heading is also "not in the
        // allowlist" — but it should produce md_forbidden_section ONLY, not
        // md_unexpected_section as well.
        let content = "# Title\n\n## Overview\n\nbody\n\n## Draft\n\nstub\n";
        let check = MarkdownCheck {
            config: MarkdownConfig {
                enabled: true,
                required_h2_all: vec!["Overview".to_string()],
                forbidden_h2: vec!["Draft".to_string()],
                ..Default::default()
            },
            root: None,
        };
        let diags = check.check(Path::new("test.md"), content);
        let forbidden_hits = diags
            .iter()
            .filter(|d| d.code == "md_forbidden_section")
            .count();
        let unexpected_hits = diags
            .iter()
            .filter(|d| d.code == "md_unexpected_section" && d.message.contains("Draft"))
            .count();
        assert_eq!(
            forbidden_hits, 1,
            "expected exactly one md_forbidden_section: {:?}",
            diags
        );
        assert_eq!(
            unexpected_hits, 0,
            "forbidden heading must not double as md_unexpected_section: {:?}",
            diags
        );
    }
}
