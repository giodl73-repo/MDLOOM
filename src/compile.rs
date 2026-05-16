use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::compile_chart;
use crate::compile_crop;
use crate::compile_directive;
use crate::compile_format;
use crate::compile_math;
use crate::compile_prose;
use crate::compile_source;
use crate::compile_symbol;
use crate::compile_toc;
use crate::compile_tree;
use crate::config::GlintConfig;
use crate::davinci::evaluate_invariant;
use crate::diagnostic::Severity;
#[cfg(test)]
use crate::element::{ElementAlign, ElementKind};
use crate::layout::{self, extract_content_lines, Align, Direction, LayoutConfig};
use crate::runner::Runner;

// ─────────────────────────────────────────────────────────
// Public result types
// ─────────────────────────────────────────────────────────

pub struct CompileResult {
    pub output_path: PathBuf,
    pub directives_resolved: usize,
    pub violations: Vec<CompileViolation>,
    pub from_cache: bool,
    pub written: bool,
    /// Files resolved during compilation (for watch-mode dependency tracking)
    pub resolved_files: Vec<PathBuf>,
}

pub struct CompileViolation {
    pub code: &'static str,
    pub severity: ViolationSeverity,
    pub uri: String,
    pub figure_id: Option<String>,
    pub invariant: String,
    pub message: String,
    pub source_line: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ViolationSeverity {
    Error,
    Warning,
}

// ─────────────────────────────────────────────────────────
// Directive parser facade
// ─────────────────────────────────────────────────────────

use crate::compile_directive::{collect_directives, Directive, ElementAttrs};
#[cfg(test)]
use crate::compile_directive::{proof_directive_kind, LayoutAttrs};

pub fn parse_directives(source: &str) -> Vec<(usize, usize, String, String)> {
    compile_directive::parse_directives(source)
}

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
        return compile_dashboard_file(source_path, output_path, root, config);
    }

    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;
    let (_, source_body, source_line_offset) = split_frontmatter(&source_text);
    let compile_attrs = format!(r#"{{"frontmatter_offset":{}}}"#, source_line_offset);
    let directives = collect_directives(source_body);

    // ── Tier 3 cache check ──────────────────────────────────────────────
    // Build a minimal directive-attrs JSON for cache keying, then check Tier 3.
    // On hit: write cached output (skip if identical), return early with from_cache=true.
    let mut path_index = crate::cache::load_path_index(root);
    let resolved_files = compile_crop::side_info_dependencies(&directives, root);
    let dependency_parse_keys =
        compile_crop::dependency_parse_keys(&resolved_files, &mut path_index);
    {
        let source_parse_key =
            crate::cache::get_or_compute_parse_key(source_path, &source_text, &mut path_index);
        let cache_key =
            crate::cache::compile_key(&source_parse_key, &dependency_parse_keys, &compile_attrs);
        if let Some(entry) = crate::cache::load_compile_cache(root, &cache_key) {
            let current = std::fs::read_to_string(output_path).unwrap_or_default();
            let written = current != entry.compiled_text;
            if written {
                let tmp = output_path.with_extension("proof_tmp");
                let _ = std::fs::write(&tmp, &entry.compiled_text);
                let _ = std::fs::rename(&tmp, output_path);
            }
            crate::cache::save_path_index(root, &path_index);
            return Ok(CompileResult {
                output_path: output_path.to_path_buf(),
                directives_resolved: entry.directives_resolved,
                violations: vec![],
                from_cache: true,
                resolved_files,
                written,
            });
        }
    }
    // ────────────────────────────────────────────────────────────────────

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
            Directive::Include { uri, pin, .. } => {
                // If pin=id is declared inline, warn when no matching [[davinci]] entry exists.
                if let Some(pin_id) = pin {
                    let has_pin = config.davinci.iter().any(|d| &d.id == pin_id);
                    if !has_pin {
                        violations.push(CompileViolation {
                            code: "COMPILE-007",
                            severity: ViolationSeverity::Warning,
                            uri: uri.clone(),
                            figure_id: Some(pin_id.clone()),
                            invariant: String::new(),
                            message: format!(
                                "Figure '{}' declares pin={:?} but no [[davinci]] entry with that ID exists — run `proof pin {} --id {}`",
                                uri, pin_id, uri, pin_id
                            ),
                            source_line: line_start + 1 + source_line_offset,
                        });
                    }
                }
                match compile_source::resolve_uri_cached(uri, root, &mut path_index) {
                    Ok((content, fig_file)) => {
                        lint_figure(
                            uri,
                            &content,
                            &fig_file,
                            line_start + 1 + source_line_offset,
                            &runner,
                            &mut violations,
                        );
                        validate_davinci(
                            uri,
                            &content,
                            config,
                            line_start + source_line_offset,
                            &mut violations,
                        );
                        resolved_count += 1;
                        compile_format::include_block(uri, &content)
                    }
                    Err(e) => {
                        violations.push(CompileViolation {
                            code: "COMPILE-002",
                            severity: ViolationSeverity::Error,
                            uri: uri.clone(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("{}", e),
                            source_line: line_start + 1 + source_line_offset,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    }
                }
            }

            Directive::Layout { uris, attrs, .. } => {
                let mut figures: Vec<Vec<String>> = Vec::new();
                let mut any_err = false;
                for uri in uris {
                    match compile_source::resolve_uri_cached(uri, root, &mut path_index) {
                        Ok((content, fig_file)) => {
                            lint_figure(
                                uri,
                                &content,
                                &fig_file,
                                line_start + 1 + source_line_offset,
                                &runner,
                                &mut violations,
                            );
                            validate_davinci(
                                uri,
                                &content,
                                config,
                                line_start + source_line_offset,
                                &mut violations,
                            );
                            figures.push(extract_content_lines(&content));
                            resolved_count += 1;
                        }
                        Err(e) => {
                            violations.push(CompileViolation {
                                code: "COMPILE-002",
                                severity: ViolationSeverity::Error,
                                uri: uri.clone(),
                                figure_id: None,
                                invariant: String::new(),
                                message: format!("{}", e),
                                source_line: line_start + 1 + source_line_offset,
                            });
                            any_err = true;
                        }
                    }
                }

                if any_err || figures.is_empty() {
                    source_lines[line_start..=line_end].join("\n")
                } else {
                    // Convert attrs directly to LayoutConfig — no re-serialization to avoid
                    // label corruption (labels with spaces would be split by the string parser)
                    let layout_config = LayoutConfig {
                        gap: attrs.gap,
                        align: Align::parse(&attrs.align).unwrap_or(Align::Top),
                        labels: attrs.labels.clone(),
                        cols: attrs.cols,
                        width: attrs.width,
                        direction: Direction::parse(&attrs.direction)
                            .unwrap_or(Direction::Horizontal),
                        border: attrs.border,
                    };

                    let composed = layout::layout(figures, &layout_config);
                    // Strip outer ``` wrapper — compile embeds content inline
                    let inner = composed
                        .strip_prefix("```\n")
                        .and_then(|s| s.strip_suffix("\n```"))
                        .unwrap_or(&composed);
                    compile_format::layout_block(uris, inner)
                }
            }

            Directive::Table { uri, .. } => {
                match compile_source::resolve_uri_cached(uri, root, &mut path_index) {
                    Ok((content, fig_file)) => {
                        lint_figure(
                            uri,
                            &content,
                            &fig_file,
                            line_start + 1 + source_line_offset,
                            &runner,
                            &mut violations,
                        );
                        validate_davinci(
                            uri,
                            &content,
                            config,
                            line_start + source_line_offset,
                            &mut violations,
                        );
                        resolved_count += 1;
                        compile_format::include_block(uri, &content)
                    }
                    Err(e) => {
                        violations.push(CompileViolation {
                            code: "COMPILE-002",
                            severity: ViolationSeverity::Error,
                            uri: uri.clone(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("{}", e),
                            source_line: line_start + 1 + source_line_offset,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    }
                }
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
                    line_start + source_line_offset,
                    &mut tree_warnings,
                ) {
                    Ok(block) => {
                        resolved_count += 1;
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
                        block
                    }
                    Err(e) => {
                        // stub=true: WIP directive — downgrade error to warning, keep source block
                        let severity = if attrs.stub {
                            ViolationSeverity::Warning
                        } else {
                            ViolationSeverity::Error
                        };
                        violations.push(CompileViolation {
                            code: "COMPILE-002",
                            severity,
                            uri: source.clone().unwrap_or_default(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!(
                                "tree generation failed: {}{}",
                                e,
                                if attrs.stub {
                                    " (stub — skipped)"
                                } else {
                                    ""
                                }
                            ),
                            source_line: line_start + 1 + source_line_offset,
                        });
                        source_fallback(&source_lines, line_start, line_end)
                    }
                }
            }

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
            } => match compile_symbol::render_symbol_compiled(name, *size) {
                Ok(rendered) => {
                    resolved_count += 1;
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
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_lines[line_start..=line_end].join("\n")
                }
            },

            Directive::Shape { attrs, .. } => match compile_symbol::render_shape_compiled(attrs) {
                Ok(rendered) => {
                    resolved_count += 1;
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
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_lines[line_start..=line_end].join("\n")
                }
            },

            Directive::Region { name, .. } => {
                // proof:region is only valid inside a .dashboard.source.md file —
                // and dashboard files are routed through compile_dashboard_file before
                // this loop is ever reached. Reaching this branch means the directive
                // appeared in a non-dashboard file.
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!(
                        "proof:region {:?} is only valid in .dashboard.source.md files",
                        name
                    ),
                    source_line: line_start + 1 + source_line_offset,
                });
                source_lines[line_start..=line_end].join("\n")
            }

            Directive::Math {
                expr,
                width,
                align,
                no_chrome,
                ..
            } => {
                let rendered = compile_math::render_math_compiled(expr, *width, *align, *no_chrome);
                resolved_count += 1;
                for d in &rendered.diagnostics {
                    violations.push(CompileViolation {
                        code: d.code,
                        severity: ViolationSeverity::Warning,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: d.message.clone(),
                        source_line: line_start + 1 + source_line_offset,
                    });
                }
                rendered.block
            }

            Directive::Toc {
                source,
                max_depth,
                style,
                section,
                ..
            } => {
                let content_opt: Option<String> = if let Some(uri) = source {
                    match compile_source::resolve_source_for_compile(uri, root) {
                        Ok(c) => Some(c),
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
                        resolved_count += 1;
                        let toc = compile_toc::generate_toc(
                            &content,
                            *max_depth,
                            style,
                            section.as_deref(),
                        );
                        format!(
                            "<!-- proof:compiled from=\"proof:toc\" -->\n{}\n<!-- /proof:compiled -->",
                            toc
                        )
                    }
                    None => source_fallback(&source_lines, line_start, line_end),
                }
            }

            Directive::Xref {
                uri, label, format, ..
            } => match compile_prose::render_xref(uri, label.as_deref(), format, root) {
                Ok(rendered) => {
                    resolved_count += 1;
                    format!(
                        "<!-- proof:compiled from=\"proof:xref\" -->\n{}\n<!-- /proof:compiled -->",
                        rendered
                    )
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: uri.clone(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("xref error: {}", e),
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_fallback(&source_lines, line_start, line_end)
                }
            },

            Directive::Blockquote {
                text,
                attribution,
                style,
                ..
            } => {
                resolved_count += 1;
                let rendered =
                    compile_prose::render_blockquote(text, attribution.as_deref(), style);
                format!(
                    "<!-- proof:compiled from=\"proof:blockquote\" -->\n{}\n<!-- /proof:compiled -->",
                    rendered
                )
            }

            Directive::Backlinks {
                target,
                source,
                format,
                ..
            } => match compile_crop::render_backlinks(root, source.as_deref(), target, format) {
                Ok(rendered) => {
                    resolved_count += 1;
                    rendered
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: target.clone(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("backlinks error: {}", e),
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_fallback(&source_lines, line_start, line_end)
                }
            },
            Directive::Links {
                source_doc,
                status,
                source,
                format,
                ..
            } => match compile_crop::render_links(
                root,
                source.as_deref(),
                source_doc,
                status,
                format,
            ) {
                Ok(rendered) => {
                    resolved_count += 1;
                    rendered
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: source_doc.clone().unwrap_or_default(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("links error: {}", e),
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_fallback(&source_lines, line_start, line_end)
                }
            },
            Directive::Headings {
                source_doc,
                source,
                format,
                ..
            } => match compile_crop::render_headings(root, source.as_deref(), source_doc, format) {
                Ok(rendered) => {
                    resolved_count += 1;
                    rendered
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: source_doc.clone(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("headings error: {}", e),
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_fallback(&source_lines, line_start, line_end)
                }
            },
            Directive::Frontmatter {
                field,
                value,
                op,
                source,
                format,
                ..
            } => match compile_crop::render_frontmatter(
                root,
                source.as_deref(),
                field,
                value,
                op,
                format,
            ) {
                Ok(rendered) => {
                    resolved_count += 1;
                    rendered
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: field.clone().unwrap_or_default(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("frontmatter error: {}", e),
                        source_line: line_start + 1 + source_line_offset,
                    });
                    source_fallback(&source_lines, line_start, line_end)
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
                            resolved_count += 1;
                            let rendered = lines.join("\n");
                            if attrs.no_chrome {
                                format!("```\n{}\n```", rendered)
                            } else {
                                format!(
                                        "<!-- proof:compiled from=\"proof:chart\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
                                        rendered
                                    )
                            }
                        }
                        Err(e) => {
                            violations.push(CompileViolation {
                                code: e.code,
                                severity: ViolationSeverity::Error,
                                uri: source.clone().unwrap_or_default(),
                                figure_id: None,
                                invariant: String::new(),
                                message: e.message,
                                source_line: line_start + 1 + source_line_offset,
                            });
                            source_lines[line_start..=line_end].join("\n")
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
                            source_line: line_start + 1 + source_line_offset,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    }
                }
            }
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
    let mut output_text = apply_replacements(&source_lines, &replacements);
    if had_trailing_newline && !output_text.ends_with('\n') {
        output_text.push('\n');
    }

    // Atomic write: temp-then-rename
    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing temp output {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path)
        .map_err(|e| anyhow::anyhow!("renaming output {}: {}", output_path.display(), e))?;

    // ── Store to Tier 3 cache ───────────────────────────────────────────
    {
        let source_parse_key =
            crate::cache::get_or_compute_parse_key(source_path, &source_text, &mut path_index);
        let cache_key =
            crate::cache::compile_key(&source_parse_key, &dependency_parse_keys, &compile_attrs);
        let entry = crate::cache::CompileCacheEntry {
            compile_key: cache_key,
            source_path: source_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            compiled_text: output_text.clone(),
            resolved_uris: resolved_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            proof_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            directives_resolved: resolved_count,
        };
        crate::cache::save_compile_cache(root, &entry);
        crate::cache::save_path_index(root, &path_index);
    }
    // ────────────────────────────────────────────────────────────────────

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
// URI resolution + figure lint validation
// ─────────────────────────────────────────────────────────

/// Validate figure content with the proof linter before embedding.
/// Emits COMPILE-007 warnings for each lint error found in the figure.
fn lint_figure(
    uri: &str,
    content: &str,
    figure_file: &Path,
    source_line: usize,
    runner: &Runner,
    violations: &mut Vec<CompileViolation>,
) {
    // Build a synthetic file content: wrap content in a fenced block for checking
    let synthetic = format!("```\n{}\n```\n", content);
    let diags = runner.lint_content(&synthetic, figure_file);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    if !errors.is_empty() {
        violations.push(CompileViolation {
            code: "COMPILE-007",
            severity: ViolationSeverity::Warning,
            uri: uri.to_string(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "figure has {} lint error{} — embedded output may be misaligned ({})",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" },
                errors.iter().map(|d| d.code).collect::<Vec<_>>().join(", ")
            ),
            source_line,
        });
    }
}

// ─────────────────────────────────────────────────────────
// DaVinci validation
// ─────────────────────────────────────────────────────────

fn validate_davinci(
    uri: &str,
    content: &str,
    config: &GlintConfig,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
) {
    for entry in &config.davinci {
        // Match by URI or by uri suffix
        let uri_matches = entry.uri == uri || uri.ends_with(&entry.uri) || entry.uri.ends_with(uri);
        if !uri_matches {
            continue;
        }
        for inv in &entry.invariants {
            if let Err(msg) = evaluate_invariant(inv, content) {
                use crate::config::ProtectionTier;
                let (code, sev) = match entry.protection {
                    ProtectionTier::Error | ProtectionTier::Lock => {
                        ("COMPILE-001", ViolationSeverity::Error)
                    }
                    ProtectionTier::Warn => ("COMPILE-003", ViolationSeverity::Warning),
                };
                violations.push(CompileViolation {
                    code,
                    severity: sev,
                    uri: uri.to_string(),
                    figure_id: Some(entry.id.clone()),
                    invariant: format!("{:?}", inv),
                    message: msg,
                    source_line: source_line + 1,
                });
            }
        }
    }
}

// ─────────────────────────────────────────────────────────
// Output formatting with traceability
// ─────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────
// Source reconstruction
// ─────────────────────────────────────────────────────────

fn apply_replacements(source_lines: &[&str], replacements: &[(usize, usize, String)]) -> String {
    if replacements.is_empty() {
        return source_lines.join("\n");
    }

    let mut out: Vec<String> = Vec::new();
    let mut cursor = 0usize;

    for (start, end, replacement) in replacements {
        // Pass through lines before this directive
        for line in &source_lines[cursor..*start] {
            out.push(line.to_string());
        }
        // Insert replacement
        out.push(replacement.clone());
        // Skip over the original directive block (start..=end)
        cursor = end + 1;
    }

    // Trailing lines after last replacement
    for line in &source_lines[cursor..] {
        out.push(line.to_string());
    }

    out.join("\n")
}

// ─────────────────────────────────────────────────────────
// Output path derivation
// ─────────────────────────────────────────────────────────

/// Derive output path from source path.
/// `foo.source.md` → `foo.md` (drops `.source.`).
/// Any other `.md` file → None (require explicit -o).
// ─────────────────────────────────────────────────────────
// proof:element compile arm
// ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
/// Safe fallback: return source lines for the directive block, guarded against OOB.
pub(crate) fn source_fallback(
    source_lines: &[&str],
    source_line: usize,
    line_end: usize,
) -> String {
    if source_line <= line_end && line_end < source_lines.len() {
        source_lines[source_line..=line_end].join("\n")
    } else {
        String::new()
    }
}

// ─────────────────────────────────────────────────────────
// Dashboard compile pipeline
// ─────────────────────────────────────────────────────────
//
// .dashboard.source.md files are routed here from compile_file. The pipeline:
//   1. Read source, split YAML front-matter from body
//   2. Parse front-matter → DashboardMeta + Vec<RegionGeometry>
//   3. Validate region geometry (D-2, D-3) → DASHBOARD-001..003
//   4. For each proof:region directive in body:
//        - look up declared region by name (else DASHBOARD-004)
//        - render body lines (literals verbatim; directives via inner compile pass
//          with no-chrome implied)
//   5. Hand the (meta, regions, content map) to dashboard::region::compile_dashboard
//   6. Write the canvas string to the output path

fn compile_dashboard_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &GlintConfig,
) -> Result<CompileResult> {
    use crate::dashboard::region::{
        compile_dashboard, parse_dashboard_frontmatter, DashboardError, RegionGeometry,
    };

    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let mut violations: Vec<CompileViolation> = Vec::new();
    let mut resolved_count = 0usize;

    // ── 1. Split YAML front-matter from body ──────────────
    let (frontmatter, body, body_offset) = split_frontmatter(&source_text);

    // ── 2. Parse front-matter ─────────────────────────────
    let (meta, regions) = parse_dashboard_frontmatter(&frontmatter);

    // DASHBOARD-006: canvas wider than the standard terminal threshold
    const CANVAS_WARN_WIDTH: usize = 220;
    if meta.width > CANVAS_WARN_WIDTH {
        violations.push(CompileViolation {
            code: "DASHBOARD-006",
            severity: ViolationSeverity::Warning,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "Canvas width {} exceeds terminal threshold {} — reduce or set a --width flag",
                meta.width, CANVAS_WARN_WIDTH
            ),
            source_line: 1,
        });
    }

    // ── 3. Collect proof:region directives from the body ──
    let directives = collect_directives(body);

    // Build runner once for nested figure linting
    let runner = Runner::new(root, config.clone())?;

    // Build a map of region name → declared geometry for quick lookup
    let mut region_by_name: std::collections::HashMap<String, &RegionGeometry> =
        std::collections::HashMap::new();
    for r in &regions {
        region_by_name.insert(r.name.clone(), r);
    }

    // ── 4. Render each proof:region's content ─────────────
    let mut region_contents: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for directive in &directives {
        if let Directive::Region {
            name,
            body,
            line_start,
            ..
        } = directive
        {
            let abs_line = body_offset + line_start;

            // DASHBOARD-004: region name not declared in front-matter
            if !region_by_name.contains_key(name) {
                violations.push(CompileViolation {
                    code: "DASHBOARD-004",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!(
                        "proof:region {:?} has no matching front-matter declaration",
                        name
                    ),
                    source_line: abs_line + 1,
                });
                continue;
            }

            // Render body lines: literals verbatim, directives via inner compile pass
            let rendered = render_region_body(
                body,
                root,
                config,
                &runner,
                abs_line,
                &mut violations,
                &mut resolved_count,
            );
            region_contents.insert(name.clone(), rendered);
        }
    }

    // Stop here on any error before painting the canvas
    let has_errors = violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error);
    if has_errors {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    // ── 5. Composite the canvas ───────────────────────────
    let (canvas_text, dashboard_errors) = compile_dashboard(&meta, &regions, &region_contents);

    for de in dashboard_errors {
        let DashboardError { code, message } = de;
        let severity = match code {
            // 005 (height overflow) is a warning per spec; 001/002/003/004 are errors
            "DASHBOARD-005" => ViolationSeverity::Warning,
            _ => ViolationSeverity::Error,
        };
        violations.push(CompileViolation {
            code,
            severity,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message,
            source_line: 1,
        });
    }

    let has_errors = violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error);
    if has_errors {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    // ── 6. Wrap in fence + traceability comment, write atomically ─
    let title_attr = if meta.title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", meta.title)
    };
    let output_text = format!(
        "<!-- proof:compiled from=\"proof:dashboard\"{} -->\n```dashboard\n{}```\n<!-- /proof:compiled -->\n",
        title_attr, canvas_text
    );

    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing temp output {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path)
        .map_err(|e| anyhow::anyhow!("renaming output {}: {}", output_path.display(), e))?;

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: resolved_count,
        violations,
        from_cache: false,
        resolved_files: vec![],
        written: true,
    })
}

/// Split a `.dashboard.source.md` source into (frontmatter_yaml, body, body_offset_in_lines).
/// Front-matter is the block between the opening `---` on line 0 and the next `---`.
/// If no front-matter is present, returns ("", source, 0).
fn split_frontmatter(source: &str) -> (String, &str, usize) {
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return (String::new(), source, 0);
    }
    // Find closing ---
    let close_idx = match lines.iter().skip(1).position(|l| l.trim() == "---") {
        Some(i) => i + 1,
        None => return (String::new(), source, 0),
    };
    let fm = lines[1..close_idx].join("\n");
    // Compute byte offset to end of closing --- + newline
    let mut byte_offset = 0usize;
    for line in &lines[..=close_idx] {
        byte_offset += line.len() + 1; // +1 for the '\n'
    }
    let byte_offset = byte_offset.min(source.len());
    let body = &source[byte_offset..];
    let body_offset_lines = close_idx + 1;
    (fm, body, body_offset_lines)
}

