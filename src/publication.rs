use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PUBLICATION_AST_SCHEMA: &str = "proof.publication_ast.v1";
pub const THEME_PLAIN: &str = "plain";
pub const THEME_PROFESSIONAL: &str = "professional";
pub const THEME_DENSE: &str = "dense";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationDocument {
    pub schema: String,
    pub kind: PublicationKind,
    pub title: String,
    pub metadata: BTreeMap<String, String>,
    pub theme: String,
    pub blocks: Vec<PublicationBlock>,
}

impl PublicationDocument {
    pub fn new(kind: PublicationKind, title: impl Into<String>) -> Self {
        Self {
            schema: PUBLICATION_AST_SCHEMA.to_string(),
            kind,
            title: title.into(),
            metadata: BTreeMap::new(),
            theme: THEME_PLAIN.to_string(),
            blocks: Vec::new(),
        }
    }

    pub fn with_theme(mut self, theme: impl Into<String>) -> Self {
        self.theme = theme.into();
        self
    }

    pub fn push_block(&mut self, block: PublicationBlock) {
        self.blocks.push(block);
    }

    pub fn from_resolved_markdown(markdown: &str, fallback_title: &str) -> Self {
        let blocks = markdown_blocks(markdown);
        let title = blocks
            .iter()
            .find_map(|block| match block {
                PublicationBlock::Heading { text, .. } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_else(|| fallback_title.to_string());
        let mut doc = Self::new(PublicationKind::Document, title);
        let mut heading_path: Vec<String> = Vec::new();
        for block in &blocks {
            if let PublicationBlock::Heading { level, text, id } = block {
                heading_path.truncate(level.saturating_sub(1));
                heading_path.push(text.clone());
                doc.metadata
                    .insert(format!("heading_path.{id}"), heading_path.join(" > "));
            }
        }
        doc.blocks = blocks;
        doc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationKind {
    Document,
    Deck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PublicationBlock {
    Heading {
        level: usize,
        text: String,
        id: String,
    },
    Paragraph {
        inlines: Vec<PublicationInline>,
    },
    List {
        ordered: bool,
        items: Vec<PublicationListItem>,
    },
    CodeBlock {
        language: Option<String>,
        text: String,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Figure {
        source: String,
        alt: String,
        caption: Option<String>,
    },
    Note {
        kind: PublicationNoteKind,
        blocks: Vec<PublicationBlock>,
    },
    Slide {
        title: String,
        subtitle: Option<String>,
        blocks: Vec<PublicationBlock>,
        notes: Vec<PublicationBlock>,
    },
}

impl PublicationBlock {
    pub fn heading(level: usize, text: impl Into<String>, id: impl Into<String>) -> Self {
        Self::Heading {
            level,
            text: text.into(),
            id: id.into(),
        }
    }

    pub fn paragraph_text(text: impl Into<String>) -> Self {
        Self::Paragraph {
            inlines: vec![PublicationInline::Text { text: text.into() }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationListItem {
    pub blocks: Vec<PublicationBlock>,
}

impl PublicationListItem {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            blocks: vec![PublicationBlock::paragraph_text(text)],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationNoteKind {
    Note,
    Speaker,
    Sidebar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PublicationInline {
    Text {
        text: String,
    },
    Emphasis {
        children: Vec<PublicationInline>,
    },
    Strong {
        children: Vec<PublicationInline>,
    },
    Code {
        text: String,
    },
    Link {
        href: String,
        children: Vec<PublicationInline>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublicationTheme {
    pub name: String,
    pub fonts: ThemeFonts,
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub typography: ThemeTypography,
    pub slide: ThemeSlide,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub body: String,
    pub heading: String,
    pub monospace: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub text: String,
    pub muted: String,
    pub background: String,
    pub accent: String,
    pub code_background: String,
    pub border: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub page_margin: f32,
    pub block_gap: f32,
    pub list_indent: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub body_size: f32,
    pub heading_scale: f32,
    pub line_height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeSlide {
    pub aspect_ratio: String,
    pub title_size: f32,
    pub body_size: f32,
    pub max_bullets: usize,
    pub bullet_indent: f32,
}

impl PublicationTheme {
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            THEME_PLAIN => Some(Self::plain()),
            THEME_PROFESSIONAL => Some(Self::professional()),
            THEME_DENSE => Some(Self::dense()),
            _ => None,
        }
    }

    pub fn builtin_names() -> &'static [&'static str] {
        &[THEME_PLAIN, THEME_PROFESSIONAL, THEME_DENSE]
    }

    pub fn plain() -> Self {
        Self {
            name: THEME_PLAIN.to_string(),
            fonts: ThemeFonts {
                body: "system-ui".to_string(),
                heading: "system-ui".to_string(),
                monospace: "ui-monospace".to_string(),
            },
            colors: ThemeColors {
                text: "#111111".to_string(),
                muted: "#666666".to_string(),
                background: "#ffffff".to_string(),
                accent: "#2563eb".to_string(),
                code_background: "#f5f5f5".to_string(),
                border: "#d4d4d4".to_string(),
            },
            spacing: ThemeSpacing {
                page_margin: 1.0,
                block_gap: 0.75,
                list_indent: 1.5,
            },
            typography: ThemeTypography {
                body_size: 11.0,
                heading_scale: 1.35,
                line_height: 1.4,
            },
            slide: ThemeSlide {
                aspect_ratio: "16:9".to_string(),
                title_size: 34.0,
                body_size: 22.0,
                max_bullets: 5,
                bullet_indent: 0.35,
            },
        }
    }

    pub fn professional() -> Self {
        Self {
            name: THEME_PROFESSIONAL.to_string(),
            fonts: ThemeFonts {
                body: "Aptos".to_string(),
                heading: "Aptos Display".to_string(),
                monospace: "Cascadia Mono".to_string(),
            },
            colors: ThemeColors {
                text: "#111827".to_string(),
                muted: "#6b7280".to_string(),
                background: "#ffffff".to_string(),
                accent: "#2563eb".to_string(),
                code_background: "#f8fafc".to_string(),
                border: "#d1d5db".to_string(),
            },
            spacing: ThemeSpacing {
                page_margin: 1.1,
                block_gap: 0.9,
                list_indent: 1.4,
            },
            typography: ThemeTypography {
                body_size: 11.0,
                heading_scale: 1.45,
                line_height: 1.45,
            },
            slide: ThemeSlide {
                aspect_ratio: "16:9".to_string(),
                title_size: 38.0,
                body_size: 24.0,
                max_bullets: 5,
                bullet_indent: 0.38,
            },
        }
    }

    pub fn dense() -> Self {
        Self {
            name: THEME_DENSE.to_string(),
            fonts: ThemeFonts {
                body: "Arial".to_string(),
                heading: "Arial".to_string(),
                monospace: "Consolas".to_string(),
            },
            colors: ThemeColors {
                text: "#111111".to_string(),
                muted: "#525252".to_string(),
                background: "#ffffff".to_string(),
                accent: "#0f766e".to_string(),
                code_background: "#f3f4f6".to_string(),
                border: "#a3a3a3".to_string(),
            },
            spacing: ThemeSpacing {
                page_margin: 0.75,
                block_gap: 0.5,
                list_indent: 1.1,
            },
            typography: ThemeTypography {
                body_size: 10.0,
                heading_scale: 1.25,
                line_height: 1.25,
            },
            slide: ThemeSlide {
                aspect_ratio: "16:9".to_string(),
                title_size: 30.0,
                body_size: 19.0,
                max_bullets: 7,
                bullet_indent: 0.3,
            },
        }
    }
}

fn markdown_blocks(markdown: &str) -> Vec<PublicationBlock> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            let fence = if trimmed.starts_with("~~~") {
                "~~~"
            } else {
                "```"
            };
            let language = trimmed
                .strip_prefix(fence)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
            index += 1;
            let mut code = Vec::new();
            while index < lines.len() && !lines[index].trim_start().starts_with(fence) {
                code.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            blocks.push(PublicationBlock::CodeBlock {
                language,
                text: code.join("\n"),
            });
            continue;
        }
        if let Some((level, title)) = markdown_heading(line) {
            blocks.push(PublicationBlock::heading(
                level,
                inline_plain_text(title),
                slugify(title),
            ));
            index += 1;
            continue;
        }
        if is_markdown_table_start(&lines, index) {
            let headers = table_cells(lines[index]);
            index += 2;
            let mut rows = Vec::new();
            while index < lines.len()
                && lines[index].contains('|')
                && !lines[index].trim().is_empty()
            {
                rows.push(table_cells(lines[index]));
                index += 1;
            }
            blocks.push(PublicationBlock::Table { headers, rows });
            continue;
        }
        if markdown_list_item(line).is_some() {
            let (list, next) = parse_list(&lines, index, list_indent(line));
            blocks.push(list);
            index = next;
            continue;
        }

        let mut paragraph = vec![trimmed.to_string()];
        index += 1;
        while index < lines.len() {
            let next = lines[index];
            if next.trim().is_empty()
                || next.trim_start().starts_with("```")
                || next.trim_start().starts_with("~~~")
                || markdown_heading(next).is_some()
                || markdown_list_item(next).is_some()
                || is_markdown_table_start(&lines, index)
            {
                break;
            }
            paragraph.push(next.trim().to_string());
            index += 1;
        }
        blocks.push(PublicationBlock::Paragraph {
            inlines: markdown_inlines(&paragraph.join(" ")),
        });
    }

    blocks
}

fn parse_list(lines: &[&str], start: usize, base_indent: usize) -> (PublicationBlock, usize) {
    let mut index = start;
    let mut ordered = false;
    let mut items: Vec<PublicationListItem> = Vec::new();

    while index < lines.len() {
        if lines[index].trim().is_empty() {
            break;
        }
        let indent = list_indent(lines[index]);
        if indent < base_indent {
            break;
        }
        if indent > base_indent {
            if let Some(last) = items.last_mut() {
                let (nested, next) = parse_list(lines, index, indent);
                last.blocks.push(nested);
                index = next;
                continue;
            }
            break;
        }
        let Some((is_ordered, text)) = markdown_list_item(lines[index]) else {
            break;
        };
        ordered |= is_ordered;
        items.push(PublicationListItem {
            blocks: vec![PublicationBlock::Paragraph {
                inlines: markdown_inlines(text),
            }],
        });
        index += 1;
    }

    (PublicationBlock::List { ordered, items }, index)
}

fn markdown_inlines(text: &str) -> Vec<PublicationInline> {
    let mut output = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix("**") {
            if let Some(end) = stripped.find("**") {
                output.push(PublicationInline::Strong {
                    children: markdown_inlines(&stripped[..end]),
                });
                rest = &stripped[end + 2..];
                continue;
            }
        }
        if let Some(stripped) = rest.strip_prefix('*') {
            if let Some(end) = stripped.find('*') {
                output.push(PublicationInline::Emphasis {
                    children: markdown_inlines(&stripped[..end]),
                });
                rest = &stripped[end + 1..];
                continue;
            }
        }
        if let Some(stripped) = rest.strip_prefix('`') {
            if let Some(end) = stripped.find('`') {
                output.push(PublicationInline::Code {
                    text: stripped[..end].to_string(),
                });
                rest = &stripped[end + 1..];
                continue;
            }
        }
        if let Some(stripped) = rest.strip_prefix('[') {
            if let Some(close) = stripped.find("](") {
                if let Some(end) = stripped[close + 2..].find(')') {
                    let href_end = close + 2 + end;
                    output.push(PublicationInline::Link {
                        href: stripped[close + 2..href_end].to_string(),
                        children: markdown_inlines(&stripped[..close]),
                    });
                    rest = &stripped[href_end + 1..];
                    continue;
                }
            }
        }

        let next = rest
            .char_indices()
            .skip(1)
            .find_map(|(index, c)| matches!(c, '*' | '`' | '[').then_some(index))
            .unwrap_or(rest.len());
        output.push(PublicationInline::Text {
            text: rest[..next].to_string(),
        });
        rest = &rest[next..];
    }
    output
}

fn inline_plain_text(text: &str) -> String {
    markdown_inlines(text)
        .iter()
        .map(inline_text)
        .collect::<Vec<_>>()
        .join("")
}

fn inline_text(inline: &PublicationInline) -> String {
    match inline {
        PublicationInline::Text { text } | PublicationInline::Code { text } => text.clone(),
        PublicationInline::Emphasis { children } | PublicationInline::Strong { children } => {
            children.iter().map(inline_text).collect()
        }
        PublicationInline::Link { children, .. } => children.iter().map(inline_text).collect(),
    }
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = trimmed.get(hashes..)?;
    if !rest.starts_with(' ') {
        return None;
    }
    let title = rest.trim();
    (!title.is_empty()).then_some((hashes, title))
}

fn markdown_list_item(line: &str) -> Option<(bool, &str)> {
    let trimmed = line.trim_start();
    if let Some(text) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
    {
        return Some((false, text.trim()));
    }
    let dot = trimmed.find('.')?;
    if dot == 0 || !trimmed[..dot].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    trimmed[dot + 1..]
        .strip_prefix(' ')
        .map(|text| (true, text.trim()))
}

fn list_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_markdown_table_start(lines: &[&str], index: usize) -> bool {
    index + 1 < lines.len()
        && lines[index].contains('|')
        && lines[index + 1]
            .chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' '))
        && lines[index + 1].contains('-')
}

fn table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| inline_plain_text(cell.trim()))
        .collect()
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_theme_lookup_returns_builtin_tokens() {
        assert_eq!(
            PublicationTheme::builtin_names(),
            &[THEME_PLAIN, THEME_PROFESSIONAL, THEME_DENSE]
        );

        let professional = PublicationTheme::builtin(THEME_PROFESSIONAL).unwrap();
        assert_eq!(professional.name, THEME_PROFESSIONAL);
        assert_eq!(professional.fonts.body, "Aptos");
        assert_eq!(professional.fonts.heading, "Aptos Display");
        assert_eq!(professional.fonts.monospace, "Cascadia Mono");
        assert_eq!(professional.colors.accent, "#2563eb");
        assert_eq!(professional.slide.aspect_ratio, "16:9");

        let dense = PublicationTheme::builtin(THEME_DENSE).unwrap();
        assert!(dense.typography.body_size < professional.typography.body_size);
        assert!(PublicationTheme::builtin("missing").is_none());
    }

    #[test]
    fn publication_ast_serializes_schema_and_blocks() {
        let mut doc =
            PublicationDocument::new(PublicationKind::Document, "Guide").with_theme(THEME_DENSE);
        doc.metadata
            .insert("status".to_string(), "draft".to_string());
        doc.push_block(PublicationBlock::heading(1, "Guide", "guide"));
        doc.push_block(PublicationBlock::Paragraph {
            inlines: vec![
                PublicationInline::Text {
                    text: "See ".to_string(),
                },
                PublicationInline::Link {
                    href: "README.md".to_string(),
                    children: vec![PublicationInline::Text {
                        text: "home".to_string(),
                    }],
                },
            ],
        });
        doc.push_block(PublicationBlock::List {
            ordered: false,
            items: vec![PublicationListItem::text("one")],
        });

        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json["schema"], PUBLICATION_AST_SCHEMA);
        assert_eq!(json["kind"], "document");
        assert_eq!(json["theme"], THEME_DENSE);
        assert_eq!(json["blocks"][0]["type"], "heading");
        assert_eq!(json["blocks"][1]["inlines"][1]["type"], "link");
        assert_eq!(
            json["blocks"][2]["items"][0]["blocks"][0]["type"],
            "paragraph"
        );
    }

    #[test]
    fn publication_markdown_extracts_common_blocks() {
        let doc = PublicationDocument::from_resolved_markdown(
            "# Guide\n\nIntro with [home](README.md), `code`, and **bold** text.\n\n- one\n  - nested\n- two\n\n1. first\n2. second\n\n| A | B |\n|---|---|\n| x | y |\n\n```rust\nlet x = 1;\n```\n\n## Details\n\nMore.\n",
            "fallback",
        );

        assert_eq!(doc.schema, PUBLICATION_AST_SCHEMA);
        assert_eq!(doc.title, "Guide");
        assert_eq!(doc.metadata["heading_path.guide"], "Guide");
        assert_eq!(doc.metadata["heading_path.details"], "Guide > Details");
        assert!(matches!(
            doc.blocks[0],
            PublicationBlock::Heading { level: 1, .. }
        ));

        let PublicationBlock::Paragraph { inlines } = &doc.blocks[1] else {
            panic!("expected paragraph");
        };
        assert!(inlines.iter().any(|inline| matches!(
            inline,
            PublicationInline::Link { href, .. } if href == "README.md"
        )));
        assert!(inlines
            .iter()
            .any(|inline| matches!(inline, PublicationInline::Code { text } if text == "code")));
        assert!(inlines
            .iter()
            .any(|inline| matches!(inline, PublicationInline::Strong { .. })));

        let PublicationBlock::List { ordered, items } = &doc.blocks[2] else {
            panic!("expected unordered list");
        };
        assert!(!ordered);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0].blocks[1], PublicationBlock::List { .. }));

        let PublicationBlock::List { ordered, items } = &doc.blocks[3] else {
            panic!("expected ordered list");
        };
        assert!(*ordered);
        assert_eq!(items.len(), 2);

        let PublicationBlock::Table { headers, rows } = &doc.blocks[4] else {
            panic!("expected table");
        };
        assert_eq!(headers, &["A".to_string(), "B".to_string()]);
        assert_eq!(rows[0], vec!["x".to_string(), "y".to_string()]);

        let PublicationBlock::CodeBlock { language, text } = &doc.blocks[5] else {
            panic!("expected code block");
        };
        assert_eq!(language.as_deref(), Some("rust"));
        assert_eq!(text, "let x = 1;");
    }
}
