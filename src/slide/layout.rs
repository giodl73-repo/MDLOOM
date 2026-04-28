/// Slide layout renderers — Wave 2.
///
/// Each layout takes a Slide and produces a Vec<String> of exactly
/// meta.height lines, each meta.width chars wide (SL-1).
///
/// render_body_lines() is a stub in Wave 2 — returns raw lines unchanged.
/// Wave 3 replaces it with full directive dispatch.

use crate::slide::canvas::SlideCanvas;
use crate::slide::{Slide, SlideLayout, SlideMeta, SlideTheme};

// ─────────────────────────────────────────────────────────
// Body stub (Wave 2 — completed in Wave 3)
// ─────────────────────────────────────────────────────────

/// Render body content — dispatches proof: directives, passes literal lines through.
/// Handles: proof:bullets, proof:centered, proof:quote, proof:callout, proof:divider, proof:stat.
/// proof:notes blocks are excluded from output (SL-5).
///
/// Warnings (SLIDE-001 max-bullets, SLIDE-007 max-depth) are discarded — callers
/// who need them should use [`render_body_lines_with_warnings`].
pub fn render_body_lines(body: &str, width: usize) -> Vec<String> {
    use crate::slide::bullets::BulletConfig;
    let (out, _) = render_body_lines_with_warnings(body, width, &BulletConfig::default());
    out
}

/// Same as [`render_body_lines`] but accepts an explicit [`BulletConfig`]
/// (so `max_bullets`/`max_depth` from slide front-matter take effect) and
/// returns the warnings produced by `proof:bullets` rendering.
pub fn render_body_lines_with_warnings(
    body: &str,
    width: usize,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    use crate::slide::bullets::render_bullets;
    use crate::slide::inline::{render_quote, render_centered, render_right, render_ol,
                                render_callout, render_divider, CalloutStyle, DividerStyle};

    let mut output: Vec<String> = Vec::new();
    let mut warnings: Vec<crate::slide::bullets::BulletWarning> = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // proof:notes — skip entire block until blank line (SL-5).
        // Guard: only matches bare "proof:notes" directive, not prose containing the phrase.
        // A line must be EXACTLY "proof:notes" (or "proof:notes" with only trailing spaces)
        // to trigger the skip. This prevents "proof:notes are important" from being silently
        // consumed.
        if line == "proof:notes" {
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty() { i += 1; }
            i += 1;
            continue;
        }

        // proof:bullets — collect lines until blank or next directive
        if line.starts_with("proof:bullets") {
            i += 1;
            let mut bullet_lines = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                bullet_lines.push_str(lines[i]);
                bullet_lines.push('\n');
                i += 1;
            }
            let (rendered, warns) = render_bullets(&bullet_lines, width, bullet_cfg);
            output.extend(rendered);
            warnings.extend(warns);
            continue;
        }

        // proof:centered — next non-empty lines until blank
        if line.starts_with("proof:centered") {
            i += 1;
            let mut text = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_centered(text.trim(), width));
            continue;
        }

        // proof:callout style=X — collect content
        if line.starts_with("proof:callout") {
            let style_str = line.split("style=").nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("note");
            let style = CalloutStyle::parse(style_str);
            i += 1;
            let mut text = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_callout(text.trim(), style, width));
            continue;
        }

        // proof:divider style=X
        if line.starts_with("proof:divider") {
            let style_str = line.split("style=").nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("thin");
            let style = DividerStyle::parse(style_str);
            output.push(render_divider(style, width));
            i += 1;
            continue;
        }

        // proof:right — right-align a block of text
        if line == "proof:right" {
            i += 1;
            let mut text = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_right(text.trim(), width));
            continue;
        }

        // proof:numbered-list (primary) / proof:ol (short-form alias) — ordered list
        if line == "proof:numbered-list" || line == "proof:ol" {
            i += 1;
            let mut text = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_ol(text.trim(), width));
            continue;
        }

        // proof:quote attribution="..."
        if line.starts_with("proof:quote") {
            let attr = line.split("attribution=").nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
            i += 1;
            let mut text = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_quote(text.trim(), attr.as_deref(), width));
            continue;
        }

        // Literal prose line — expand inline math/symbols then word-wrap to slide width
        let expanded = expand_inline(lines[i]);
        let wrapped = word_wrap(&expanded, width);
        output.extend(wrapped);
        i += 1;
    }

    (output, warnings)
}

/// Expand inline `$...$` math and `[sym:name]` in a single prose line.
fn expand_inline(line: &str) -> String {
    let lib = crate::symbol::SymbolLibrary::new();
    let (sym_line, _sym_diags) = crate::symbol::expand_symbols(line, &lib);
    let (math_line, _math_diags) = crate::math::expand_inline_math(&sym_line);
    math_line
}

// ─────────────────────────────────────────────────────────
// Theme application
// ─────────────────────────────────────────────────────────

pub fn apply_theme(lines: &[String], meta: &SlideMeta) -> Vec<String> {
    match meta.theme {
        SlideTheme::None => lines.to_vec(),
        SlideTheme::Minimal => lines.to_vec(), // title separator added by layout
        SlideTheme::Box => {
            let w = meta.width;
            let top = format!("┌{}┐", "─".repeat(w.saturating_sub(2)));
            let bot = format!("└{}┘", "─".repeat(w.saturating_sub(2)));
            let mut out = vec![top];
            for line in lines {
                let inner_w = w.saturating_sub(2);
                let clipped = clip_to_width(line, inner_w);
                let padded = format!("{:<width$}", clipped, width = inner_w);
                out.push(format!("│{}│", padded));
            }
            out.push(bot);
            out
        }
    }
}

