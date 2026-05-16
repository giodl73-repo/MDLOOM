use anyhow::Result;
use std::path::Path;

use crate::compile_directive::TreeAttrs;
use crate::compile_output;
use crate::compile_source;
use crate::compile_types::{CompileViolation, ViolationSeverity};
use crate::tree::dirtree::{generate as dirtree_generate, DirtreeOptions};
use crate::tree::schema::{
    generate_decision, generate_dependency, generate_org, generate_outline, generate_taxonomy,
    FieldMap,
};

pub(crate) struct TreeRenderWarning {
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) source_line: usize,
}

pub(crate) fn generate_tree_block(
    kind: &str,
    source: Option<&str>,
    inline_body: &[String],
    attrs: &TreeAttrs,
    root: &Path,
    source_line: usize,
    warnings: &mut Vec<TreeRenderWarning>,
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
                let content = compile_source::resolve_source_for_compile(src_uri, root)?;
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
                    "decision" => {
                        generate_decision(&content, &attrs.format, &mut map, attrs.indent_width)?
                    }
                    other => anyhow::bail!("unknown tree kind {:?}", other),
                }
            } else if !inline_body.is_empty() {
                let content = inline_body.join("\n");
                let mut map = FieldMap {
                    name: attrs.name.clone(),
                    parent: attrs.parent.clone(),
                    label: attrs.label.clone(),
                    ..Default::default()
                };
                match kind {
                    "org" | "taxonomy" | "dependency" => {
                        render_inline_tree(&content, attrs.indent_width)?
                    }
                    "outline" => render_inline_outline(
                        &content,
                        attrs.indent_width,
                        source_line + 1,
                        warnings,
                    )?,
                    "decision" => {
                        generate_decision(&content, &attrs.format, &mut map, attrs.indent_width)?
                    }
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_tree(
    kind: &str,
    source: Option<&String>,
    inline_body: &[String],
    attrs: &TreeAttrs,
    root: &Path,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    let mut tree_warnings = Vec::new();
    match generate_tree_block(
        kind,
        source.map(|s| s.as_str()),
        inline_body,
        attrs,
        root,
        line_start + source_line_offset,
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
            block
        }
        Err(e) => {
            let severity = if attrs.stub {
                ViolationSeverity::Warning
            } else {
                ViolationSeverity::Error
            };
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity,
                uri: source.cloned().unwrap_or_default(),
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
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

pub(crate) fn render_inline_tree(content: &str, indent_width: usize) -> Result<String> {
    let render_iw = indent_width.max(2);
    let mut parsed: Vec<(usize, String, bool)> = Vec::new();
    for line in content.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            continue;
        }

        if let Some(rest) = trimmed_end.strip_prefix("root:") {
            parsed.push((0, rest.trim().to_string(), false));
            continue;
        }

        let ws_len = line.len() - line.trim_start().len();
        let after_ws = &line[ws_len..];
        let (has_bullet, label_start) = if let Some(rest) = after_ws.strip_prefix("- ") {
            (true, rest)
        } else if after_ws == "-" {
            (true, "")
        } else {
            (false, after_ws)
        };
        let label = label_start.trim().to_string();
        if label.is_empty() {
            continue;
        }
        parsed.push((ws_len, label, has_bullet));
    }

    if parsed.is_empty() {
        anyhow::bail!("inline tree body is empty");
    }

    let parse_iw = parsed
        .iter()
        .filter_map(|(ws, _, bullet)| if *bullet && *ws > 0 { Some(*ws) } else { None })
        .min()
        .unwrap_or(2);

    let mut nodes: Vec<(usize, String)> = Vec::with_capacity(parsed.len());
    let mut have_root = false;
    for (i, (ws, label, has_bullet)) in parsed.iter().enumerate() {
        if !has_bullet && i == 0 && !have_root {
            nodes.push((0, label.clone()));
            have_root = true;
            continue;
        }
        let depth = (ws / parse_iw) + 1;
        nodes.push((depth, label.clone()));
    }

    let mut out = String::new();
    for (i, (depth, label)) in nodes.iter().enumerate() {
        if *depth == 0 {
            out.push_str(label);
            out.push('\n');
            continue;
        }
        let mut prefix = String::new();
        for ancestor in 1..*depth {
            if is_ancestor_level_open(&nodes, i, ancestor) {
                prefix.push('│');
                for _ in 0..render_iw.saturating_sub(1) {
                    prefix.push(' ');
                }
            } else {
                for _ in 0..render_iw {
                    prefix.push(' ');
                }
            }
        }
        let is_last = nodes[i + 1..]
            .iter()
            .find(|(d, _)| *d <= *depth)
            .is_none_or(|(d, _)| *d < *depth);
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(&prefix);
        out.push_str(connector);
        out.push_str(label);
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

fn is_ancestor_level_open(nodes: &[(usize, String)], pos: usize, level: usize) -> bool {
    for (d, _) in &nodes[pos + 1..] {
        if *d < level {
            return false;
        }
        if *d == level {
            return true;
        }
    }
    false
}

pub(crate) fn render_inline_outline(
    content: &str,
    indent_width: usize,
    source_line: usize,
    warnings: &mut Vec<TreeRenderWarning>,
) -> Result<String> {
    let has_dash_bullet = content.lines().any(|line| {
        let after_ws = line.trim_start();
        after_ws.starts_with("- ") || after_ws == "-"
    });
    if has_dash_bullet {
        warnings.push(TreeRenderWarning {
            code: "TREE-001",
            message: "kind=outline expects numbered bullets (e.g. '1. Foo', '1.1 Bar') for inline content; rendering as kind=taxonomy. Did you mean kind=taxonomy?".to_string(),
            source_line,
        });
        return render_inline_tree(content, indent_width);
    }

    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match parse_outline_number_prefix(trimmed) {
            Some((depth, number, label)) => {
                let indent = " ".repeat(depth.saturating_mul(indent_width));
                if label.is_empty() {
                    out.push_str(&format!("{}{}\n", indent, number));
                } else {
                    out.push_str(&format!("{}{} {}\n", indent, number, label));
                }
            }
            None => {
                out.push_str(trimmed);
                out.push('\n');
            }
        }
    }
    Ok(out.trim_end().to_string())
}

fn parse_outline_number_prefix(s: &str) -> Option<(usize, String, String)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return None;
    }
    let mut had_digit = false;
    let mut dot_count = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_digit() {
            had_digit = true;
            i += 1;
            continue;
        }
        if b == b'.' {
            if i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
                dot_count += 1;
                i += 1;
                continue;
            }
            i += 1;
            break;
        }
        break;
    }
    if !had_digit {
        return None;
    }
    if i < bytes.len() {
        let b = bytes[i];
        if b != b' ' && b != b'\t' {
            return None;
        }
    }
    let raw_number = &s[..i];
    let label = s[i..].trim_start().to_string();
    let trimmed_number = raw_number.trim_end_matches('.');
    let normalized = if dot_count == 0 {
        format!("{}.", trimmed_number)
    } else {
        trimmed_number.to_string()
    };
    Some((dot_count, normalized, label))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_tree_two_space_indent_renders_nested() {
        let body = "root: Plugin runtime channels\n\
                    - Typed plugin hooks\n  \
                    - File: src/plugins/hook-types.ts\n  \
                    - Style: pull, modify\n\
                    - Diagnostic event stream\n  \
                    - File: src/infra/diagnostic-events.ts\n  \
                    - Style: push, observe";
        let out = render_inline_tree(body, 4).expect("render must succeed");

        assert!(
            out.starts_with("Plugin runtime channels"),
            "root must come first: {}",
            out
        );
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
        let body = "root: Top\n\
                    - One\n    \
                    - One.A\n\
                    - Two";
        let out = render_inline_tree(body, 4).unwrap();
        assert!(out.contains("├── One"), "got:\n{}", out);
        assert!(out.contains("│   └── One.A"), "got:\n{}", out);
        assert!(out.contains("└── Two"), "got:\n{}", out);
    }

    #[test]
    fn inline_tree_last_sibling_uses_corner() {
        let body = "root: R\n- A\n  - A1\n  - A2\n- B";
        let out = render_inline_tree(body, 4).unwrap();
        assert!(
            out.contains("│   └── A2"),
            "last child A2 should be Corner under non-last parent A:\n{}",
            out
        );
    }

    #[test]
    fn inline_outline_dash_bullets_warn_and_promote() {
        let body = "root: Plugin lifecycle\n- 1 Discovery\n- 2 Manifest read\n- 3 Activation";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 4, 1, &mut warnings).expect("must render");

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
        let body = "1. First\n1.1 Sub\n2. Second";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 4, 1, &mut warnings).unwrap();
        assert!(
            warnings.is_empty(),
            "no warnings expected for numbered body"
        );
        assert!(out.contains("1. First"));
    }

    #[test]
    fn inline_outline_numbered_bullets_auto_indent() {
        let body =
            "1. Installation\n1.1 From source\n1.2 From crates.io\n2. Configuration\n2.1 Basics";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        assert!(warnings.is_empty(), "no warnings for numbered input");
        let expected =
            "1. Installation\n   1.1 From source\n   1.2 From crates.io\n2. Configuration\n   2.1 Basics";
        assert_eq!(out, expected, "depth-based indent normalization");
    }

    #[test]
    fn inline_outline_numbered_three_levels() {
        let body = "1. A\n1.1 B\n1.1.1 C\n2. D";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 3, 1, &mut warnings).unwrap();
        let expected = "1. A\n   1.1 B\n      1.1.1 C\n2. D";
        assert_eq!(out, expected);
    }

    #[test]
    fn inline_outline_preserves_trailing_period_only_at_depth_zero() {
        let body = "1. A\n1.1. B";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 3, 1, &mut warnings).unwrap();
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
        let body = "Project plan:\n1. Phase one\n1.1 Step";
        let mut warnings = Vec::new();
        let out = render_inline_outline(body, 3, 1, &mut warnings).unwrap();
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
