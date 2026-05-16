use std::path::Path;

use crate::compile::{CompileViolation, ViolationSeverity};
use crate::compile_chart;
use crate::compile_directive::{collect_directives, Directive, ElementAttrs};
use crate::compile_math;
use crate::compile_source;
use crate::compile_symbol;
use crate::compile_tree;
use crate::compile_validation::{lint_figure, validate_davinci};
use crate::config::GlintConfig;
use crate::layout::extract_content_lines;
use crate::runner::Runner;

/// Render the body of a proof:region directive: literal lines kept verbatim,
/// directive lines dispatched through per-directive renderers with no-chrome
/// implied so the canvas paste sees raw glyphs only.
pub(crate) fn render_region_body(
    body: &[String],
    root: &Path,
    config: &GlintConfig,
    runner: &Runner,
    abs_line: usize,
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line = &body[i];
        if let Some(header) = top_level_region_directive_header(line) {
            let mut j = i + 1;
            while j < body.len() && top_level_region_directive_header(&body[j]).is_none() {
                j += 1;
            }
            let body_slice: Vec<String> = body[i + 1..j].to_vec();
            let synth = if body_slice.is_empty() {
                format!("```{}\n```", header)
            } else {
                format!("```{}\n{}\n```", header, body_slice.join("\n"))
            };
            let nested = collect_directives(&synth);
            if let Some(directive) = nested.into_iter().next() {
                let rendered = render_one_directive_no_chrome(
                    &directive,
                    root,
                    config,
                    runner,
                    abs_line + i,
                    violations,
                    resolved_count,
                );
                for rline in rendered.lines() {
                    output.push(rline.to_string());
                }
            } else {
                output.push(line.clone());
                for b in &body_slice {
                    output.push(b.clone());
                }
            }
            i = j;
        } else {
            output.push(line.clone());
            i += 1;
        }
    }
    output
}

fn top_level_region_directive_header(line: &str) -> Option<&str> {
    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }
    const HEADERS: &[&str] = &[
        "proof:element",
        "proof:tree",
        "proof:chart",
        "proof:row",
        "proof:symbol",
        "proof:shape",
        "proof:bullets",
        "proof:centered",
        "proof:stat",
    ];
    for h in HEADERS {
        if line.starts_with(h) {
            let next = line.as_bytes().get(h.len()).copied();
            if next.is_none() || next == Some(b' ') || next == Some(b'\t') {
                return Some(line);
            }
        }
    }
    None
}