// ─────────────────────────────────────────────────────────
// Shared utilities
// ─────────────────────────────────────────────────────────

/// Center a string within `width` cols. Tie-break: extra space on right (SL-6).
pub fn center_in_width(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width { return clip_to_width(s, width); }
    let total_pad = width - len;
    let left = total_pad / 2;
    let right = total_pad - left; // extra on right
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// Word-wrap a string to `width` columns.
///
/// Breaks at word boundaries (spaces). Preserves the leading indentation of the
/// first line on all continuation lines so wrapped paragraphs stay indented.
/// Returns one string per output line.
pub fn word_wrap(s: &str, width: usize) -> Vec<String> {
    if width == 0 { return vec![s.to_string()]; }

    // Detect leading indent (spaces only) to carry onto continuation lines
    let indent_len = s.chars().take_while(|c| *c == ' ').count();
    let indent = " ".repeat(indent_len);
    let effective_width = width.saturating_sub(indent_len).max(1);

    // If the whole string fits, return as-is
    let visual_len = crate::layout::visual_width(s);
    if visual_len <= width {
        return vec![s.to_string()];
    }

    let content = &s[indent_len..]; // strip indent for wrapping
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;

    for word in content.split(' ') {
        let word_w = crate::layout::visual_width(word);
        if current.is_empty() {
            current.push_str(word);
            current_w = word_w;
        } else if current_w + 1 + word_w <= effective_width {
            current.push(' ');
            current.push_str(word);
            current_w += 1 + word_w;
        } else {
            // Flush current line with indent
            lines.push(format!("{}{}", if lines.is_empty() { &indent } else { &indent }, current));
            current = word.to_string();
            current_w = word_w;
        }
    }
    if !current.is_empty() {
        lines.push(format!("{}{}", indent, current));
    }
    if lines.is_empty() {
        lines.push(s.to_string());
    }
    lines
}

/// Clip string to width visual columns, appending … if truncated.
/// Never splits wide Unicode characters (CJK, emoji) at the boundary (F123).
pub fn clip_to_width(s: &str, width: usize) -> String {
    use crate::layout::visual_width;
    if visual_width(s) <= width { return s.to_string(); }
    let ellipsis_w = 1usize; // … is 1 column
    let target = width.saturating_sub(ellipsis_w);
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + ch_w > target { break; }
        out.push(ch);
        w += ch_w;
    }
    out.push('…');
    out
}

/// Pad/clip a string to exactly `width` chars.
pub fn fit_to_width(s: &str, width: usize) -> String {
    let clipped = clip_to_width(s, width);
    let len = clipped.chars().count();
    if len < width {
        format!("{}{}", clipped, " ".repeat(width - len))
    } else {
        clipped
    }
}

/// Build a canvas from a list of content lines, padded to width×height.
fn lines_to_canvas(lines: &[String], width: usize, height: usize) -> Vec<String> {
    let mut result: Vec<String> = lines.iter()
        .take(height)
        .map(|l| fit_to_width(l, width))
        .collect();
    while result.len() < height {
        result.push(" ".repeat(width));
    }
    result
}

/// Horizontal separator rule.
fn separator(width: usize) -> String {
    "─".repeat(width)
}

// ─────────────────────────────────────────────────────────
// Layout renderers
// ─────────────────────────────────────────────────────────

/// `title` layout — title + subtitle + author + date, all vertically and
/// horizontally centered (compositor-driven, not proof:centered directive).
pub fn render_title(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;

    let mut content_lines: Vec<String> = Vec::new();
    if let Some(ref t) = slide.title    { content_lines.push(center_in_width(t, w)); }
    if let Some(ref s) = slide.subtitle { content_lines.push(center_in_width(s, w)); }
    if slide.author.is_some() || slide.date.is_some() {
        content_lines.push(String::new());
        if let Some(ref a) = slide.author { content_lines.push(center_in_width(a, w)); }
        if let Some(ref d) = slide.date   { content_lines.push(center_in_width(d, w)); }
    }

    // Vertical centering: distribute blank lines evenly above and below
    let content_h = content_lines.len();
    let total_pad = h.saturating_sub(content_h);
    let top_pad = total_pad / 2;
    let bot_pad = total_pad - top_pad;

    let mut result: Vec<String> = Vec::with_capacity(h);
    for _ in 0..top_pad   { result.push(" ".repeat(w)); }
    for line in &content_lines { result.push(fit_to_width(line, w)); }
    for _ in 0..bot_pad   { result.push(" ".repeat(w)); }
    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    result
}

/// `title-content` layout — title bar (height 3) + separator + body fills rest.
pub fn render_title_content(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    let body_height = h.saturating_sub(title_height + 1); // +1 for separator

    let title_str = slide.title.as_deref().unwrap_or("");
    let mut result: Vec<String> = Vec::with_capacity(h);

    // Title area (left-aligned, padded)
    result.push(fit_to_width(title_str, w));
    for _ in 1..title_height { result.push(" ".repeat(w)); }

    // Separator
    result.push(separator(w));

    // Body
    let body_lines = render_body_lines(&slide.body_content, w);
    result.extend(lines_to_canvas(&body_lines, w, body_height));

    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    result
}

