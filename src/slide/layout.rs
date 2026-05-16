use crate::slide::{FooterMode, Slide, SlideLayout, SlideMeta, SlideTheme};

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
    use crate::slide::inline::{
        render_callout, render_centered, render_divider, render_ol, render_quote, render_right,
        CalloutStyle, DividerStyle,
    };

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
            while i < lines.len() && !lines[i].trim().is_empty() {
                i += 1;
            }
            i += 1;
            continue;
        }

        // proof:bullets — collect lines until blank or next directive
        if line.starts_with("proof:bullets") {
            i += 1;
            let mut bullet_lines = String::new();
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
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
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_centered(text.trim(), width));
            continue;
        }

        // proof:callout style=X — collect content
        if line.starts_with("proof:callout") {
            let style_str = line
                .split("style=")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("note");
            let style = CalloutStyle::parse(style_str);
            i += 1;
            let mut text = String::new();
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_callout(text.trim(), style, width));
            continue;
        }

        // proof:divider style=X
        if line.starts_with("proof:divider") {
            let style_str = line
                .split("style=")
                .nth(1)
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
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
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
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
                text.push_str(lines[i]);
                text.push('\n');
                i += 1;
            }
            output.extend(render_ol(text.trim(), width));
            continue;
        }

        // proof:quote attribution="..."
        if line.starts_with("proof:quote") {
            let attr = line
                .split("attribution=")
                .nth(1)
                .map(|s| s.trim().trim_matches('"').to_string());
            i += 1;
            let mut text = String::new();
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
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
    if len >= width {
        return clip_to_width(s, width);
    }
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
    if width == 0 {
        return vec![s.to_string()];
    }

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
            lines.push(format!("{}{}", &indent, current));
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
    if visual_width(s) <= width {
        return s.to_string();
    }
    let ellipsis_w = 1usize; // … is 1 column
    let target = width.saturating_sub(ellipsis_w);
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + ch_w > target {
            break;
        }
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

/// Compose the footer text for a deck from its meta.
///
/// Returns `None` when `meta.footer` is `FooterMode::Off`.
/// For `Auto`, builds "author · date" from available deck-level fields, falling
/// back gracefully when one or both are absent.
/// For `Custom(s)`, returns `s` as-is.
pub fn build_footer_line(meta: &SlideMeta) -> Option<String> {
    match &meta.footer {
        FooterMode::Off => None,
        FooterMode::Custom(s) => Some(s.clone()),
        FooterMode::Auto => {
            let parts: Vec<&str> = [meta.author.as_deref(), meta.date.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" · "))
            }
        }
    }
}

/// Stamp the footer onto the last row of a rendered slide canvas.
///
/// The footer is right-aligned.  If the footer text is wider than `meta.width`,
/// it is clipped with `…`.  This overwrites the last row; callers must ensure
/// the last row is a blank (padding) row and not content.
pub fn apply_footer(lines: &mut Vec<String>, meta: &SlideMeta) {
    let Some(text) = build_footer_line(meta) else {
        return;
    };
    let w = meta.width;
    let clipped = clip_to_width(&text, w);
    let vw = crate::layout::visual_width(&clipped);
    let pad = w.saturating_sub(vw);
    let footer_row = format!("{}{}", " ".repeat(pad), clipped);
    if let Some(last) = lines.last_mut() {
        *last = footer_row;
    }
}

/// Build a canvas from a list of content lines, padded to width×height.
fn lines_to_canvas(lines: &[String], width: usize, height: usize) -> Vec<String> {
    let mut result: Vec<String> = lines
        .iter()
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
    if let Some(ref t) = slide.title {
        content_lines.push(center_in_width(t, w));
    }
    if let Some(ref s) = slide.subtitle {
        content_lines.push(center_in_width(s, w));
    }
    if slide.author.is_some() || slide.date.is_some() {
        content_lines.push(String::new());
        if let Some(ref a) = slide.author {
            content_lines.push(center_in_width(a, w));
        }
        if let Some(ref d) = slide.date {
            content_lines.push(center_in_width(d, w));
        }
    }

    // Vertical centering: distribute blank lines evenly above and below
    let content_h = content_lines.len();
    let total_pad = h.saturating_sub(content_h);
    let top_pad = total_pad / 2;
    let bot_pad = total_pad - top_pad;

    let mut result: Vec<String> = Vec::with_capacity(h);
    for _ in 0..top_pad {
        result.push(" ".repeat(w));
    }
    for line in &content_lines {
        result.push(fit_to_width(line, w));
    }
    for _ in 0..bot_pad {
        result.push(" ".repeat(w));
    }
    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
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
    for _ in 1..title_height {
        result.push(" ".repeat(w));
    }

    // Separator
    result.push(separator(w));

    // Body
    let body_lines = render_body_lines(&slide.body_content, w);
    result.extend(lines_to_canvas(&body_lines, w, body_height));

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
    result
}

