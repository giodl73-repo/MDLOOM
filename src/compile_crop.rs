use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::compile::{CompileViolation, ViolationSeverity};
use crate::compile_directive::Directive;
use crate::compile_output;
use crate::crop_side_info;

#[derive(Clone, Copy)]
pub(crate) enum SideInfoKind {
    Links,
    Backlinks,
    Headings,
    Frontmatter,
}

impl SideInfoKind {
    fn filename(self) -> &'static str {
        match self {
            SideInfoKind::Links => "links.json",
            SideInfoKind::Backlinks => "backlinks.json",
            SideInfoKind::Headings => "headings.json",
            SideInfoKind::Frontmatter => "frontmatter.json",
        }
    }
}

pub(crate) fn side_info_path(root: &Path, explicit: Option<&str>, kind: SideInfoKind) -> PathBuf {
    explicit
        .map(|p| root.join(p))
        .unwrap_or_else(|| root.join(".proof").join("side-info").join(kind.filename()))
}

pub(crate) fn side_info_dependencies(directives: &[Directive], root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for directive in directives {
        let path = match directive {
            Directive::Backlinks { source, .. } => {
                side_info_path(root, source.as_deref(), SideInfoKind::Backlinks)
            }
            Directive::Links { source, .. } => {
                side_info_path(root, source.as_deref(), SideInfoKind::Links)
            }
            Directive::Headings { source, .. } => {
                side_info_path(root, source.as_deref(), SideInfoKind::Headings)
            }
            Directive::Frontmatter { source, .. } => {
                side_info_path(root, source.as_deref(), SideInfoKind::Frontmatter)
            }
            _ => {
                continue;
            }
        };
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

pub(crate) fn dependency_parse_keys(
    paths: &[PathBuf],
    path_index: &mut crate::cache::PathIndex,
) -> Vec<String> {
    paths
        .iter()
        .map(|path| match std::fs::read_to_string(path) {
            Ok(content) => crate::cache::get_or_compute_parse_key(path, &content, path_index),
            Err(_) => format!("missing:{}", path.display()),
        })
        .collect()
}

pub(crate) fn frontmatter_filter(
    field: &Option<String>,
    value: &Option<String>,
    op: &str,
) -> Result<crop_side_info::FrontmatterFilter> {
    let op = match op {
        "has" => crop_side_info::FrontmatterMatch::Has,
        "eq" => crop_side_info::FrontmatterMatch::Eq,
        _ => bail!("frontmatter match op must be 'has' or 'eq'"),
    };
    Ok(crop_side_info::FrontmatterFilter {
        field: field.clone(),
        value: value.clone(),
        op,
    })
}

pub(crate) fn link_filter(
    source: &Option<String>,
    status: &str,
) -> Result<crop_side_info::LinkFilter> {
    let status = match status {
        "all" => Some("all".to_string()),
        "ok" | "broken" => Some(status.to_string()),
        _ => bail!("link status must be 'all', 'ok', or 'broken'"),
    };
    Ok(crop_side_info::LinkFilter {
        source: source.clone(),
        status,
    })
}

pub(crate) fn render_backlinks(
    root: &Path,
    side_info: Option<&str>,
    target: &str,
    format: &str,
) -> Result<String> {
    let report_path = side_info_path(root, side_info, SideInfoKind::Backlinks);
    let rendered = crop_side_info::render_backlinks(target, &report_path, format)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:backlinks\" target=\"{}\" -->\n{}\n<!-- /proof:compiled -->",
        target, rendered
    ))
}

pub(crate) fn render_links(
    root: &Path,
    side_info: Option<&str>,
    source_doc: &Option<String>,
    status: &str,
    format: &str,
) -> Result<String> {
    let report_path = side_info_path(root, side_info, SideInfoKind::Links);
    let filter = link_filter(source_doc, status)?;
    let rendered = crop_side_info::render_links(&report_path, &filter, format)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:links\" -->\n{}\n<!-- /proof:compiled -->",
        rendered
    ))
}