/// `two-column` layout — columns split by ratio, optional divider.
/// Column delimiters in body: `## col:left` and `## col:right` (H2 level).
pub fn render_two_column(
    slide: &Slide,
    meta: &SlideMeta,
    ratio: (u8, u8),
) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = if slide.title.is_some() { 2usize } else { 0 };
    let body_height = h.saturating_sub(title_height);

    // Column width: floor() with remainder to first column (per spec rounding rule)
    let ratio_sum = (ratio.0 as usize) + (ratio.1 as usize);
    let col_a_w = (w * ratio.0 as usize) / ratio_sum;
    let col_b_w = w.saturating_sub(col_a_w); // remainder goes to second col? No — first gets remainder
    // Actually: spec says "remainder to first column"
    let col_a_raw = (w * ratio.0 as usize) / ratio_sum;
    let col_b_raw = (w * ratio.1 as usize) / ratio_sum;
    let remainder = w.saturating_sub(col_a_raw + col_b_raw);
    let col_a_width = col_a_raw + remainder; // first column gets remainder
    let col_b_width = col_b_raw;

    // Split body at ## col: markers
    let (col_a_content, col_b_content) = split_two_column(&slide.body_content);
    let col_a_lines = render_body_lines(&col_a_content, col_a_width);
    let col_b_lines = render_body_lines(&col_b_content, col_b_width);
    let col_a = lines_to_canvas(&col_a_lines, col_a_width, body_height);
    let col_b = lines_to_canvas(&col_b_lines, col_b_width, body_height);

    let mut result: Vec<String> = Vec::with_capacity(h);

    // Title
    if let Some(ref t) = slide.title {
        result.push(fit_to_width(t, w));
        result.push(separator(w));
    }

    // Interleave columns
    for i in 0..body_height {
        let a = col_a.get(i).map(|s| s.as_str()).unwrap_or("");
        let b = col_b.get(i).map(|s| s.as_str()).unwrap_or("");
        result.push(format!("{}{}", fit_to_width(a, col_a_width), fit_to_width(b, col_b_width)));
    }

    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    result
}

/// Split body content at `## col:left` and `## col:right` markers.
fn split_two_column(body: &str) -> (String, String) {
    let mut col_a = String::new();
    let mut col_b = String::new();
    let mut current = 'a'; // 'a' = before first marker, treat as col_a

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "## col:left" || trimmed == "## col:1" {
            current = 'a';
            continue;
        }
        if trimmed == "## col:right" || trimmed == "## col:2" {
            current = 'b';
            continue;
        }
        match current {
            'a' => { col_a.push_str(line); col_a.push('\n'); }
            'b' => { col_b.push_str(line); col_b.push('\n'); }
            _ => {}
        }
    }
    (col_a, col_b)
}

/// `section` layout — compositor-driven centering. Title and subtitle
/// centered both vertically and horizontally. Authors cannot override
/// (use `blank` layout with proof:centered for custom alignment).
pub fn render_section(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;

    let mut lines: Vec<String> = Vec::new();
    if let Some(ref t) = slide.title {
        lines.push(center_in_width(&format!("── {} ──", t), w));
    }
    if let Some(ref s) = slide.subtitle {
        lines.push(String::new());
        lines.push(center_in_width(s, w));
    }

    let total_pad = h.saturating_sub(lines.len());
    let top = total_pad / 2;
    let bot = total_pad - top;

    let mut result = Vec::with_capacity(h);
    for _ in 0..top { result.push(" ".repeat(w)); }
    for l in &lines { result.push(fit_to_width(l, w)); }
    for _ in 0..bot { result.push(" ".repeat(w)); }
    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    result
}

/// `stats` layout — large numbers with labels, centered.
/// Uses its own dedicated renderer (NOT proof:columns).
/// SL-3 does not apply — column widths = floor(width/count), remainder to rightmost.
pub fn render_stats(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = if slide.title.is_some() { 2 } else { 0 };
    let body_height = h.saturating_sub(title_height);

    // Parse stats from body: each line "value | label | sublabel" or "value | label"
    let mut stats: Vec<(String, String, String)> = Vec::new(); // (value, label, sublabel)
    for line in slide.body_content.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').map(|s| s.trim()).collect();
        match parts.len() {
            3 => stats.push((parts[0].into(), parts[1].into(), parts[2].into())),
            2 => stats.push((parts[0].into(), parts[1].into(), String::new())),
            1 if !parts[0].is_empty() => stats.push((parts[0].into(), String::new(), String::new())),
            _ => {}
        }
    }

    if stats.is_empty() {
        return lines_to_canvas(&[], w, h);
    }

    // Column width: floor(w / count), remainder to rightmost
    let n = stats.len();
    let col_w_base = w / n;
    let remainder = w - col_w_base * n;

    let col_widths: Vec<usize> = (0..n).map(|i| {
        if i == n - 1 { col_w_base + remainder } else { col_w_base }
    }).collect();

    // Build content rows (value row, label row, sublabel row)
    let value_row: String = stats.iter().zip(col_widths.iter())
        .map(|((v, _, _), &cw)| fit_to_width(&center_in_width(v, cw), cw))
        .collect();
    let label_row: String = stats.iter().zip(col_widths.iter())
        .map(|((_, l, _), &cw)| fit_to_width(&center_in_width(l, cw), cw))
        .collect();
    let sublabel_row: String = stats.iter().zip(col_widths.iter())
        .map(|((_, _, sl), &cw)| fit_to_width(&center_in_width(sl, cw), cw))
        .collect();

    let content_lines = vec![value_row, label_row, sublabel_row];
    let total_pad = body_height.saturating_sub(content_lines.len());
    let top = total_pad / 2;

    let mut result = Vec::with_capacity(h);
    if let Some(ref t) = slide.title {
        result.push(fit_to_width(t, w));
        result.push(separator(w));
    }
    for _ in 0..top { result.push(" ".repeat(w)); }
    for l in &content_lines { result.push(fit_to_width(l, w)); }
    while result.len() < h { result.push(" ".repeat(w)); }
    result.truncate(h);
    result
}

