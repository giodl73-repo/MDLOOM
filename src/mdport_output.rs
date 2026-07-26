use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct MdportDocument {
    schema: &'static str,
    kind: &'static str,
    title: String,
    source: String,
    format: &'static str,
    metadata: BTreeMap<String, String>,
    sections: Vec<MdportSection>,
    refs: Vec<String>,
}

#[derive(Serialize)]
struct MdportSection {
    id: String,
    path: Vec<String>,
    level: usize,
    line: usize,
    metadata: BTreeMap<String, String>,
    text: String,
}

pub(crate) fn document_json(
    markdown: &str,
    fallback_title: &str,
    source: String,
    refs: impl IntoIterator<Item = impl Into<String>>,
) -> String {
    let parsed = parse_frontmatter(markdown);
    let title = parsed
        .metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| document_title(parsed.content, fallback_title));
    let document = MdportDocument {
        schema: "mdport.v1",
        kind: "document",
        title,
        source,
        format: "markdown",
        metadata: parsed.metadata.clone(),
        sections: sections(parsed.content, &parsed.metadata, parsed.start_line),
        refs: refs.into_iter().map(Into::into).collect(),
    };
    serde_json::to_string(&document).expect("serializing Mdport document cannot fail")
}

pub(crate) fn document_title(markdown: &str, fallback: &str) -> String {
    markdown
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("# ")
                .map(str::trim)
                .filter(|title| !title.is_empty())
        })
        .unwrap_or(fallback)
        .to_string()
}

fn sections(
    markdown: &str,
    metadata: &BTreeMap<String, String>,
    start_line: usize,
) -> Vec<MdportSection> {
    let mut sections = Vec::new();
    let mut current_start = start_line;
    let mut current_level = 0;
    let mut current_path = Vec::new();
    let mut current_text = String::new();
    let mut heading_stack: Vec<(usize, String)> = Vec::new();

    for (index, line) in markdown.lines().enumerate() {
        let line_number = index + start_line;
        if let Some((level, heading)) = parse_heading(line) {
            push_section(
                &mut sections,
                current_start,
                current_level,
                &current_path,
                &current_text,
                metadata,
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
        metadata,
    );
    if sections.is_empty() {
        sections.push(MdportSection {
            id: "document".to_string(),
            path: Vec::new(),
            level: 0,
            line: start_line,
            metadata: metadata.clone(),
            text: String::new(),
        });
    }
    sections
}

fn push_section(
    sections: &mut Vec<MdportSection>,
    line: usize,
    level: usize,
    path: &[String],
    text: &str,
    metadata: &BTreeMap<String, String>,
) {
    let text = text.trim().to_string();
    if text.is_empty() {
        return;
    }
    let base = path.last().map_or("preamble", String::as_str);
    let base = slugify(base);
    let base = if base.is_empty() {
        "section".to_string()
    } else {
        base
    };
    let mut id = base.clone();
    let mut suffix = 2;
    while sections.iter().any(|section| section.id == id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    sections.push(MdportSection {
        id,
        path: path.to_vec(),
        level,
        line,
        metadata: metadata.clone(),
        text,
    });
}

struct ParsedMarkdown<'a> {
    metadata: BTreeMap<String, String>,
    content: &'a str,
    start_line: usize,
}

fn parse_frontmatter(markdown: &str) -> ParsedMarkdown<'_> {
    let mut lines = markdown.split_inclusive('\n');
    let Some(first_line) = lines.next() else {
        return plain_markdown(markdown);
    };
    if first_line.trim_end_matches(['\r', '\n']).trim() != "---" {
        return plain_markdown(markdown);
    }

    let mut metadata = BTreeMap::new();
    let mut consumed_bytes = first_line.len();
    for (line_number, line_with_newline) in (2usize..).zip(lines) {
        let line = line_with_newline.trim_end_matches(['\r', '\n']);
        if line.trim() == "---" {
            consumed_bytes += line_with_newline.len();
            return ParsedMarkdown {
                metadata,
                content: markdown.get(consumed_bytes..).unwrap_or(""),
                start_line: line_number + 1,
            };
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if !key.is_empty() {
                metadata.insert(
                    key.to_string(),
                    value
                        .trim()
                        .trim_matches(|character| character == '"' || character == '\'')
                        .to_string(),
                );
            }
        }
        consumed_bytes += line_with_newline.len();
    }
    plain_markdown(markdown)
}

fn plain_markdown(markdown: &str) -> ParsedMarkdown<'_> {
    ParsedMarkdown {
        metadata: BTreeMap::new(),
        content: markdown,
        start_line: 1,
    }
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let heading = line.get(level..)?;
    heading.starts_with(' ').then(|| (level, heading.trim()))
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for character in text.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
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
