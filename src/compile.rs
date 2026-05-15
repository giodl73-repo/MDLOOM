use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::compile_chart::resolve_chart_data;
use crate::compile_directive::{collect_directives, Directive, ElementAttrs, TreeAttrs};
#[cfg(test)]
use crate::compile_prose::heading_slug;
use crate::compile_prose::{render_blockquote, render_xref};
use crate::compile_source::resolve_source_for_compile;
use crate::compile_toc::generate_toc;
use crate::compile_tree::{render_inline_outline, render_inline_tree};
use crate::config::GlintConfig;
use crate::davinci::evaluate_invariant;
use crate::diagnostic::Severity;
use crate::element::row::{render_row_foreach, validate_r1, RowConfig, RowElement};
use crate::element::{
    render_element, ElementAlign, ElementConfig, ElementData, ElementError, ElementKind,
};
use crate::layout::{self, extract_content_lines};
use crate::runner::Runner;
use crate::tree::dirtree::{generate as dirtree_generate, DirtreeOptions};
use crate::tree::schema::{
    generate_dependency, generate_org, generate_outline, generate_taxonomy, parse_json_source,
    parse_md_table, FieldMap,
};

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
// Directive parsing
// ─────────────────────────────────────────────────────────

pub fn parse_directives(source: &str) -> Vec<(usize, usize, String, String)> {
    crate::compile_directive::parse_directives(source)
}

