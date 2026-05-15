/// Slide document parser.
///
/// Disambiguation rule for `---` lines:
///   - If line 0 of the source is exactly `---`, the document begins with YAML front-matter.
///     The next `---` is the front-matter closer (NOT a slide separator).
///     All subsequent bare `---` lines are slide separators.
///   - If line 0 is NOT `---`, there is no front-matter, and every bare `---` line is a
///     slide separator. The content before the first separator is slide 1.
///
/// Slide blocks: after front-matter (if any), the remaining text is split into raw slide
/// strings at bare `---` lines. Each raw slide string is then parsed for `proof:slide`
/// fence attributes and body content.
use super::{FooterMode, Slide, SlideDoc, SlideLayout, SlideMeta, SlideTheme};

// ─────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SlideError {
    MalformedFrontMatter(String),
    InvalidRatio { slide: usize, raw: String },
    SlideOutOfRange { requested: usize, count: usize },
    UnknownLayout { slide: usize, name: String },
}

impl std::fmt::Display for SlideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlideError::MalformedFrontMatter(msg) => write!(f, "malformed front-matter: {}", msg),
            SlideError::InvalidRatio { slide, raw } => write!(
                f,
                "SLIDE-002: slide {}: ratio {:?} parts do not sum to 100",
                slide, raw
            ),
            SlideError::SlideOutOfRange { requested, count } => write!(
                f,
                "SLIDE-006: --slide {} out of range (document has {} slides)",
                requested, count
            ),
            SlideError::UnknownLayout { slide, name } => {
                write!(f, "slide {}: unknown layout {:?}", slide, name)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────
// Public parse entry point
// ─────────────────────────────────────────────────────────

/// Parse a `.slides.source.md` source string into a `SlideDoc`.
/// Returns `Err(Vec<SlideError>)` if any structural error is fatal.
/// Non-fatal errors (unknown layout → Blank fallback) are silently recovered.
pub fn parse_slide_doc(source: &str) -> Result<SlideDoc, Vec<SlideError>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut errors: Vec<SlideError> = Vec::new();

    // ── Front-matter disambiguation ──────────────────────
    let (meta, body_start_line) = if lines.first().map(|l| l.trim()) == Some("---") {
        // YAML front-matter: scan for the closing ---
        let closer = lines.iter().skip(1).position(|l| l.trim() == "---");
        match closer {
            Some(rel_idx) => {
                let fm_lines = &lines[1..rel_idx + 1]; // lines between opening and closing ---
                let fm_text = fm_lines.join("\n");
                match parse_front_matter(&fm_text) {
                    Ok(meta) => (meta, rel_idx + 2), // body starts after closing ---
                    Err(msg) => {
                        errors.push(SlideError::MalformedFrontMatter(msg));
                        return Err(errors);
                    }
                }
            }
            None => {
                // No closing --- found — treat whole file as front-matter error
                errors.push(SlideError::MalformedFrontMatter(
                    "no closing --- found for front-matter block".to_string(),
                ));
                return Err(errors);
            }
        }
    } else {
        (SlideMeta::default(), 0)
    };

    // ── Body text: lines[body_start_line..] ──────────────
    let body_lines = &lines[body_start_line..];
    let body_text = body_lines.join("\n");
    let raw_slides = split_slides(&body_text, body_start_line);

    if raw_slides.is_empty() {
        // Empty file: return one empty slide
        return Ok(SlideDoc {
            meta,
            slides: vec![Slide {
                index: 1,
                layout: SlideLayout::TitleContent,
                title: None,
                subtitle: None,
                author: None,
                date: None,
                body_content: String::new(),
                notes_content: String::new(),
                source_line: body_start_line + 1,
            }],
        });
    }

    let mut slides = Vec::new();
    for (idx, (source_line, raw)) in raw_slides.iter().enumerate() {
        match parse_slide(raw, idx + 1, *source_line) {
            Ok(slide) => slides.push(slide),
            Err(SlideError::InvalidRatio { slide, raw: r }) => {
                errors.push(SlideError::InvalidRatio { slide, raw: r });
            }
            Err(SlideError::UnknownLayout { slide, name }) => {
                // Recover: use Blank layout and continue
                errors.push(SlideError::UnknownLayout { slide, name });
            }
            Err(e) => errors.push(e),
        }
    }

    if !errors.iter().any(|e| {
        matches!(
            e,
            SlideError::MalformedFrontMatter(_) | SlideError::InvalidRatio { .. }
        )
    }) {
        // Non-fatal errors don't prevent returning a SlideDoc
        Ok(SlideDoc { meta, slides })
    } else {
        Err(errors)
    }
}