/// Render a single directive with `no-chrome` semantics — strips the
/// traceability HTML comments and the surrounding fence so the canvas
/// paste sees raw glyph rows. Returns the inner text (may be multi-line).
pub(crate) fn render_one_directive_no_chrome(
    directive: &Directive,
    root: &Path,
    config: &GlintConfig,
    runner: &Runner,
    abs_line: usize,
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    let line_start = directive.line_start();
    match directive {
        Directive::Symbol { name, size, .. } => match compile_symbol::render_symbol(name, *size) {
            Ok(rendered) => {
                *resolved_count += 1;
                rendered
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: e.code,
                    severity: if e.is_warning {
                        ViolationSeverity::Warning
                    } else {
                        ViolationSeverity::Error
                    },
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: e.message,
                    source_line: abs_line + 1,
                });
                String::new()
            }
        },
        Directive::Shape { attrs, .. } => match compile_symbol::render_shape_inline(attrs) {
            Ok(rendered) => {
                *resolved_count += 1;
                rendered
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: e.code,
                    severity: if e.is_warning {
                        ViolationSeverity::Warning
                    } else {
                        ViolationSeverity::Error
                    },
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: e.message,
                    source_line: abs_line + 1,
                });
                String::new()
            }
        },
        Directive::Element {
            kind,
            source,
            field,
            inline_value,
            attrs,
            ..
        } => {
            // Force no-chrome regardless of what the author wrote
            let attrs = ElementAttrs {
                width: attrs.width,
                align: attrs.align.clone(),
                format: attrs.format.clone(),
                no_chrome: true,
                max: attrs.max,
                fill: attrs.fill,
                empty: attrs.empty,
            };
            // compile_element returns the rendered text directly when no_chrome=true
            let dummy_src_lines: Vec<&str> = Vec::new();
            crate::compile_element::compile_element(
                kind,
                source.as_deref(),
                field.as_deref(),
                inline_value.as_deref(),
                &attrs,
                root,
                line_start,
                violations,
                &dummy_src_lines,
                line_start,
                resolved_count,
            )
        }
        Directive::Row {
            source_uri,
            separator,
            declared_width,
            elements,
            ..
        } => {
            let dummy_src_lines: Vec<&str> = Vec::new();
            crate::compile_element::compile_row(
                source_uri,
                separator,
                *declared_width,
                elements,
                /* no_chrome = */ true,
                root,
                line_start,
                violations,
                &dummy_src_lines,
                line_start,
                resolved_count,
            )
        }
        Directive::Tree {
            kind,
            source,
            inline_body,
            attrs,
            ..
        } => {
            let mut tree_warnings = Vec::new();
            match compile_tree::generate_tree_block(
                kind,
                source.as_deref(),
                inline_body,
                attrs,
                root,
                line_start,
                &mut tree_warnings,
            ) {
                Ok(block) => {
                    *resolved_count += 1;
                    for warning in tree_warnings {
                        violations.push(CompileViolation {
                            code: warning.code,
                            severity: ViolationSeverity::Warning,
                            uri: String::new(),
                            figure_id: None,
                            invariant: String::new(),
                            message: warning.message,
                            source_line: warning.source_line,
                        });
                    }
                    strip_compiled_chrome(&block)
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: source.clone().unwrap_or_default(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("tree generation failed: {}", e),
                        source_line: abs_line + 1,
                    });
                    String::new()
                }
            }
        }
        Directive::Include { uri, .. } => match compile_source::resolve_uri(uri, root) {
            Ok((content, fig_file)) => {
                lint_figure(uri, &content, &fig_file, abs_line + 1, runner, violations);
                validate_davinci(uri, &content, config, abs_line, violations);
                *resolved_count += 1;
                extract_content_lines(&content).join("\n")
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: uri.clone(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("{}", e),
                    source_line: abs_line + 1,
                });
                String::new()
            }
        },
        Directive::Chart {
            attrs,
            source,
            label_field,
            value_field,
            inline_body,
            ..
        } => {
            let data_result = compile_chart::resolve_chart_data(
                source.as_deref(),
                label_field.as_deref(),
                value_field.as_deref(),
                inline_body,
                root,
            );
            match data_result {
                Ok(data) => match crate::chart::render_chart(&data, attrs) {
                    Ok(lines) => {
                        *resolved_count += 1;
                        lines.join("\n")
                    }
                    Err(e) => {
                        violations.push(CompileViolation {
                            code: e.code,
                            severity: ViolationSeverity::Error,
                            uri: source.clone().unwrap_or_default(),
                            figure_id: None,
                            invariant: String::new(),
                            message: e.message,
                            source_line: abs_line + 1,
                        });
                        String::new()
                    }
                },
                Err(msg) => {
                    violations.push(CompileViolation {
                        code: "CHART-002",
                        severity: ViolationSeverity::Error,
                        uri: source.clone().unwrap_or_default(),
                        figure_id: None,
                        invariant: String::new(),
                        message: msg,
                        source_line: abs_line + 1,
                    });
                    String::new()
                }
            }
        }
        Directive::Math {
            expr, width, align, ..
        } => {
            let rendered = compile_math::render_math_inline(expr, *width, *align);
            *resolved_count += 1;
            for d in &rendered.diagnostics {
                violations.push(CompileViolation {
                    code: d.code,
                    severity: ViolationSeverity::Warning,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: d.message.clone(),
                    source_line: abs_line + 1,
                });
            }
            rendered.block
        }
        // Layout, Table, Region, Toc, Xref, Blockquote not supported inline within a region.
        // (They produce wrapper chrome / external content unsuited to canvas paste.)
        _ => String::new(),
    }
}

/// Strip `<!-- proof:compiled ... -->` HTML chrome and outer ``` fence from
/// a rendered block, returning only the inner text rows.
pub(crate) fn strip_compiled_chrome(block: &str) -> String {
    let mut lines: Vec<&str> = block.lines().collect();
    // Drop leading "<!-- proof:compiled ... -->" lines
    while lines
        .first()
        .map(|l| l.trim_start().starts_with("<!-- proof:compiled"))
        .unwrap_or(false)
    {
        lines.remove(0);
    }
    // Drop trailing "<!-- /proof:compiled -->" lines
    while lines
        .last()
        .map(|l| l.trim_start().starts_with("<!-- /proof:compiled"))
        .unwrap_or(false)
    {
        lines.pop();
    }
    // Drop a single outer ```...``` fence pair if present
    if lines
        .first()
        .map(|l| l.trim_start().starts_with("```"))
        .unwrap_or(false)
    {
        lines.remove(0);
    }
    if lines.last().map(|l| l.trim() == "```").unwrap_or(false) {
        lines.pop();
    }
    lines.join("\n")
}
