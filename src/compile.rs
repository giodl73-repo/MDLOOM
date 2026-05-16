use anyhow::Result;
use std::path::Path;

use crate::compile_cache;
use crate::compile_chart;
use crate::compile_crop;
use crate::compile_directive;
use crate::compile_figure;
use crate::compile_math;
use crate::compile_output;
use crate::compile_prose;
use crate::compile_symbol;
use crate::compile_toc;
use crate::compile_tree;
pub use crate::compile_types::{CompileResult, CompileViolation, ViolationSeverity};
use crate::config::GlintConfig;
use crate::runner::Runner;

use crate::compile_directive::{collect_directives, Directive};

pub fn parse_directives(source: &str) -> Vec<(usize, usize, String, String)> {
    compile_directive::parse_directives(source)
}

pub use compile_output::derive_output_path;

// ─────────────────────────────────────────────────────────

pub fn compile_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &GlintConfig,
) -> Result<CompileResult> {
    // Dispatch: .slides.source.md files use the slide compositor.
    if source_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".slides.source.md"))
        .unwrap_or(false)
    {
        return crate::compile_slides::compile_slides_file(source_path, output_path);
    }

    // Dispatch: .dashboard.source.md files use the canvas-based dashboard compiler.
    if source_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".dashboard.source.md"))
        .unwrap_or(false)
    {
        return crate::compile_dashboard::compile_dashboard_file(
            source_path,
            output_path,
            root,
            config,
        );
    }

    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;
    let (_, source_body, source_line_offset) = compile_output::split_frontmatter(&source_text);
    let compile_attrs = format!(r#"{{"frontmatter_offset":{}}}"#, source_line_offset);
    let directives = collect_directives(source_body);

    let mut path_index = crate::cache::load_path_index(root);
    let resolved_files = compile_crop::side_info_dependencies(&directives, root);
    let dependency_parse_keys =
        compile_crop::dependency_parse_keys(&resolved_files, &mut path_index);

    if let Some(result) = compile_cache::restore_compile_cache(
        root,
        source_path,
        output_path,
        &source_text,
        &compile_attrs,
        &resolved_files,
        &dependency_parse_keys,
        &mut path_index,
    )? {
        return Ok(result);
    }

    let source_lines: Vec<&str> = source_body.lines().collect();

    // Build a runner for figure lint validation
    let runner = Runner::new(root, config.clone())?;

    let mut violations: Vec<CompileViolation> = Vec::new();
    let mut resolved_count = 0usize;

    // (line_start, line_end, replacement_text)
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for directive in &directives {
        let line_start = directive.line_start();
        let line_end = directive.line_end();

        let replacement = match directive {
            Directive::Include { uri, pin, .. } => compile_figure::compile_include(
                uri,
                pin.as_ref(),
                root,
                config,
                &runner,
                &mut path_index,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Layout { uris, attrs, .. } => compile_figure::compile_layout(
                uris,
                attrs,
                root,
                config,
                &runner,
                &mut path_index,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Table { uri, .. } => compile_figure::compile_table(
                uri,
                root,
                config,
                &runner,
                &mut path_index,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Tree {
                kind,
                source,
                inline_body,
                attrs,
                ..
            } => compile_tree::compile_tree(
                kind,
                source.as_ref(),
                inline_body,
                attrs,
                root,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Element {
                kind,
                source,
                field,
                inline_value,
                attrs,
                ..
            } => crate::compile_element::compile_element(
                kind,
                source.as_deref(),
                field.as_deref(),
                inline_value.as_deref(),
                attrs,
                root,
                line_start + source_line_offset,
                &mut violations,
                &source_lines,
                line_end,
                &mut resolved_count,
            ),

            Directive::Row {
                source_uri,
                var_name: _,
                separator,
                declared_width,
                elements,
                no_chrome,
                ..
            } => crate::compile_element::compile_row(
                source_uri,
                separator,
                *declared_width,
                elements,
                *no_chrome,
                root,
                line_start + source_line_offset,
                &mut violations,
                &source_lines,
                line_end,
                &mut resolved_count,
            ),

            Directive::Symbol {
                name,
                size,
                align: _,
                ..
            } => compile_symbol::compile_symbol(
                name,
                *size,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Shape { attrs, .. } => compile_symbol::compile_shape(
                attrs,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Region { name, .. } => crate::compile_region::compile_invalid_region(
                name,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
            ),

            Directive::Math {
                expr,
                width,
                align,
                no_chrome,
                ..
            } => compile_math::compile_math(
                expr,
                *width,
                *align,
                *no_chrome,
                line_start,
                source_line_offset,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Toc {
                source,
                max_depth,
                style,
                section,
                ..
            } => compile_toc::compile_toc(
                source.as_ref(),
                *max_depth,
                style,
                section.as_ref(),
                root,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Xref {
                uri, label, format, ..
            } => compile_prose::compile_xref(
                uri,
                label.as_ref(),
                format,
                root,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Blockquote {
                text,
                attribution,
                style,
                ..
            } => compile_prose::compile_blockquote(
                text,
                attribution.as_ref(),
                style,
                &mut resolved_count,
            ),

            Directive::Backlinks {
                target,
                source,
                format,
                ..
            } => compile_crop::compile_backlinks(
                root,
                source.as_ref(),
                target,
                format,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),
            Directive::Links {
                source_doc,
                status,
                source,
                format,
                ..
            } => compile_crop::compile_links(
                root,
                source.as_ref(),
                source_doc,
                status,
                format,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),
            Directive::Headings {
                source_doc,
                source,
                format,
                ..
            } => compile_crop::compile_headings(
                root,
                source.as_ref(),
                source_doc,
                format,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),
            Directive::Frontmatter {
                field,
                value,
                op,
                source,
                format,
                ..
            } => compile_crop::compile_frontmatter(
                root,
                source.as_ref(),
                field,
                value,
                op,
                format,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),

            Directive::Chart {
                attrs,
                source,
                label_field,
                value_field,
                inline_body,
                ..
            } => compile_chart::compile_chart(
                attrs,
                source.as_ref(),
                label_field.as_ref(),
                value_field.as_ref(),
                inline_body,
                root,
                line_start,
                line_end,
                source_line_offset,
                &source_lines,
                &mut violations,
                &mut resolved_count,
            ),
        };

        replacements.push((line_start, line_end, replacement));
    }

    // Collect all error-level violations
    let has_errors = violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error);
    if has_errors {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files,
            written: false,
        });
    }

    // Rebuild source with replacements applied, preserving trailing newline
    let had_trailing_newline = source_body.ends_with('\n');
    let mut output_text = compile_output::apply_replacements(&source_lines, &replacements);
    if had_trailing_newline && !output_text.ends_with('\n') {
        output_text.push('\n');
    }

    compile_output::atomic_write(output_path, &output_text)?;
    compile_cache::store_compile_cache(
        root,
        source_path,
        output_path,
        &source_text,
        &output_text,
        &compile_attrs,
        &resolved_files,
        &dependency_parse_keys,
        resolved_count,
        &mut path_index,
    );

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: resolved_count,
        violations,
        from_cache: false,
        resolved_files,
        written: true,
    })
}

// ─────────────────────────────────────────────────────────
// Output formatting with traceability
// ─────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_pin_missing_in_config_emits_compile_007() {
        let dir = tempfile::tempdir().unwrap();
        // Write the figure file the source will include
        std::fs::write(dir.path().join("myfig.md"), "```\ncontent\n```\n").unwrap();

        let src_path = dir.path().join("test.source.md");
        let out_path = dir.path().join("test.md");
        let src = "```proof:include pin=my-pin\nmd://myfig.md\n```\n";
        std::fs::write(&src_path, src).unwrap();

        let cfg = GlintConfig::default(); // no davinci entries
        let result = compile_file(&src_path, &out_path, dir.path(), &cfg).expect("compile ok");

        assert!(
            result.violations.iter().any(|v| v.code == "COMPILE-007"),
            "missing DaVinci pin should emit COMPILE-007, got: {:?}",
            result.violations.iter().map(|v| v.code).collect::<Vec<_>>()
        );
    }
}