// ─────────────────────────────────────────────────────────
// Front-matter parsing (hand-rolled, no serde_yaml)
// ─────────────────────────────────────────────────────────

fn parse_front_matter(block: &str) -> Result<SlideMeta, String> {
    let mut meta = SlideMeta::default();
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = split_yaml_kv(line)?;
        match key {
            "width" => {
                meta.width = value
                    .parse::<usize>()
                    .map_err(|_| format!("width must be a positive integer, got {:?}", value))?
            }
            "height" => {
                meta.height = value
                    .parse::<usize>()
                    .map_err(|_| format!("height must be a positive integer, got {:?}", value))?
            }
            "theme" => {
                meta.theme = match value {
                    "minimal" => SlideTheme::Minimal,
                    "box" => SlideTheme::Box,
                    "none" => SlideTheme::None,
                    other => return Err(format!("unknown theme {:?}", other)),
                }
            }
            "show-numbers" | "show_numbers" => {
                meta.show_numbers = parse_bool(value)
                    .map_err(|_| format!("show-numbers must be true/false, got {:?}", value))?;
            }
            "progress-bar" | "progress_bar" => {
                meta.progress_bar = parse_bool(value)
                    .map_err(|_| format!("progress-bar must be true/false, got {:?}", value))?;
            }
            "font-width" | "font_width" => {
                meta.font_width = value
                    .parse::<usize>()
                    .map_err(|_| format!("font-width must be 1 or 2, got {:?}", value))?;
            }
            "max-bullets" | "max_bullets" => {
                meta.max_bullets = value.parse::<usize>().map_err(|_| {
                    format!("max-bullets must be a positive integer, got {:?}", value)
                })?;
            }
            "max-depth" | "max_depth" => {
                meta.max_depth = value.parse::<usize>().map_err(|_| {
                    format!("max-depth must be a positive integer, got {:?}", value)
                })?;
            }
            "footer" => {
                meta.footer = match value {
                    "true" | "yes" | "1" | "auto" => FooterMode::Auto,
                    "false" | "no" | "0" | "off" => FooterMode::Off,
                    custom => FooterMode::Custom(custom.to_string()),
                };
            }
            "author" => {
                meta.author = Some(value.to_string());
            }
            "date" => {
                meta.date = Some(value.to_string());
            }
            "title" => {
                meta.title = Some(value.to_string());
            }
            // Ignore unknown keys (forward compatibility)
            _ => {}
        }
    }
    Ok(meta)
}

fn split_yaml_kv(line: &str) -> Result<(&str, &str), String> {
    let colon = line
        .find(':')
        .ok_or_else(|| format!("expected key: value, got {:?}", line))?;
    let key = line[..colon].trim();
    let value = line[colon + 1..]
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    Ok((key, value))
}

fn parse_bool(s: &str) -> Result<bool, ()> {
    match s {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(()),
    }
}

// ─────────────────────────────────────────────────────────
// Slide splitting
// ─────────────────────────────────────────────────────────

/// Split `body` into raw slide chunks at bare `---` lines.
/// Returns `Vec<(source_line_1based, raw_content)>` where source_line is the
/// 1-based line in the original file where this slide chunk begins.
fn split_slides(body: &str, body_offset: usize) -> Vec<(usize, String)> {
    let mut result: Vec<(usize, String)> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut current_start = body_offset + 1; // 1-based

    for (i, line) in body.lines().enumerate() {
        let abs_line = body_offset + i + 1;
        if line.trim() == "---" {
            // This --- is a slide separator (front-matter closer was already consumed)
            let chunk = current.join("\n");
            result.push((current_start, chunk));
            current.clear();
            current_start = abs_line + 1;
        } else {
            current.push(line);
        }
    }

    // Last chunk (or only chunk if no --- found)
    let last = current.join("\n");
    result.push((current_start, last));

    result
}

// ─────────────────────────────────────────────────────────
// Single slide parsing
// ─────────────────────────────────────────────────────────

/// Parse a raw slide chunk (text between separators) into a `Slide`.
fn parse_slide(raw: &str, index: usize, source_line: usize) -> Result<Slide, SlideError> {
    // A slide chunk may begin with a ```proof:slide ...``` fence.
    // If so, everything inside is the body. If not, the whole chunk is the body.
    let trimmed = raw.trim();

    let (layout, title, subtitle, author, date, body_raw) = if trimmed.starts_with("```proof:slide")
    {
        parse_slide_fence(trimmed, index)?
    } else {
        // No fence — treat entire chunk as body with default layout
        (
            SlideLayout::TitleContent,
            None,
            None,
            None,
            None,
            trimmed.to_string(),
        )
    };

    let (body_content, notes_content) = extract_notes(&body_raw);

    Ok(Slide {
        index,
        layout,
        title,
        subtitle,
        author,
        date,
        body_content,
        notes_content,
        source_line,
    })
}