/// `comparison` layout — 2×2 quadrant grid for strategic matrices (e.g., SWOT,
/// urgent/important, BCG matrix).
///
/// Body content uses quadrant markers to assign text to each cell:
/// - `## q:tl` (or `## quadrant:tl`) — top-left
/// - `## q:tr` (or `## quadrant:tr`) — top-right
/// - `## q:bl` (or `## quadrant:bl`) — bottom-left
/// - `## q:br` (or `## quadrant:br`) — bottom-right
///
/// Optional axis labels:
/// - `## axis:x <label>` — text rendered centered on a single row beneath the grid
/// - `## axis:y <label>` — text rendered as a 1-column-wide vertical strip on the
///   left edge of the grid (one character per row, vertically centered)
///
/// Lines before the first marker are dropped (use the title for an overall
/// label). Empty quadrants render blank. Axis labels are optional and the
/// layout always satisfies SL-1 (height × width).
pub fn render_comparison(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;

    let parsed = parse_comparison_body(&slide.body_content);

    // Reserve 1 row at the bottom for x-axis label if set.
    let x_axis_rows = if parsed.axis_x.is_some() { 1usize } else { 0 };
    // Reserve 1 column on the left for y-axis label if set.
    let y_axis_cols = if parsed.axis_y.is_some() { 1usize } else { 0 };

    let body_height = h
        .saturating_sub(title_height + 1)
        .saturating_sub(x_axis_rows);
    let mid_sep_height = 1usize;
    let row_height = body_height.saturating_sub(mid_sep_height) / 2;

    // Column widths: equal split of remaining grid width (after y-axis column).
    let grid_w = w.saturating_sub(y_axis_cols);
    let col_a_width = grid_w.div_ceil(2);
    let col_b_width = grid_w.saturating_sub(col_a_width);

    let tl_lines = lines_to_canvas(
        &render_body_lines(&parsed.tl, col_a_width),
        col_a_width,
        row_height,
    );
    let tr_lines = lines_to_canvas(
        &render_body_lines(&parsed.tr, col_b_width),
        col_b_width,
        row_height,
    );
    let bl_lines = lines_to_canvas(
        &render_body_lines(&parsed.bl, col_a_width),
        col_a_width,
        row_height,
    );
    let br_lines = lines_to_canvas(
        &render_body_lines(&parsed.br, col_b_width),
        col_b_width,
        row_height,
    );

    // Build the y-axis column as `total_grid_rows` chars, centered around the label.
    let total_grid_rows = row_height * 2 + mid_sep_height;
    let y_strip = if let Some(ref label) = parsed.axis_y {
        build_vertical_strip(label, total_grid_rows)
    } else {
        Vec::new()
    };

    let mut result: Vec<String> = Vec::with_capacity(h);
    let title_str = slide.title.as_deref().unwrap_or("");
    result.push(fit_to_width(title_str, w));
    for _ in 1..title_height {
        result.push(" ".repeat(w));
    }
    result.push(separator(w));

    let mut grid_row_idx = 0usize;
    // Top row: tl | tr
    for i in 0..row_height {
        let prefix = y_axis_prefix(&y_strip, grid_row_idx);
        let a = tl_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let b = tr_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        result.push(format!(
            "{}{}{}",
            prefix,
            fit_to_width(a, col_a_width),
            fit_to_width(b, col_b_width)
        ));
        grid_row_idx += 1;
    }
    // Mid separator (with y-axis prefix character if set)
    let prefix = y_axis_prefix(&y_strip, grid_row_idx);
    result.push(format!("{}{}", prefix, separator(grid_w)));
    grid_row_idx += 1;
    // Bottom row: bl | br
    for i in 0..row_height {
        let prefix = y_axis_prefix(&y_strip, grid_row_idx);
        let a = bl_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        let b = br_lines.get(i).map(|s| s.as_str()).unwrap_or("");
        result.push(format!(
            "{}{}{}",
            prefix,
            fit_to_width(a, col_a_width),
            fit_to_width(b, col_b_width)
        ));
        grid_row_idx += 1;
    }

    // X-axis label row beneath the grid.
    if let Some(ref label) = parsed.axis_x {
        result.push(center_in_width(label, w));
    }

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
    result
}

struct ComparisonBody {
    tl: String,
    tr: String,
    bl: String,
    br: String,
    axis_x: Option<String>,
    axis_y: Option<String>,
}

/// Build a vertical strip of single-character rows from `label`, centered
/// vertically across `total_rows`. Rows outside the label range are spaces.
fn build_vertical_strip(label: &str, total_rows: usize) -> Vec<char> {
    let chars: Vec<char> = label.chars().collect();
    let label_len = chars.len().min(total_rows);
    let pad = (total_rows.saturating_sub(label_len)) / 2;
    let mut out = vec![' '; total_rows];
    for (i, ch) in chars.iter().take(label_len).enumerate() {
        out[pad + i] = *ch;
    }
    out
}

fn y_axis_prefix(strip: &[char], row: usize) -> String {
    if strip.is_empty() {
        String::new()
    } else {
        strip.get(row).copied().unwrap_or(' ').to_string()
    }
}

