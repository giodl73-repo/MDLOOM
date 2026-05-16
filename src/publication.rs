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
}