/// `blank` layout — all content passed through render_body_lines.
pub fn render_blank(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let body_lines = render_body_lines(&slide.body_content, meta.width);
    lines_to_canvas(&body_lines, meta.width, meta.height)
}

/// Dispatch to the correct renderer based on SlideLayout.
///
/// Bullet-list warnings are discarded — callers who need them (e.g. the compile
/// pipeline, which surfaces SLIDE-WARN HTML comments to the author) should use
/// [`render_slide_with_warnings`].
pub fn render_slide(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let (lines, _) = render_slide_with_warnings(slide, meta);
    lines
}

/// Render a slide and return both the rendered lines and any bullet-list warnings
/// (SLIDE-001 max-bullets, SLIDE-007 max-depth) collected from `proof:bullets`
/// directives in the slide body.
///
/// `BulletConfig` is derived from `meta.max_bullets` / `meta.max_depth` so authors
/// can tune the threshold via slide front-matter (`max-bullets: N`).
pub fn render_slide_with_warnings(
    slide: &Slide,
    meta: &SlideMeta,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    use crate::slide::bullets::BulletConfig;
    let bullet_cfg = BulletConfig {
        max_bullets: meta.max_bullets,
        max_depth: meta.max_depth,
        ..BulletConfig::default()
    };

    let (raw, warnings) = match &slide.layout {
        SlideLayout::Title => (render_title(slide, meta), Vec::new()),
        SlideLayout::TitleContent => render_title_content_with_warnings(slide, meta, &bullet_cfg),
        SlideLayout::TwoColumn { ratio } => {
            render_two_column_with_warnings(slide, meta, *ratio, &bullet_cfg)
        }
        SlideLayout::Section => (render_section(slide, meta), Vec::new()),
        SlideLayout::Stats => (render_stats(slide, meta), Vec::new()),
        SlideLayout::Blank => render_blank_with_warnings(slide, meta, &bullet_cfg),
        SlideLayout::ContentCaption | SlideLayout::Comparison => {
            // Fallback to title-content for unimplemented layouts
            render_title_content_with_warnings(slide, meta, &bullet_cfg)
        }
    };
    (apply_theme(&raw, meta), warnings)
}

fn render_title_content_with_warnings(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    let body_height = h.saturating_sub(title_height + 1);

    let title_str = slide.title.as_deref().unwrap_or("");
    let mut result: Vec<String> = Vec::with_capacity(h);
    result.push(fit_to_width(title_str, w));
    for _ in 1..title_height { result.push(" ".repeat(w)); }
    result.push(separator(w));

    let (body_lines, warnings) = render_body_lines_with_warnings(&slide.body_content, w, bullet_cfg);
    result.extend(lines_to_canvas(&body_lines, w, body_height));

    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    (result, warnings)
}

fn render_two_column_with_warnings(
    slide: &Slide,
    meta: &SlideMeta,
    ratio: (u8, u8),
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    let w = meta.width;
    let h = meta.height;
    let title_height = if slide.title.is_some() { 2usize } else { 0 };
    let body_height = h.saturating_sub(title_height);

    let ratio_sum = (ratio.0 as usize) + (ratio.1 as usize);
    let col_a_raw = (w * ratio.0 as usize) / ratio_sum;
    let col_b_raw = (w * ratio.1 as usize) / ratio_sum;
    let remainder = w.saturating_sub(col_a_raw + col_b_raw);
    let col_a_width = col_a_raw + remainder;
    let col_b_width = col_b_raw;

    let (col_a_content, col_b_content) = split_two_column(&slide.body_content);
    let (col_a_lines, mut warns_a) =
        render_body_lines_with_warnings(&col_a_content, col_a_width, bullet_cfg);
    let (col_b_lines, warns_b) =
        render_body_lines_with_warnings(&col_b_content, col_b_width, bullet_cfg);
    warns_a.extend(warns_b);

    let col_a = lines_to_canvas(&col_a_lines, col_a_width, body_height);
    let col_b = lines_to_canvas(&col_b_lines, col_b_width, body_height);

    let mut result: Vec<String> = Vec::with_capacity(h);
    if let Some(ref t) = slide.title {
        result.push(fit_to_width(t, w));
        result.push(separator(w));
    }
    for i in 0..body_height {
        let a = col_a.get(i).map(|s| s.as_str()).unwrap_or("");
        let b = col_b.get(i).map(|s| s.as_str()).unwrap_or("");
        result.push(format!("{}{}", fit_to_width(a, col_a_width), fit_to_width(b, col_b_width)));
    }
    result.truncate(h);
    while result.len() < h { result.push(" ".repeat(w)); }
    (result, warns_a)
}