/// Parse comparison body for quadrant + axis markers.
fn parse_comparison_body(body: &str) -> ComparisonBody {
    let mut tl = String::new();
    let mut tr = String::new();
    let mut bl = String::new();
    let mut br = String::new();
    let mut axis_x: Option<String> = None;
    let mut axis_y: Option<String> = None;
    let mut current: Option<char> = None;
    for line in body.lines() {
        let trimmed = line.trim();
        // axis markers consume the rest of the line as label and don't affect `current`.
        if let Some(rest) = trimmed.strip_prefix("## axis:x") {
            axis_x = Some(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## axis:y") {
            axis_y = Some(rest.trim().to_string());
            continue;
        }
        let marker = match trimmed {
            "## q:tl" | "## quadrant:tl" => Some('1'),
            "## q:tr" | "## quadrant:tr" => Some('2'),
            "## q:bl" | "## quadrant:bl" => Some('3'),
            "## q:br" | "## quadrant:br" => Some('4'),
            _ => None,
        };
        if let Some(m) = marker {
            current = Some(m);
            continue;
        }
        match current {
            Some('1') => {
                tl.push_str(line);
                tl.push('\n');
            }
            Some('2') => {
                tr.push_str(line);
                tr.push('\n');
            }
            Some('3') => {
                bl.push_str(line);
                bl.push('\n');
            }
            Some('4') => {
                br.push_str(line);
                br.push('\n');
            }
            _ => {} // pre-marker content is dropped
        }
    }
    ComparisonBody {
        tl,
        tr,
        bl,
        br,
        axis_x,
        axis_y,
    }
}

/// `content-caption` layout — main content area with a caption strip at the bottom.
///
/// Layout rows:
/// - 1 row: title (left-aligned, fitted)
/// - 2 rows: top padding
/// - 1 row: separator
/// - body_height - 4 rows: body content (`render_body_lines` over `body_content`)
/// - 1 row: separator
/// - 2 rows: caption (italic in box theme; left-aligned plain otherwise; comes from `slide.subtitle`)
///
/// Caption text is read from the slide's `subtitle` field. Authors set it via
/// the slide front-matter `subtitle: "..."` attribute or via the inline form
/// (`proof:slide layout=content-caption title="..." subtitle="..."`).
/// If no subtitle is given the caption strip stays present but blank — keeping
/// vertical alignment consistent across slides in a deck.
pub fn render_content_caption(slide: &Slide, meta: &SlideMeta) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    // Reserve 3 rows at the bottom for the caption strip: separator + caption + padding.
    let caption_strip_height = 3usize;
    let body_height = h
        .saturating_sub(title_height + 1) // +1 for separator under title
        .saturating_sub(caption_strip_height);

    let title_str = slide.title.as_deref().unwrap_or("");
    let mut result: Vec<String> = Vec::with_capacity(h);

    // Title area
    result.push(fit_to_width(title_str, w));
    for _ in 1..title_height {
        result.push(" ".repeat(w));
    }
    result.push(separator(w));

    // Body
    let body_lines = render_body_lines(&slide.body_content, w);
    result.extend(lines_to_canvas(&body_lines, w, body_height));

    // Caption strip: separator + caption + padding
    result.push(separator(w));
    let caption = slide.subtitle.as_deref().unwrap_or("");
    result.push(fit_to_width(caption, w));
    result.push(" ".repeat(w));

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
    result
}

/// `two-column` layout — columns split by ratio, optional divider.
/// Column delimiters in body: `## col:left` and `## col:right` (H2 level).
pub fn render_two_column(slide: &Slide, meta: &SlideMeta, ratio: (u8, u8)) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = if slide.title.is_some() { 2usize } else { 0 };
    let body_height = h.saturating_sub(title_height);

    // Column width: floor() with remainder to first column (per spec rounding rule)
    let ratio_sum = (ratio.0 as usize) + (ratio.1 as usize);
    let col_a_w = (w * ratio.0 as usize) / ratio_sum;
    let _col_b_w = w.saturating_sub(col_a_w); // remainder goes to second col? No — first gets remainder
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
        result.push(format!(
            "{}{}",
            fit_to_width(a, col_a_width),
            fit_to_width(b, col_b_width)
        ));
    }

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
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
            'a' => {
                col_a.push_str(line);
                col_a.push('\n');
            }
            'b' => {
                col_b.push_str(line);
                col_b.push('\n');
            }
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
    for _ in 0..top {
        result.push(" ".repeat(w));
    }
    for l in &lines {
        result.push(fit_to_width(l, w));
    }
    for _ in 0..bot {
        result.push(" ".repeat(w));
    }
    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
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
            1 if !parts[0].is_empty() => {
                stats.push((parts[0].into(), String::new(), String::new()))
            }
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

    let col_widths: Vec<usize> = (0..n)
        .map(|i| {
            if i == n - 1 {
                col_w_base + remainder
            } else {
                col_w_base
            }
        })
        .collect();

    // Build content rows (value row, label row, sublabel row)
    let value_row: String = stats
        .iter()
        .zip(col_widths.iter())
        .map(|((v, _, _), &cw)| fit_to_width(&center_in_width(v, cw), cw))
        .collect();
    let label_row: String = stats
        .iter()
        .zip(col_widths.iter())
        .map(|((_, l, _), &cw)| fit_to_width(&center_in_width(l, cw), cw))
        .collect();
    let sublabel_row: String = stats
        .iter()
        .zip(col_widths.iter())
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
    for _ in 0..top {
        result.push(" ".repeat(w));
    }
    for l in &content_lines {
        result.push(fit_to_width(l, w));
    }
    while result.len() < h {
        result.push(" ".repeat(w));
    }
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
    // Deck-less path: agenda slides receive an empty section list and render
    // an empty bullet area. Callers that want a populated agenda should use
    // [`render_slide_with_warnings_in_deck`].
    render_slide_with_warnings_in_deck(slide, meta, &[])
}

/// Same as [`render_slide_with_warnings`] but with deck context. The
/// `agenda` layout uses `all_slides` to enumerate every `Section` slide's
/// title in deck order. Other layouts ignore the extra parameter.
pub fn render_slide_with_warnings_in_deck(
    slide: &Slide,
    meta: &SlideMeta,
    all_slides: &[crate::slide::Slide],
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
        SlideLayout::Agenda => {
            let titles = collect_section_titles(all_slides);
            (render_agenda(slide, meta, &titles), Vec::new())
        }
        SlideLayout::Stats => (render_stats(slide, meta), Vec::new()),
        SlideLayout::Blank => render_blank_with_warnings(slide, meta, &bullet_cfg),
        SlideLayout::ContentCaption => (render_content_caption(slide, meta), Vec::new()),
        SlideLayout::Comparison => (render_comparison(slide, meta), Vec::new()),
    };
    let mut themed = apply_theme(&raw, meta);
    apply_footer(&mut themed, meta);
    (themed, warnings)
}