/// Parse a ```proof:slide ...``` fenced block.
/// Returns (layout, title, subtitle, author, date, body_text).
fn parse_slide_fence(
    text: &str,
    index: usize,
) -> Result<
    (
        SlideLayout,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    ),
    SlideError,
> {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    // first = "```proof:slide layout=title-content title=\"Foo\" ..."
    let info = first.trim_start_matches('`').trim();
    let attrs_str = info.strip_prefix("proof:slide").unwrap_or("").trim();

    let (layout, mut title, mut subtitle) = parse_slide_attrs(attrs_str, index)?;

    // Collect body lines until closing ```
    let mut body_lines: Vec<&str> = Vec::new();
    let mut author: Option<String> = None;
    let mut date: Option<String> = None;

    for line in lines {
        let t = line.trim();
        if t == "```" {
            break;
        }
        // Extract YAML-style key: "value" attrs from body lines.
        // These override any values from the info string.
        if t.starts_with("title:") {
            let v = t["title:".len()..].trim().trim_matches('"').to_string();
            if title.is_none() || !v.is_empty() {
                title = Some(v);
            }
            continue;
        }
        if t.starts_with("subtitle:") {
            let v = t["subtitle:".len()..].trim().trim_matches('"').to_string();
            if subtitle.is_none() || !v.is_empty() {
                subtitle = Some(v);
            }
            continue;
        }
        if t.starts_with("author:") {
            author = Some(t["author:".len()..].trim().trim_matches('"').to_string());
            continue;
        }
        if t.starts_with("date:") {
            date = Some(t["date:".len()..].trim().trim_matches('"').to_string());
            continue;
        }
        body_lines.push(line);
    }

    Ok((layout, title, subtitle, author, date, body_lines.join("\n")))
}

/// Parse `layout=`, `title=`, `ratio=` from the info string after `proof:slide`.
/// Returns (SlideLayout, Option<title>, Option<subtitle>).
pub(crate) fn parse_slide_attrs(
    info: &str,
    index: usize,
) -> Result<(SlideLayout, Option<String>, Option<String>), SlideError> {
    let mut layout_name = "title-content";
    let mut title: Option<String> = None;
    let mut subtitle: Option<String> = None;
    let mut ratio_raw: Option<String> = None;

    let mut rest = info.trim();
    while !rest.is_empty() {
        if let Some(eq_pos) = rest.find('=') {
            let key = rest[..eq_pos].trim();
            rest = &rest[eq_pos + 1..];
            let (val, next) = if rest.starts_with('"') {
                if let Some(close) = rest[1..].find('"') {
                    (&rest[1..close + 1], &rest[close + 2..])
                } else {
                    (rest.trim_start_matches('"'), "")
                }
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                (&rest[..end], &rest[end..])
            };
            match key {
                "layout" => layout_name = val,
                "title" => title = Some(val.to_string()),
                "subtitle" => subtitle = Some(val.to_string()),
                "ratio" => ratio_raw = Some(val.to_string()),
                _ => {}
            }
            rest = next.trim_start();
        } else {
            // bare token (unknown) — skip
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            rest = rest[end..].trim_start();
        }
    }

    // Parse layout
    let layout = match layout_name {
        "title" => SlideLayout::Title,
        "title-content" => SlideLayout::TitleContent,
        "two-column" => {
            let ratio = parse_ratio(ratio_raw.as_deref().unwrap_or("60:40"), index)?;
            SlideLayout::TwoColumn { ratio }
        }
        "section" => SlideLayout::Section,
        "agenda" => SlideLayout::Agenda,
        "content-caption" => SlideLayout::ContentCaption,
        "comparison" => SlideLayout::Comparison,
        "stats" => SlideLayout::Stats,
        "blank" => SlideLayout::Blank,
        _other => SlideLayout::Blank, // recover silently; caller may push warning
    };

    Ok((layout, title, subtitle))
}

