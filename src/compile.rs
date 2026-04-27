use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::config::GlintConfig;
use crate::davinci::evaluate_invariant;
use crate::element::{ElementConfig, ElementData, ElementKind, ElementAlign, ElementError, render_element};
use crate::element::row::{RowConfig, RowElement, render_row_foreach, validate_r1};
use crate::layout::{self, extract_content_lines, Align, Direction, LayoutConfig};
use crate::runner::Runner;
use crate::diagnostic::Severity;
use crate::tree::schema::{FieldMap, parse_md_table, parse_json_source, generate_org, generate_taxonomy, generate_dependency, generate_outline};
use crate::tree::dirtree::{DirtreeOptions, generate as dirtree_generate};

// ─────────────────────────────────────────────────────────
// Public result types
// ─────────────────────────────────────────────────────────

pub struct CompileResult {
    pub output_path: PathBuf,
    pub directives_resolved: usize,
    pub violations: Vec<CompileViolation>,
    pub from_cache: bool,
    pub written: bool,
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
// Directive types
// ─────────────────────────────────────────────────────────

#[derive(Debug)]
enum Directive {
    Include {
        uri: String,
        line_start: usize,
        line_end: usize,
    },
    Layout {
        uris: Vec<String>,
        attrs: LayoutAttrs,
        line_start: usize,
        line_end: usize,
    },
    Table {
        uri: String,
        line_start: usize,
        line_end: usize,
    },
    Tree {
        kind: String,                   // dirtree | org | taxonomy | dependency | outline
        source: Option<String>,         // md:// URI for schema-driven kinds
        attrs: TreeAttrs,
        line_start: usize,
        line_end: usize,
    },
    Element {
        kind: String,                   // value | delta | sparkline | mini-bar | label | badge
        source: Option<String>,         // md:// URI (absent if inline_value is set)
        field: Option<String>,          // column name in source table
        inline_value: Option<String>,   // from value="..." attribute
        attrs: ElementAttrs,
        line_start: usize,
        line_end: usize,
    },
    Row {
        source_uri: String,
        var_name: String,
        separator: String,
        declared_width: Option<usize>,
        elements: Vec<RowElement>,
        no_chrome: bool,
        line_start: usize,
        line_end: usize,
    },
}

/// Parsed attributes from a proof:tree directive.
#[derive(Debug, Default)]
pub struct TreeAttrs {
    pub name: Option<String>,
    pub parent: Option<String>,
    pub label: Option<String>,
    pub format: String,              // "table" (default) or "json"
    pub indent_width: usize,         // default: 4
    pub root: Option<String>,        // for dirtree: filesystem root
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>,
}

impl TreeAttrs {
    fn parse(attrs_str: &str) -> Self {
        let mut out = TreeAttrs {
            format: "table".to_string(),
            indent_width: 4,
            ..Default::default()
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            if let Some(eq) = rest.find('=') {
                let key = rest[..eq].trim();
                rest = &rest[eq + 1..];
                let (val, next) = if rest.starts_with('"') {
                    if let Some(close) = rest[1..].find('"') {
                        (&rest[1..close + 1], &rest[close + 2..])
                    } else { ("", "") }
                } else {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (&rest[..end], &rest[end..])
                };
                match key {
                    "name"         => out.name = Some(val.to_string()),
                    "parent"       => out.parent = Some(val.to_string()),
                    "label"        => out.label = Some(val.to_string()),
                    "format"       => out.format = val.to_string(),
                    "indent-width" => out.indent_width = val.parse().unwrap_or(4),
                    "root"         => out.root = Some(val.to_string()),
                    "max-depth"    => out.max_depth = val.parse().ok(),
                    "exclude"      => out.exclude = val.split(',').map(|s| s.trim().to_string()).collect(),
                    _ => {}
                }
                rest = next.trim_start();
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                rest = &rest[end..].trim_start();
            }
        }
        out
    }
}

/// Parsed attributes from a proof:element directive.
#[derive(Debug, Default)]
pub struct ElementAttrs {
    pub width: Option<usize>,
    pub align: String,      // "left" | "right" | "center" — default "left"
    pub format: String,     // "{:.1}" etc. — default "{}"
    pub no_chrome: bool,
    pub max: Option<f64>,
    pub fill: char,         // default '█'
    pub empty: char,        // default '░'
}

impl ElementAttrs {
    fn parse(attrs_str: &str) -> Self {
        let mut out = ElementAttrs {
            align: "left".to_string(),
            format: "{}".to_string(),
            fill: '█',
            empty: '░',
            ..Default::default()
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            // Find the next whitespace-delimited token
            let tok_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let tok = &rest[..tok_end];

            if let Some(eq) = tok.find('=') {
                // key=value token
                let key = tok[..eq].trim();
                let after_eq = &tok[eq + 1..];
                // Value may span into quoted region — re-parse from rest after key=
                let val_start = &rest[eq + 1..];
                let (val, consumed) = if val_start.starts_with('"') {
                    if let Some(close) = val_start[1..].find('"') {
                        (&val_start[1..close + 1], eq + 1 + close + 2)
                    } else { (after_eq, tok_end) }
                } else {
                    (after_eq, tok_end)
                };
                match key {
                    "width"     => out.width = val.parse().ok(),
                    "align"     => out.align = val.to_string(),
                    "format"    => out.format = val.to_string(),
                    "max"       => out.max = val.parse().ok(),
                    "fill"      => out.fill = val.chars().next().unwrap_or('█'),
                    "empty"     => out.empty = val.chars().next().unwrap_or('░'),
                    "no-chrome" => out.no_chrome = matches!(val, "true" | "1" | ""),
                    _ => {}
                }
                rest = rest[consumed..].trim_start();
            } else {
                // Bare flag (no '=' in token)
                if tok == "no-chrome" { out.no_chrome = true; }
                rest = rest[tok_end..].trim_start();
            }
        }
        out
    }
}

impl Directive {
    fn line_start(&self) -> usize {
        match self {
            Directive::Include { line_start, .. } => *line_start,
            Directive::Layout { line_start, .. } => *line_start,
            Directive::Table { line_start, .. } => *line_start,
            Directive::Tree { line_start, .. } => *line_start,
            Directive::Element { line_start, .. } => *line_start,
            Directive::Row { line_start, .. } => *line_start,
        }
    }
    fn line_end(&self) -> usize {
        match self {
            Directive::Include { line_end, .. } => *line_end,
            Directive::Layout { line_end, .. } => *line_end,
            Directive::Table { line_end, .. } => *line_end,
            Directive::Tree { line_end, .. } => *line_end,
            Directive::Element { line_end, .. } => *line_end,
            Directive::Row { line_end, .. } => *line_end,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Layout attribute parsing
// ─────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct LayoutAttrs {
    gap: usize,
    align: String,
    labels: Vec<String>,
    cols: Option<usize>,
    width: usize,
    direction: String,
    border: bool,
}

impl LayoutAttrs {
    fn parse(attrs_str: &str) -> Self {
        let mut out = LayoutAttrs {
            gap: 3,
            align: "top".to_string(),
            labels: Vec::new(),
            cols: None,
            width: 120,
            direction: "horizontal".to_string(),
            border: false,
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            if let Some(eq_pos) = rest.find('=') {
                let key = rest[..eq_pos].trim();
                rest = &rest[eq_pos + 1..];
                let (val, next) = if rest.starts_with('"') {
                    if let Some(close) = rest[1..].find('"') {
                        (&rest[1..close + 1], &rest[close + 2..])
                    } else {
                        ("", "")
                    }
                } else {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (&rest[..end], &rest[end..])
                };
                match key {
                    "gap"       => out.gap = val.parse().unwrap_or(3),
                    "align"     => out.align = val.to_string(),
                    "labels"    => out.labels = val.split(',').map(|s| s.to_string()).collect(),
                    "cols"      => out.cols = val.parse().ok(),
                    "width"     => out.width = val.parse().unwrap_or(120),
                    "direction" => out.direction = val.to_string(),
                    "border"    => out.border = matches!(val, "true" | "1" | ""),
                    _ => {}
                }
                rest = next.trim_start();
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                let key = rest[..end].trim();
                if key == "border" { out.border = true; }
                rest = &rest[end..].trim_start();
            }
        }
        out
    }

    fn to_layout_config(self) -> LayoutConfig {
        LayoutConfig {
            gap: self.gap,
            align: Align::parse(&self.align).unwrap_or(Align::Top),
            labels: self.labels,
            cols: self.cols,
            width: self.width,
            direction: Direction::parse(&self.direction).unwrap_or(Direction::Horizontal),
            border: self.border,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Directive parsing
// ─────────────────────────────────────────────────────────

pub fn parse_directives(source: &str) -> Vec<(usize, usize, String, String)> {
    // Returns (line_start, line_end, kind, body) for quick inspection — used by tests
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if let Some(kind) = proof_directive_kind(trimmed) {
            let start = i;
            let info = trimmed[3..].to_string(); // after ```
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() {
                let l = lines[i].trim();
                if l == "```" || l == "~~~" { break; }
                body.push(lines[i]);
                i += 1;
            }
            out.push((start, i, kind.to_string(), body.join("\n")));
        }
        i += 1;
    }
    out
}

fn collect_directives(source: &str) -> Vec<Directive> {
    let lines: Vec<&str> = source.lines().collect();
    let mut directives = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if let Some(kind) = proof_directive_kind(trimmed) {
            let line_start = i;
            let info_after_backticks = trimmed[3..].to_string(); // "proof:layout gap=4 ..."
            let mut body: Vec<&str> = Vec::new();
            i += 1;
            while i < lines.len() {
                let l = lines[i].trim();
                if l == "```" || l == "~~~" { break; }
                body.push(lines[i]);
                i += 1;
            }
            let line_end = i;
            match kind {
                "include" => {
                    if let Some(uri) = body.iter().find_map(|l| {
                        let t = l.trim();
                        if !t.is_empty() { Some(t.to_string()) } else { None }
                    }) {
                        directives.push(Directive::Include { uri, line_start, line_end });
                    }
                }
                "layout" => {
                    let attrs_str = info_after_backticks
                        .strip_prefix("proof:layout")
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let attrs = LayoutAttrs::parse(&attrs_str);
                    let uris: Vec<String> = body.iter()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .map(|l| l.to_string())
                        .collect();
                    directives.push(Directive::Layout { uris, attrs, line_start, line_end });
                }
                "table" => {
                    let uri = body.iter()
                        .find_map(|l| {
                            let t = l.trim();
                            if t.starts_with("md://") { Some(t.to_string()) } else { None }
                        })
                        .unwrap_or_default();
                    if !uri.is_empty() {
                        directives.push(Directive::Table { uri, line_start, line_end });
                    }
                }
                "tree" => {
                    // Info string: "proof:tree kind=org name="Employee" parent="Manager""
                    let info_after = info_after_backticks
                        .strip_prefix("proof:tree")
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    // Extract kind from attrs (first key=value or standalone word)
                    let kind = info_after
                        .split_whitespace()
                        .find_map(|tok| {
                            if tok.starts_with("kind=") {
                                Some(tok.strip_prefix("kind=").unwrap_or("dirtree")
                                    .trim_matches('"').to_string())
                            } else if !tok.contains('=') {
                                Some(tok.to_string()) // bare kind name
                            } else { None }
                        })
                        .unwrap_or_else(|| "dirtree".to_string());

                    let attrs = TreeAttrs::parse(&info_after);

                    // Source URI is the first md:// line in the body (for schema kinds)
                    let source = body.iter().find_map(|l| {
                        let t = l.trim();
                        if t.starts_with("md://") { Some(t.to_string()) } else { None }
                    });

                    directives.push(Directive::Tree { kind, source, attrs, line_start, line_end });
                }
                "element" => {
                    // Info string: "proof:element kind=value field=pts_82 width=8 align=right"
                    let info_after = info_after_backticks
                        .strip_prefix("proof:element")
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    // Extract kind= from attrs
                    let kind = extract_attr_value(&info_after, "kind")
                        .unwrap_or_else(|| "value".to_string());

                    // Extract field= and value= (inline literal)
                    let field = extract_attr_value(&info_after, "field");
                    let inline_value = extract_attr_value(&info_after, "value");

                    let attrs = ElementAttrs::parse(&info_after);

                    // Source URI is the first md:// line in the body
                    let source = body.iter().find_map(|l| {
                        let t = l.trim();
                        if t.starts_with("md://") { Some(t.to_string()) } else { None }
                    });

                    directives.push(Directive::Element {
                        kind, source, field, inline_value, attrs, line_start, line_end,
                    });
                }
                "row" => {
                    // Info string: "proof:row foreach=player in md://stats.md#edm:table:0 separator=" " width=120"
                    let info_after = info_after_backticks
                        .strip_prefix("proof:row")
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    // Parse foreach=VAR in URI
                    let (var_name, source_uri) = parse_foreach(&info_after);
                    let separator = extract_attr_value(&info_after, "separator")
                        .unwrap_or_else(|| " ".to_string());
                    let declared_width = extract_attr_value(&info_after, "width")
                        .and_then(|v| v.parse().ok());
                    let no_chrome = info_after.split_whitespace()
                        .any(|t| t == "no-chrome" || t == "no-chrome=true" || t == "no-chrome=1");

                    // Body lines: each "proof:element ..." line becomes a RowElement
                    let elements: Vec<RowElement> = body.iter()
                        .filter_map(|l| parse_row_element_line(l.trim()))
                        .collect();

                    if !source_uri.is_empty() {
                        directives.push(Directive::Row {
                            source_uri,
                            var_name,
                            separator,
                            declared_width,
                            elements,
                            no_chrome,
                            line_start,
                            line_end,
                        });
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    directives
}

/// Extract a quoted or unquoted value for `key=` from an attribute string.
fn extract_attr_value(attrs: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    let mut rest = attrs;
    while !rest.is_empty() {
        if let Some(pos) = rest.find(&prefix) {
            // Ensure it's a word boundary (not mid-identifier)
            if pos > 0 {
                let prev = rest.as_bytes()[pos - 1] as char;
                if prev.is_alphanumeric() || prev == '-' || prev == '_' {
                    rest = &rest[pos + 1..];
                    continue;
                }
            }
            let after = &rest[pos + prefix.len()..];
            let val = if after.starts_with('"') {
                after[1..].find('"').map(|e| after[1..e + 1].to_string())
            } else {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                if end > 0 { Some(after[..end].to_string()) } else { None }
            };
            return val;
        } else {
            break;
        }
    }
    None
}

fn proof_directive_kind(line: &str) -> Option<&'static str> {
    let line = line.trim_start();
    if !line.starts_with("```proof:") { return None; }
    let rest = &line[9..]; // after "```proof:"
    if rest.starts_with("include") { Some("include") }
    else if rest.starts_with("layout")  { Some("layout") }
    else if rest.starts_with("table")   { Some("table") }
    else if rest.starts_with("tree")    { Some("tree") }
    else if rest.starts_with("element") { Some("element") }
    else if rest.starts_with("row")     { Some("row") }
    else { None }
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
    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let source_lines: Vec<&str> = source_text.lines().collect();
    let directives = collect_directives(&source_text);

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
            Directive::Include { uri, .. } => {
                match resolve_uri(uri, root) {
                    Ok((content, fig_file)) => {
                        lint_figure(uri, &content, &fig_file, line_start + 1, &runner, &mut violations);
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
                    match resolve_uri(uri, root) {
                        Ok((content, fig_file)) => {
                            lint_figure(uri, &content, &fig_file, line_start + 1, &runner, &mut violations);
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
                    let layout_config = LayoutConfig {
                        gap: attrs.gap,
                        align: Align::parse(&attrs.align).unwrap_or(Align::Top),
                        labels: attrs.labels.clone(),
                        cols: attrs.cols,
                        width: attrs.width,
                        direction: Direction::parse(&attrs.direction).unwrap_or(Direction::Horizontal),
                        border: attrs.border,
                    };

                    let composed = layout::layout(figures, &layout_config);
                    // Strip outer ``` wrapper — compile embeds content inline
                    let inner = composed
                        .strip_prefix("```\n")
                        .and_then(|s| s.strip_suffix("\n```"))
                        .unwrap_or(&composed);
                    format_layout_block(uris, inner)
                }
            }

            Directive::Table { uri, .. } => {
                match resolve_uri(uri, root) {
                    Ok((content, fig_file)) => {
                        lint_figure(uri, &content, &fig_file, line_start + 1, &runner, &mut violations);
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

            Directive::Tree { kind, source, attrs, .. } => {
                generate_tree_block(kind, source.as_deref(), attrs, root, line_start, &mut violations)
                    .unwrap_or_else(|e| {
                        violations.push(CompileViolation {
                            code: "COMPILE-002",
                            severity: ViolationSeverity::Error,
                            uri: source.clone().unwrap_or_default(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("tree generation failed: {}", e),
                            source_line: line_start + 1,
                        });
                        source_lines[line_start..=line_end].join("\n")
                    })
            }

            Directive::Element { kind, source, field, inline_value, attrs, .. } => {
                compile_element(
                    kind, source.as_deref(), field.as_deref(), inline_value.as_deref(),
                    attrs, root, line_start,
                    &mut violations, &source_lines, line_end,
                    &mut resolved_count,
                )
            }

            Directive::Row { source_uri, var_name: _, separator, declared_width, elements, no_chrome, .. } => {
                compile_row(
                    source_uri, separator, *declared_width, elements, *no_chrome,
                    root, line_start, &mut violations, &source_lines, line_end,
                    &mut resolved_count,
                )
            }
        };

        replacements.push((line_start, line_end, replacement));
    }

    // Collect all error-level violations
    let has_errors = violations.iter().any(|v| v.severity == ViolationSeverity::Error);
    if has_errors {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            written: false,
        });
    }

    // Rebuild source with replacements applied, preserving trailing newline
    let had_trailing_newline = source_text.ends_with('\n');
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

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: resolved_count,
        violations,
        from_cache: false,
        written: true,
    })
}

// ─────────────────────────────────────────────────────────
// URI resolution + figure lint validation
// ─────────────────────────────────────────────────────────

fn resolve_uri(uri: &str, root: &Path) -> Result<(String, PathBuf)> {
    let parsed = mdpath::parse(uri)
        .map_err(|e| anyhow::anyhow!("invalid md:// URI {:?}: {}", uri, e))?;
    let element = mdpath::resolve(&parsed, root)
        .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", uri, e))?;
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
    let errors: Vec<_> = diags.iter()
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
        let uri_matches = entry.uri == uri
            || uri.ends_with(&entry.uri)
            || entry.uri.ends_with(uri);
        if !uri_matches {
            continue;
        }
        for inv in &entry.invariants {
            if let Err(msg) = evaluate_invariant(inv, content) {
                use crate::config::ProtectionTier;
                let (code, sev) = match entry.protection {
                    ProtectionTier::Error | ProtectionTier::Lock =>
                        ("COMPILE-001", ViolationSeverity::Error),
                    ProtectionTier::Warn =>
                        ("COMPILE-003", ViolationSeverity::Warning),
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
    attrs: &TreeAttrs,
    root: &Path,
    source_line: usize,
    _violations: &mut Vec<CompileViolation>,
) -> Result<String> {
    let body = match kind {
        "dirtree" => {
            let tree_root = attrs.root.as_ref()
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
            let src_uri = source.ok_or_else(|| {
                anyhow::anyhow!("proof:tree kind={} requires a source URI in the body", kind)
            })?;
            let content = resolve_source_for_compile(src_uri, root)?;
            let mut map = FieldMap::default();
            match kind {
                "org"        => generate_org(&content, &attrs.format, &mut map, attrs.indent_width)?,
                "taxonomy"   => generate_taxonomy(&content, &attrs.format, &mut map, attrs.indent_width)?,
                "dependency" => generate_dependency(&content, &attrs.format, &mut map, attrs.indent_width)?,
                "outline"    => generate_outline(&content, attrs.indent_width)?,
                other => anyhow::bail!("unknown tree kind {:?}", other),
            }
        }
    };

    let uris = source.map(|s| s.to_string()).unwrap_or_default();
    Ok(format!(
        "<!-- proof:compiled from=\"proof:tree kind={}\" uri=\"{}\" -->\n```{}\n{}\n```\n<!-- /proof:compiled -->",
        kind, uris, kind, body
    ))
}

fn resolve_source_for_compile(src: &str, root: &Path) -> Result<String> {
    if src.starts_with("md://") {
        let parsed = mdpath::parse(src)
            .map_err(|e| anyhow::anyhow!("invalid URI {:?}: {}", src, e))?;
        let element = mdpath::resolve(&parsed, root)
            .map_err(|e| anyhow::anyhow!("cannot resolve {:?}: {}", src, e))?;
        Ok(element.content)
    } else {
        let path = root.join(src);
        std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e))
    }
}

// ─────────────────────────────────────────────────────────
// proof:element compile arm
// ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
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
                    message: "proof:element requires either value=\"...\" or a source URI in the body".to_string(),
                    source_line: source_line + 1,
                });
                return source_lines[source_line..=line_end].join("\n");
            }
        };

        // Resolve URI content
        let content = match resolve_source_for_compile(src_uri, root) {
            Ok(c) => { *resolved_count += 1; c }
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
                return source_lines[source_line..=line_end].join("\n");
            }
        };

        // Parse source table
        let format = if src_uri.ends_with(".json") { "json" } else { "table" };
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
                return source_lines[source_line..=line_end].join("\n");
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
                    message: "proof:element with a source URI requires field=\"ColumnName\"".to_string(),
                    source_line: source_line + 1,
                });
                return source_lines[source_line..=line_end].join("\n");
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
                return source_lines[source_line..=line_end].join("\n");
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
                return source_lines[source_line..=line_end].join("\n");
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
            return source_lines[source_line..=line_end].join("\n");
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
            return source_lines[source_line..=line_end].join("\n");
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
                    return source_lines[source_line..=line_end].join("\n");
                }
            }
        }
        ElementKind::Label | ElementKind::Badge => {
            ElementData::Text(raw_value.clone())
        }
        _ => {
            // value, delta, mini-bar: scalar
            match raw_value.parse::<f64>() {
                Ok(v) => ElementData::Scalar(v),
                Err(_) => {
                    violations.push(CompileViolation {
                        code: "ELEMENT-002",
                        severity: ViolationSeverity::Error,
                        uri: uri_str.to_string(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("element kind={} requires a numeric value; got {:?}", kind, raw_value),
                        source_line: source_line + 1,
                    });
                    return source_lines[source_line..=line_end].join("\n");
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
                message: format!("rendered element width {} exceeds budget {}", actual, budget),
                source_line: source_line + 1,
            });
            source_lines[source_line..=line_end].join("\n")
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
            source_lines[source_line..=line_end].join("\n")
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
// proof:row foreach parsing helpers
// ─────────────────────────────────────────────────────────

/// Parse `foreach=VAR in URI` from the info string after `proof:row`.
/// Returns (var_name, source_uri). Both empty strings on parse failure.
fn parse_foreach(info: &str) -> (String, String) {
    // Expect: foreach=VARNAME in md://...
    let mut var_name = String::new();
    let mut source_uri = String::new();

    for tok in info.split_whitespace() {
        if tok.starts_with("foreach=") {
            var_name = tok["foreach=".len()..].to_string();
        } else if tok.starts_with("md://") && !var_name.is_empty() {
            source_uri = tok.to_string();
        }
    }
    (var_name, source_uri)
}

/// Parse a body line of the form `proof:element kind=X field=Y width=N ...`
/// into a RowElement. Returns None if the line doesn't start with `proof:element`.
fn parse_row_element_line(line: &str) -> Option<RowElement> {
    let rest = line.strip_prefix("proof:element")?.trim();
    let attrs = ElementAttrs::parse(rest);
    let kind_str = extract_attr_value(rest, "kind").unwrap_or_else(|| "value".to_string());
    let kind = ElementKind::parse(&kind_str)?;
    let field = extract_attr_value(rest, "field").unwrap_or_default();
    let width = attrs.width.unwrap_or(0);
    if field.is_empty() || width == 0 { return None; }
    Some(RowElement {
        kind,
        field,
        width,
        align: ElementAlign::parse(&attrs.align),
        format: attrs.format,
        max: attrs.max,
        fill_char: attrs.fill,
        empty_char: attrs.empty,
    })
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
        Ok(c) => { *resolved_count += 1; c }
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
            return source_lines[source_line..=line_end].join("\n");
        }
    };

    // Parse table
    let format = if source_uri.ends_with(".json") { "json" } else { "table" };
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
            code: "ELEMENT-007",
            severity: ViolationSeverity::Error,
            uri: source_uri.to_string(),
            figure_id: None,
            invariant: String::new(),
            message: "proof:row source resolved to zero rows".to_string(),
            source_line: source_line + 1,
        });
        return source_lines[source_line..=line_end].join("\n");
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

