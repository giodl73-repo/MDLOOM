use pulldown_cmark::{html, Event, Options, Parser};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub fn markdown_to_html_document(markdown: &str, title: &str) -> String {
    let title = document_title(markdown, title);
    let body = markdown_to_html_fragment(markdown);

    format!(
        concat!(
            "<!doctype html>\n",
            "<html lang=\"en\">\n",
            "<head>\n",
            "<meta charset=\"utf-8\">\n",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "<title>{}</title>\n",
            "<style>\n",
            ":root {{ color-scheme: light dark; }}\n",
            "body {{ font-family: system-ui, sans-serif; line-height: 1.55; max-width: 72ch; margin: 2rem auto; padding: 0 1rem; }}\n",
            "pre, code {{ font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }}\n",
            "pre {{ overflow-x: auto; padding: 1rem; border: 1px solid color-mix(in srgb, currentColor 20%, transparent); border-radius: .5rem; }}\n",
            "table {{ border-collapse: collapse; width: 100%; }}\n",
            "th, td {{ border: 1px solid color-mix(in srgb, currentColor 20%, transparent); padding: .35rem .5rem; }}\n",
            "img {{ max-width: 100%; }}\n",
            "</style>\n",
            "</head>\n",
            "<body>\n",
            "{}",
            "</body>\n",
            "</html>\n"
        ),
        escape_html(&title),
        body
    )
}

pub fn markdown_to_pebble_document(
    markdown: &str,
    fallback_title: &str,
    source_path: &Path,
    resolved_files: &[PathBuf],
) -> String {
    let title = document_title(markdown, fallback_title);
    let pebble = PebbleDocument {
        schema: "pebble.v1",
        kind: "document",
        title,
        source: path_string(source_path),
        format: "markdown",
        sections: markdown_sections(markdown),
        refs: resolved_files
            .iter()
            .map(|path| path_string(path))
            .collect(),
    };
    serde_json::to_string(&pebble).expect("serializing Pebble document cannot fail")
}

#[derive(Serialize)]
struct PebbleDocument {
    schema: &'static str,
    kind: &'static str,
    title: String,
    source: String,
    format: &'static str,
    sections: Vec<PebbleSection>,
    refs: Vec<String>,
}

#[derive(Serialize)]
struct PebbleSection {
    id: String,
    path: Vec<String>,
    level: usize,
    line: usize,
    text: String,
}

fn markdown_to_html_fragment(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, markdown_options()).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

fn document_title(markdown: &str, fallback: &str) -> String {
    markdown
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("# ")
                .map(str::trim)
                .filter(|title| !title.is_empty())
        })
        .unwrap_or(fallback)
        .to_string()
}

fn markdown_sections(markdown: &str) -> Vec<PebbleSection> {
    let mut sections = Vec::new();
    let mut current_start = 1usize;
    let mut current_level = 0usize;
    let mut current_path: Vec<String> = Vec::new();
    let mut current_text = String::new();
    let mut heading_stack: Vec<(usize, String)> = Vec::new();

    for (index, line) in markdown.lines().enumerate() {
        let line_number = index + 1;
        if let Some((level, heading)) = parse_heading(line) {
            push_section(
                &mut sections,
                current_start,
                current_level,
                &current_path,
                &current_text,
            );
            while heading_stack
                .last()
                .is_some_and(|(stack_level, _)| *stack_level >= level)
            {
                heading_stack.pop();
            }
            heading_stack.push((level, heading.to_string()));
            current_path = heading_stack
                .iter()
                .map(|(_, heading)| heading.clone())
                .collect();
            current_start = line_number;
            current_level = level;
            current_text.clear();
        }
        current_text.push_str(line);
        current_text.push('\n');
    }

    push_section(
        &mut sections,
        current_start,
        current_level,
        &current_path,
        &current_text,
    );

    if sections.is_empty() {
        sections.push(PebbleSection {
            id: "document".to_string(),
            path: Vec::new(),
            level: 0,
            line: 1,
            text: String::new(),
        });
    }

    sections
}

fn push_section(
    sections: &mut Vec<PebbleSection>,
    line: usize,
    level: usize,
    path: &[String],
    text: &str,
) {
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }
    let base = path.last().map_or("preamble", String::as_str);
    let id = unique_section_id(sections, base);
    sections.push(PebbleSection {
        id,
        path: path.to_vec(),
        level,
        line,
        text,
    });
}

fn unique_section_id(sections: &[PebbleSection], heading: &str) -> String {
    let base = slugify(heading);
    let base = if base.is_empty() {
        "section".to_string()
    } else {
        base
    };
    let mut id = base.clone();
    let mut suffix = 2usize;
    while sections.iter().any(|section| section.id == id) {
        id = format!("{}-{}", base, suffix);
        suffix += 1;
    }
    id
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    if last_dash {
        slug.pop();
    }
    slug
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
    Some((hashes, rest.trim()))
}

fn path_string(path: &Path) -> String {
    path.display().to_string()
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_backend_renders_common_markdown_blocks() {
        let html = markdown_to_html_document(
            "# Guide\n\n- one\n- two\n\n| A | B |\n|---|---|\n| x | y |\n\n[home](README.md)\n",
            "fallback",
        );

        assert!(html.contains("<title>Guide</title>"), "got:\n{}", html);
        assert!(html.contains("<ul>"), "got:\n{}", html);
        assert!(html.contains("<li>one</li>"), "got:\n{}", html);
        assert!(html.contains("<table>"), "got:\n{}", html);
        assert!(
            html.contains("<a href=\"README.md\">home</a>"),
            "got:\n{}",
            html
        );
    }

    #[test]
    fn html_backend_escapes_raw_html() {
        let html = markdown_to_html_document("# Safe\n\n<script>alert(1)</script>\n", "fallback");

        assert!(!html.contains("<script>"), "got:\n{}", html);
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn html_backend_escapes_title_fallback() {
        let html = markdown_to_html_document("Body only\n", "A < B");

        assert!(html.contains("<title>A &lt; B</title>"), "got:\n{}", html);
    }

    #[test]
    fn pebble_backend_chunks_markdown_for_transfer() {
        let pebble = markdown_to_pebble_document(
            "# Guide\n\nIntro.\n\n## Steps\n\n- one\n- two\n",
            "fallback",
            Path::new("guide.source.md"),
            &[PathBuf::from(".proof\\side-info\\links.json")],
        );
        let json: serde_json::Value = serde_json::from_str(&pebble).unwrap();

        assert_eq!(json["schema"], "pebble.v1");
        assert_eq!(json["title"], "Guide");
        assert_eq!(json["source"], "guide.source.md");
        assert_eq!(json["refs"].as_array().unwrap().len(), 1);
        assert_eq!(json["sections"][0]["id"], "guide");
        assert_eq!(json["sections"][1]["path"][1], "Steps");
        assert!(json["sections"][1]["text"]
            .as_str()
            .unwrap()
            .contains("- one"));
    }
}
