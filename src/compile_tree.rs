use anyhow::Result;
use std::path::Path;

use crate::compile_directive::TreeAttrs;
use crate::compile_source;
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
            .map_or(true, |(d, _)| *d < *depth);
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