pub fn derive_output_path(source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?.to_str()?;
    if let Some(stem) = name.strip_suffix(".source.md") {
        let out_name = format!("{}.md", stem);
        Some(source.parent().unwrap_or(Path::new(".")).join(out_name))
    } else {
        None
    }
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
        let lines = vec!["before", "```proof:include", "md://a", "```", "middle", "```proof:include", "md://b", "```", "after"];
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
            Directive::Include { uri, line_start, line_end } => {
                assert_eq!(uri, "md://fig.md#:0");
                assert_eq!(*line_start, 2);
                assert_eq!(*line_end, 4);
            }
            _ => panic!("expected Include"),
        }
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
            Directive::Row { source_uri, var_name, elements, .. } => {
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
        assert_eq!(proof_directive_kind("```proof:row foreach=p in md://x.md"), Some("row"));
    }

    // ── parse_foreach ────────────────────────────────────

    #[test]
    fn test_parse_foreach_extracts_var_and_uri() {
        let (var, uri) = parse_foreach("foreach=player in md://stats.md#edm:table:0 separator=\" \"");
        assert_eq!(var, "player");
        assert_eq!(uri, "md://stats.md#edm:table:0");
    }

    // ── parse_row_element_line ────────────────────────────

    #[test]
    fn test_parse_row_element_line_label() {
        let elem = parse_row_element_line("proof:element kind=label field=name width=12 align=left").unwrap();
        assert_eq!(elem.field, "name");
        assert_eq!(elem.width, 12);
        assert!(matches!(elem.kind, ElementKind::Label));
    }

    #[test]
    fn test_parse_row_element_line_mini_bar_with_max() {
        let elem = parse_row_element_line("proof:element kind=mini-bar field=pts width=10 max=200").unwrap();
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
            RowElement { kind: ElementKind::Label, field: "n".into(), width: 10, align: ElementAlign::Left, format: "{}".into(), max: None, fill_char: '█', empty_char: '░' },
            RowElement { kind: ElementKind::Value, field: "p".into(), width: 5, align: ElementAlign::Right, format: "{}".into(), max: None, fill_char: '█', empty_char: '░' },
        ];
        // sum=15, sep_len=1, n=2 → total=16, declared=16 → OK
        assert!(validate_r1(&elems, 1, Some(16)).is_none());
    }

    #[test]
    fn test_validate_r1_violation() {
        use crate::element::row::validate_r1;
        use crate::element::row::RowElement;
        let elems = vec![
            RowElement { kind: ElementKind::Label, field: "n".into(), width: 10, align: ElementAlign::Left, format: "{}".into(), max: None, fill_char: '█', empty_char: '░' },
            RowElement { kind: ElementKind::Value, field: "p".into(), width: 5, align: ElementAlign::Right, format: "{}".into(), max: None, fill_char: '█', empty_char: '░' },
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
            Directive::Element { kind, field, attrs, .. } => {
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
        let attrs = ElementAttrs::parse("kind=value field=pts width=8 align=right format=\"{:.1}\" max=200 fill=▓ empty=░");
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
        assert_eq!(proof_directive_kind("```proof:element kind=value width=4"), Some("element"));
    }

    // E2E tests using compile_element directly (no file I/O)

    #[test]
    fn test_e2e_element_value_inline() {
        let attrs = ElementAttrs { width: Some(4), align: "right".to_string(), format: "{}".to_string(), no_chrome: false, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=value value=\"42\" width=4 align=right", "```"];
        let out = compile_element("value", None, None, Some("42"), &attrs, Path::new("."), 0, &mut violations, &lines, 1, &mut 0);
        assert!(violations.is_empty(), "should have no violations: {:?}", violations.iter().map(|v| v.message.as_str()).collect::<Vec<_>>());
        assert!(out.contains("42"), "output should contain value: {:?}", out);
        let value_w = crate::layout::visual_width(&" 42");
        assert_eq!(value_w, 3);
    }

    #[test]
    fn test_e2e_element_label_inline() {
        let attrs = ElementAttrs { width: Some(8), align: "left".to_string(), format: "{}".to_string(), no_chrome: false, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=label value=\"McDavid\" width=8 align=left", "```"];
        let out = compile_element("label", None, None, Some("McDavid"), &attrs, Path::new("."), 0, &mut violations, &lines, 1, &mut 0);
        assert!(violations.is_empty(), "should have no violations: {:?}", violations.iter().map(|v| v.message.as_str()).collect::<Vec<_>>());
        assert!(out.contains("McDavid"), "output should contain label: {:?}", out);
    }

    #[test]
    fn test_e2e_element_badge_inline() {
        let attrs = ElementAttrs { width: Some(5), align: "left".to_string(), format: "{}".to_string(), no_chrome: false, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=badge value=\"UFA\" width=5", "```"];
        let out = compile_element("badge", None, None, Some("UFA"), &attrs, Path::new("."), 0, &mut violations, &lines, 1, &mut 0);
        assert!(violations.is_empty(), "violations: {:?}", violations.iter().map(|v| v.message.as_str()).collect::<Vec<_>>());
        assert!(out.contains("UFA"), "output: {:?}", out);
    }

    #[test]
    fn test_e2e_element_no_chrome_true() {
        let attrs = ElementAttrs { width: Some(5), align: "left".to_string(), format: "{}".to_string(), no_chrome: true, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=label value=\"Hi\" width=5 no-chrome", "```"];
        let out = compile_element("label", None, None, Some("Hi"), &attrs, Path::new("."), 0, &mut violations, &lines, 1, &mut 0);
        assert!(violations.is_empty(), "violations: {:?}", violations.iter().map(|v| v.message.as_str()).collect::<Vec<_>>());
        assert!(!out.contains("```"), "no-chrome should have no fence: {:?}", out);
        assert!(!out.contains("<!--"), "no-chrome should have no HTML comment: {:?}", out);
    }

    #[test]
    fn test_e2e_element_no_chrome_false_has_wrapper() {
        let attrs = ElementAttrs { width: Some(5), align: "left".to_string(), format: "{}".to_string(), no_chrome: false, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=label value=\"Hi\" width=5", "```"];
        let out = compile_element("label", None, None, Some("Hi"), &attrs, Path::new("."), 0, &mut violations, &lines, 1, &mut 0);
        assert!(violations.is_empty(), "violations: {:?}", violations.iter().map(|v| v.message.as_str()).collect::<Vec<_>>());
        assert!(out.contains("<!-- proof:compiled"), "should have traceability comment: {:?}", out);
        assert!(out.contains("```"), "should have fence: {:?}", out);
    }

    #[test]
    fn test_e2e_element_missing_field_emits_element_005() {
        // Simulate a source table with a known header, but ask for a missing field
        let attrs = ElementAttrs { width: Some(6), align: "left".to_string(), format: "{}".to_string(), no_chrome: false, ..Default::default() };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=value field=absent width=6", "md://test", "```"];
        // Use inline value to avoid file I/O, but pass field= with no source → triggers ELEMENT-005 (missing source)
        compile_element("value", None, Some("absent"), None, &attrs, Path::new("."), 0, &mut violations, &lines, 2, &mut 0);
        // Should emit ELEMENT-005 because source is None and inline_value is None
        let codes: Vec<&str> = violations.iter().map(|v| v.code).collect();
        assert!(codes.contains(&"ELEMENT-005"), "expected ELEMENT-005, got: {:?}", codes);
    }
}
