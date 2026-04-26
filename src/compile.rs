use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::config::GlintConfig;
use crate::davinci::evaluate_invariant;
use crate::layout::{self, extract_content_lines, Align, Direction, LayoutConfig};
use crate::runner::Runner;
use crate::diagnostic::Severity;

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
}

impl Directive {
    fn line_start(&self) -> usize {
        match self {
            Directive::Include { line_start, .. } => *line_start,
            Directive::Layout { line_start, .. } => *line_start,
            Directive::Table { line_start, .. } => *line_start,
        }
    }
    fn line_end(&self) -> usize {
        match self {
            Directive::Include { line_end, .. } => *line_end,
            Directive::Layout { line_end, .. } => *line_end,
            Directive::Table { line_end, .. } => *line_end,
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
                _ => {}
            }
        }
        i += 1;
    }
    directives
}

fn proof_directive_kind(line: &str) -> Option<&'static str> {
    let line = line.trim_start();
    if !line.starts_with("```proof:") { return None; }
    let rest = &line[9..]; // after "```proof:"
    if rest.starts_with("include") { Some("include") }
    else if rest.starts_with("layout") { Some("layout") }
    else if rest.starts_with("table")  { Some("table") }
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
                    let layout_config = LayoutAttrs::parse(
                        // Re-parse from attrs since we can't move out of the match binding
                        &format!(
                            "gap={} align={} width={} direction={} border={}{}{}",
                            attrs.gap,
                            attrs.align,
                            attrs.width,
                            attrs.direction,
                            if attrs.border { "true" } else { "false" },
                            if let Some(c) = attrs.cols { format!(" cols={}", c) } else { String::new() },
                            if !attrs.labels.is_empty() { format!(" labels={:?}", attrs.labels.join(",")) } else { String::new() },
                        )
                    ).to_layout_config();

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

    // Rebuild source with replacements applied
    let output_text = apply_replacements(&source_lines, &replacements);

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
}
