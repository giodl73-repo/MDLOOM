use std::path::Path;

use crate::cache::PathIndex;
use crate::compile_directive::LayoutAttrs;
use crate::compile_format;
use crate::compile_output;
use crate::compile_source;
use crate::compile_types::{CompileViolation, ViolationSeverity};
use crate::config::MdloomConfig;
use crate::layout::{self, extract_content_lines};
use crate::runner::Runner;

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_include(
    uri: &str,
    pin: Option<&String>,
    root: &Path,
    config: &MdloomConfig,
    runner: &Runner,
    path_index: &mut PathIndex,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    if let Some(pin_id) = pin {
        warn_missing_pin(
            uri,
            pin_id,
            config,
            line_start,
            source_line_offset,
            violations,
        );
    }

    match resolve_validated(
        uri,
        root,
        config,
        runner,
        path_index,
        line_start,
        source_line_offset,
        violations,
    ) {
        Ok(content) => {
            *resolved_count += 1;
            compile_format::include_block(uri, &content)
        }
        Err(e) => {
            push_resolve_error(uri, e, line_start, source_line_offset, violations);
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_table(
    uri: &str,
    root: &Path,
    config: &MdloomConfig,
    runner: &Runner,
    path_index: &mut PathIndex,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match resolve_validated(
        uri,
        root,
        config,
        runner,
        path_index,
        line_start,
        source_line_offset,
        violations,
    ) {
        Ok(content) => {
            *resolved_count += 1;
            compile_format::include_block(uri, &content)
        }
        Err(e) => {
            push_resolve_error(uri, e, line_start, source_line_offset, violations);
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_layout(
    uris: &[String],
    attrs: &LayoutAttrs,
    root: &Path,
    config: &MdloomConfig,
    runner: &Runner,
    path_index: &mut PathIndex,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    let mut figures: Vec<Vec<String>> = Vec::new();
    let mut any_err = false;

    for uri in uris {
        match resolve_validated(
            uri,
            root,
            config,
            runner,
            path_index,
            line_start,
            source_line_offset,
            violations,
        ) {
            Ok(content) => {
                figures.push(extract_content_lines(&content));
                *resolved_count += 1;
            }
            Err(e) => {
                push_resolve_error(uri, e, line_start, source_line_offset, violations);
                any_err = true;
            }
        }
    }

    if any_err || figures.is_empty() {
        compile_output::source_fallback(source_lines, line_start, line_end)
    } else {
        let layout_config = attrs.to_layout_config();
        let composed = layout::layout(figures, &layout_config);
        let inner = composed
            .strip_prefix("```\n")
            .and_then(|s| s.strip_suffix("\n```"))
            .unwrap_or(&composed);
        compile_format::layout_block(uris, inner)
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_validated(
    uri: &str,
    root: &Path,
    config: &MdloomConfig,
    runner: &Runner,
    path_index: &mut PathIndex,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
) -> anyhow::Result<String> {
    let (content, fig_file) = compile_source::resolve_uri_cached(uri, root, path_index)?;
    crate::compile_validation::lint_figure(
        uri,
        &content,
        &fig_file,
        line_start + 1 + source_line_offset,
        runner,
        violations,
    );
    crate::compile_validation::validate_davinci(
        uri,
        &content,
        config,
        line_start + source_line_offset,
        violations,
    );
    Ok(content)
}

fn warn_missing_pin(
    uri: &str,
    pin_id: &str,
    config: &MdloomConfig,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
) {
    let has_pin = config.davinci.iter().any(|d| d.id == pin_id);
    if has_pin {
        return;
    }
    violations.push(CompileViolation {
        code: "COMPILE-007",
        severity: ViolationSeverity::Warning,
        uri: uri.to_string(),
        figure_id: Some(pin_id.to_string()),
        invariant: String::new(),
        message: format!(
            "Figure '{}' declares pin={:?} but no [[davinci]] entry with that ID exists — run `mdloom pin {} --id {}`",
            uri, pin_id, uri, pin_id
        ),
        source_line: line_start + 1 + source_line_offset,
    });
}

fn push_resolve_error(
    uri: &str,
    error: anyhow::Error,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
) {
    violations.push(CompileViolation {
        code: "COMPILE-002",
        severity: ViolationSeverity::Error,
        uri: uri.to_string(),
        figure_id: None,
        invariant: String::new(),
        message: format!("{}", error),
        source_line: line_start + 1 + source_line_offset,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_pin_missing_in_config_emits_compile_007() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("myfig.md"), "```\ncontent\n```\n").unwrap();

        let cfg = MdloomConfig::default();
        let runner = Runner::new(dir.path(), cfg.clone()).expect("runner");
        let mut path_index = PathIndex::new();
        let mut violations = Vec::new();
        let source_lines = vec!["```mdloom:include pin=my-pin", "md://myfig.md", "```"];
        let mut resolved_count = 0usize;

        let rendered = compile_include(
            "md://myfig.md",
            Some(&"my-pin".to_string()),
            dir.path(),
            &cfg,
            &runner,
            &mut path_index,
            0,
            2,
            0,
            &source_lines,
            &mut violations,
            &mut resolved_count,
        );

        assert!(rendered.contains("content"), "include should resolve");
        assert_eq!(resolved_count, 1);
        assert!(
            violations.iter().any(|v| v.code == "COMPILE-007"),
            "missing DaVinci pin should emit COMPILE-007, got: {:?}",
            violations.iter().map(|v| v.code).collect::<Vec<_>>()
        );
    }
}