pub(crate) fn render_headings(
    root: &Path,
    side_info: Option<&str>,
    source_doc: &str,
    format: &str,
) -> Result<String> {
    let report_path = side_info_path(root, side_info, SideInfoKind::Headings);
    let rendered = crop_side_info::render_headings(source_doc, &report_path, format)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:headings\" source=\"{}\" -->\n{}\n<!-- /proof:compiled -->",
        source_doc, rendered
    ))
}

pub(crate) fn render_frontmatter(
    root: &Path,
    side_info: Option<&str>,
    field: &Option<String>,
    value: &Option<String>,
    op: &str,
    format: &str,
) -> Result<String> {
    let report_path = side_info_path(root, side_info, SideInfoKind::Frontmatter);
    let filter = frontmatter_filter(field, value, op)?;
    let rendered = crop_side_info::render_frontmatter(&report_path, &filter, format)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:frontmatter\" -->\n{}\n<!-- /proof:compiled -->",
        rendered
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_backlinks(
    root: &Path,
    side_info: Option<&String>,
    target: &str,
    format: &str,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_backlinks(root, side_info.map(|s| s.as_str()), target, format) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_crop_error(
                "backlinks",
                target.to_string(),
                e,
                line_start,
                source_line_offset,
                violations,
            );
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_links(
    root: &Path,
    side_info: Option<&String>,
    source_doc: &Option<String>,
    status: &str,
    format: &str,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_links(
        root,
        side_info.map(|s| s.as_str()),
        source_doc,
        status,
        format,
    ) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_crop_error(
                "links",
                source_doc.clone().unwrap_or_default(),
                e,
                line_start,
                source_line_offset,
                violations,
            );
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_headings(
    root: &Path,
    side_info: Option<&String>,
    source_doc: &str,
    format: &str,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_headings(root, side_info.map(|s| s.as_str()), source_doc, format) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_crop_error(
                "headings",
                source_doc.to_string(),
                e,
                line_start,
                source_line_offset,
                violations,
            );
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_frontmatter(
    root: &Path,
    side_info: Option<&String>,
    field: &Option<String>,
    value: &Option<String>,
    op: &str,
    format: &str,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_frontmatter(
        root,
        side_info.map(|s| s.as_str()),
        field,
        value,
        op,
        format,
    ) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_crop_error(
                "frontmatter",
                field.clone().unwrap_or_default(),
                e,
                line_start,
                source_line_offset,
                violations,
            );
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

fn push_crop_error(
    kind: &str,
    uri: String,
    error: anyhow::Error,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
) {
    violations.push(CompileViolation {
        code: "COMPILE-002",
        severity: ViolationSeverity::Error,
        uri,
        figure_id: None,
        invariant: String::new(),
        message: format!("{} error: {}", kind, error),
        source_line: line_start + 1 + source_line_offset,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_info_path_uses_explicit_or_default_report_path() {
        let root = Path::new("repo");

        assert_eq!(
            side_info_path(root, Some("reports/links.json"), SideInfoKind::Links),
            PathBuf::from("repo").join("reports").join("links.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Backlinks),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("backlinks.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Headings),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("headings.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Frontmatter),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("frontmatter.json")
        );
    }

    #[test]
    fn validates_crop_side_info_filters() {
        assert!(matches!(
            frontmatter_filter(&Some("tags".to_string()), &Some("guide".to_string()), "has")
                .unwrap()
                .op,
            crop_side_info::FrontmatterMatch::Has
        ));
        assert!(frontmatter_filter(&None, &None, "approx").is_err());

        assert_eq!(
            link_filter(&Some("README.md".to_string()), "broken")
                .unwrap()
                .status,
            Some("broken".to_string())
        );
        assert!(link_filter(&None, "unknown").is_err());
    }
}
