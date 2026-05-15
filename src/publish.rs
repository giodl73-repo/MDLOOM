pub fn markdown_to_html_document(markdown: &str, title: &str) -> String {
    let mut body = String::new();
    let mut paragraph = Vec::new();
    let mut in_code = false;
    let mut code = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut body, &mut paragraph);
            if in_code {
                body.push_str("<pre><code>");
                body.push_str(&escape_html(code.trim_end_matches('\n')));
                body.push_str("</code></pre>\n");
                code.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }

        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut body, &mut paragraph);
            continue;
        }

        if let Some((level, text)) = parse_heading(trimmed) {
            flush_paragraph(&mut body, &mut paragraph);
            body.push_str(&format!(
                "<h{level}>{}</h{level}>\n",
                escape_html(text.trim())
            ));
            continue;
        }

        paragraph.push(trimmed.to_string());
    }

    if in_code {
        body.push_str("<pre><code>");
        body.push_str(&escape_html(code.trim_end_matches('\n')));
        body.push_str("</code></pre>\n");
    }
    flush_paragraph(&mut body, &mut paragraph);

    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{}</title>\n</head>\n<body>\n{}</body>\n</html>\n",
        escape_html(title),
        body
    )
}

fn flush_paragraph(body: &mut String, paragraph: &mut Vec<String>) {
    if paragraph.is_empty() {
        return;
    }
    body.push_str("<p>");
    body.push_str(&escape_html(&paragraph.join(" ")));
    body.push_str("</p>\n");
    paragraph.clear();
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = line.get(hashes..)?;
    if !rest.starts_with(' ') {
        return None;
    }
    Some((hashes, rest))
}

fn escape_html(text: &str) -> String {
    text.chars()
        .flat_map(|c| match c {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect::<Vec<_>>(),
            '>' => "&gt;".chars().collect::<Vec<_>>(),
            '"' => "&quot;".chars().collect::<Vec<_>>(),
            '\'' => "&#39;".chars().collect::<Vec<_>>(),
            _ => vec![c],
        })
        .collect()
}