// ─────────────────────────────────────────────────────────
// Main compile function
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
        return compile_slides_file(source_path, output_path, config);
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
    let source_body = crate::frontmatter::parse(&source_text)
        .map(|parsed| parsed.body)
        .unwrap_or(&source_text);

    // ── Tier 3 cache check ──────────────────────────────────────────────
    // Build a minimal directive-attrs JSON for cache keying, then check Tier 3.
    // On hit: write cached output (skip if identical), return early with from_cache=true.
    let mut path_index = crate::cache::load_path_index(root);
    {
        let source_parse_key =
            crate::cache::get_or_compute_parse_key(source_path, &source_text, &mut path_index);
        // Collect resolved file deps from resolved_files for key computation
        // (empty on first compile; populated on cache store below)
        let cache_key = crate::cache::compile_key(&source_parse_key, &[], "{}");
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
                directives_resolved: 0,
                violations: vec![],
                from_cache: true,
                resolved_files: vec![],
                written,
            });
        }
    }
    // ────────────────────────────────────────────────────────────────────

    let source_lines: Vec<&str> = source_body.lines().collect();
    let directives = collect_directives(source_body);

    // Build a runner for figure lint validation
    let runner = Runner::new(root, config.clone())?;

    let mut violations: Vec<CompileViolation> = Vec::new();
    let mut resolved_count = 0usize;
    let resolved_files: Vec<PathBuf> = Vec::new();

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
                            source_line: line_start + 1,
                        });
                    }
                }
                match resolve_uri_cached(uri, root, &mut path_index) {
                    Ok((content, fig_file)) => {
                        lint_figure(
                            uri,
                            &content,
                            &fig_file,
                            line_start + 1,
                            &runner,
                            &mut violations,
                        );
                        validate_davinci(uri, &content, config, line_start, &mut violations);
                        resolved_count += 1;
                        format_include_block(uri, &content)
                    }
                    Err(e) => {
                        violations.push(CompileViolation {
                            code: "COMPILE-002",
                            severity: ViolationSeverity::Error,
                            uri: uri.clone(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("{}", e),
                            source_line: line_start + 1,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    }
                }
            }

            Directive::Layout { uris, attrs, .. } => {
                let mut figures: Vec<Vec<String>> = Vec::new();
                let mut any_err = false;
                for uri in uris {
                    match resolve_uri_cached(uri, root, &mut path_index) {
                        Ok((content, fig_file)) => {
                            lint_figure(
                                uri,
                                &content,
                                &fig_file,
                                line_start + 1,
                                &runner,
                                &mut violations,
                            );
                            validate_davinci(uri, &content, config, line_start, &mut violations);
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
                                source_line: line_start + 1,
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
                    let layout_config = attrs.to_layout_config();

                    let composed = layout::layout(figures, &layout_config);
                    // Strip outer ``` wrapper — compile embeds content inline
                    let inner = composed
                        .strip_prefix("```\n")
                        .and_then(|s| s.strip_suffix("\n```"))
                        .unwrap_or(&composed);
                    format_layout_block(uris, inner)
                }
            }

            Directive::Table { uri, .. } => match resolve_uri_cached(uri, root, &mut path_index) {
                Ok((content, fig_file)) => {
                    lint_figure(
                        uri,
                        &content,
                        &fig_file,
                        line_start + 1,
                        &runner,
                        &mut violations,
                    );
                    validate_davinci(uri, &content, config, line_start, &mut violations);
                    resolved_count += 1;
                    format_include_block(uri, &content)
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: "COMPILE-002",
                        severity: ViolationSeverity::Error,
                        uri: uri.clone(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("{}", e),
                        source_line: line_start + 1,
                    });
                    source_lines[line_start..=line_end].join("\n")
                }
            },

            Directive::Tree {
                kind,
                source,
                inline_body,
                attrs,
                ..
            } => {
                generate_tree_block(
                    kind,
                    source.as_deref(),
                    inline_body,
                    attrs,
                    root,
                    line_start,
                    &mut violations,
                )
                .unwrap_or_else(|e| {
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
                        source_line: line_start + 1,
                    });
                    source_fallback(&source_lines, line_start, line_end)
                })
            }

            Directive::Element {
                kind,
                source,
                field,
                inline_value,
                attrs,
                ..
            } => compile_element(
                kind,
                source.as_deref(),
                field.as_deref(),
                inline_value.as_deref(),
                attrs,
                root,
                line_start,
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
            } => compile_row(
                source_uri,
                separator,
                *declared_width,
                elements,
                *no_chrome,
                root,
                line_start,
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
            } => {
                let lib = crate::symbol::SymbolLibrary::new();
                match crate::symbol::resolve(name, &lib) {
                    Some(sym) => {
                        resolved_count += 1;
                        let rendered = crate::symbol::render_symbol_block(&sym, *size);
                        format!(
                            "<!-- proof:compiled from=\"proof:symbol\" name=\"{}\" size=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
                            name, size, rendered
                        )
                    }
                    None => {
                        let hint = crate::symbol::suggest_symbol(name, &lib)
                            .map(|s| format!(" — did you mean '{}'?", s))
                            .unwrap_or_default();
                        violations.push(CompileViolation {
                            code: "SYMBOL-001",
                            severity: ViolationSeverity::Warning,
                            uri: String::new(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("Unknown symbol '{}'{}", name, hint),
                            source_line: line_start + 1,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    }
                }
            }

            Directive::Shape { attrs, .. } => match crate::symbol::shape::render_shape(attrs) {
                Ok(rendered) => {
                    resolved_count += 1;
                    format!(
                            "<!-- proof:compiled from=\"proof:shape\" name=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
                            attrs.name, rendered
                        )
                }
                Err(e) => {
                    violations.push(CompileViolation {
                        code: e.code,
                        severity: ViolationSeverity::Error,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: e.message,
                        source_line: line_start + 1,
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
                    source_line: line_start + 1,
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
                let (math_lines, math_diags) =
                    crate::math::render_display_math(expr, *width, *align);
                resolved_count += 1;
                for d in &math_diags {
                    violations.push(CompileViolation {
                        code: d.code,
                        severity: ViolationSeverity::Warning,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: d.message.clone(),
                        source_line: line_start + 1,
                    });
                }
                let rendered = math_lines.join("\n");
                if *no_chrome {
                    format!("```\n{}\n```", rendered)
                } else {
                    format!(
                        "<!-- proof:compiled from=\"proof:math\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
                        rendered
                    )
                }
            }

            Directive::Toc {
                source,
                max_depth,
                style,
                section,
                ..
            } => {
                let content_opt: Option<String> = if let Some(uri) = source {
                    match resolve_source_for_compile(uri, root) {
                        Ok(c) => Some(c),
                        Err(e) => {
                            violations.push(CompileViolation {
                                code: "COMPILE-002",
                                severity: ViolationSeverity::Error,
                                uri: uri.clone(),
                                figure_id: None,
                                invariant: String::new(),
                                message: format!("toc source error: {}", e),
                                source_line: line_start + 1,
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
                        let toc = generate_toc(&content, *max_depth, style, section.as_deref());
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
            } => match render_xref(uri, label.as_deref(), format, root) {
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
                        source_line: line_start + 1,
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
                let rendered = render_blockquote(text, attribution.as_deref(), style);
                format!(
                    "<!-- proof:compiled from=\"proof:blockquote\" -->\n{}\n<!-- /proof:compiled -->",
                    rendered
                )
            }

            Directive::Chart {
                attrs,
                source,
                label_field,
                value_field,
                inline_body,
                ..
            } => {
                let data_result = resolve_chart_data(
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
                                source_line: line_start + 1,
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
                            source_line: line_start + 1,
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
            resolved_files: vec![],
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
        let cache_key = crate::cache::compile_key(&source_parse_key, &[], "{}");
        let entry = crate::cache::CompileCacheEntry {
            compile_key: cache_key,
            source_path: source_path.to_string_lossy().to_string(),
            output_path: output_path.to_string_lossy().to_string(),
            compiled_text: output_text.clone(),
            resolved_uris: vec![],
            proof_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
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

fn resolve_uri(uri: &str, root: &Path) -> Result<(String, PathBuf)> {
    let parsed =
        mdpath::parse(uri).map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", uri, e))?;
    let element = mdpath::resolve(&parsed, root)
        .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", uri, e))?;
    Ok((element.content, element.file))
}

/// Tier 2 cached version of `resolve_uri`.
/// Checks `.proof/cache/resolve/` before calling mdpath. On hit the figure file
/// is not re-read or re-parsed; on miss the result is stored for future runs.
fn resolve_uri_cached(
    uri: &str,
    root: &Path,
    path_index: &mut crate::cache::PathIndex,
) -> Result<(String, PathBuf)> {
    let parsed =
        mdpath::parse(uri).map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", uri, e))?;

    let target_file = root.join(&parsed.path);
    if !target_file.exists() {
        return resolve_uri(uri, root);
    }
    let target_content = match std::fs::read_to_string(&target_file) {
        Ok(c) => c,
        Err(_) => return resolve_uri(uri, root),
    };
    if let Some(cached) =
        crate::cache::try_resolve_cache_hit(root, &target_file, &target_content, uri, path_index)
    {
        return Ok((cached, target_file));
    }
    let element = mdpath::resolve(&parsed, root)
        .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", uri, e))?;
    crate::cache::store_resolve_cache(
        root,
        &target_file,
        &target_content,
        uri,
        &element.content,
        path_index,
    );
    Ok((element.content, element.file))
}

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

fn format_include_block(uri: &str, content: &str) -> String {
    // Content from mdpath may or may not be fenced — normalize
    let lines = extract_content_lines(content);
    let body = lines.join("\n");
    format!(
        "<!-- proof:compiled from=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uri, body
    )
}

fn format_layout_block(uris: &[String], composed_inner: &str) -> String {
    let uris_str = uris.join(",");
    format!(
        "<!-- proof:compiled from=\"proof:layout\"\n     uris=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uris_str, composed_inner
    )
}

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
/// Generate a tree block for embedding in compiled output.
fn generate_tree_block(
    kind: &str,
    source: Option<&str>,
    inline_body: &[String],
    attrs: &TreeAttrs,
    root: &Path,
    _source_line: usize,
    _violations: &mut Vec<CompileViolation>,
) -> Result<String> {
    let body = match kind {
        "dirtree" => {
            let tree_root = attrs
                .root
                .as_ref()
                .map(|r| root.join(r))
                .unwrap_or_else(|| root.to_path_buf());
            let opts = DirtreeOptions {
                root: tree_root,
                max_depth: attrs.max_depth,
                exclude: attrs.exclude.clone(),
                wrap_fence: false,
                indent_width: attrs.indent_width,
                ..Default::default()
            };
            dirtree_generate(&opts)?
        }
        _ => {
            if let Some(src_uri) = source {
                let content = resolve_source_for_compile(src_uri, root)?;
                let mut map = FieldMap {
                    name: attrs.name.clone(),
                    parent: attrs.parent.clone(),
                    label: attrs.label.clone(),
                    ..Default::default()
                };
                match kind {
                    "org" => generate_org(&content, &attrs.format, &mut map, attrs.indent_width)?,
                    "taxonomy" => {
                        generate_taxonomy(&content, &attrs.format, &mut map, attrs.indent_width)?
                    }
                    "dependency" => {
                        generate_dependency(&content, &attrs.format, &mut map, attrs.indent_width)?
                    }
                    "outline" => generate_outline(&content, attrs.indent_width)?,
                    other => anyhow::bail!("unknown tree kind {:?}", other),
                }
            } else if !inline_body.is_empty() {
                let content = inline_body.join("\n");
                match kind {
                    "org" | "taxonomy" | "dependency" => {
                        render_inline_tree(&content, attrs.indent_width)?
                    }
                    "outline" => render_inline_outline(&content)?,
                    other => anyhow::bail!("unknown tree kind {:?}", other),
                }
            } else {
                anyhow::bail!(
                    "proof:tree kind={} requires either source=md://... or an inline body",
                    kind
                )
            }
        }
    };

    if body.trim().is_empty() {
        anyhow::bail!(
            "proof:tree kind={} produced empty output — check source table columns (name={:?}, parent={:?})",
            kind,
            attrs.name.as_deref().unwrap_or("name"),
            attrs.parent.as_deref().unwrap_or("parent"),
        );
    }

    let uris = source.map(|s| s.to_string()).unwrap_or_default();
    Ok(format!(
        "<!-- proof:compiled from=\"proof:tree kind={}\" uri=\"{}\" -->\n```{}\n{}\n```\n<!-- /proof:compiled -->",
        kind, uris, kind, body
    ))
}

// ─────────────────────────────────────────────────────────
// proof:element compile arm
// ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
/// Safe fallback: return source lines for the directive block, guarded against OOB.
fn source_fallback(source_lines: &[&str], source_line: usize, line_end: usize) -> String {
    if source_line <= line_end && line_end < source_lines.len() {
        source_lines[source_line..=line_end].join("\n")
    } else {
        String::new()
    }
}

fn compile_element(
    kind: &str,
    source: Option<&str>,
    field: Option<&str>,
    inline_value: Option<&str>,
    attrs: &ElementAttrs,
    root: &Path,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
    source_lines: &[&str],
    line_end: usize,
    resolved_count: &mut usize,
) -> String {
    let uri_str = source.unwrap_or("inline");

    // Resolve data
    let raw_value: String = if let Some(lit) = inline_value {
        lit.to_string()
    } else {
        let src_uri = match source {
            Some(s) => s,
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: uri_str.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message:
                        "proof:element requires either value=\"...\" or a source URI in the body"
                            .to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        // Resolve URI content
        let content = match resolve_source_for_compile(src_uri, root) {
            Ok(c) => {
                *resolved_count += 1;
                c
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("{}", e),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        // Parse source table
        let format = if src_uri.ends_with(".json") {
            "json"
        } else {
            "table"
        };
        let rows = match if format == "json" {
            parse_json_source(&content)
        } else {
            parse_md_table(&content)
        } {
            Ok((_, r)) => r,
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("source parse error: {}", e),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        // Extract field value from first row (single-value extraction)
        let col = match field {
            Some(f) => f,
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: "proof:element with a source URI requires field=\"ColumnName\""
                        .to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        let first_row = match rows.first() {
            Some(r) => r,
            None => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: "source resolved to empty table".to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        match first_row.get(col) {
            Some(v) => v.clone(),
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("field {:?} not found in source table headers", col),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        }
    };

    // Parse element kind
    let elem_kind = match ElementKind::parse(kind) {
        Some(k) => k,
        None => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("unknown element kind {:?} — use value, delta, sparkline, mini-bar, label, or badge", kind),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    // Build ElementConfig
    let width = match attrs.width {
        Some(w) => w,
        None => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: "proof:element requires width=N".to_string(),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    let cfg = ElementConfig {
        kind: elem_kind,
        width,
        align: ElementAlign::parse(&attrs.align),
        format: attrs.format.clone(),
        no_chrome: attrs.no_chrome,
        max: attrs.max,
        fill_char: attrs.fill,
        empty_char: attrs.empty,
    };

    // Coerce data
    let data = match elem_kind {
        ElementKind::Sparkline => {
            let series: Result<Vec<f64>, _> = raw_value
                .split(',')
                .map(|s| s.trim().parse::<f64>())
                .collect();
            match series {
                Ok(v) => {
                    if v.len() < width {
                        violations.push(CompileViolation {
                            code: "ELEMENT-003",
                            severity: ViolationSeverity::Warning,
                            uri: uri_str.to_string(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("sparkline series ({} values) shorter than width ({}) — values will be repeated", v.len(), width),
                            source_line: source_line + 1,
                        });
                    }
                    ElementData::Series(v)
                }
                Err(_) => {
                    violations.push(CompileViolation {
                        code: "ELEMENT-002",
                        severity: ViolationSeverity::Error,
                        uri: uri_str.to_string(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("sparkline field value {:?} cannot be parsed as comma-separated numbers", raw_value),
                        source_line: source_line + 1,
                    });
                    return source_fallback(source_lines, source_line, line_end);
                }
            }
        }
        ElementKind::Label | ElementKind::Badge => ElementData::Text(raw_value.clone()),
        ElementKind::Value => {
            // F79: accept formatted display strings ("1,024", "99.9%", "142ms")
            // Strip commas and trailing % then try numeric; fall back to Text display.
            let cleaned = raw_value.replace(',', "");
            let cleaned = cleaned.trim_end_matches('%');
            match cleaned.parse::<f64>() {
                Ok(v) => ElementData::Scalar(v),
                Err(_) => ElementData::Text(raw_value.clone()),
            }
        }
        _ => {
            // delta, mini-bar: strictly numeric
            match raw_value.parse::<f64>() {
                Ok(v) => ElementData::Scalar(v),
                Err(_) => {
                    violations.push(CompileViolation {
                        code: "ELEMENT-002",
                        severity: ViolationSeverity::Error,
                        uri: uri_str.to_string(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!(
                            "element kind={} requires a numeric value; got {:?}",
                            kind, raw_value
                        ),
                        source_line: source_line + 1,
                    });
                    return source_fallback(source_lines, source_line, line_end);
                }
            }
        }
    };

    // Render
    match render_element(&data, &cfg) {
        Ok(rendered) => {
            if attrs.no_chrome {
                rendered
            } else {
                format_element_block(uri_str, &rendered)
            }
        }
        Err(ElementError::WidthExceeded { actual, budget }) => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!(
                    "rendered element width {} exceeds budget {}",
                    actual, budget
                ),
                source_line: source_line + 1,
            });
            source_fallback(source_lines, source_line, line_end)
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("element render error: {}", e),
                source_line: source_line + 1,
            });
            source_fallback(source_lines, source_line, line_end)
        }
    }
}

fn format_element_block(uri: &str, rendered: &str) -> String {
    format!(
        "<!-- proof:compiled from=\"proof:element\" uri=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uri, rendered
    )
}

fn format_row_block(uri: &str, rendered: &str) -> String {
    format!(
        "<!-- proof:compiled from=\"proof:row\" uri=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uri, rendered
    )
}

// ─────────────────────────────────────────────────────────
// proof:row compile handler
// ─────────────────────────────────────────────────────────

fn compile_row(
    source_uri: &str,
    separator: &str,
    declared_width: Option<usize>,
    elements: &[RowElement],
    no_chrome: bool,
    root: &Path,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
    source_lines: &[&str],
    line_end: usize,
    resolved_count: &mut usize,
) -> String {
    // Resolve source
    let content = match resolve_source_for_compile(source_uri, root) {
        Ok(c) => {
            *resolved_count += 1;
            c
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("{}", e),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    // Parse table
    let format = if source_uri.ends_with(".json") {
        "json"
    } else {
        "table"
    };
    let rows = match if format == "json" {
        parse_json_source(&content)
    } else {
        parse_md_table(&content)
    } {
        Ok((_, r)) => r,
        Err(e) => {
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("source parse error: {}", e),
                source_line: source_line + 1,
            });
            return source_lines[source_line..=line_end].join("\n");
        }
    };

    if rows.is_empty() {
        violations.push(CompileViolation {
            code: "COMPILE-004",
            severity: ViolationSeverity::Warning,
            uri: source_uri.to_string(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "proof:row produced no output — source table {:?} has 0 data rows",
                source_uri
            ),
            source_line: source_line + 1,
        });
    }

    // R-1 invariant check
    let sep_len = separator.chars().count();
    if let Some((found, expected)) = validate_r1(elements, sep_len, declared_width) {
        violations.push(CompileViolation {
            code: "ELEMENT-004",
            severity: ViolationSeverity::Error,
            uri: source_uri.to_string(),
            figure_id: None,
            invariant: format!("R-1: sum(widths) + (n-1)*sep_len = row_width"),
            message: format!(
                "ELEMENT-004: row width mismatch — found={} expected={} (sum of element widths + separators must equal declared width={})",
                found, expected, expected
            ),
            source_line: source_line + 1,
        });
        return source_lines[source_line..=line_end].join("\n");
    }

    let row_cfg = RowConfig {
        source_uri: source_uri.to_string(),
        var_name: String::new(),
        separator: separator.to_string(),
        declared_width,
        elements: elements.to_vec(),
        no_chrome,
    };

    match render_row_foreach(&rows, &row_cfg) {
        Ok(lines) => {
            let rendered = lines.join("\n");
            if no_chrome {
                rendered
            } else {
                format_row_block(source_uri, &rendered)
            }
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "ELEMENT-005",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("row render error: {}", e),
                source_line: source_line + 1,
            });
            source_lines[source_line..=line_end].join("\n")
        }
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

/// Compile a .slides.source.md file into a .slides.md output.
/// Each slide is rendered as a fixed-width ASCII canvas block, separated
/// by a slide divider header showing the slide number.
fn compile_slides_file(
    source_path: &Path,
    output_path: &Path,
    _config: &GlintConfig,
) -> Result<CompileResult> {
    use crate::slide::bullets::has_reveal_markers;
    use crate::slide::layout::{render_slide_pages, render_slide_with_warnings_in_deck};
    use crate::slide::parser::parse_slide_doc;

    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let mut violations: Vec<CompileViolation> = Vec::new();

    let doc = match parse_slide_doc(&source_text) {
        Ok(d) => d,
        Err(errs) => {
            let mut vv = Vec::new();
            for e in errs {
                vv.push(CompileViolation {
                    code: "SLIDE-002",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: e.to_string(),
                    source_line: 0,
                });
            }
            return Ok(CompileResult {
                output_path: output_path.to_path_buf(),
                directives_resolved: 0,
                violations: vv,
                from_cache: false,
                resolved_files: vec![],
                written: false,
            });
        }
    };

    let total = doc.slides.len();
    let meta = &doc.meta;

    // Build output: separator header + slide canvas per slide
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "<!-- proof:compiled from=\"proof:slides\" count={} -->",
        total
    ));
    parts.push(format!("```slides"));

    for slide in &doc.slides {
        let n = slide.index; // parser already 1-indexes slides

        // Use reveal-aware multi-page rendering when [N] markers are present.
        let use_reveal = has_reveal_markers(&slide.body_content);
        let (pages, warnings) = if use_reveal {
            let pgs = render_slide_pages(slide, meta);
            (pgs, Vec::new()) // reveal path: warnings collected via render_slide_with_warnings_in_deck below
        } else {
            let (rendered, warns) = render_slide_with_warnings_in_deck(slide, meta, &doc.slides);
            (vec![rendered], warns)
        };

        // For reveal slides, also run the warning check on the first page render
        let warnings = if use_reveal {
            render_slide_with_warnings_in_deck(slide, meta, &doc.slides).1
        } else {
            warnings
        };

        // Surface bullet warnings
        if !warnings.is_empty() {
            let mut seen: std::collections::HashSet<(&'static str, String)> = Default::default();
            for w in &warnings {
                if seen.insert((w.code, w.message.clone())) {
                    parts.push(format!(
                        "<!-- SLIDE-WARN {} slide={}: {} -->",
                        w.code, n, w.message
                    ));
                    violations.push(CompileViolation {
                        code: w.code,
                        severity: ViolationSeverity::Warning,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("slide {}: {}", n, w.message),
                        source_line: slide.source_line,
                    });
                }
            }
        }

        // Emit one canvas block per reveal page (single page for non-reveal slides)
        let num_pages = pages.len();
        for (page_idx, rendered) in pages.into_iter().enumerate() {
            let separator = format!(
                "SLIDE {} {}",
                n,
                "─".repeat(meta.width.saturating_sub(format!("SLIDE {}  ", n).len()))
            );
            if use_reveal && num_pages > 1 {
                parts.push(format!(
                    "{} {}/{} (reveal {}/{})",
                    separator,
                    n,
                    total,
                    page_idx + 1,
                    num_pages
                ));
            } else {
                parts.push(format!("{} {}/{}", separator, n, total));
            }
            // Progress bar (outside the canvas, between separator and canvas content)
            if meta.progress_bar && total > 0 {
                parts.push(render_progress_bar(n, total, meta.width));
            }
            parts.extend(rendered);
        }
    }
    parts.push("```".to_string());
    parts.push("<!-- /proof:compiled -->".to_string());

    let output_text = parts.join("\n") + "\n";

    // Atomic write
    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path).map_err(|e| anyhow::anyhow!("renaming output: {}", e))?;

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: doc.slides.len(),
        violations,
        from_cache: false,
        resolved_files: vec![],
        written: true,
    })
}

/// Render a `████░░░  N/M` progress bar for slide N of M.
/// Width is `canvas_width`. Bar occupies the full width minus a ` N/M` label.
fn render_progress_bar(n: usize, total: usize, width: usize) -> String {
    let label = format!(" {}/{}", n, total);
    let bar_width = width.saturating_sub(label.len());
    let filled = (bar_width * n / total).min(bar_width);
    let empty = bar_width - filled;
    format!("{}{}{}", "█".repeat(filled), "░".repeat(empty), label)
}

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
    use crate::dashboard::region::{classify_region_line, RegionLine};

    let mut output: Vec<String> = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line = &body[i];
        match classify_region_line(line) {
            RegionLine::Literal(lit) => {
                output.push(lit.to_string());
                i += 1;
            }
            RegionLine::Directive(d) => {
                // Strategy: build a synthetic ```proof:foo ...``` fenced block,
                // run collect_directives on it, dispatch the resulting Directive
                // through render_one_directive_no_chrome.
                let synth = format!("```{}\n```", d);
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
                    for line in rendered.lines() {
                        output.push(line.to_string());
                    }
                } else {
                    // Unrecognized directive — fall through as literal
                    output.push(line.clone());
                }
                i += 1;
            }
        }
    }
    output
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
        Directive::Symbol { name, size, .. } => {
            let lib = crate::symbol::SymbolLibrary::new();
            match crate::symbol::resolve(name, &lib) {
                Some(sym) => {
                    *resolved_count += 1;
                    crate::symbol::render_symbol_block(&sym, *size)
                }
                None => {
                    let hint = crate::symbol::suggest_symbol(name, &lib)
                        .map(|s| format!(" — did you mean '{}'?", s))
                        .unwrap_or_default();
                    violations.push(CompileViolation {
                        code: "SYMBOL-001",
                        severity: ViolationSeverity::Warning,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("Unknown symbol '{}'{}", name, hint),
                        source_line: abs_line + 1,
                    });
                    String::new()
                }
            }
        }
        Directive::Shape { attrs, .. } => match crate::symbol::shape::render_shape(attrs) {
            Ok(rendered) => {
                *resolved_count += 1;
                rendered
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: e.code,
                    severity: ViolationSeverity::Error,
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
            compile_element(
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
            compile_row(
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
            match generate_tree_block(
                kind,
                source.as_deref(),
                inline_body,
                attrs,
                root,
                line_start,
                violations,
            ) {
                Ok(block) => {
                    // generate_tree_block wraps in chrome — strip it for canvas paste.
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
        Directive::Include { uri, .. } => match resolve_uri(uri, root) {
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
        // Layout, Table, Region not supported inline within a region
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
    use crate::compile_directive::{parse_foreach, parse_row_element_line, LayoutAttrs};

    use super::*;
    use crate::compile_directive::proof_directive_kind;

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
        let out = format_include_block("md://figures/foo.md#:0", "CONTENT\nLINE2");
        assert!(out.contains("<!-- proof:compiled from=\"md://figures/foo.md#:0\" -->"));
        assert!(out.contains("<!-- /proof:compiled -->"));
        assert!(out.contains("CONTENT"));
        assert!(out.contains("LINE2"));
    }

    #[test]
    fn test_format_include_block_strips_fence() {
        // Content that arrives already-fenced from older resolve paths
        let out = format_include_block("md://x.md#:0", "```\nFOO\nBAR\n```");
        // Should strip the fence and re-wrap
        assert!(out.contains("FOO"));
        assert!(out.contains("BAR"));
    }

    #[test]
    fn test_format_layout_block_has_uris() {
        let uris = vec!["md://a.md#:0".to_string(), "md://b.md#:0".to_string()];
        let out = format_layout_block(&uris, "COMPOSED");
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
        let (var, uri) =
            parse_foreach("foreach=player in md://stats.md#edm:table:0 separator=\" \"");
        assert_eq!(var, "player");
        assert_eq!(uri, "md://stats.md#edm:table:0");
    }

    // ── parse_row_element_line ────────────────────────────

    #[test]
    fn test_parse_row_element_line_label() {
        let elem =
            parse_row_element_line("proof:element kind=label field=name width=12 align=left")
                .unwrap();
        assert_eq!(elem.field, "name");
        assert_eq!(elem.width, 12);
        assert!(matches!(elem.kind, ElementKind::Label));
    }

    #[test]
    fn test_parse_row_element_line_mini_bar_with_max() {
        let elem = parse_row_element_line("proof:element kind=mini-bar field=pts width=10 max=200")
            .unwrap();
        assert_eq!(elem.field, "pts");
        assert_eq!(elem.width, 10);
        assert_eq!(elem.max, Some(200.0));
        assert!(matches!(elem.kind, ElementKind::MiniBar));
    }

    #[test]
    fn test_parse_row_element_line_non_element_returns_none() {
        assert!(parse_row_element_line("# Comment").is_none());
        assert!(parse_row_element_line("md://stats.md").is_none());
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
        let out = compile_element(
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
        let out = compile_element(
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
        let out = compile_element(
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
        let out = compile_element(
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
        let out = compile_element(
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
        compile_element(
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
        // section= can target any heading, not just H2
        let out = generate_toc(SAMPLE_DOC, 4, "list", Some("Endpoints"));
        assert!(out.contains("GET /widgets"));
        assert!(out.contains("POST /widgets"));
        // Must stop at sibling ### Authentication
        assert!(!out.contains("Authentication"));
    }

    #[test]
    fn toc_section_numbered_renumbers_from_section() {
        let out = generate_toc(SAMPLE_DOC, 4, "numbered", Some("API Reference"));
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
        assert_eq!(heading_slug("Authentication"), "authentication");
        assert_eq!(heading_slug("API Reference"), "api-reference");
        assert_eq!(heading_slug("What's New?"), "whats-new");
    }

    #[test]
    fn xref_inline_renders_see_link() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("api.md");
        std::fs::write(&target, "# API Guide\n\n## Authentication\n\nContent.\n").unwrap();

        let result = render_xref("md://api.md#authentication", None, "inline", dir.path())
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
        let result = render_xref("md://ref.md#background", None, "note", dir.path()).unwrap();
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
        let result = render_xref(
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
        let result = render_xref("md://nonexistent.md", None, "inline", dir.path());
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
        let out = render_blockquote("To be or not to be.", None, "indent");
        assert_eq!(out, "> To be or not to be.");
    }

    #[test]
    fn blockquote_indent_with_attribution() {
        let out = render_blockquote("To be or not to be.", Some("Hamlet"), "indent");
        // Body line, blank quote line, attribution line.
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines, vec!["> To be or not to be.", ">", "> — Hamlet"]);
    }

    #[test]
    fn blockquote_indent_multi_paragraph_preserves_blank_lines() {
        // Inner blank lines stay as `>` (so the rendered markdown is still one
        // contiguous quote, not two adjacent ones).
        let text = "First paragraph.\n\nSecond paragraph.";
        let out = render_blockquote(text, None, "indent");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines,
            vec!["> First paragraph.", ">", "> Second paragraph."]
        );
    }

    #[test]
    fn blockquote_indent_trims_leading_and_trailing_blank_lines() {
        let text = "\n\nThe quote.\n\n";
        let out = render_blockquote(text, None, "indent");
        assert_eq!(out, "> The quote.");
    }

    #[test]
    fn blockquote_unknown_style_falls_back_to_indent() {
        let out_unknown = render_blockquote("hi", None, "marble");
        let out_indent = render_blockquote("hi", None, "indent");
        assert_eq!(
            out_unknown, out_indent,
            "unknown style must fall back to indent (permissive parsing)"
        );
    }

    #[test]
    fn blockquote_boxed_renders_frame() {
        let out = render_blockquote("Hello world", None, "boxed");
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
        let out = render_blockquote("To be.", Some("Hamlet"), "boxed");
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
}