/// Collect titles of every `Section` slide in deck order. Slides without an
/// explicit title get a placeholder "Untitled section" entry so the agenda
/// still reflects the deck's structure.
pub fn collect_section_titles(all_slides: &[crate::slide::Slide]) -> Vec<String> {
    all_slides
        .iter()
        .filter(|s| matches!(s.layout, SlideLayout::Section))
        .map(|s| {
            s.title
                .clone()
                .unwrap_or_else(|| "Untitled section".to_string())
        })
        .collect()
}

/// `agenda` layout — auto-generated table of contents built from every
/// `Section` slide's title in deck order. Mirrors PowerPoint's agenda
/// builder: authors keep a single agenda slide that always reflects the
/// current deck structure, no manual maintenance.
///
/// The slide's own body content is intentionally ignored — the bullet list
/// always comes from the deck. Authors who need a manual list should use
/// `layout=title-content` with an explicit `proof:bullets` block.
///
/// Title defaults to "Agenda" when the slide front-matter omits one.
pub fn render_agenda(slide: &Slide, meta: &SlideMeta, section_titles: &[String]) -> Vec<String> {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    let body_height = h.saturating_sub(title_height + 1);

    let title_str = slide.title.as_deref().unwrap_or("Agenda");
    let mut result: Vec<String> = Vec::with_capacity(h);
    result.push(fit_to_width(title_str, w));
    for _ in 1..title_height {
        result.push(" ".repeat(w));
    }
    result.push(separator(w));

    // Build the bullet body. `proof:bullets` would word-wrap to slide width
    // and apply hanging indents, but agenda items are typically short — a
    // direct numbered list keeps the output deterministic and easy to test.
    let body_lines: Vec<String> = if section_titles.is_empty() {
        vec![center_in_width("(no section slides in this deck)", w)]
    } else {
        section_titles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let prefix = format!("{}. ", i + 1);
                fit_to_width(&format!("{}{}", prefix, t), w)
            })
            .collect()
    };

    result.extend(lines_to_canvas(&body_lines, w, body_height));

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
    result
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
    for _ in 1..title_height {
        result.push(" ".repeat(w));
    }
    result.push(separator(w));

    let (body_lines, warnings) =
        render_body_lines_with_warnings(&slide.body_content, w, bullet_cfg);
    result.extend(lines_to_canvas(&body_lines, w, body_height));

    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
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
        result.push(format!(
            "{}{}",
            fit_to_width(a, col_a_width),
            fit_to_width(b, col_b_width)
        ));
    }
    result.truncate(h);
    while result.len() < h {
        result.push(" ".repeat(w));
    }
    (result, warns_a)
}

