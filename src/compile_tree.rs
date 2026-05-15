use anyhow::Result;

pub(crate) fn render_inline_tree(content: &str, indent_width: usize) -> Result<String> {
    let iw = indent_width.max(2);
    let mut nodes: Vec<(usize, String)> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("root:") {
            nodes.push((0, rest.trim().to_string()));
            continue;
        }
        let leading = line.len() - line.trim_start_matches([' ', '-']).len();
        if leading == 0 && nodes.is_empty() && !trimmed.starts_with('-') {
            nodes.push((0, trimmed.trim_start_matches([' ', '-']).trim().to_string()));
            continue;
        }
        let depth = (leading / iw).max(1);
        let label = trimmed.trim_start_matches([' ', '-']).trim();
        if label.is_empty() {
            continue;
        }
        nodes.push((depth, label.to_string()));
    }

    if nodes.is_empty() {
        anyhow::bail!("inline tree body is empty");
    }

    let mut out = String::new();
    for (i, (depth, label)) in nodes.iter().enumerate() {
        if *depth == 0 {
            out.push_str(label);
            out.push('\n');
            continue;
        }
        let prefix = " ".repeat((*depth - 1) * iw);
        let is_last = !nodes[i + 1..]
            .iter()
            .any(|(d, _)| *d == *depth || *d < *depth);
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(&prefix);
        out.push_str(connector);
        out.push_str(label);
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

pub(crate) fn render_inline_outline(content: &str) -> Result<String> {
    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}
