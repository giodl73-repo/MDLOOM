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

    // ── inline tree rendering: 2-space-indent nesting (issue #2) ─────────

    #[test]
    fn inline_tree_two_space_indent_renders_nested() {
        // Reproduces issue #2: 2-space-indented children under a `- foo` parent
        // must produce a nested tree, not flatten everything to siblings.
        let body = "root: Plugin runtime channels\n\
                    - Typed plugin hooks\n  \
                    - File: src/plugins/hook-types.ts\n  \
                    - Style: pull, modify\n\
                    - Diagnostic event stream\n  \
                    - File: src/infra/diagnostic-events.ts\n  \
                    - Style: push, observe";
        let out = compile_tree::render_inline_tree(body, 4).expect("render must succeed");
        // Root.
        assert!(
            out.starts_with("Plugin runtime channels"),
            "root must come first: {}",
            out
        );
        // First-level children: Typed plugin hooks (Tee), Diagnostic event stream (Corner).
        assert!(
            out.contains("├── Typed plugin hooks"),
            "first parent should be Tee:\n{}",
            out
        );
        assert!(
            out.contains("└── Diagnostic event stream"),
            "second parent should be Corner:\n{}",
            out
        );
        // Children must be indented under their parent and use Tee/Corner correctly.
        assert!(
            out.contains("│   ├── File: src/plugins/hook-types.ts"),
            "first parent's first child should be Tee under │:\n{}",
            out
        );
        assert!(
            out.contains("│   └── Style: pull, modify"),
            "first parent's last child should be Corner under │:\n{}",
            out
        );
        assert!(
            out.contains("    ├── File: src/infra/diagnostic-events.ts"),
            "second parent's first child should be Tee under spaces:\n{}",
            out
        );
        assert!(
            out.contains("    └── Style: push, observe"),
            "second parent's last child should be Corner under spaces:\n{}",
            out
        );
    }

    #[test]
    fn inline_tree_four_space_indent_also_nests() {
        // Auto-detect: 4-space input should also nest correctly.
        let body = "root: Top\n\
                    - One\n    \
                    - One.A\n\
                    - Two";
        let out = compile_tree::render_inline_tree(body, 4).unwrap();
        assert!(out.contains("├── One"), "got:\n{}", out);
        assert!(out.contains("│   └── One.A"), "got:\n{}", out);
        assert!(out.contains("└── Two"), "got:\n{}", out);
    }

    #[test]
    fn inline_tree_last_sibling_uses_corner() {
        // The buggy is_last logic flagged last-of-non-last-parent as Tee instead of Corner.
        let body = "root: R\n- A\n  - A1\n  - A2\n- B";
        let out = compile_tree::render_inline_tree(body, 4).unwrap();
        // A2 is last child of A → must be Corner.
        assert!(
            out.contains("│   └── A2"),
            "last child A2 should be Corner under non-last parent A:\n{}",
            out
        );
    }

    // ── inline outline: dash-bullet detection + auto-promote (issue #1) ──

    #[test]
    fn inline_outline_dash_bullets_warn_and_promote() {
        let body = "root: Plugin lifecycle\n- 1 Discovery\n- 2 Manifest read\n- 3 Activation";
        let mut warnings = Vec::new();
        let out =
            compile_tree::render_inline_outline(body, 4, 1, &mut warnings).expect("must render");
        // Warning emitted with TREE-001 code.
        assert_eq!(
            warnings.len(),
            1,
            "expected exactly one warning, got {:?}",
            warnings.iter().map(|v| v.code).collect::<Vec<_>>()
        );
        assert_eq!(warnings[0].code, "TREE-001");
        assert!(
            warnings[0].message.contains("kind=taxonomy"),
            "warning should suggest kind=taxonomy: {}",
            warnings[0].message
        );
        // Body promoted to a rendered tree, not emitted verbatim.
        assert!(
            out.contains("├──") || out.contains("└──"),
            "must contain tree connectors:\n{}",
            out
        );
        assert!(
            out.contains("Plugin lifecycle"),
            "root must appear:\n{}",
            out
        );
    }

    #[test]
    fn inline_outline_no_bullets_no_warning() {
        // Numbered/heading content should NOT warn.
        let body = "1. First\n1.1 Sub\n2. Second";
        let mut warnings = Vec::new();
        let out = compile_tree::render_inline_outline(body, 4, 1, &mut warnings).unwrap();
        assert!(
            warnings.is_empty(),
            "no warnings expected for numbered body"
        );
        assert!(out.contains("1. First"));
    }

    // ── inline outline: numbered-bullet auto-indent (task #2) ────────────

    #[test]
    fn inline_outline_numbered_bullets_auto_indent() {
        // Author types unindented numbered bullets — output re-indents by dot depth.
        let body =
            "1. Installation\n1.1 From source\n1.2 From crates.io\n2. Configuration\n2.1 Basics";
        let mut warnings = Vec::new();
        let out = compile_tree::render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        assert!(warnings.is_empty(), "no warnings for numbered input");
        let expected =
            "1. Installation\n   1.1 From source\n   1.2 From crates.io\n2. Configuration\n   2.1 Basics";
        assert_eq!(out, expected, "depth-based indent normalization");
    }

    #[test]
    fn inline_outline_numbered_three_levels() {
        let body = "1. A\n1.1 B\n1.1.1 C\n2. D";
        let mut warnings = Vec::new();
        let out = compile_tree::render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        let expected = "1. A\n   1.1 B\n      1.1.1 C\n2. D";
        assert_eq!(out, expected);
    }

    #[test]
    fn inline_outline_preserves_trailing_period_only_at_depth_zero() {
        // "1." stays "1.", but "1.1." normalizes to "1.1" (no trailing dot at depth ≥ 1).
        let body = "1. A\n1.1. B";
        let mut warnings = Vec::new();
        let out = compile_tree::render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        assert!(out.contains("1. A"));
        assert!(
            out.contains("1.1 B"),
            "trailing period dropped at depth 1: got\n{}",
            out
        );
        assert!(
            !out.contains("1.1. B"),
            "trailing period must not survive: got\n{}",
            out
        );
    }

    #[test]
    fn inline_outline_unnumbered_line_passes_through() {
        // Lines without a numeric prefix are emitted verbatim at depth 0.
        let body = "Project plan:\n1. Phase one\n1.1 Step";
        let mut warnings = Vec::new();
        let out = compile_tree::render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        assert!(
            out.starts_with("Project plan:"),
            "header preserved at top:\n{}",
            out
        );
        assert!(
            out.contains("\n1. Phase one"),
            "depth-0 numbered line at column 0:\n{}",
            out
        );
        assert!(
            out.contains("\n   1.1 Step"),
            "depth-1 numbered line indented:\n{}",
            out
        );
    }
}
