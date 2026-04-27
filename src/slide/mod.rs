pub mod canvas;
pub mod parser;

pub use canvas::SlideCanvas;
pub use parser::{parse_slide_doc, SlideError};

// ─────────────────────────────────────────────────────────
// Core structs
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SlideMeta {
    pub width: usize,
    pub height: usize,
    pub theme: SlideTheme,
    pub show_numbers: bool,
    pub font_width: usize,
    pub max_bullets: usize,
    pub max_depth: usize,
}

impl Default for SlideMeta {
    fn default() -> Self {
        SlideMeta {
            width: 120,
            height: 34,
            theme: SlideTheme::Minimal,
            show_numbers: false,
            font_width: 1,
            max_bullets: 6,
            max_depth: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlideTheme {
    Minimal,
    Box,
    None,
}

#[derive(Debug, Clone)]
pub struct Slide {
    pub index: usize,
    pub layout: SlideLayout,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub body_content: String,
    pub notes_content: String,
    pub source_line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlideLayout {
    Title,
    TitleContent,
    TwoColumn { ratio: (u8, u8) },
    Section,
    ContentCaption,
    Comparison,
    Stats,
    Blank,
}

#[derive(Debug)]
pub struct SlideDoc {
    pub meta: SlideMeta,
    pub slides: Vec<Slide>,
}
