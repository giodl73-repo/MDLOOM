use anyhow::Result;
use std::path::Path;

use crate::compile_output;
use crate::compile_types::{CompileViolation, ViolationSeverity};

/// Render a `proof:xref` directive as a formatted cross-reference.
///
/// Resolves the heading text from `uri` (e.g. `md://api.md#authentication`) by
/// reading the target file and finding the heading whose slug matches.
/// Falls back to the URI path if no specific heading is found.
pub(crate) fn render_xref(
    uri: &str,
    label: Option<&str>,
    format: &str,
    root: &Path,
) -> Result<String> {
    let parsed =
        mdpath::parse(uri).map_err(|e| anyhow::anyhow!("invalid xref URI {:?}: {}", uri, e))?;

    let target_path = root.join(&parsed.path);
    if !target_path.exists() {
        anyhow::bail!("xref target file not found: {:?}", parsed.path);
    }

    let heading_text: String = if parsed.heading_path.is_empty() {
        target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&parsed.path)
            .replace('-', " ")
            .replace('_', " ")
    } else {
        let content = std::fs::read_to_string(&target_path)?;
        let slug_target = parsed.heading_path.last().map(|s| s.as_str()).unwrap_or("");
        find_heading_by_slug(&content, slug_target).unwrap_or_else(|| slug_target.replace('-', " "))
    };

    let display_label = label.unwrap_or(&heading_text);

    let anchor = if parsed.heading_path.is_empty() {
        String::new()
    } else {
        let slug = heading_slug(&heading_text);
        format!("#{}", slug)
    };
    let link = format!("{}{}", parsed.path, anchor);

    let rendered = match format {
        "note" => format!("> **See also:** [{}]({})", display_label, link),
        "callout" => format!("→ [{}]({})", display_label, link),
        _ => format!("*See: [{}]({})*", display_label, link),
    };

    Ok(rendered)
}

/// Find a heading in `content` whose GitHub-style slug matches `target_slug`.
fn find_heading_by_slug(content: &str, target_slug: &str) -> Option<String> {
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
            if !text.is_empty() && heading_slug(text) == target_slug {
                return Some(text.to_string());
            }
        }
    }
    None
}

/// Produce a GitHub-style heading anchor slug from heading text.
/// Lowercase, spaces -> hyphens, strip non-alphanumeric/non-hyphen.
pub(crate) fn heading_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Render a `proof:blockquote` directive for prose documents.
///
/// Distinct from `proof:quote` (slide-only, centered, curly-quoted): this is
/// left-aligned, indented, with optional attribution on a trailing line.
pub(crate) fn render_blockquote(text: &str, attribution: Option<&str>, style: &str) -> String {
    let body_lines: Vec<&str> = text.lines().collect();
    let trimmed_body = trim_blank_edges(&body_lines);

    match style {
        "boxed" => render_blockquote_boxed(&trimmed_body, attribution),
        _ => render_blockquote_indent(&trimmed_body, attribution),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_xref(
    uri: &str,
    label: Option<&String>,
    format: &str,
    root: &Path,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_xref(uri, label.map(|s| s.as_str()), format, root) {
        Ok(rendered) => {
            *resolved_count += 1;
            format!(
                "<!-- proof:compiled from=\"proof:xref\" -->\n{}\n<!-- /proof:compiled -->",
                rendered
            )
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity: ViolationSeverity::Error,
                uri: uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("xref error: {}", e),
                source_line: line_start + 1 + source_line_offset,
            });
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

pub(crate) fn compile_blockquote(
    text: &str,
    attribution: Option<&String>,
    style: &str,
    resolved_count: &mut usize,
) -> String {
    *resolved_count += 1;
    let rendered = render_blockquote(text, attribution.map(|s| s.as_str()), style);
    format!(
        "<!-- proof:compiled from=\"proof:blockquote\" -->\n{}\n<!-- /proof:compiled -->",
        rendered
    )
}

fn trim_blank_edges<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map(|i| i + 1)
        .unwrap_or(0);
    if start >= end {
        Vec::new()
    } else {
        lines[start..end].to_vec()
    }
}

fn render_blockquote_indent(body: &[&str], attribution: Option<&str>) -> String {
    let mut out: Vec<String> = body
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                ">".to_string()
            } else {
                format!("> {}", line)
            }
        })
        .collect();
    if let Some(by) = attribution {
        if !out.is_empty() {
            out.push(">".to_string());
        }
        out.push(format!("> — {}", by));
    }
    out.join("\n")
}

fn render_blockquote_boxed(body: &[&str], attribution: Option<&str>) -> String {
    use crate::layout::visual_width;

    if body.is_empty() && attribution.is_none() {
        return String::new();
    }

    let body_max = body.iter().map(|l| visual_width(l)).max().unwrap_or(0);
    let attr_w = attribution.map(|a| visual_width(a) + 2).unwrap_or(0);
    let inner_w = body_max.max(attr_w);

    let horizontal = "─".repeat(inner_w + 2);
    let top = format!("┌{}┐", horizontal);
    let bot = format!("└{}┘", horizontal);

    let mut out = vec![top];
    for line in body {
        let pad = inner_w.saturating_sub(visual_width(line));
        out.push(format!("│ {}{} │", line, " ".repeat(pad)));
    }
    if let Some(by) = attribution {
        if !body.is_empty() {
            out.push(format!("│ {} │", " ".repeat(inner_w)));
        }
        let attr_text = format!("— {}", by);
        let pad = inner_w.saturating_sub(visual_width(&attr_text));
        out.push(format!("│ {}{} │", " ".repeat(pad), attr_text));
    }
    out.push(bot);
    out.join("\n")
}
