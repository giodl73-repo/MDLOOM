/// Markdown structure validator.
///
/// Checks heading counts, required sections, and required content patterns.

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
        let mut diags = Vec::new();

        // H1 count
        if let Some(max_h1) = self.config.max_h1 {
            let h1_lines: Vec<usize> = lines.iter().enumerate()
                .filter(|(_, l)| l.starts_with("# ") && !l.starts_with("## "))
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

        // Required H2 sections (any one)
        if !self.config.required_h2.is_empty() {
            let h2_headings: Vec<&str> = lines.iter()
                .filter(|l| l.starts_with("## "))
                .map(|l| l.trim_start_matches("## ").trim())
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

        // Required H2 sections (all)
        for required in &self.config.required_h2_all {
            let found = lines.iter().any(|l| {
                l.starts_with("## ") && l.trim_start_matches("## ").trim() == required.as_str()
            });
            if !found {
                diags.push(Diagnostic::warning(
                    path.to_path_buf(), 1, 1,
                    "md_missing_section",
                    format!("missing required section: \"{}\"", required),
                ));
            }
        }

        // Required content patterns
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
