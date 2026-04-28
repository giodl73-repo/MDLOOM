pub mod bullets;
pub mod canvas;
pub mod inline;
pub mod layout;
pub mod parser;

pub use canvas::SlideCanvas;
pub use bullets::{render_bullets, BulletConfig, BulletWarning,
                  parse_reveal_step, has_reveal_markers, render_bullets_pages};
pub use inline::{render_quote, render_centered, render_right, render_ol,
                 render_stat, render_callout, render_divider, CalloutStyle, DividerStyle};
pub use layout::{render_slide, render_slide_with_warnings, render_slide_with_warnings_in_deck,
                 render_slide_pages,
                 render_title, render_title_content,
                 render_two_column, render_section, render_agenda, render_stats, render_blank,
                 collect_section_titles, apply_theme,
                 center_in_width, render_body_lines, render_body_lines_with_warnings,
                 render_body_lines_pages};
pub use parser::{parse_slide_doc, SlideError};

// ─────────────────────────────────────────────────────────
// Core structs
// ─────────────────────────────────────────────────────────

/// Footer mode for the deck.
///
/// - `Off` — no footer (default)
/// - `Auto` — compose "author · date" from deck-level author/date fields on the title slide
/// - `Custom(s)` — render `s` verbatim as the footer text
#[derive(Debug, Clone, PartialEq)]
pub enum FooterMode {
    Off,
    Auto,
    Custom(String),
}

impl Default for FooterMode {
    fn default() -> Self { FooterMode::Off }
}

#[derive(Debug, Clone)]
pub struct SlideMeta {
    pub width: usize,
    pub height: usize,
    pub theme: SlideTheme,
    pub show_numbers: bool,
    /// When true, a `████░░░` progress bar is emitted after each slide separator
    /// in the compiled output (outside the canvas — SL-1 still holds).
    pub progress_bar: bool,
    pub font_width: usize,
    pub max_bullets: usize,
    pub max_depth: usize,
    pub footer: FooterMode,
    /// Deck-level author propagated from the title slide (or front-matter).
    pub author: Option<String>,
    /// Deck-level date propagated from the title slide (or front-matter).
    pub date: Option<String>,
    /// Deck-level title propagated from front-matter.
    pub title: Option<String>,
}

impl Default for SlideMeta {
    fn default() -> Self {
        SlideMeta {
            width: 120,
            height: 34,
            theme: SlideTheme::Minimal,
            show_numbers: false,
            progress_bar: false,
            font_width: 1,
            max_bullets: 4,   // 30-second rule — see bullets.rs SLIDE-001
            max_depth: 4,
            footer: FooterMode::Off,
            author: None,
            date: None,
            title: None,
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
    /// Auto-generated agenda built from every `Section` slide's title in deck order.
    /// The slide's own body content is ignored; the bullet list comes from the deck.
    Agenda,
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