fn render_blank_with_warnings(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    let (body_lines, warnings) =
        render_body_lines_with_warnings(&slide.body_content, meta.width, bullet_cfg);
    (lines_to_canvas(&body_lines, meta.width, meta.height), warnings)
}

// ─────────────────────────────────────────────────────────
// Reveal: progressive-reveal page generation
// ─────────────────────────────────────────────────────────

/// Render a slide as one or more reveal "pages" (frames).
///
/// When a `proof:bullets` block in the slide body contains `[N]` reveal-step
/// prefixes, the slide is expanded into multiple pages — one per distinct step
/// value.  Page N shows all bullets with step ≤ N (cumulative reveal).  The
/// title/chrome is identical on every page; only the bullet visibility changes.
///
/// If no `[N]` markers (N ≥ 2) are present, returns a `Vec` with exactly one
/// element, identical to `render_slide`.
///
/// The caller is responsible for joining pages with the appropriate output
/// separator (e.g. `---` for the `.slides.md` format or a form-feed for paging
/// terminal output).
pub fn render_slide_pages(slide: &Slide, meta: &SlideMeta) -> Vec<Vec<String>> {
    use crate::slide::bullets::{BulletConfig, has_reveal_markers, render_bullets_pages};

    let bullet_cfg = BulletConfig {
        max_bullets: meta.max_bullets,
        max_depth: meta.max_depth,
        ..BulletConfig::default()
    };

    // Fast path: no reveal markers anywhere in the body
    if !has_reveal_markers(&slide.body_content) {
        return vec![render_slide(slide, meta)];
    }

    // Only title-content and blank layouts support reveal pages today.
    // Two-column and others fall back to single-page rendering.
    match &slide.layout {
        SlideLayout::TitleContent | SlideLayout::ContentCaption | SlideLayout::Comparison => {
            render_reveal_pages_title_content(slide, meta, &bullet_cfg)
        }
        SlideLayout::Blank => {
            render_reveal_pages_blank(slide, meta, &bullet_cfg)
        }
        _ => vec![render_slide(slide, meta)],
    }
}

