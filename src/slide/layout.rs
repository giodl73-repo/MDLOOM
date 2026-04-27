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
pub fn render_body_lines(body: &str, width: usize) -> Vec<String> {
    use crate::slide::bullets::{render_bullets, BulletConfig};
    use crate::slide::inline::{render_quote, render_centered, render_callout,
                                render_divider, CalloutStyle, DividerStyle};

    let mut output: Vec<String> = Vec::new();
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
            let cfg = BulletConfig::default();
            let (rendered, _) = render_bullets(&bullet_lines, width, &cfg);
            output.extend(rendered);
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

        // Literal line
        output.push(lines[i].to_string());
        i += 1;
    }

    output
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

/// Clip string to width, appending … if truncated.
pub fn clip_to_width(s: &str, width: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width { return s.to_string(); }
    let mut out: String = chars[..width.saturating_sub(1)].iter().collect();
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
pub fn render_slide(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let raw = match &slide.layout {
        SlideLayout::Title       => render_title(slide, meta),
        SlideLayout::TitleContent => render_title_content(slide, meta),
        SlideLayout::TwoColumn { ratio } => render_two_column(slide, meta, *ratio),
        SlideLayout::Section     => render_section(slide, meta),
        SlideLayout::Stats       => render_stats(slide, meta),
        SlideLayout::Blank       => render_blank(slide, meta),
        SlideLayout::ContentCaption | SlideLayout::Comparison => {
            // Fallback to title-content for unimplemented layouts
            render_title_content(slide, meta)
        }
    };
    apply_theme(&raw, meta)
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
}