/// Render the body of a proof:region directive: literal lines kept verbatim,
/// directive lines (proof:element/proof:row/proof:tree/proof:symbol/proof:shape)
/// dispatched through the same per-directive renderers used by compile_file —
/// with `no-chrome` implied so the canvas paste sees raw glyphs only.
fn render_region_body(
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
            // Gobble body lines until the next column-0 directive header or end.
            // Indented proof:* lines stay with the parent (e.g. proof:row + indented
            // proof:element children). Blank and literal lines are also body.
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
                // Couldn't synthesize — fall back to literal lines so the user sees something.
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

/// Return the directive header (trimmed of the leading column-0 anchor) if
/// `line` begins at column 0 with a known proof:* directive name. Returns
/// None for indented lines (they belong to the enclosing directive body) and
/// for plain text. The set must match `classify_region_line`.
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
            // Require word-boundary so e.g. "proof:rowx" doesn't match "proof:row".
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
fn render_one_directive_no_chrome(
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
fn strip_compiled_chrome(block: &str) -> String {
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

pub fn derive_output_path(source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?.to_str()?;
    let parent = source.parent().unwrap_or(Path::new("."));
    // Check longer suffixes before shorter ones (longest-first)
    if let Some(stem) = name.strip_suffix(".slides.source.md") {
        return Some(parent.join(format!("{}.slides.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".dashboard.source.md") {
        return Some(parent.join(format!("{}.dashboard.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".source.md") {
        return Some(parent.join(format!("{}.md", stem)));
    }
    None
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_directives ──────────────────────────────────

    #[test]
    fn test_parse_include_directive() {
        let src = "## Section\n\n```proof:include\nmd://figures/foo.md#:0\n```\n\nAfter.";
        let dirs = parse_directives(src);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].2, "include");
        assert!(dirs[0].3.contains("md://figures/foo.md#:0"));
    }

    #[test]
    fn test_parse_layout_directive() {
        let src = "```proof:layout gap=4 align=top\nmd://a.md#:0\nmd://b.md#:0\n```";
        let dirs = parse_directives(src);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].2, "layout");
        assert!(dirs[0].3.contains("md://a.md#:0"));
        assert!(dirs[0].3.contains("md://b.md#:0"));
    }

    #[test]
    fn test_parse_table_directive() {
        let src = "```proof:table\nmd://data.md#table:0\n```";
        let dirs = parse_directives(src);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].2, "table");
    }

    #[test]
    fn test_parse_no_directives() {
        let src = "# Title\n\nRegular paragraph.\n\n```rust\nfn main() {}\n```";
        let dirs = parse_directives(src);
        assert_eq!(dirs.len(), 0);
    }

    #[test]
    fn test_parse_multiple_directives() {
        let src = "```proof:include\nmd://a.md#:0\n```\n\nBetween.\n\n```proof:include\nmd://b.md#:0\n```";
        let dirs = parse_directives(src);
        assert_eq!(dirs.len(), 2);
    }

    // ── layout_attrs_parse ────────────────────────────────

    #[test]
    fn test_attrs_parse_gap() {
        let attrs = LayoutAttrs::parse("gap=4");
        assert_eq!(attrs.gap, 4);
    }

    #[test]
    fn test_attrs_parse_labels_quoted() {
        let attrs = LayoutAttrs::parse("labels=\"Go,Rust,C#\"");
        assert_eq!(attrs.labels, vec!["Go", "Rust", "C#"]);
    }

    #[test]
    fn test_attrs_parse_border_flag() {
        let attrs = LayoutAttrs::parse("border");
        assert!(attrs.border);
    }

    #[test]
    fn test_attrs_parse_defaults() {
        let attrs = LayoutAttrs::parse("");
        assert_eq!(attrs.gap, 3);
        assert_eq!(attrs.align, "top");
        assert_eq!(attrs.width, 120);
        assert!(!attrs.border);
    }

    #[test]
    fn test_attrs_parse_combined() {
        let attrs = LayoutAttrs::parse("gap=2 align=center cols=3 width=200");
        assert_eq!(attrs.gap, 2);
        assert_eq!(attrs.align, "center");
        assert_eq!(attrs.cols, Some(3));
        assert_eq!(attrs.width, 200);
    }

    // ── derive_output_path ────────────────────────────────

    #[test]
    fn test_derive_output_source_md() {
        let src = Path::new("languages/10-GO.source.md");
        let out = derive_output_path(src).unwrap();
        assert_eq!(out, PathBuf::from("languages/10-GO.md"));
    }

    #[test]
    fn test_derive_output_plain_md_returns_none() {
        let src = Path::new("languages/10-GO.md");
        assert!(derive_output_path(src).is_none());
    }

    #[test]
    fn test_derive_output_root_level() {
        let src = Path::new("overview.source.md");
        let out = derive_output_path(src).unwrap();
        assert_eq!(out, PathBuf::from("overview.md"));
    }

    // ── apply_replacements ────────────────────────────────

    #[test]
    fn test_apply_replacements_single() {
        let lines = vec!["line0", "```proof:include", "md://x", "```", "line4"];
        let replacements = vec![(1, 3, "REPLACED".to_string())];
        let out = apply_replacements(&lines, &replacements);
        assert_eq!(out, "line0\nREPLACED\nline4");
    }

    #[test]
    fn test_apply_replacements_none() {
        let lines = vec!["a", "b", "c"];
        let out = apply_replacements(&lines, &[]);
        assert_eq!(out, "a\nb\nc");
    }

    #[test]
    fn test_apply_replacements_multiple() {
        let lines = vec![
            "before",
            "```proof:include",
            "md://a",
            "```",
            "middle",
            "```proof:include",
            "md://b",
            "```",
            "after",
        ];
        let replacements = vec![
            (1, 3, "A_RESOLVED".to_string()),
            (5, 7, "B_RESOLVED".to_string()),
        ];
        let out = apply_replacements(&lines, &replacements);
        assert_eq!(out, "before\nA_RESOLVED\nmiddle\nB_RESOLVED\nafter");
    }

    // ── format helpers ────────────────────────────────────

    #[test]
    fn test_format_include_block_has_traceability() {
        let out = compile_format::include_block("md://figures/foo.md#:0", "CONTENT\nLINE2");
        assert!(out.contains("<!-- proof:compiled from=\"md://figures/foo.md#:0\" -->"));
        assert!(out.contains("<!-- /proof:compiled -->"));
        assert!(out.contains("CONTENT"));
        assert!(out.contains("LINE2"));
    }

    #[test]
    fn test_format_include_block_strips_fence() {
        // Content that arrives already-fenced from older resolve paths
        let out = compile_format::include_block("md://x.md#:0", "```\nFOO\nBAR\n```");
        // Should strip the fence and re-wrap
        assert!(out.contains("FOO"));
        assert!(out.contains("BAR"));
    }

    #[test]
    fn test_format_layout_block_has_uris() {
        let uris = vec!["md://a.md#:0".to_string(), "md://b.md#:0".to_string()];
        let out = compile_format::layout_block(&uris, "COMPOSED");
        assert!(out.contains("proof:layout"));
        assert!(out.contains("md://a.md#:0"));
        assert!(out.contains("md://b.md#:0"));
        assert!(out.contains("COMPOSED"));
    }

    // ── end-to-end compile (no mdpath) ────────────────────

    #[test]
    fn test_collect_directives_include() {
        let src = "Before.\n\n```proof:include\nmd://fig.md#:0\n```\n\nAfter.";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Include {
                uri,
                pin,
                line_start,
                line_end,
            } => {
                assert_eq!(uri, "md://fig.md#:0");
                assert_eq!(*line_start, 2);
                assert_eq!(*line_end, 4);
                assert!(pin.is_none(), "no pin= in plain include");
            }
            _ => panic!("expected Include"),
        }
    }

    #[test]
    fn include_pin_attribute_parsed() {
        let src = "```proof:include pin=arch-diagram\nmd://figures/arch.md#:0\n```\n";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Include { uri, pin, .. } => {
                assert_eq!(uri, "md://figures/arch.md#:0");
                assert_eq!(pin.as_deref(), Some("arch-diagram"));
            }
            _ => panic!("expected Include"),
        }
    }

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

    #[test]
    fn test_collect_directives_layout() {
        let src = "```proof:layout gap=2 labels=\"A,B\"\nmd://a.md#:0\nmd://b.md#:0\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Layout { uris, attrs, .. } => {
                assert_eq!(uris.len(), 2);
                assert_eq!(attrs.gap, 2);
                assert_eq!(attrs.labels, vec!["A", "B"]);
            }
            _ => panic!("expected Layout"),
        }
    }

    // ── proof:row parsing ─────────────────────────────────

    #[test]
    fn test_collect_directives_row_parsed() {
        let src = "```proof:row foreach=player in md://stats.md#edm:table:0\nproof:element kind=label field=name width=12\nproof:element kind=value field=pts width=6\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1, "should parse one Row directive");
        match &dirs[0] {
            Directive::Row {
                source_uri,
                var_name,
                elements,
                ..
            } => {
                assert_eq!(source_uri, "md://stats.md#edm:table:0");
                assert_eq!(var_name, "player");
                assert_eq!(elements.len(), 2, "should parse 2 RowElements");
            }
            _ => panic!("expected Row, got {:?}", dirs[0]),
        }
    }

    #[test]
    fn test_collect_directives_row_body_element_lines() {
        let src = "```proof:row foreach=p in md://x.md#:table:0\nproof:element kind=label field=name width=10 align=left\nproof:element kind=mini-bar field=pts width=8 max=200\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Row { elements, .. } => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0].field, "name");
                assert_eq!(elements[0].width, 10);
                assert_eq!(elements[1].field, "pts");
                assert_eq!(elements[1].width, 8);
                assert_eq!(elements[1].max, Some(200.0));
            }
            _ => panic!("expected Row"),
        }
    }

    #[test]
    fn test_collect_directives_row_default_separator() {
        let src = "```proof:row foreach=p in md://x.md#:table:0\nproof:element kind=label field=name width=8\n```";
        let dirs = collect_directives(src);
        match &dirs[0] {
            Directive::Row { separator, .. } => {
                assert_eq!(separator, " ", "default separator should be single space");
            }
            _ => panic!("expected Row"),
        }
    }

    #[test]
    fn test_collect_directives_row_explicit_separator() {
        let src = "```proof:row foreach=p in md://x.md#:table:0 separator=\",\"\nproof:element kind=label field=name width=8\n```";
        let dirs = collect_directives(src);
        match &dirs[0] {
            Directive::Row { separator, .. } => {
                assert_eq!(separator, ",");
            }
            _ => panic!("expected Row"),
        }
    }

    // ── proof_directive_kind ──────────────────────────────

    #[test]
    fn test_proof_directive_kind_row() {
        assert_eq!(
            proof_directive_kind("```proof:row foreach=p in md://x.md"),
            Some("row")
        );
    }

    // ── parse_foreach ────────────────────────────────────

    #[test]
    fn test_parse_foreach_extracts_var_and_uri() {
        let (var, uri) = compile_directive::parse_foreach(
            "foreach=player in md://stats.md#edm:table:0 separator=\" \"",
        );
        assert_eq!(var, "player");
        assert_eq!(uri, "md://stats.md#edm:table:0");
    }

    // ── parse_row_element_line ────────────────────────────

    #[test]
    fn test_parse_row_element_line_label() {
        let elem = compile_directive::parse_row_element_line(
            "proof:element kind=label field=name width=12 align=left",
        )
        .unwrap();
        assert_eq!(elem.field, "name");
        assert_eq!(elem.width, 12);
        assert!(matches!(elem.kind, ElementKind::Label));
    }

    #[test]
    fn test_parse_row_element_line_mini_bar_with_max() {
        let elem = compile_directive::parse_row_element_line(
            "proof:element kind=mini-bar field=pts width=10 max=200",
        )
        .unwrap();
        assert_eq!(elem.field, "pts");
        assert_eq!(elem.width, 10);
        assert_eq!(elem.max, Some(200.0));
        assert!(matches!(elem.kind, ElementKind::MiniBar));
    }

    #[test]
    fn test_parse_row_element_line_non_element_returns_none() {
        assert!(compile_directive::parse_row_element_line("# Comment").is_none());
        assert!(compile_directive::parse_row_element_line("md://stats.md").is_none());
    }

    // ── R-1 violation via compile (no I/O — inline table) ─

    #[test]
    fn test_validate_r1_correct() {
        use crate::element::row::validate_r1;
        use crate::element::row::RowElement;
        let elems = vec![
            RowElement {
                kind: ElementKind::Label,
                field: "n".into(),
                width: 10,
                align: ElementAlign::Left,
                format: "{}".into(),
                max: None,
                fill_char: '█',
                empty_char: '░',
            },
            RowElement {
                kind: ElementKind::Value,
                field: "p".into(),
                width: 5,
                align: ElementAlign::Right,
                format: "{}".into(),
                max: None,
                fill_char: '█',
                empty_char: '░',
            },
        ];
        // sum=15, sep_len=1, n=2 → total=16, declared=16 → OK
        assert!(validate_r1(&elems, 1, Some(16)).is_none());
    }

    #[test]
    fn test_validate_r1_violation() {
        use crate::element::row::validate_r1;
        use crate::element::row::RowElement;
        let elems = vec![
            RowElement {
                kind: ElementKind::Label,
                field: "n".into(),
                width: 10,
                align: ElementAlign::Left,
                format: "{}".into(),
                max: None,
                fill_char: '█',
                empty_char: '░',
            },
            RowElement {
                kind: ElementKind::Value,
                field: "p".into(),
                width: 5,
                align: ElementAlign::Right,
                format: "{}".into(),
                max: None,
                fill_char: '█',
                empty_char: '░',
            },
        ];
        // actual=16, declared=20 → violation
        let result = validate_r1(&elems, 1, Some(20));
        assert_eq!(result, Some((16, 20)));
    }

    // ── Wave 2: proof:element directive tests ─────────────

    #[test]
    fn test_collect_directives_element_kind_value() {
        let src = "```proof:element kind=value field=pts width=6\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1, "should parse one Element directive");
        match &dirs[0] {
            Directive::Element {
                kind, field, attrs, ..
            } => {
                assert_eq!(kind, "value");
                assert_eq!(field.as_deref(), Some("pts"));
                assert_eq!(attrs.width, Some(6));
            }
            _ => panic!("expected Element, got {:?}", dirs[0]),
        }
    }

    #[test]
    fn test_collect_directives_element_sparkline_no_chrome() {
        let src = "```proof:element kind=sparkline field=trend width=10 no-chrome\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Element { kind, attrs, .. } => {
                assert_eq!(kind, "sparkline");
                assert!(attrs.no_chrome, "no-chrome flag should be set");
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn test_element_attrs_parse_all_keys() {
        let attrs = ElementAttrs::parse(
            "kind=value field=pts width=8 align=right format=\"{:.1}\" max=200 fill=▓ empty=░",
        );
        assert_eq!(attrs.width, Some(8));
        assert_eq!(attrs.align, "right");
        assert_eq!(attrs.format, "{:.1}");
        assert_eq!(attrs.max, Some(200.0));
    }

    #[test]
    fn test_element_attrs_parse_no_chrome_flag() {
        let attrs = ElementAttrs::parse("no-chrome width=6");
        assert!(attrs.no_chrome, "bare no-chrome should set flag");
        assert_eq!(attrs.width, Some(6));
    }

    #[test]
    fn test_element_attrs_parse_defaults() {
        let attrs = ElementAttrs::parse("");
        assert_eq!(attrs.align, "left");
        assert_eq!(attrs.format, "{}");
        assert!(!attrs.no_chrome);
        assert_eq!(attrs.max, None);
        assert_eq!(attrs.width, None);
    }

    #[test]
    fn test_proof_directive_kind_element() {
        assert_eq!(
            proof_directive_kind("```proof:element kind=value width=4"),
            Some("element")
        );
    }

    // E2E tests using compile_element directly (no file I/O)

    #[test]
    fn test_e2e_element_value_inline() {
        let attrs = ElementAttrs {
            width: Some(4),
            align: "right".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=value value=\"42\" width=4 align=right",
            "```",
        ];
        let out = crate::compile_element::compile_element(
            "value",
            None,
            None,
            Some("42"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "should have no violations: {:?}",
            violations
                .iter()
                .map(|v| v.message.as_str())
                .collect::<Vec<_>>()
        );
        assert!(out.contains("42"), "output should contain value: {:?}", out);
        let value_w = crate::layout::visual_width(&" 42");
        assert_eq!(value_w, 3);
    }

    #[test]
    fn test_e2e_element_label_inline() {
        let attrs = ElementAttrs {
            width: Some(8),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=label value=\"McDavid\" width=8 align=left",
            "```",
        ];
        let out = crate::compile_element::compile_element(
            "label",
            None,
            None,
            Some("McDavid"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "should have no violations: {:?}",
            violations
                .iter()
                .map(|v| v.message.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            out.contains("McDavid"),
            "output should contain label: {:?}",
            out
        );
    }

    #[test]
    fn test_e2e_element_badge_inline() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=badge value=\"UFA\" width=5", "```"];
        let out = crate::compile_element::compile_element(
            "badge",
            None,
            None,
            Some("UFA"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violations
                .iter()
                .map(|v| v.message.as_str())
                .collect::<Vec<_>>()
        );
        assert!(out.contains("UFA"), "output: {:?}", out);
    }

    #[test]
    fn test_e2e_element_no_chrome_true() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: true,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=label value=\"Hi\" width=5 no-chrome",
            "```",
        ];
        let out = crate::compile_element::compile_element(
            "label",
            None,
            None,
            Some("Hi"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violations
                .iter()
                .map(|v| v.message.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            !out.contains("```"),
            "no-chrome should have no fence: {:?}",
            out
        );
        assert!(
            !out.contains("<!--"),
            "no-chrome should have no HTML comment: {:?}",
            out
        );
    }

    #[test]
    fn test_e2e_element_no_chrome_false_has_wrapper() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=label value=\"Hi\" width=5", "```"];
        let out = crate::compile_element::compile_element(
            "label",
            None,
            None,
            Some("Hi"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violations
                .iter()
                .map(|v| v.message.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            out.contains("<!-- proof:compiled"),
            "should have traceability comment: {:?}",
            out
        );
        assert!(out.contains("```"), "should have fence: {:?}", out);
    }

    #[test]
    fn test_e2e_element_missing_field_emits_element_005() {
        // Simulate a source table with a known header, but ask for a missing field
        let attrs = ElementAttrs {
            width: Some(6),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=value field=absent width=6",
            "md://test",
            "```",
        ];
        // Use inline value to avoid file I/O, but pass field= with no source → triggers ELEMENT-005 (missing source)
        crate::compile_element::compile_element(
            "value",
            None,
            Some("absent"),
            None,
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            2,
            &mut 0,
        );
        // Should emit ELEMENT-005 because source is None and inline_value is None
        let codes: Vec<&str> = violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"ELEMENT-005"),
            "expected ELEMENT-005, got: {:?}",
            codes
        );
    }

    // ── Wave 3: dashboard pipeline ────────────────────────

    #[test]
    fn test_proof_directive_kind_region() {
        assert_eq!(
            proof_directive_kind("```proof:region name=header"),
            Some("region")
        );
    }

    #[test]
    fn test_proof_directive_kind_ol_alias() {
        // Both names resolve to the same kind, dispatching to render_ol().
        assert_eq!(proof_directive_kind("```proof:numbered-list"), Some("ol"));
        assert_eq!(proof_directive_kind("```proof:ol"), Some("ol"));
    }

    #[test]
    fn test_collect_directives_region() {
        let src = "```proof:region name=header\nHello world\nproof:element kind=label value=\"X\" width=5\n```";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Region { name, body, .. } => {
                assert_eq!(name, "header");
                assert_eq!(body.len(), 2);
                assert_eq!(body[0], "Hello world");
                assert!(body[1].starts_with("proof:element"));
            }
            _ => panic!("expected Region"),
        }
    }

    #[test]
    fn test_split_frontmatter_with_yaml() {
        let src = "---\ndashboard:\n  width: 80\n---\nbody line 1\nbody line 2\n";
        let (fm, body, offset) = split_frontmatter(src);
        assert!(fm.contains("dashboard:"));
        assert!(fm.contains("width: 80"));
        assert!(body.starts_with("body line 1"));
        assert_eq!(offset, 4); // lines: --- / dashboard: / width / --- → body at line 4
    }

    #[test]
    fn test_split_frontmatter_no_yaml() {
        let src = "no frontmatter here\nplain content\n";
        let (fm, body, offset) = split_frontmatter(src);
        assert!(fm.is_empty());
        assert_eq!(body, src);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_strip_compiled_chrome_removes_html_and_fence() {
        let block = "<!-- proof:compiled from=\"x\" -->\n```\ninner content\nrow 2\n```\n<!-- /proof:compiled -->";
        let stripped = strip_compiled_chrome(block);
        assert_eq!(stripped, "inner content\nrow 2");
    }

    #[test]
    fn test_dashboard_compile_two_regions_e2e() {
        // End-to-end: write a .dashboard.source.md to a temp dir, compile, read output
        use std::io::Write;

        let tmp = std::env::temp_dir().join(format!(
            "proof-dash-{}.dashboard.source.md",
            std::process::id()
        ));
        let out =
            std::env::temp_dir().join(format!("proof-dash-{}.dashboard.md", std::process::id()));
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let src = "---\ndashboard:\n  width: 20\n  height: 4\n  title: \"Test\"\n  regions:\n    top: { x: 0, y: 0, width: 20, height: 2 }\n    bot: { x: 0, y: 2, width: 20, height: 2 }\n---\n\n```proof:region name=top\nHEADER LINE\n```\n\n```proof:region name=bot\nFOOTER LINE\n```\n";
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(src.as_bytes()).expect("write tmp");
        drop(f);

        let cfg = GlintConfig::default();
        let result =
            compile_file(&tmp, &out, &std::env::temp_dir(), &cfg).expect("compile_file ok");

        let _ = std::fs::remove_file(&tmp);

        assert!(
            result
                .violations
                .iter()
                .all(|v| v.severity != ViolationSeverity::Error),
            "unexpected errors: {:?}",
            result
                .violations
                .iter()
                .map(|v| (v.code, &v.message))
                .collect::<Vec<_>>()
        );
        assert!(result.written, "should have written output");

        let written = std::fs::read_to_string(&out).expect("read output");
        let _ = std::fs::remove_file(&out);

        assert!(
            written.contains("```dashboard"),
            "should have dashboard fence: {}",
            written
        );
        assert!(written.contains("HEADER LINE"), "top region not rendered");
        assert!(written.contains("FOOTER LINE"), "bot region not rendered");

        // Verify D-6: every line inside the fence is exactly the canvas width
        let inner: Vec<&str> = written
            .lines()
            .skip_while(|l| !l.starts_with("```dashboard"))
            .skip(1)
            .take_while(|l| *l != "```")
            .collect();
        assert_eq!(
            inner.len(),
            4,
            "canvas should be height=4 lines, got {}: {:?}",
            inner.len(),
            inner
        );
        for line in &inner {
            assert_eq!(line.chars().count(), 20, "row width != 20: {:?}", line);
        }
    }

    #[test]
    fn test_dashboard_unknown_region_emits_dashboard_004() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join(format!(
            "proof-dash-bad-{}.dashboard.source.md",
            std::process::id()
        ));
        let out = std::env::temp_dir().join(format!(
            "proof-dash-bad-{}.dashboard.md",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        // front-matter declares only "header"; body has region "ghost" that's not declared
        let src = "---\ndashboard:\n  width: 20\n  height: 2\n  regions:\n    header: { x: 0, y: 0, width: 20, height: 2 }\n---\n\n```proof:region name=ghost\nNo such region\n```\n";
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(src.as_bytes()).expect("write tmp");
        drop(f);

        let cfg = GlintConfig::default();
        let result =
            compile_file(&tmp, &out, &std::env::temp_dir(), &cfg).expect("compile_file ok");

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-004"),
            "expected DASHBOARD-004, got: {:?}",
            codes
        );
    }

    #[test]
    fn test_dashboard_overlap_emits_dashboard_003() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join(format!(
            "proof-dash-ovl-{}.dashboard.source.md",
            std::process::id()
        ));
        let out = std::env::temp_dir().join(format!(
            "proof-dash-ovl-{}.dashboard.md",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let src = "---\ndashboard:\n  width: 40\n  height: 10\n  regions:\n    a: { x: 0, y: 0, width: 30, height: 5 }\n    b: { x: 20, y: 0, width: 20, height: 5 }\n---\n";
        let mut f = std::fs::File::create(&tmp).expect("create tmp");
        f.write_all(src.as_bytes()).expect("write tmp");
        drop(f);

        let cfg = GlintConfig::default();
        let result =
            compile_file(&tmp, &out, &std::env::temp_dir(), &cfg).expect("compile_file ok");

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-003"),
            "expected DASHBOARD-003 (overlap), got: {:?}",
            codes
        );
    }

    #[test]
    fn test_dashboard_wide_canvas_emits_dashboard_006() {
        use std::io::Write;
        let pid = std::process::id();
        let tmp = std::env::temp_dir().join(format!("proof-dash-wide-{}.dashboard.source.md", pid));
        let out = std::env::temp_dir().join(format!("proof-dash-wide-{}.dashboard.md", pid));
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let src = "---\ndashboard:\n  width: 300\n  height: 10\n---\n\n```proof:region name=r1\nhello\n```\n";
        std::fs::File::create(&tmp)
            .unwrap()
            .write_all(src.as_bytes())
            .unwrap();

        let cfg = GlintConfig::default();
        let result = compile_file(&tmp, &out, &std::env::temp_dir(), &cfg).expect("compile ok");

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);

        let codes: Vec<&str> = result.violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"DASHBOARD-006"),
            "expected DASHBOARD-006 for canvas width 300 > 220, got: {:?}",
            codes
        );
    }

    // ── generate_toc: section= scoping ────────────────────────────────────────

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
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "list", None);
        assert!(out.contains("API Reference"));
        assert!(out.contains("Endpoints"));
        assert!(out.contains("Migration"));
        assert!(out.contains("Upgrade Steps"));
    }

    #[test]
    fn toc_section_filters_to_descendants() {
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "list", Some("API Reference"));
        assert!(out.contains("Endpoints"));
        assert!(out.contains("Authentication"));
        assert!(out.contains("GET /widgets"));
        // The anchor heading itself is NOT listed — only its children
        assert!(
            !out.contains("API Reference"),
            "section anchor heading must be excluded from output, got:\n{}",
            out
        );
        // Sibling sections must NOT appear
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
        // max-depth=3 + section="API Reference" => H3 only (H4 endpoints excluded)
        let out = compile_toc::generate_toc(SAMPLE_DOC, 3, "list", Some("API Reference"));
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
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "list", Some("api reference"));
        assert!(
            out.contains("Endpoints"),
            "section match must be case-insensitive, got:\n{}",
            out
        );
    }

    #[test]
    fn toc_section_not_found_returns_empty() {
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "list", Some("Nonexistent Section"));
        assert!(
            out.is_empty(),
            "missing section should produce empty TOC, got:\n{}",
            out
        );
    }

    #[test]
    fn toc_section_works_for_h3_anchor() {
        // section= can target any heading, not just H2
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "list", Some("Endpoints"));
        assert!(out.contains("GET /widgets"));
        assert!(out.contains("POST /widgets"));
        // Must stop at sibling ### Authentication
        assert!(!out.contains("Authentication"));
    }

    #[test]
    fn toc_section_numbered_renumbers_from_section() {
        let out = compile_toc::generate_toc(SAMPLE_DOC, 4, "numbered", Some("API Reference"));
        // Within the section, the first H3 is "1." (re-rooted by min_level)
        assert!(
            out.starts_with("1. Endpoints"),
            "numbered TOC must renumber from the section root, got:\n{}",
            out
        );
    }

    #[test]
    fn toc_directive_parses_section_attr() {
        let src = "```proof:toc section=\"API Reference\" max-depth=3\n```\n";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Toc {
                section, max_depth, ..
            } => {
                assert_eq!(section.as_deref(), Some("API Reference"));
                assert_eq!(*max_depth, 3);
            }
            _ => panic!("expected Directive::Toc"),
        }
    }

    // ── proof:xref ─────────────────────────────────────────────────────────────

    #[test]
    fn xref_parses_uri_and_format() {
        let src = "```proof:xref uri=\"md://api.md#authentication\" format=note\n```\n";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Xref {
                uri, format, label, ..
            } => {
                assert_eq!(uri, "md://api.md#authentication");
                assert_eq!(format, "note");
                assert!(label.is_none());
            }
            _ => panic!("expected Directive::Xref"),
        }
    }

    #[test]
    fn xref_parses_label_override() {
        let src = "```proof:xref uri=\"md://guide.md\" label=\"the guide\"\n```\n";
        let dirs = collect_directives(src);
        match &dirs[0] {
            Directive::Xref { label, .. } => {
                assert_eq!(label.as_deref(), Some("the guide"));
            }
            _ => panic!("expected Directive::Xref"),
        }
    }

    #[test]
    fn heading_slug_basic() {
        assert_eq!(
            compile_prose::heading_slug("Authentication"),
            "authentication"
        );
        assert_eq!(
            compile_prose::heading_slug("API Reference"),
            "api-reference"
        );
        assert_eq!(compile_prose::heading_slug("What's New?"), "whats-new");
    }

    #[test]
    fn xref_inline_renders_see_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("api.md");
        std::fs::write(&target, "# API Guide\n\n## Authentication\n\nContent.\n").unwrap();

        let result =
            compile_prose::render_xref("md://api.md#authentication", None, "inline", dir.path())
                .expect("render_xref should succeed");
        assert!(
            result.contains("See:"),
            "inline format should start with See:"
        );
        assert!(
            result.contains("Authentication"),
            "should resolve heading text"
        );
        assert!(
            result.contains("api.md#authentication"),
            "should include link"
        );
    }

    #[test]
    fn xref_note_format_renders_blockquote() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("ref.md"),
            "# Ref\n\n## Background\n\nContent.\n",
        )
        .unwrap();
        let result =
            compile_prose::render_xref("md://ref.md#background", None, "note", dir.path()).unwrap();
        assert!(
            result.starts_with("> **See also:**"),
            "note format must use blockquote"
        );
        assert!(result.contains("Background"));
    }

    #[test]
    fn xref_label_override_used_instead_of_heading() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("guide.md"),
            "# Guide\n\n## Setup\n\nContent.\n",
        )
        .unwrap();
        let result = compile_prose::render_xref(
            "md://guide.md#setup",
            Some("the setup section"),
            "inline",
            dir.path(),
        )
        .unwrap();
        assert!(
            result.contains("the setup section"),
            "label override must appear in output"
        );
        assert!(!result.contains("Setup") || result.contains("the setup section"));
    }

    #[test]
    fn xref_missing_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = compile_prose::render_xref("md://nonexistent.md", None, "inline", dir.path());
        assert!(result.is_err(), "missing target file should return Err");
    }

    // ── proof:blockquote ─────────────────────────────────────

    #[test]
    fn blockquote_directive_kind_detected() {
        assert_eq!(
            proof_directive_kind("```proof:blockquote"),
            Some("blockquote")
        );
        assert_eq!(
            proof_directive_kind("```proof:blockquote attribution=\"Author\""),
            Some("blockquote"),
        );
    }

    #[test]
    fn blockquote_indent_default_no_attribution() {
        let out = compile_prose::render_blockquote("To be or not to be.", None, "indent");
        assert_eq!(out, "> To be or not to be.");
    }

    #[test]
    fn blockquote_indent_with_attribution() {
        let out = compile_prose::render_blockquote("To be or not to be.", Some("Hamlet"), "indent");
        // Body line, blank quote line, attribution line.
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["> To be or not to be.", ">", "> — Hamlet"]);
    }

    #[test]
    fn blockquote_indent_multi_paragraph_preserves_blank_lines() {
        // Inner blank lines stay as `>` (so the rendered markdown is still one
        // contiguous quote, not two adjacent ones).
        let text = "First paragraph.\n\nSecond paragraph.";
        let out = compile_prose::render_blockquote(text, None, "indent");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines,
            vec!["> First paragraph.", ">", "> Second paragraph."]
        );
    }

    #[test]
    fn blockquote_indent_trims_leading_and_trailing_blank_lines() {
        let text = "\n\nThe quote.\n\n";
        let out = compile_prose::render_blockquote(text, None, "indent");
        assert_eq!(out, "> The quote.");
    }

    #[test]
    fn blockquote_unknown_style_falls_back_to_indent() {
        let out_unknown = compile_prose::render_blockquote("hi", None, "marble");
        let out_indent = compile_prose::render_blockquote("hi", None, "indent");
        assert_eq!(
            out_unknown, out_indent,
            "unknown style must fall back to indent (permissive parsing)"
        );
    }

    #[test]
    fn blockquote_boxed_renders_frame() {
        let out = compile_prose::render_blockquote("Hello world", None, "boxed");
        let lines: Vec<&str> = out.lines().collect();
        // Top, content, bottom — at minimum.
        assert!(lines.len() >= 3);
        assert!(lines.first().unwrap().starts_with('┌'));
        assert!(lines.first().unwrap().ends_with('┐'));
        assert!(lines.last().unwrap().starts_with('└'));
        assert!(lines.last().unwrap().ends_with('┘'));
        // Content row contains the text and is bracketed by │ ... │.
        assert!(lines
            .iter()
            .any(|l| l.starts_with('│') && l.contains("Hello world") && l.ends_with('│')));
    }

    #[test]
    fn blockquote_boxed_with_attribution_right_aligned() {
        let out = compile_prose::render_blockquote("To be.", Some("Hamlet"), "boxed");
        let lines: Vec<&str> = out.lines().collect();
        // Last content line (before bottom border) should hold the attribution.
        let attr_line = lines[lines.len() - 2];
        assert!(
            attr_line.contains("— Hamlet"),
            "expected attribution in penultimate line, got {:?}",
            attr_line
        );
        assert!(attr_line.starts_with('│') && attr_line.ends_with('│'));
    }

    #[test]
    fn blockquote_collected_from_directive_block() {
        // End-to-end: collect_directives should pick up a proof:blockquote fence.
        let src = "Before.\n\n```proof:blockquote attribution=\"Ada\"\nThe Analytical Engine has no pretensions.\n```\n\nAfter.\n";
        let dirs = collect_directives(src);
        assert_eq!(dirs.len(), 1, "expected exactly one Blockquote directive");
        match &dirs[0] {
            Directive::Blockquote {
                text,
                attribution,
                style,
                ..
            } => {
                assert!(text.contains("Analytical Engine"));
                assert_eq!(attribution.as_deref(), Some("Ada"));
                assert_eq!(style, "indent", "default style is indent");
            }
            other => panic!("expected Blockquote, got {:?}", other),
        }
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