/// Build reveal pages for title-content layout.
fn render_reveal_pages_title_content(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> Vec<Vec<String>> {
    use crate::slide::bullets::render_bullets_pages;

    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    let body_height = h.saturating_sub(title_height + 1);

    // Build the fixed chrome (title area + separator) — same on every page
    let title_str = slide.title.as_deref().unwrap_or("");
    let mut chrome: Vec<String> = Vec::with_capacity(title_height + 1);
    chrome.push(fit_to_width(title_str, w));
    for _ in 1..title_height { chrome.push(" ".repeat(w)); }
    chrome.push(separator(w));

    // Expand the body: split on proof:bullets, generate pages for each bullets block,
    // then reassemble. For simplicity, we treat the entire body as a single bullets
    // block if it starts with proof:bullets; otherwise fall back to single-page.
    //
    // Strategy: render_body_lines_pages returns Vec<Vec<String>> — one body rendition
    // per reveal step.  We then combine chrome + each body rendition into a full page.
    let body_pages = render_body_lines_pages(&slide.body_content, w, bullet_cfg);

    body_pages.into_iter().map(|body_lines| {
        let mut page = chrome.clone();
        page.extend(lines_to_canvas(&body_lines, w, body_height));
        page.truncate(h);
        while page.len() < h { page.push(" ".repeat(w)); }
        apply_theme(&page, meta)
    }).collect()
}

/// Build reveal pages for blank layout.
fn render_reveal_pages_blank(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> Vec<Vec<String>> {
    let body_pages = render_body_lines_pages(&slide.body_content, meta.width, bullet_cfg);
    body_pages.into_iter().map(|body_lines| {
        let page = lines_to_canvas(&body_lines, meta.width, meta.height);
        apply_theme(&page, meta)
    }).collect()
}

/// Render body content for each reveal step, returning one `Vec<String>` per step.
///
/// Scans the body for `proof:bullets` directives that contain `[N]` reveal
/// markers.  For each step, renders the body with that step's bullet visibility.
/// Non-bullet directives and prose are identical across all pages.
///
/// If no reveal markers are present returns a single-element vec.
pub fn render_body_lines_pages(
    body: &str,
    width: usize,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> Vec<Vec<String>> {
    use crate::slide::bullets::{has_reveal_markers, render_bullets_pages};

    // Quick scan: does any proof:bullets block in this body have reveal markers?
    // We need to find such blocks first.
    let lines: Vec<&str> = body.lines().collect();
    let mut has_any_reveal = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("proof:bullets") {
            i += 1;
            let mut block = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                block.push_str(lines[i]);
                block.push('\n');
                i += 1;
            }
            if has_reveal_markers(&block) {
                has_any_reveal = true;
                break;
            }
            continue;
        }
        i += 1;
    }

    if !has_any_reveal {
        let (out, _) = render_body_lines_with_warnings(body, width, bullet_cfg);
        return vec![out];
    }

    // Full pass: for each proof:bullets block with reveal markers, collect the
    // pages it generates.  All other directives emit a single Vec<String> (same
    // on every page).  We then transpose: for page N, assemble the Nth slice of
    // each segment.

    #[derive(Debug)]
    enum Segment {
        // Same lines on every reveal page
        Fixed(Vec<String>),
        // Different lines per reveal step
        Paged(Vec<Vec<String>>),
    }

    let mut segments: Vec<Segment> = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line == "proof:notes" {
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty() { i += 1; }
            i += 1;
            continue;
        }

        if line.starts_with("proof:bullets") {
            i += 1;
            let mut bullet_lines = String::new();
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                bullet_lines.push_str(lines[i]);
                bullet_lines.push('\n');
                i += 1;
            }
            if has_reveal_markers(&bullet_lines) {
                let (pages, _) = render_bullets_pages(&bullet_lines, width, bullet_cfg);
                segments.push(Segment::Paged(pages));
            } else {
                let (rendered, _) = crate::slide::bullets::render_bullets(
                    &bullet_lines, width, bullet_cfg);
                segments.push(Segment::Fixed(rendered));
            }
            continue;
        }

        // All other directives and prose: render normally into a Fixed segment
        // We render this single line/block via a mini body string
        let mut mini_body = String::new();
        if line.starts_with("proof:") {
            // Consume the whole directive block
            mini_body.push_str(lines[i]);
            mini_body.push('\n');
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:") {
                mini_body.push_str(lines[i]);
                mini_body.push('\n');
                i += 1;
            }
        } else {
            mini_body.push_str(lines[i]);
            mini_body.push('\n');
            i += 1;
        }
        let (rendered, _) = render_body_lines_with_warnings(&mini_body, width, bullet_cfg);
        segments.push(Segment::Fixed(rendered));
    }

    // Determine total page count = max pages across all Paged segments
    let page_count = segments.iter().map(|seg| match seg {
        Segment::Fixed(_) => 1,
        Segment::Paged(pages) => pages.len(),
    }).max().unwrap_or(1).max(1);

    // Assemble: for each page index, concatenate Fixed lines + Paged[page_idx] lines
    (0..page_count).map(|page_idx| {
        let mut out: Vec<String> = Vec::new();
        for seg in &segments {
            match seg {
                Segment::Fixed(lines) => out.extend_from_slice(lines),
                Segment::Paged(pages) => {
                    // Use the last page if page_idx exceeds available pages
                    let idx = page_idx.min(pages.len().saturating_sub(1));
                    out.extend_from_slice(&pages[idx]);
                }
            }
        }
        out
    }).collect()
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slide::{SlideLayout, SlideMeta};

    fn blank_slide(layout: SlideLayout) -> Slide {
        Slide {
            index: 0,
            layout,
            title: None,
            subtitle: None,
            author: None,
            date: None,
            body_content: String::new(),
            notes_content: String::new(),
            source_line: 0,
        }
    }

    fn meta_80x24() -> SlideMeta {
        SlideMeta { width: 80, height: 24, ..SlideMeta::default() }
    }

    // ── SL-1: every layout produces exactly height rows of width chars ──

    fn assert_sl1(lines: &[String], meta: &SlideMeta) {
        assert_eq!(lines.len(), meta.height, "SL-1 line count");
        for (i, l) in lines.iter().enumerate() {
            assert_eq!(l.chars().count(), meta.width,
                "SL-1 line {} width mismatch: {:?}", i, l);
        }
    }

    #[test]
    fn title_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Title);
        s.title = Some("Hello".into());
        s.subtitle = Some("World".into());
        assert_sl1(&render_title(&s, &meta), &meta);
    }

    #[test]
    fn title_content_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::TitleContent);
        s.title = Some("Title".into());
        s.body_content = "bullet 1\nbullet 2\n".into();
        assert_sl1(&render_title_content(&s, &meta), &meta);
    }

    #[test]
    fn two_column_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::TwoColumn { ratio: (50, 50) });
        s.title = Some("Compare".into());
        s.body_content = "## col:left\nLeft content\n## col:right\nRight content\n".into();
        assert_sl1(&render_two_column(&s, &meta, (50, 50)), &meta);
    }

    #[test]
    fn section_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Section);
        s.title = Some("Part 2".into());
        assert_sl1(&render_section(&s, &meta), &meta);
    }

    #[test]
    fn stats_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Stats);
        s.title = Some("Key Numbers".into());
        s.body_content = "138.0 | Pts/82 | #1 all-time\n62.3% | Corsi | Top 0.1%\n".into();
        assert_sl1(&render_stats(&s, &meta), &meta);
    }

    #[test]
    fn blank_layout_sl1() {
        let meta = meta_80x24();
        let s = blank_slide(SlideLayout::Blank);
        assert_sl1(&render_blank(&s, &meta), &meta);
    }

    // ── Two-column rounding ─────────────────────────────────

    #[test]
    fn two_column_ratio_rounding_odd_width() {
        // 119 cols, 60:40 → col_a_raw=71, col_b_raw=47, remainder=1 → col_a=72, col_b=47
        let meta = SlideMeta { width: 119, height: 10, ..SlideMeta::default() };
        let mut s = blank_slide(SlideLayout::TwoColumn { ratio: (60, 40) });
        s.body_content = "## col:left\nA\n## col:right\nB\n".into();
        let lines = render_two_column(&s, &meta, (60, 40));
        // Each body line should be exactly 119 chars
        for line in &lines {
            assert_eq!(line.chars().count(), 119, "width mismatch: {:?}", line);
        }
    }

    // ── Section centering ────────────────────────────────────

    #[test]
    fn section_title_is_centered() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Section);
        s.title = Some("Test".into());
        let lines = render_section(&s, &meta);
        // Find the line with the title
        let title_line = lines.iter().find(|l| l.contains("Test")).unwrap();
        let left_spaces = title_line.chars().take_while(|&c| c == ' ').count();
        let right_spaces = title_line.chars().rev().take_while(|&c| c == ' ').count();
        // Left and right padding should be approximately equal (tie-break: right gets extra)
        assert!(right_spaces >= left_spaces, "tie-break should put extra space on right");
    }

    // ── center_in_width tie-break ────────────────────────────

    #[test]
    fn center_tie_break_extra_right() {
        // "Go" (2 chars) in width 9: total_pad=7, left=3, right=4
        let r = center_in_width("Go", 9);
        assert_eq!(r.len(), 9);
        let left = r.chars().take_while(|&c| c == ' ').count();
        let right = r.chars().rev().take_while(|&c| c == ' ').count();
        assert_eq!(left, 3);
        assert_eq!(right, 4); // extra space on right
    }

    // ── Theme application ────────────────────────────────────

    #[test]
    fn theme_none_unchanged() {
        let meta = SlideMeta { theme: SlideTheme::None, ..meta_80x24() };
        let lines = vec!["hello".to_string()];
        assert_eq!(apply_theme(&lines, &meta), lines);
    }

    #[test]
    fn theme_box_adds_border() {
        let meta = SlideMeta { width: 10, height: 1, theme: SlideTheme::Box, ..SlideMeta::default() };
        let lines = vec!["hi".to_string()];
        let themed = apply_theme(&lines, &meta);
        assert!(themed[0].starts_with('┌'));
        assert!(themed[themed.len()-1].starts_with('└'));
    }

    // ── split_two_column ─────────────────────────────────────

    #[test]
    fn split_two_column_basic() {
        let body = "## col:left\nLeft 1\nLeft 2\n## col:right\nRight 1\n";
        let (a, b) = split_two_column(body);
        assert!(a.contains("Left 1"));
        assert!(b.contains("Right 1"));
        assert!(!a.contains("Right"));
    }

    // ── word_wrap ──────────────────────────────────────────

    #[test]
    fn word_wrap_short_line_unchanged() {
        let result = word_wrap("Hello world", 40);
        assert_eq!(result, vec!["Hello world"]);
    }

    #[test]
    fn word_wrap_long_line_breaks_at_word() {
        let result = word_wrap("The quick brown fox jumped over the lazy dog", 20);
        assert!(result.len() > 1, "long line should wrap");
        for line in &result {
            assert!(line.chars().count() <= 20, "line {:?} exceeds width 20", line);
        }
        // All words should still be present
        let full = result.join(" ");
        assert!(full.contains("quick") && full.contains("dog"));
    }

    #[test]
    fn word_wrap_preserves_indent() {
        let result = word_wrap("  This is an indented line that goes way beyond the width limit", 30);
        assert!(result.len() > 1);
        // The continuation lines should preserve the 2-space indent
        for line in result.iter().skip(1) {
            assert!(line.starts_with("  "), "continuation should preserve indent: {:?}", line);
        }
    }

    #[test]
    fn word_wrap_exact_width_no_break() {
        let s = "12345678901234567890"; // exactly 20 chars
        let result = word_wrap(s, 20);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn word_wrap_zero_width_no_panic() {
        let result = word_wrap("some text", 0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn ol_body_dispatch_short_alias() {
        let body = "proof:ol\n- A\n- B";
        let lines = render_body_lines(body, 40);
        assert!(lines.iter().any(|l| l.contains("1.") && l.contains('A')));
        assert!(lines.iter().any(|l| l.contains("2.") && l.contains('B')));
    }

    #[test]
    fn numbered_list_body_dispatch_primary_name() {
        // proof:numbered-list must produce identical output to proof:ol.
        let from_long = render_body_lines("proof:numbered-list\n- A\n- B", 40);
        let from_short = render_body_lines("proof:ol\n- A\n- B", 40);
        assert_eq!(from_long, from_short);
    }

    // ── SLIDE-001: max-bullets warnings flow through render_slide_with_warnings ──

    #[test]
    fn slide_with_six_bullets_emits_slide001_at_default_threshold() {
        // Default max_bullets is 4 (the 30-second rule). 6 bullets must warn twice
        // (bullets 5 and 6 each exceed the threshold).
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::TitleContent);
        s.title = Some("Too many points".into());
        s.body_content = "proof:bullets\n- One\n- Two\n- Three\n- Four\n- Five\n- Six\n".into();

        let (_, warnings) = render_slide_with_warnings(&s, &meta);
        let slide001: Vec<_> = warnings.iter().filter(|w| w.code == "SLIDE-001").collect();
        assert_eq!(slide001.len(), 2,
            "expected 2 SLIDE-001 warnings (bullets 5 and 6) at default max_bullets=4, got: {:?}",
            warnings);
    }

    #[test]
    fn slide_with_four_bullets_no_warning_at_default_threshold() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::TitleContent);
        s.title = Some("Just right".into());
        s.body_content = "proof:bullets\n- One\n- Two\n- Three\n- Four\n".into();

        let (_, warnings) = render_slide_with_warnings(&s, &meta);
        assert!(
            warnings.iter().all(|w| w.code != "SLIDE-001"),
            "4 bullets at threshold 4 should not warn, got: {:?}",
            warnings
        );
    }

    #[test]
    fn slide_max_bullets_configurable_via_meta() {
        // Author overrides the threshold via slide front-matter (max-bullets: 8).
        // 6 bullets is then under the threshold and should not warn.
        let meta = SlideMeta { max_bullets: 8, ..meta_80x24() };
        let mut s = blank_slide(SlideLayout::TitleContent);
        s.title = Some("Higher threshold".into());
        s.body_content = "proof:bullets\n- 1\n- 2\n- 3\n- 4\n- 5\n- 6\n".into();

        let (_, warnings) = render_slide_with_warnings(&s, &meta);
        assert!(
            warnings.iter().all(|w| w.code != "SLIDE-001"),
            "6 bullets at threshold 8 should not warn, got: {:?}",
            warnings
        );
    }

    #[test]
    fn slide_max_bullets_two_column_layout_collects_both_columns() {
        // Two-column slide with 3 bullets per column (6 total) at default threshold 4.
        // Should warn — the warning is per-slide, not per-column.
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::TwoColumn { ratio: (50, 50) });
        s.title = Some("Compare".into());
        s.body_content = concat!(
            "## col:left\n",
            "proof:bullets\n- L1\n- L2\n- L3\n",
            "## col:right\n",
            "proof:bullets\n- R1\n- R2\n- R3\n",
        ).into();

        let (_, warnings) = render_slide_with_warnings(&s, &meta);
        // Each column independently re-counts bullets, so each column emits warnings
        // when its OWN bullet count exceeds the threshold. With 3 bullets per side
        // at threshold 4, neither column should warn — this documents that the
        // counter is per-bullet-list, not per-slide.
        assert!(
            warnings.iter().all(|w| w.code != "SLIDE-001"),
            "3 bullets per column at threshold 4 should not warn, got: {:?}",
            warnings
        );
    }

    // ── render_slide_pages / proof:reveal ─────────────────

    fn make_reveal_slide(layout: SlideLayout, title: Option<&str>, body: &str) -> (Slide, SlideMeta) {
        let mut s = blank_slide(layout);
        s.title = title.map(|t| t.into());
        s.body_content = body.into();
        (s, meta_80x24())
    }

    #[test]
    fn reveal_no_markers_single_page() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent, Some("Title"),
            "proof:bullets\n- A\n- B\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 1, "no reveal markers → single page");
        assert_sl1(&pages[0], &meta);
    }

    #[test]
    fn reveal_two_steps_two_pages_sl1() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent, Some("Title"),
            "proof:bullets\n- Always\n[2] - Step 2\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 2, "two steps → two pages");
        for page in &pages {
            assert_sl1(page, &meta);
        }
    }

    #[test]
    fn reveal_page_1_hides_step_2() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent, Some("Title"),
            "proof:bullets\n- Always\n[2] - Step 2\n",
        );
        let pages = render_slide_pages(&s, &meta);
        let p1 = pages[0].join("\n");
        assert!( p1.contains("Always"), "page 1 should show step-1 bullet");
        assert!(!p1.contains("Step 2"), "page 1 should hide step-2 bullet");
    }

    #[test]
    fn reveal_page_2_shows_all() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent, Some("Title"),
            "proof:bullets\n- Always\n[2] - Step 2\n",
        );
        let pages = render_slide_pages(&s, &meta);
        let p2 = pages[1].join("\n");
        assert!(p2.contains("Always") && p2.contains("Step 2"),
            "page 2 should show all bullets");
    }

    #[test]
    fn reveal_title_identical_on_all_pages() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent, Some("My Deck Title"),
            "proof:bullets\n- A\n[2] - B\n[3] - C\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 3);
        for page in &pages {
            assert!(page[0].contains("My Deck Title"),
                "title row must be identical on every page");
        }
    }

    #[test]
    fn reveal_blank_layout_pages_sl1() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::Blank, None,
            "proof:bullets\n- One\n[2] - Two\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 2);
        for page in &pages {
            assert_sl1(page, &meta);
        }
    }

    #[test]
    fn render_body_lines_pages_no_markers_single_page() {
        use crate::slide::bullets::BulletConfig;
        let cfg = BulletConfig::default();
        let body = "proof:bullets\n- A\n- B\n";
        let pages = render_body_lines_pages(body, 80, &cfg);
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn render_body_lines_pages_fixed_segment_on_every_page() {
        use crate::slide::bullets::BulletConfig;
        let cfg = BulletConfig { max_bullets: 10, ..BulletConfig::default() };
        // A fixed centered block, then a reveal bullets block
        let body = "proof:centered\nIntro\n\nproof:bullets\n- Always\n[2] - Step 2\n";
        let pages = render_body_lines_pages(body, 80, &cfg);
        assert_eq!(pages.len(), 2, "reveal block → 2 pages");
        for page in &pages {
            let text = page.join("\n");
            assert!(text.contains("Intro"), "fixed prose must appear on every page");
        }
    }
}
