fn build_numbered_label(headings: &[(usize, String)], min_level: usize) -> String {
    let (target_level, _) = headings.last().unwrap();
    let target_depth = target_level - min_level;
    let mut counters: Vec<usize> = vec![0; target_depth + 1];
    for (level, _) in headings {
        let depth = level - min_level;
        if depth <= target_depth {
            counters[depth] += 1;
            for d in (depth + 1)..=target_depth {
                counters[d] = 0;
            }
        }
    }
    let parts: Vec<String> = counters[..=target_depth]
        .iter()
        .map(|n| n.to_string())
        .collect();
    format!("{}.", parts.join("."))
}

pub(crate) fn generate_toc(
    content: &str,
    max_depth: usize,
    style: &str,
    section: Option<&str>,
) -> String {
    let mut all: Vec<(usize, String)> = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed[level..].trim().to_string();
            if !text.is_empty() {
                all.push((level, text));
            }
        }
    }

    let scoped: Vec<(usize, String)> = if let Some(target) = section {
        let want = target.trim().to_lowercase();
        let start = all
            .iter()
            .position(|(_, t)| t.trim().to_lowercase() == want);
        match start {
            Some(idx) => {
                let parent_level = all[idx].0;
                let mut out = Vec::new();
                for (level, text) in all.iter().skip(idx + 1) {
                    if *level <= parent_level {
                        break;
                    }
                    out.push((*level, text.clone()));
                }
                out
            }
            None => Vec::new(),
        }
    } else {
        all
    };

    let headings: Vec<(usize, String)> = scoped
        .into_iter()
        .filter(|(level, _)| *level <= max_depth)
        .collect();

    if headings.is_empty() {
        return String::new();
    }
    let min_level = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);
    let mut out = String::new();
    for (i, (level, text)) in headings.iter().enumerate() {
        let depth = level - min_level;
        let indent = "  ".repeat(depth);
        if style == "tree" && depth > 0 {
            let is_last = !headings[i + 1..].iter().any(|(l, _)| *l <= *level);
            let connector = if is_last { "└── " } else { "├── " };
            let parent_indent = "  ".repeat(depth.saturating_sub(1));
            out.push_str(&format!("{}  {}{}\n", parent_indent, connector, text));
        } else if style == "numbered" {
            let number = build_numbered_label(&headings[..=i], min_level);
            out.push_str(&format!("{}{} {}\n", indent, number, text));
        } else {
            out.push_str(&format!("{}- {}\n", indent, text));
        }
    }
    out.trim_end().to_string()
}