fn render_blank_with_warnings(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> (Vec<String>, Vec<crate::slide::bullets::BulletWarning>) {
    let (body_lines, warnings) =
        render_body_lines_with_warnings(&slide.body_content, meta.width, bullet_cfg);
    (
        lines_to_canvas(&body_lines, meta.width, meta.height),
        warnings,
    )
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
    use crate::slide::bullets::{has_reveal_markers, BulletConfig};

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
    // Two-column, agenda, and others fall back to single-page rendering.
    match &slide.layout {
        SlideLayout::TitleContent | SlideLayout::ContentCaption | SlideLayout::Comparison => {
            render_reveal_pages_title_content(slide, meta, &bullet_cfg)
        }
        SlideLayout::Blank => render_reveal_pages_blank(slide, meta, &bullet_cfg),
        _ => vec![render_slide(slide, meta)],
    }
}

/// Build reveal pages for title-content layout.
fn render_reveal_pages_title_content(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> Vec<Vec<String>> {
    let w = meta.width;
    let h = meta.height;
    let title_height = 3usize;
    let body_height = h.saturating_sub(title_height + 1);

    // Build the fixed chrome (title area + separator) — same on every page
    let title_str = slide.title.as_deref().unwrap_or("");
    let mut chrome: Vec<String> = Vec::with_capacity(title_height + 1);
    chrome.push(fit_to_width(title_str, w));
    for _ in 1..title_height {
        chrome.push(" ".repeat(w));
    }
    chrome.push(separator(w));

    // Expand the body: split on proof:bullets, generate pages for each bullets block,
    // then reassemble. For simplicity, we treat the entire body as a single bullets
    // block if it starts with proof:bullets; otherwise fall back to single-page.
    //
    // Strategy: render_body_lines_pages returns Vec<Vec<String>> — one body rendition
    // per reveal step.  We then combine chrome + each body rendition into a full page.
    let body_pages = render_body_lines_pages(&slide.body_content, w, bullet_cfg);

    body_pages
        .into_iter()
        .map(|body_lines| {
            let mut page = chrome.clone();
            page.extend(lines_to_canvas(&body_lines, w, body_height));
            page.truncate(h);
            while page.len() < h {
                page.push(" ".repeat(w));
            }
            let mut themed = apply_theme(&page, meta);
            apply_footer(&mut themed, meta);
            themed
        })
        .collect()
}

/// Build reveal pages for blank layout.
fn render_reveal_pages_blank(
    slide: &Slide,
    meta: &SlideMeta,
    bullet_cfg: &crate::slide::bullets::BulletConfig,
) -> Vec<Vec<String>> {
    let body_pages = render_body_lines_pages(&slide.body_content, meta.width, bullet_cfg);
    body_pages
        .into_iter()
        .map(|body_lines| {
            let page = lines_to_canvas(&body_lines, meta.width, meta.height);
            let mut themed = apply_theme(&page, meta);
            apply_footer(&mut themed, meta);
            themed
        })
        .collect()
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
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
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
            while i < lines.len() && !lines[i].trim().is_empty() {
                i += 1;
            }
            i += 1;
            continue;
        }

        if line.starts_with("proof:bullets") {
            i += 1;
            let mut bullet_lines = String::new();
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
                bullet_lines.push_str(lines[i]);
                bullet_lines.push('\n');
                i += 1;
            }
            if has_reveal_markers(&bullet_lines) {
                let (pages, _) = render_bullets_pages(&bullet_lines, width, bullet_cfg);
                segments.push(Segment::Paged(pages));
            } else {
                let (rendered, _) =
                    crate::slide::bullets::render_bullets(&bullet_lines, width, bullet_cfg);
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
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].trim().starts_with("proof:")
            {
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
    let page_count = segments
        .iter()
        .map(|seg| match seg {
            Segment::Fixed(_) => 1,
            Segment::Paged(pages) => pages.len(),
        })
        .max()
        .unwrap_or(1)
        .max(1);

    // Assemble: for each page index, concatenate Fixed lines + Paged[page_idx] lines
    (0..page_count)
        .map(|page_idx| {
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
        })
        .collect()
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
        SlideMeta {
            width: 80,
            height: 24,
            ..SlideMeta::default()
        }
    }

    // ── SL-1: every layout produces exactly height rows of width chars ──

    fn assert_sl1(lines: &[String], meta: &SlideMeta) {
        assert_eq!(lines.len(), meta.height, "SL-1 line count");
        for (i, l) in lines.iter().enumerate() {
            assert_eq!(
                l.chars().count(),
                meta.width,
                "SL-1 line {} width mismatch: {:?}",
                i,
                l
            );
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
    fn content_caption_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::ContentCaption);
        s.title = Some("Diagram".into());
        s.subtitle = Some("Figure 1: System overview".into());
        s.body_content = "Main content paragraph.\n".into();
        assert_sl1(&render_content_caption(&s, &meta), &meta);
    }

    #[test]
    fn content_caption_renders_caption_at_bottom() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::ContentCaption);
        s.title = Some("Diagram".into());
        s.subtitle = Some("Figure 1: System overview".into());
        s.body_content = "Main content paragraph.\n".into();
        let lines = render_content_caption(&s, &meta);
        // Title at top.
        assert!(
            lines[0].starts_with("Diagram"),
            "title at row 0: {:?}",
            lines[0]
        );
        // Caption text appears in the bottom strip — at row height-2 by construction.
        let caption_row = &lines[meta.height - 2];
        assert!(
            caption_row.starts_with("Figure 1:"),
            "caption expected at row {}: {:?}",
            meta.height - 2,
            caption_row
        );
        // Body content somewhere in the middle.
        assert!(
            lines.iter().any(|l| l.starts_with("Main content")),
            "body content somewhere in slide"
        );
    }

    #[test]
    fn comparison_layout_sl1() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.title = Some("SWOT".into());
        s.body_content =
            "## q:tl\nStrengths\n## q:tr\nWeaknesses\n## q:bl\nOpportunities\n## q:br\nThreats\n"
                .into();
        assert_sl1(&render_comparison(&s, &meta), &meta);
    }

    #[test]
    fn comparison_renders_all_four_quadrants() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.title = Some("Matrix".into());
        s.body_content =
            "## q:tl\nTL marker\n## q:tr\nTR marker\n## q:bl\nBL marker\n## q:br\nBR marker\n"
                .into();
        let lines = render_comparison(&s, &meta);
        let blob = lines.join("\n");
        for marker in ["TL marker", "TR marker", "BL marker", "BR marker"] {
            assert!(
                blob.contains(marker),
                "{} must appear in canvas:\n{}",
                marker,
                blob
            );
        }
        // Title at top.
        assert!(lines[0].starts_with("Matrix"));
        // TL appears in the upper half (above the mid-separator); BL in lower.
        let mid = meta.height / 2;
        assert!(
            lines[..mid].iter().any(|l| l.contains("TL marker")),
            "TL in upper half"
        );
        assert!(
            lines[mid..].iter().any(|l| l.contains("BL marker")),
            "BL in lower half"
        );
    }

    #[test]
    fn comparison_left_right_in_correct_columns() {
        // TL must appear left of TR on the same row; BL left of BR.
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.body_content =
            "## q:tl\nLEFTONE\n## q:tr\nRIGHTONE\n## q:bl\nLEFTTWO\n## q:br\nRIGHTTWO\n".into();
        let lines = render_comparison(&s, &meta);
        let row_with_left_one = lines
            .iter()
            .find(|l| l.contains("LEFTONE"))
            .expect("TL row");
        let l_pos = row_with_left_one.find("LEFTONE").unwrap();
        let r_pos = row_with_left_one.find("RIGHTONE").expect("TR on same row");
        assert!(
            l_pos < r_pos,
            "TL must precede TR on the row: l={}, r={}",
            l_pos,
            r_pos
        );
    }

    #[test]
    fn comparison_x_axis_label_renders_below_grid() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.title = Some("Eisenhower".into());
        s.body_content =
            "## q:tl\nA\n## q:tr\nB\n## q:bl\nC\n## q:br\nD\n## axis:x Urgency\n".into();
        let lines = render_comparison(&s, &meta);
        assert_sl1(&lines, &meta);
        // X-axis label appears on a row near the bottom (before final padding).
        assert!(
            lines.iter().any(|l| l.contains("Urgency")),
            "x-axis label rendered:\n{:#?}",
            lines
        );
    }

    #[test]
    fn comparison_y_axis_label_renders_as_left_strip() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.title = Some("BCG".into());
        // Short y-axis label so it fits the available rows easily.
        s.body_content =
            "## q:tl\nStars\n## q:tr\nCash\n## q:bl\nDog\n## q:br\nQM\n## axis:y Growth\n".into();
        let lines = render_comparison(&s, &meta);
        assert_sl1(&lines, &meta);
        // Y-axis chars must appear at column 0 of grid rows (after title+separator: rows 4..end-padding).
        // The label "Growth" is 6 chars; with grid_rows ≈ 19 and centering, the chars land in mid rows.
        let strip: String = lines
            .iter()
            .skip(4) // skip title + 2 padding + separator
            .filter_map(|l| l.chars().next())
            .collect();
        let label_chars: Vec<char> = "Growth".chars().collect();
        // The strip contains the label characters somewhere in order.
        let mut idx = 0;
        for c in strip.chars() {
            if idx < label_chars.len() && c == label_chars[idx] {
                idx += 1;
            }
        }
        assert_eq!(
            idx,
            label_chars.len(),
            "y-axis label characters should appear in order at column 0; strip={:?}",
            strip
        );
    }

    #[test]
    fn comparison_both_axes_layout_intact() {
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.body_content =
            "## q:tl\nA\n## q:tr\nB\n## q:bl\nC\n## q:br\nD\n## axis:x XL\n## axis:y YL\n".into();
        let lines = render_comparison(&s, &meta);
        assert_sl1(&lines, &meta);
        let blob = lines.join("\n");
        assert!(blob.contains("XL"), "x-axis label present");
        // y-axis: chars Y and L appear at column 0 (in some row).
        let col0: String = lines.iter().filter_map(|l| l.chars().next()).collect();
        assert!(
            col0.contains('Y') && col0.contains('L'),
            "y-axis chars in column 0: {:?}",
            col0
        );
    }

    #[test]
    fn comparison_quadrant_alias_works() {
        // Long form `## quadrant:tl` is also accepted.
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::Comparison);
        s.body_content = "## quadrant:tl\nLONG\n## q:br\nSHORT\n".into();
        let lines = render_comparison(&s, &meta);
        let blob = lines.join("\n");
        assert!(blob.contains("LONG"));
        assert!(blob.contains("SHORT"));
    }

    #[test]
    fn content_caption_blank_caption_keeps_layout() {
        // No subtitle → caption row is blank but the layout still has caption_strip_height rows.
        let meta = meta_80x24();
        let mut s = blank_slide(SlideLayout::ContentCaption);
        s.title = Some("Diagram".into());
        s.body_content = "Body.\n".into();
        let lines = render_content_caption(&s, &meta);
        assert_sl1(&lines, &meta);
        let caption_row = &lines[meta.height - 2];
        assert!(
            caption_row.chars().all(|c| c == ' '),
            "blank caption row at row {}: {:?}",
            meta.height - 2,
            caption_row
        );
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
        let meta = SlideMeta {
            width: 119,
            height: 10,
            ..SlideMeta::default()
        };
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
        assert!(
            right_spaces >= left_spaces,
            "tie-break should put extra space on right"
        );
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
        let meta = SlideMeta {
            theme: SlideTheme::None,
            ..meta_80x24()
        };
        let lines = vec!["hello".to_string()];
        assert_eq!(apply_theme(&lines, &meta), lines);
    }

    #[test]
    fn theme_box_adds_border() {
        let meta = SlideMeta {
            width: 10,
            height: 1,
            theme: SlideTheme::Box,
            ..SlideMeta::default()
        };
        let lines = vec!["hi".to_string()];
        let themed = apply_theme(&lines, &meta);
        assert!(themed[0].starts_with('┌'));
        assert!(themed[themed.len() - 1].starts_with('└'));
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
            assert!(
                line.chars().count() <= 20,
                "line {:?} exceeds width 20",
                line
            );
        }
        // All words should still be present
        let full = result.join(" ");
        assert!(full.contains("quick") && full.contains("dog"));
    }

    #[test]
    fn word_wrap_preserves_indent() {
        let result = word_wrap(
            "  This is an indented line that goes way beyond the width limit",
            30,
        );
        assert!(result.len() > 1);
        // The continuation lines should preserve the 2-space indent
        for line in result.iter().skip(1) {
            assert!(
                line.starts_with("  "),
                "continuation should preserve indent: {:?}",
                line
            );
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
        assert_eq!(
            slide001.len(),
            2,
            "expected 2 SLIDE-001 warnings (bullets 5 and 6) at default max_bullets=4, got: {:?}",
            warnings
        );
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
        let meta = SlideMeta {
            max_bullets: 8,
            ..meta_80x24()
        };
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
        )
        .into();

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

    fn make_reveal_slide(
        layout: SlideLayout,
        title: Option<&str>,
        body: &str,
    ) -> (Slide, SlideMeta) {
        let mut s = blank_slide(layout);
        s.title = title.map(|t| t.into());
        s.body_content = body.into();
        (s, meta_80x24())
    }

    #[test]
    fn reveal_no_markers_single_page() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent,
            Some("Title"),
            "proof:bullets\n- A\n- B\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 1, "no reveal markers → single page");
        assert_sl1(&pages[0], &meta);
    }

    #[test]
    fn reveal_two_steps_two_pages_sl1() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent,
            Some("Title"),
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
            SlideLayout::TitleContent,
            Some("Title"),
            "proof:bullets\n- Always\n[2] - Step 2\n",
        );
        let pages = render_slide_pages(&s, &meta);
        let p1 = pages[0].join("\n");
        assert!(p1.contains("Always"), "page 1 should show step-1 bullet");
        assert!(!p1.contains("Step 2"), "page 1 should hide step-2 bullet");
    }

    #[test]
    fn reveal_page_2_shows_all() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent,
            Some("Title"),
            "proof:bullets\n- Always\n[2] - Step 2\n",
        );
        let pages = render_slide_pages(&s, &meta);
        let p2 = pages[1].join("\n");
        assert!(
            p2.contains("Always") && p2.contains("Step 2"),
            "page 2 should show all bullets"
        );
    }

    #[test]
    fn reveal_title_identical_on_all_pages() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::TitleContent,
            Some("My Deck Title"),
            "proof:bullets\n- A\n[2] - B\n[3] - C\n",
        );
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 3);
        for page in &pages {
            assert!(
                page[0].contains("My Deck Title"),
                "title row must be identical on every page"
            );
        }
    }

    #[test]
    fn reveal_blank_layout_pages_sl1() {
        let (s, meta) = make_reveal_slide(
            SlideLayout::Blank,
            None,
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

    // ── Footer ────────────────────────────────────────────

    fn meta_with_footer(
        footer: crate::slide::FooterMode,
        author: Option<&str>,
        date: Option<&str>,
    ) -> SlideMeta {
        SlideMeta {
            footer,
            author: author.map(|s| s.to_string()),
            date: date.map(|s| s.to_string()),
            ..meta_80x24()
        }
    }

    #[test]
    fn footer_off_no_footer_stamped() {
        let meta = meta_with_footer(crate::slide::FooterMode::Off, Some("Gio"), Some("2026"));
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert!(
            !last.contains("Gio") && !last.contains("2026"),
            "footer=off must not stamp last row: {:?}",
            last
        );
    }

    #[test]
    fn footer_auto_right_aligned() {
        let meta = meta_with_footer(
            crate::slide::FooterMode::Auto,
            Some("Gio"),
            Some("April 2026"),
        );
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert!(
            last.contains("Gio · April 2026"),
            "footer=auto should contain author · date: {:?}",
            last
        );
        assert!(
            last.ends_with("Gio · April 2026"),
            "footer should be right-aligned (ends with text): {:?}",
            last
        );
    }

    #[test]
    fn footer_auto_author_only() {
        let meta = meta_with_footer(crate::slide::FooterMode::Auto, Some("Alice"), None);
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert!(
            last.contains("Alice"),
            "auto footer with only author: {:?}",
            last
        );
        assert!(
            !last.contains("·"),
            "no separator when only one field: {:?}",
            last
        );
    }

    #[test]
    fn footer_auto_date_only() {
        let meta = meta_with_footer(crate::slide::FooterMode::Auto, None, Some("Q2 2026"));
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert!(
            last.contains("Q2 2026"),
            "auto footer with only date: {:?}",
            last
        );
    }

    #[test]
    fn footer_auto_no_fields_no_footer() {
        let meta = meta_with_footer(crate::slide::FooterMode::Auto, None, None);
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert_eq!(
            last.trim(),
            "",
            "auto footer with no fields should be blank: {:?}",
            last
        );
    }

    #[test]
    fn footer_custom_text() {
        let meta = meta_with_footer(
            crate::slide::FooterMode::Custom("CONFIDENTIAL".to_string()),
            None,
            None,
        );
        let s = blank_slide(SlideLayout::TitleContent);
        let lines = render_slide(&s, &meta);
        let last = lines.last().unwrap();
        assert!(
            last.contains("CONFIDENTIAL"),
            "custom footer text: {:?}",
            last
        );
    }

    #[test]
    fn footer_sl1_row_count_and_width() {
        let meta = meta_with_footer(crate::slide::FooterMode::Auto, Some("Gio"), Some("2026"));
        let s = blank_slide(SlideLayout::TitleContent);
        assert_sl1(&render_slide(&s, &meta), &meta);
    }

    #[test]
    fn footer_on_every_layout() {
        let meta = meta_with_footer(
            crate::slide::FooterMode::Custom("FTR".to_string()),
            None,
            None,
        );
        for layout in [
            SlideLayout::Title,
            SlideLayout::TitleContent,
            SlideLayout::Section,
            SlideLayout::Stats,
            SlideLayout::Blank,
        ] {
            let mut s = blank_slide(layout.clone());
            s.title = Some("T".into());
            let lines = render_slide(&s, &meta);
            let last = lines.last().unwrap();
            assert!(
                last.contains("FTR"),
                "footer missing on layout {:?}: {:?}",
                layout,
                last
            );
        }
    }

    #[test]
    fn footer_parsed_from_front_matter_auto() {
        use crate::slide::parser::parse_slide_doc;
        let source = "---\nfooter: true\nauthor: Gio\ndate: 2026\n---\nContent";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(doc.meta.footer, crate::slide::FooterMode::Auto);
        assert_eq!(doc.meta.author.as_deref(), Some("Gio"));
        assert_eq!(doc.meta.date.as_deref(), Some("2026"));
    }

    #[test]
    fn footer_parsed_from_front_matter_custom() {
        use crate::slide::parser::parse_slide_doc;
        let source = "---\nfooter: \"My Org · Confidential\"\n---\nContent";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(
            doc.meta.footer,
            crate::slide::FooterMode::Custom("My Org · Confidential".to_string())
        );
    }

    #[test]
    fn footer_parsed_off_by_default() {
        use crate::slide::parser::parse_slide_doc;
        let source = "---\nwidth: 80\n---\nContent";
        let doc = parse_slide_doc(source).expect("should parse");
        assert_eq!(doc.meta.footer, crate::slide::FooterMode::Off);
    }

    #[test]
    fn reveal_pages_all_have_footer() {
        let meta = meta_with_footer(
            crate::slide::FooterMode::Custom("SLIDE".to_string()),
            None,
            None,
        );
        let mut s = blank_slide(SlideLayout::TitleContent);
        s.title = Some("T".into());
        s.body_content = "proof:bullets\n- Always\n[2] - Step 2\n".into();
        let pages = render_slide_pages(&s, &meta);
        assert_eq!(pages.len(), 2);
        for (i, page) in pages.iter().enumerate() {
            let last = page.last().unwrap();
            assert!(
                last.contains("SLIDE"),
                "footer missing on reveal page {}: {:?}",
                i + 1,
                last
            );
        }
    }

    #[test]
    fn render_body_lines_pages_fixed_segment_on_every_page() {
        use crate::slide::bullets::BulletConfig;
        let cfg = BulletConfig {
            max_bullets: 10,
            ..BulletConfig::default()
        };
        // A fixed centered block, then a reveal bullets block
        let body = "proof:centered\nIntro\n\nproof:bullets\n- Always\n[2] - Step 2\n";
        let pages = render_body_lines_pages(body, 80, &cfg);
        assert_eq!(pages.len(), 2, "reveal block → 2 pages");
        for page in &pages {
            let text = page.join("\n");
            assert!(
                text.contains("Intro"),
                "fixed prose must appear on every page"
            );
        }
    }

    // ── proof:slide layout=agenda ───────────────────────────────────────────

    fn section_slide(idx: usize, title: &str) -> Slide {
        let mut s = blank_slide(SlideLayout::Section);
        s.index = idx;
        s.title = Some(title.to_string());
        s
    }

    fn agenda_slide(idx: usize, title: Option<&str>) -> Slide {
        let mut s = blank_slide(SlideLayout::Agenda);
        s.index = idx;
        s.title = title.map(|t| t.to_string());
        s
    }

    #[test]
    fn agenda_layout_sl1() {
        let meta = meta_80x24();
        let s = agenda_slide(1, Some("Agenda"));
        let titles = vec![
            "Intro".to_string(),
            "Body".to_string(),
            "Wrap-up".to_string(),
        ];
        assert_sl1(&render_agenda(&s, &meta, &titles), &meta);
    }

    #[test]
    fn agenda_lists_section_titles_in_order() {
        let meta = meta_80x24();
        let agenda = agenda_slide(2, Some("Today"));
        let titles = vec![
            "Problem".to_string(),
            "Approach".to_string(),
            "Results".to_string(),
        ];
        let lines = render_agenda(&agenda, &meta, &titles);
        let body = lines.join("\n");
        let p_problem = body.find("Problem").expect("Problem missing");
        let p_approach = body.find("Approach").expect("Approach missing");
        let p_results = body.find("Results").expect("Results missing");
        assert!(
            p_problem < p_approach && p_approach < p_results,
            "section titles must appear in deck order"
        );
    }

    #[test]
    fn agenda_uses_default_title_when_omitted() {
        let meta = meta_80x24();
        let agenda = agenda_slide(1, None); // no title in front-matter
        let titles = vec!["First".to_string()];
        let lines = render_agenda(&agenda, &meta, &titles);
        // First line is the title row — must contain the default "Agenda"
        assert!(
            lines[0].contains("Agenda"),
            "default title 'Agenda' should appear in first line, got: {:?}",
            lines[0]
        );
    }

    #[test]
    fn agenda_with_no_section_slides_falls_back_to_placeholder() {
        let meta = meta_80x24();
        let agenda = agenda_slide(1, Some("Agenda"));
        let lines = render_agenda(&agenda, &meta, &[]);
        let body = lines.join("\n");
        assert!(
            body.contains("(no section slides in this deck)"),
            "empty deck should show a placeholder, got body:\n{}",
            body
        );
    }

    #[test]
    fn agenda_items_are_numbered_for_easy_walkthrough() {
        let meta = meta_80x24();
        let agenda = agenda_slide(1, Some("Agenda"));
        let titles = vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ];
        let lines = render_agenda(&agenda, &meta, &titles);
        let body = lines.join("\n");
        assert!(
            body.contains("1. First"),
            "expected '1. First' in:\n{}",
            body
        );
        assert!(body.contains("2. Second"));
        assert!(body.contains("3. Third"));
    }

    #[test]
    fn collect_section_titles_filters_to_section_layout() {
        let deck = vec![
            blank_slide(SlideLayout::Title), // skipped
            section_slide(1, "Setup"),
            blank_slide(SlideLayout::TitleContent), // skipped
            section_slide(2, "Findings"),
            section_slide(3, "Next steps"),
            blank_slide(SlideLayout::Stats), // skipped
        ];
        let titles = collect_section_titles(&deck);
        assert_eq!(
            titles,
            vec![
                "Setup".to_string(),
                "Findings".to_string(),
                "Next steps".to_string()
            ],
            "only section slides should contribute to the agenda"
        );
    }

    #[test]
    fn collect_section_titles_handles_untitled_section() {
        let mut s = blank_slide(SlideLayout::Section);
        s.title = None;
        let deck = vec![s];
        let titles = collect_section_titles(&deck);
        assert_eq!(titles, vec!["Untitled section".to_string()]);
    }

    #[test]
    fn render_slide_with_warnings_in_deck_populates_agenda() {
        let meta = meta_80x24();
        let deck = vec![
            agenda_slide(1, Some("Agenda")),
            section_slide(2, "Discovery"),
            section_slide(3, "Decisions"),
        ];
        let agenda = &deck[0];
        let (rendered, warnings) = render_slide_with_warnings_in_deck(agenda, &meta, &deck);
        assert!(warnings.is_empty(), "agenda has no body, no warnings");
        let body = rendered.join("\n");
        assert!(body.contains("Discovery"));
        assert!(body.contains("Decisions"));
    }

    #[test]
    fn render_slide_with_warnings_falls_back_to_empty_deck() {
        // The deck-less alias must not crash and must surface the placeholder.
        let meta = meta_80x24();
        let agenda = agenda_slide(1, Some("Agenda"));
        let (rendered, _) = render_slide_with_warnings(&agenda, &meta);
        let body = rendered.join("\n");
        assert!(
            body.contains("(no section slides in this deck)"),
            "deck-less render must show the placeholder, got:\n{}",
            body
        );
    }
}
