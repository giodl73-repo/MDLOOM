/// Markdown structure validator.
///
/// Checks heading counts, required sections, and required content patterns.
/// All heading detection skips content inside fenced code blocks to avoid
/// false positives from code comments that start with `#`.

use crate::checks::Check;
use crate::config::{MarkdownConfig, PatternSeverity};
use crate::diagnostic::Diagnostic;
use std::path::Path;

pub struct MarkdownCheck {
    pub config: MarkdownConfig,
}

impl Check for MarkdownCheck {
    fn name(&self) -> &'static str { "markdown" }

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
            let h1_lines: Vec<usize> = lines.iter().enumerate()
                .filter(|(i, l)| {
                    !in_code_block[*i]
                        && l.starts_with("# ")
                        && !l.starts_with("## ")
                })
                .map(|(i, _)| i + 1)
                .collect();
            if h1_lines.len() > max_h1 {
                for &ln in &h1_lines[max_h1..] {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(), ln, 1,
                        "md_h1_count",
                        format!("extra H1 heading (max {} per file)", max_h1),
                    ));
                }
            }
        }

        // Required H2 sections (any one) — outside code blocks only
        if !self.config.required_h2.is_empty() {
            let h2_headings: Vec<&str> = lines.iter().enumerate()
                .filter(|(i, l)| !in_code_block[*i] && l.starts_with("## "))
                .map(|(_, l)| l.trim_start_matches("## ").trim())
                .collect();
            let found_any = self.config.required_h2.iter()
                .any(|req| h2_headings.iter().any(|h| *h == req.as_str()));
            if !found_any {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(), 1, 1,
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
                    path.to_path_buf(), 1, 1,
                    "md_missing_section",
                    format!("missing required section: \"{}\"", required),
                ));
            }
        }

        // Required content patterns — search full content (patterns may
        // legitimately appear inside or outside code blocks)
        for req in &self.config.required_patterns {
            let found = content.contains(&req.pattern);
            if !found {
                let d = match req.severity {
                    PatternSeverity::Error => Diagnostic::error(
                        path.to_path_buf(), 1, 1,
                        "md_missing_pattern",
                        format!("missing required content: {} (pattern: {:?})", req.description, req.pattern),
                    ),
                    PatternSeverity::Warning => Diagnostic::warning(
                        path.to_path_buf(), 1, 1,
                        "md_missing_pattern",
                        format!("missing recommended content: {} (pattern: {:?})", req.description, req.pattern),
                    ),
                };
                diags.push(d);
            }
        }

        // Max lines
        if let Some(max) = self.config.max_lines {
            if lines.len() > max {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(), lines.len(), 1,
                    "md_file_length",
                    format!("file has {} lines, exceeds limit of {}", lines.len(), max),
                ));
            }
        }

        diags
    }
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
        };
        let diags = check.check(Path::new("test.md"), content);
        let h1_warns: Vec<_> = diags.iter().filter(|d| d.code == "md_h1_count").collect();
        assert!(h1_warns.is_empty(),
            "# inside code block must not be counted as H1, got: {:?}",
            h1_warns.iter().map(|d| &d.message).collect::<Vec<_>>());
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
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_h1_count"),
            "real extra H1 outside code block must still be detected");
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
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(diags.iter().any(|d| d.code == "md_missing_section"),
            "## inside code block must not satisfy required section check");
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
        };
        let diags = check.check(Path::new("test.md"), content);
        assert!(diags.iter().all(|d| d.code != "md_h1_count"),
            "# inside tilde fence must not be counted as H1");
    }
}
