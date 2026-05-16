use std::path::Path;

use crate::compile_output;
use crate::compile_source;
use crate::compile_types::{CompileViolation, ViolationSeverity};

fn build_numbered_label(headings: &[(usize, String)], min_level: usize) -> String {
    let (target_level, _) = headings.last().unwrap();
    let target_depth = target_level - min_level;
    let mut counters: Vec<usize> = vec![0; target_depth + 1];
    for (level, _) in headings {
        let depth = level - min_level;
        if depth <= target_depth {
            counters[depth] += 1;
            for d in (depth + 1)..=target_depth {
                counters[d] = 0;
            }
        }
    }
    let parts: Vec<String> = counters[..=target_depth]
        .iter()
        .map(|n| n.to_string())
        .collect();
    format!("{}.", parts.join("."))
}

pub(crate) fn generate_toc(
    content: &str,
    max_depth: usize,
    style: &str,
    section: Option<&str>,
) -> String {
    let mut all: Vec<(usize, String)> = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed[level..].trim().to_string();
            if !text.is_empty() {
                all.push((level, text));
            }
        }
    }

    let scoped: Vec<(usize, String)> = if let Some(target) = section {
        let want = target.trim().to_lowercase();
        let start = all
            .iter()
            .position(|(_, t)| t.trim().to_lowercase() == want);
        match start {
            Some(idx) => {
                let parent_level = all[idx].0;
                let mut out = Vec::new();
                for (level, text) in all.iter().skip(idx + 1) {
                    if *level <= parent_level {
                        break;
                    }
                    out.push((*level, text.clone()));
                }
                out
            }
            None => Vec::new(),
        }
    } else {
        all
    };

    let headings: Vec<(usize, String)> = scoped
        .into_iter()
        .filter(|(level, _)| *level <= max_depth)
        .collect();

    if headings.is_empty() {
        return String::new();
    }
    let min_level = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);
    let mut out = String::new();
    for (i, (level, text)) in headings.iter().enumerate() {
        let depth = level - min_level;
        let indent = "  ".repeat(depth);
        if style == "tree" && depth > 0 {
            let is_last = !headings[i + 1..].iter().any(|(l, _)| *l <= *level);
            let connector = if is_last { "└── " } else { "├── " };
            let parent_indent = "  ".repeat(depth.saturating_sub(1));
            out.push_str(&format!("{}  {}{}\n", parent_indent, connector, text));
        } else if style == "numbered" {
            let number = build_numbered_label(&headings[..=i], min_level);
            out.push_str(&format!("{}{} {}\n", indent, number, text));
        } else {
            out.push_str(&format!("{}- {}\n", indent, text));
        }
    }
    out.trim_end().to_string()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_toc(
    source: Option<&String>,
    max_depth: usize,
    style: &str,
    section: Option<&String>,
    root: &Path,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    let content_opt: Option<String> = if let Some(uri) = source {
        match compile_source::resolve_source_for_compile(uri, root) {
            Ok(content) => Some(content),
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: uri.clone(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("toc source error: {}", e),
                    source_line: line_start + 1 + source_line_offset,
                });
                None
            }
        }
    } else {
        Some(source_lines.join("\n"))
    };

    match content_opt {
        Some(content) => {
            *resolved_count += 1;
            let toc = generate_toc(&content, max_depth, style, section.map(|s| s.as_str()));
            format!(
                "<!-- proof:compiled from=\"proof:toc\" -->\n{}\n<!-- /proof:compiled -->",
                toc
            )
        }
        None => compile_output::source_fallback(source_lines, line_start, line_end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DOC: &str = "\
# Doc Title

## Intro

Some prose.

## API Reference

### Endpoints

#### GET /widgets

#### POST /widgets

### Authentication

## Migration

### Upgrade Steps
";

    #[test]
    fn toc_no_section_lists_everything() {
        let out = generate_toc(SAMPLE_DOC, 4, "list", None);
        assert!(out.contains("API Reference"));
        assert!(out.contains("Endpoints"));
        assert!(out.contains("Migration"));
        assert!(out.contains("Upgrade Steps"));
    }

    #[test]
    fn toc_section_filters_to_descendants() {
        let out = generate_toc(SAMPLE_DOC, 4, "list", Some("API Reference"));
        assert!(out.contains("Endpoints"));
        assert!(out.contains("Authentication"));
        assert!(out.contains("GET /widgets"));
        assert!(
            !out.contains("API Reference"),
            "section anchor heading must be excluded from output, got:\n{}",
            out
        );
        assert!(
            !out.contains("Migration"),
            "headings outside the section must be excluded, got:\n{}",
            out
        );
        assert!(!out.contains("Upgrade Steps"));
        assert!(!out.contains("Intro"));
    }

    #[test]
    fn toc_section_respects_max_depth() {
        let out = generate_toc(SAMPLE_DOC, 3, "list", Some("API Reference"));
        assert!(out.contains("Endpoints"));
        assert!(out.contains("Authentication"));
        assert!(
            !out.contains("GET /widgets"),
            "H4 must be filtered by max_depth=3, got:\n{}",
            out
        );
        assert!(!out.contains("POST /widgets"));
    }

    #[test]
    fn toc_section_case_insensitive_match() {
        let out = generate_toc(SAMPLE_DOC, 4, "list", Some("api reference"));
        assert!(
            out.contains("Endpoints"),
            "section match must be case-insensitive, got:\n{}",
            out
        );
    }

    #[test]
    fn toc_section_not_found_returns_empty() {
        let out = generate_toc(SAMPLE_DOC, 4, "list", Some("Nonexistent Section"));
        assert!(
            out.is_empty(),
            "missing section should produce empty TOC, got:\n{}",
            out
        );
    }

    #[test]
    fn toc_section_works_for_h3_anchor() {
        let out = generate_toc(SAMPLE_DOC, 4, "list", Some("Endpoints"));
        assert!(out.contains("GET /widgets"));
        assert!(out.contains("POST /widgets"));
        assert!(!out.contains("Authentication"));
    }

    #[test]
    fn toc_section_numbered_renumbers_from_section() {
        let out = generate_toc(SAMPLE_DOC, 4, "numbered", Some("API Reference"));
        assert!(
            out.starts_with("1. Endpoints"),
            "numbered TOC must renumber from the section root, got:\n{}",
            out
        );
    }
}