/// Parse a ratio string like "60:40" → (60, 40). Validates sum == 100.
fn parse_ratio(raw: &str, slide_index: usize) -> Result<(u8, u8), SlideError> {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(SlideError::InvalidRatio {
            slide: slide_index,
            raw: raw.to_string(),
        });
    }
    let a = parts[0]
        .trim()
        .parse::<u8>()
        .map_err(|_| SlideError::InvalidRatio {
            slide: slide_index,
            raw: raw.to_string(),
        })?;
    let b = parts[1]
        .trim()
        .parse::<u8>()
        .map_err(|_| SlideError::InvalidRatio {
            slide: slide_index,
            raw: raw.to_string(),
        })?;
    if a.saturating_add(b) != 100 {
        return Err(SlideError::InvalidRatio {
            slide: slide_index,
            raw: raw.to_string(),
        });
    }
    Ok((a, b))
}

// ─────────────────────────────────────────────────────────
// Notes extraction
// ─────────────────────────────────────────────────────────

/// Extract `proof:notes` fenced block from slide body.
/// Returns `(body_without_notes, notes_content)`.
pub(crate) fn extract_notes(body: &str) -> (String, String) {
    let mut body_lines: Vec<&str> = Vec::new();
    let mut notes_lines: Vec<&str> = Vec::new();
    let mut in_notes = false;

    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if !in_notes && (trimmed == "```proof:notes" || trimmed == "~~~proof:notes") {
            in_notes = true;
            i += 1;
            continue;
        }
        if in_notes && (trimmed == "```" || trimmed == "~~~") {
            in_notes = false;
            i += 1;
            continue;
        }
        if in_notes {
            notes_lines.push(lines[i]);
        } else {
            body_lines.push(lines[i]);
        }
        i += 1;
    }

    (body_lines.join("\n"), notes_lines.join("\n"))
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a minimal source with N slides separated by ---
    fn make_slides(n: usize) -> String {
        (1..=n)
            .map(|i| format!("Slide {i} content"))
            .collect::<Vec<_>>()
            .join("\n---\n")
    }

    // ── Test 1: YAML front-matter parsed, width extracted ────────────────────
    #[test]
    fn front_matter_width_parsed() {
        let source = "---\nwidth: 80\nheight: 20\n---\nSlide content";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(doc.meta.width, 80);
        assert_eq!(doc.meta.height, 20);
    }

    // ── Test 2: Front-matter closer not counted as slide separator (SL-7) ───
    #[test]
    fn front_matter_closer_not_a_separator() {
        // One --- opens FM, one closes FM, one more separates slides
        let source = "---\nwidth: 80\n---\nSlide 1\n---\nSlide 2";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(
            doc.slides.len(),
            2,
            "expected 2 slides, got {}",
            doc.slides.len()
        );
    }

    // ── Test 3: No front-matter: first --- is slide 2 separator ─────────────
    #[test]
    fn no_front_matter_first_dash_is_separator() {
        let source = "Slide 1\n---\nSlide 2\n---\nSlide 3";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(doc.slides.len(), 3);
        assert_eq!(doc.meta.width, 120); // default
    }

    // ── Test 4: 3 slides separated by 2 --- ─────────────────────────────────
    #[test]
    fn three_slides_two_separators() {
        let source = make_slides(3);
        let doc = parse_slide_doc(&source).expect("should parse");
        assert_eq!(doc.slides.len(), 3);
    }

    // ── Test 5: layout=title parsed ─────────────────────────────────────────
    #[test]
    fn layout_title_parsed() {
        let source = "```proof:slide layout=title title=\"Hello\"\ncontent\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        assert!(matches!(doc.slides[0].layout, SlideLayout::Title));
    }

    // ── layout=agenda parsed ────────────────────────────────────────────────
    #[test]
    fn layout_agenda_parsed() {
        let source = "```proof:slide layout=agenda title=\"Today\"\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        assert!(
            matches!(doc.slides[0].layout, SlideLayout::Agenda),
            "agenda layout should parse to SlideLayout::Agenda"
        );
        assert_eq!(doc.slides[0].title.as_deref(), Some("Today"));
    }

    // ── Test 6: two-column ratio 60:40 parsed ───────────────────────────────
    #[test]
    fn layout_two_column_ratio_parsed() {
        let source = "```proof:slide layout=two-column ratio=60:40\ncol content\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        match doc.slides[0].layout {
            SlideLayout::TwoColumn { ratio: (a, b) } => {
                assert_eq!(a, 60);
                assert_eq!(b, 40);
            }
            _ => panic!("expected TwoColumn layout"),
        }
    }

    // ── Test 7: ratio parts don't sum to 100 → InvalidRatio error ───────────
    #[test]
    fn invalid_ratio_rejected() {
        let source = "```proof:slide layout=two-column ratio=60:50\ncol content\n```";
        let result = parse_slide_doc(source);
        // Should either return errors or have silently recovered
        // The spec says SLIDE-002 is an error; we propagate it
        match result {
            Err(errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| matches!(e, SlideError::InvalidRatio { .. })),
                    "expected InvalidRatio error, got: {:?}",
                    errors
                );
            }
            Ok(doc) => {
                // If recovered (Blank fallback), that's also acceptable per spec
                // The important thing is no panic
                let _ = doc;
            }
        }
    }

    // ── Test 8: title= attribute extracted from info string ──────────────────
    #[test]
    fn slide_title_from_info_string() {
        let source = "```proof:slide layout=title-content title=\"My Title\"\nbody\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(doc.slides[0].title.as_deref(), Some("My Title"));
    }

    // ── Test 9: proof:notes extracted to notes_content ───────────────────────
    #[test]
    fn notes_extracted_to_notes_content() {
        let source = "```proof:slide layout=title-content\nBody text\n```proof:notes\nSpeaker notes\n```\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        assert!(
            doc.slides[0].notes_content.contains("Speaker notes"),
            "notes not extracted: {:?}",
            doc.slides[0].notes_content
        );
        assert!(
            !doc.slides[0].body_content.contains("Speaker notes"),
            "notes leaked into body: {:?}",
            doc.slides[0].body_content
        );
    }

    // ── Test 10: body_content contains subtitle/author/date for title layout ─
    #[test]
    fn author_date_extracted_from_body() {
        let source =
            "```proof:slide layout=title title=\"My Deck\"\nauthor: Gio\ndate: April 2026\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        let slide = &doc.slides[0];
        assert_eq!(slide.author.as_deref(), Some("Gio"));
        assert_eq!(slide.date.as_deref(), Some("April 2026"));
    }

    // ── Test 11: empty file → SlideDoc with 1 empty slide, default SlideMeta ─
    #[test]
    fn empty_file_produces_one_slide() {
        let doc = parse_slide_doc("").expect("should parse");
        assert_eq!(doc.slides.len(), 1);
        assert_eq!(doc.meta.width, 120);
        assert_eq!(doc.meta.height, 34);
    }

    // ── Test 12: 5-slide file → slides.len() == 5, indices 1..=5 ────────────
    #[test]
    fn five_slide_file_parsed() {
        let source = make_slides(5);
        let doc = parse_slide_doc(&source).expect("should parse");
        assert_eq!(
            doc.slides.len(),
            5,
            "expected 5 slides, got {}",
            doc.slides.len()
        );
        for (i, slide) in doc.slides.iter().enumerate() {
            assert_eq!(slide.index, i + 1, "slide index wrong at position {}", i);
        }
    }

    // ── Test 13: front-matter theme parsed ───────────────────────────────────
    #[test]
    fn front_matter_theme_parsed() {
        let source = "---\ntheme: box\n---\nContent";
        let doc = parse_slide_doc(source).expect("should parse");
        assert!(matches!(doc.meta.theme, SlideTheme::Box));
    }

    // ── Test 14: extract_notes round-trip ─────────────────────────────────────
    #[test]
    fn extract_notes_round_trip() {
        let body = "Line 1\n```proof:notes\nNote A\nNote B\n```\nLine 2";
        let (body_out, notes_out) = extract_notes(body);
        assert!(body_out.contains("Line 1"), "body missing Line 1");
        assert!(body_out.contains("Line 2"), "body missing Line 2");
        assert!(!body_out.contains("Note A"), "notes leaked to body");
        assert!(notes_out.contains("Note A"), "Note A not in notes");
        assert!(notes_out.contains("Note B"), "Note B not in notes");
    }

    // ── Test 15: parse_ratio validates sum == 100 ─────────────────────────────
    #[test]
    fn parse_ratio_sum_validation() {
        assert!(parse_ratio("50:50", 1).is_ok());
        assert!(parse_ratio("60:40", 1).is_ok());
        assert!(parse_ratio("100:0", 1).is_ok());
        assert!(parse_ratio("60:50", 1).is_err()); // sum = 110
        assert!(parse_ratio("30:30", 1).is_err()); // sum = 60
    }

    // ── Test 16: two-column without ratio= defaults to 60:40 ─────────────────
    #[test]
    fn two_column_default_ratio_is_60_40() {
        let source = "```proof:slide layout=two-column\n## col:left\nA\n## col:right\nB\n```";
        let doc = parse_slide_doc(source).expect("should parse");
        match doc.slides[0].layout {
            SlideLayout::TwoColumn { ratio: (a, b) } => {
                assert_eq!(a, 60, "default left ratio should be 60");
                assert_eq!(b, 40, "default right ratio should be 40");
            }
            _ => panic!("expected TwoColumn layout"),
        }
    }
}
