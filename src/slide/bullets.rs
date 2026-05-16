/// proof:bullets renderer — hierarchical bullet list with configurable chars.
///
/// Bullet chars by level (configurable via slide front-matter):
///   Level 1: ● (default), Level 2: ◦, Level 3: ▸, Level 4+: –
///
/// SLIDE-001: max_bullets exceeded per slide — advisory; default 4 bullets
///            (the "30-second rule": more than 4 bullets per slide is hard to read aloud
///            in 30 seconds, the typical pacing for a presentation slide).
/// SLIDE-007: bullet depth exceeds max_depth (advisory)
///
/// ## Reveal syntax
///
/// Bullets prefixed with `[N]` (where N ≥ 1) are assigned to reveal step N.
/// Unmarked bullets implicitly belong to step 1.
///
///   proof:bullets
///   - Always visible
///   [2] - Appears on step 2
///   [3] - Appears on step 3
///
/// `render_bullets_pages` returns one rendered page per step.  Each page shows
/// all bullets whose step ≤ current step (cumulative reveal).  Steps are
/// compacted: if only steps 1 and 3 are used the output is still 2 pages
/// (step 1 content, then step 1+3 content).  Bullets with no `[N]` prefix
/// are treated as step 1 (always visible from page 1 onward).

#[derive(Debug, Clone)]
pub struct BulletConfig {
    pub level_chars: [char; 4], // chars for levels 1, 2, 3, 4+
    pub indent_width: usize,    // spaces per level (default: 2)
    pub max_bullets: usize,     // SLIDE-001 threshold (default: 4 — 30-second rule)
    pub max_depth: usize,       // SLIDE-007 threshold (default: 4)
}

impl Default for BulletConfig {
    fn default() -> Self {
        BulletConfig {
            level_chars: ['●', '◦', '▸', '–'],
            indent_width: 2,
            max_bullets: 4,
            max_depth: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BulletWarning {
    pub code: &'static str,
    pub message: String,
}

/// Render a bullet list from body text.
/// Input: lines starting with `- ` for level 1, `  - ` for level 2 (2 spaces), etc.
/// Indented lines without a `-`/`*` marker are treated as continuation paragraphs
/// belonging to the most recent bullet — rendered with no glyph, indented to the
/// parent bullet's content column.
/// Returns (rendered_lines, warnings).
pub fn render_bullets(
    text: &str,
    width: usize,
    config: &BulletConfig,
) -> (Vec<String>, Vec<BulletWarning>) {
    let mut lines: Vec<String> = Vec::new();
    let mut warnings: Vec<BulletWarning> = Vec::new();
    let mut bullet_count = 0usize;
    // Content column of the most recent bullet — used to align continuation paragraphs.
    let mut last_bullet_content_col: Option<usize> = None;

    for raw_line in text.lines() {
        if raw_line.trim().is_empty() {
            lines.push(String::new());
            continue;
        }

        let trimmed = raw_line.trim();
        let leading = raw_line.len() - raw_line.trim_start().len();
        let is_bullet = trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed == "-"
            || trimmed == "*";

        // Continuation paragraph: indented prose under a bullet (no `-`/`*` marker).
        // Requires a preceding bullet — without one, falls through to bullet handling.
        if !is_bullet && leading >= config.indent_width {
            if let Some(content_col) = last_bullet_content_col {
                lines.extend(wrap_continuation(trimmed, content_col, width));
                continue;
            }
        }

        // Detect indent level: count leading spaces / indent_width
        let level = (leading / config.indent_width).min(3) + 1; // 1-indexed, max 4

        if level > config.max_depth {
            warnings.push(BulletWarning {
                code: "SLIDE-007",
                message: format!(
                    "bullet depth {} exceeds max_depth {}",
                    level, config.max_depth
                ),
            });
        }

        // Strip leading - or * bullet marker if present
        let content = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            &trimmed[2..]
        } else if trimmed.starts_with('-') || trimmed.starts_with('*') {
            &trimmed[1..]
        } else {
            trimmed
        };

        bullet_count += 1;
        if bullet_count > config.max_bullets {
            warnings.push(BulletWarning {
                code: "SLIDE-001",
                message: format!(
                    "Slide has {} bullets — reduce to {} or fewer (30-second rule)",
                    bullet_count, config.max_bullets
                ),
            });
        }

        let bullet_char = config.level_chars[level.min(4) - 1];
        let indent = " ".repeat((level - 1) * config.indent_width);
        let prefix = format!("{}{} ", indent, bullet_char);
        let prefix_w = prefix.chars().count();
        let first_line = format!("{}{}", prefix, content);

        last_bullet_content_col = Some(prefix_w);

        // Word-wrap with hanging indent — continuation lines align past the bullet char
        let wrapped = word_wrap_hanging(&first_line, prefix_w, width);
        lines.extend(wrapped);
    }

    (lines, warnings)
}

/// Word-wrap a continuation paragraph: every output line is indented to `indent`
/// columns (the parent bullet's content column) and no glyph is emitted.
fn wrap_continuation(text: &str, indent: usize, width: usize) -> Vec<String> {
    use crate::layout::visual_width;
    let pad = " ".repeat(indent);
    let body_width = width.saturating_sub(indent).max(1);

    let mut result = Vec::new();
    let mut current = String::new();
    let mut current_w = 0usize;

    for word in text.split_whitespace() {
        let ww = visual_width(word);
        if current.is_empty() {
            current.push_str(word);
            current_w = ww;
        } else if current_w + 1 + ww <= body_width {
            current.push(' ');
            current.push_str(word);
            current_w += 1 + ww;
        } else {
            result.push(format!("{}{}", pad, current));
            current = word.to_string();
            current_w = ww;
        }
    }
    if !current.is_empty() {
        result.push(format!("{}{}", pad, current));
    }
    if result.is_empty() {
        result.push(pad);
    }
    result
}

/// Word-wrap with hanging indent.
/// `hanging` = number of columns to indent continuation lines (past the bullet prefix).
fn word_wrap_hanging(s: &str, hanging: usize, width: usize) -> Vec<String> {
    use crate::layout::visual_width;
    if width == 0 || visual_width(s) <= width {
        return vec![s.to_string()];
    }
    let continuation_indent = " ".repeat(hanging);
    let cont_width = width.saturating_sub(hanging).max(1);

    let mut result = Vec::new();
    let mut first = true;
    let mut current = String::new();
    let mut current_w = 0usize;

    // Split the string: first word includes the prefix (already formatted)
    // We treat the whole first line as tokens
    for word in s.split(' ') {
        let word_w = visual_width(word);
        let max_w = if first && result.is_empty() {
            width
        } else {
            cont_width
        };
        if current.is_empty() {
            current.push_str(word);
            current_w = word_w;
        } else if current_w + 1 + word_w <= max_w {
            current.push(' ');
            current.push_str(word);
            current_w += 1 + word_w;
        } else {
            // Flush
            if first {
                result.push(current.clone());
                first = false;
            } else {
                result.push(format!("{}{}", continuation_indent, current));
            }
            current = word.to_string();
            current_w = word_w;
        }
    }
    if !current.is_empty() {
        if first {
            result.push(current);
        } else {
            result.push(format!("{}{}", continuation_indent, current));
        }
    }
    if result.is_empty() {
        result.push(s.to_string());
    }
    result
}

// ─────────────────────────────────────────────────────────
// Reveal: [N] prefix parsing and page generation
// ─────────────────────────────────────────────────────────

/// Parse an optional `[N]` reveal-step prefix from a bullet line.
///
/// Returns `(step, rest)` where `step` is the parsed step number (≥ 1) and
/// `rest` is the line with the `[N]` prefix stripped but with leading indent
/// preserved so level detection still works.  Lines without a `[N]` prefix
/// return `(1, original_line)`.
pub fn parse_reveal_step(line: &str) -> (usize, &str) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('[') {
        return (1, line);
    }
    if let Some(close) = trimmed.find(']') {
        let inner = &trimmed[1..close];
        if let Ok(n) = inner.parse::<usize>() {
            if n >= 1 {
                let indent_len = line.len() - trimmed.len();
                // Bytes consumed by "[N]" and any single trailing space
                let tag_len = close + 1; // length of "[N]"
                let after_tag = &trimmed[tag_len..];
                let stripped = after_tag.trim_start_matches(' ');
                let space_len = after_tag.len() - stripped.len();
                // Point back into `line`: indent + "[N]" + spaces
                let rest_start = indent_len + tag_len + space_len;
                return (n, &line[rest_start.min(line.len())..]);
            }
        }
    }
    (1, line)
}

/// Returns `true` if any non-blank line in `text` has a `[N]` reveal-step
/// prefix with N ≥ 2.  Used to decide whether multi-page rendering is needed.
pub fn has_reveal_markers(text: &str) -> bool {
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .any(|line| parse_reveal_step(line).0 >= 2)
}

/// Render a bullet list with progressive reveal.
///
/// Returns `(pages, warnings)` where `pages[i]` is the rendered bullet lines
/// visible at reveal step `i + 1` (cumulative — each page shows all bullets
/// with assigned step ≤ current step).  Steps are compacted to only the
/// distinct step values that appear in `text`, so gaps in numbering don't
/// produce empty pages.
///
/// If no `[N]` markers (with N ≥ 2) are present, returns a single page
/// identical to `render_bullets`.
pub fn render_bullets_pages(
    text: &str,
    width: usize,
    config: &BulletConfig,
) -> (Vec<Vec<String>>, Vec<BulletWarning>) {
    if !has_reveal_markers(text) {
        let (rendered, warnings) = render_bullets(text, width, config);
        return (vec![rendered], warnings);
    }

    // Collect (step, raw_line): blank lines get step 0 (always included for spacing)
    let tagged: Vec<(usize, &str)> = text
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                (0, line)
            } else {
                parse_reveal_step(line)
            }
        })
        .collect();

    // Sorted distinct steps (≥ 1)
    let mut steps: Vec<usize> = tagged
        .iter()
        .filter_map(|&(s, _)| if s >= 1 { Some(s) } else { None })
        .collect();
    steps.sort_unstable();
    steps.dedup();

    let mut all_warnings: Vec<BulletWarning> = Vec::new();
    let mut pages: Vec<Vec<String>> = Vec::with_capacity(steps.len());

    for &current_step in &steps {
        // Include blank lines and lines whose step ≤ current_step
        let mut filtered = String::new();
        for &(step, line) in &tagged {
            if step == 0 || step <= current_step {
                filtered.push_str(line);
                filtered.push('\n');
            }
        }
        let (rendered, warnings) = render_bullets(&filtered, width, config);
        all_warnings.extend(warnings);
        pages.push(rendered);
    }

    (pages, all_warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_1_uses_filled_circle() {
        let cfg = BulletConfig::default();
        let (lines, _) = render_bullets("- First point", 80, &cfg);
        assert_eq!(lines[0], "● First point");
    }

    #[test]
    fn level_2_uses_open_circle() {
        let cfg = BulletConfig::default();
        let (lines, _) = render_bullets("- Top\n  - Nested", 80, &cfg);
        assert!(
            lines[1].contains('◦'),
            "level 2 should use ◦: {:?}",
            lines[1]
        );
    }

    #[test]
    fn level_3_uses_right_arrow() {
        let cfg = BulletConfig::default();
        let (lines, _) = render_bullets("- Top\n  - Mid\n    - Deep", 80, &cfg);
        assert!(
            lines[2].contains('▸'),
            "level 3 should use ▸: {:?}",
            lines[2]
        );
    }

    #[test]
    fn max_bullets_warning() {
        let cfg = BulletConfig {
            max_bullets: 2,
            ..Default::default()
        };
        let text = "- A\n- B\n- C";
        let (_, warns) = render_bullets(text, 80, &cfg);
        assert!(warns.iter().any(|w| w.code == "SLIDE-001"));
    }

    #[test]
    fn max_depth_warning() {
        let cfg = BulletConfig {
            max_depth: 2,
            ..Default::default()
        };
        let text = "- A\n  - B\n    - C"; // level 3 > max_depth 2
        let (_, warns) = render_bullets(text, 80, &cfg);
        assert!(warns.iter().any(|w| w.code == "SLIDE-007"));
    }

    #[test]
    fn clips_to_width() {
        let cfg = BulletConfig::default();
        let long = "- ".to_string() + &"x".repeat(100);
        let (lines, _) = render_bullets(&long, 20, &cfg);
        assert!(lines[0].chars().count() <= 20);
    }

    #[test]
    fn bullet_long_text_wraps_not_clips() {
        let cfg = BulletConfig::default();
        let text = "- This is a very long bullet point that exceeds the slide width and should wrap onto the next line";
        let (lines, _) = render_bullets(text, 40, &cfg);
        // Should produce more than one line
        assert!(lines.len() > 1, "long bullet should wrap, not clip");
        // First line should start with the bullet
        assert!(lines[0].contains('●'), "first line should have bullet char");
        // No line should exceed width
        for line in &lines {
            assert!(
                line.chars().count() <= 40,
                "line {:?} exceeds width 40",
                line
            );
        }
    }

    #[test]
    fn bullet_continuation_has_hanging_indent() {
        let cfg = BulletConfig::default();
        let text = "- First item that is long enough to need wrapping onto the second line here";
        let (lines, _) = render_bullets(text, 30, &cfg);
        if lines.len() > 1 {
            // Continuation line should be indented to align past the bullet
            // "● " = 2 chars, so continuation has 2 spaces indent
            assert!(
                lines[1].starts_with("  "),
                "continuation line should have hanging indent: {:?}",
                lines[1]
            );
        }
    }

    #[test]
    fn nested_bullets_wrap_correctly() {
        let cfg = BulletConfig::default();
        let text = "- Top\n  - This nested bullet point is also very long and should word-wrap properly here";
        let (lines, _) = render_bullets(text, 30, &cfg);
        // Should have the top bullet + nested bullet (possibly wrapped)
        assert!(lines.len() >= 2);
        // The nested bullet line should contain ◦
        assert!(
            lines.iter().any(|l| l.contains('◦')),
            "nested bullet should use ◦ char"
        );
    }

    #[test]
    fn continuation_paragraph_under_level_1_bullet() {
        let cfg = BulletConfig::default();
        let text = "- First bullet\n    Continuation prose under it";
        let (lines, _) = render_bullets(text, 80, &cfg);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "● First bullet");
        // Level-1 bullet content starts at col 2 ("● "), so continuation aligns there.
        assert_eq!(lines[1], "  Continuation prose under it");
    }

    #[test]
    fn continuation_paragraph_has_no_glyph() {
        let cfg = BulletConfig::default();
        let text = "- A bullet\n    Indented prose";
        let (lines, _) = render_bullets(text, 80, &cfg);
        // The continuation line must NOT contain any bullet glyph.
        assert!(!lines[1].contains('●'));
        assert!(!lines[1].contains('◦'));
        assert!(!lines[1].contains('▸'));
    }

    #[test]
    fn continuation_does_not_count_as_bullet() {
        let cfg = BulletConfig {
            max_bullets: 1,
            ..Default::default()
        };
        let text = "- One\n    Continuation";
        let (_, warns) = render_bullets(text, 80, &cfg);
        // Only one real bullet — continuation should not trigger SLIDE-001.
        assert!(
            !warns.iter().any(|w| w.code == "SLIDE-001"),
            "continuation prose must not count toward max_bullets"
        );
    }

    #[test]
    fn continuation_under_nested_bullet() {
        let cfg = BulletConfig::default();
        let text = "- Top\n  - Nested\n      Continuation under nested";
        let (lines, _) = render_bullets(text, 80, &cfg);
        assert_eq!(lines[0], "● Top");
        assert!(lines[1].contains('◦'));
        // Level-2 bullet content starts at col 4 ("  ◦ "), so continuation aligns there.
        assert_eq!(lines[2], "    Continuation under nested");
    }

    #[test]
    fn continuation_paragraph_wraps_with_aligned_indent() {
        let cfg = BulletConfig::default();
        let text =
            "- Bullet\n    This continuation paragraph is long enough to wrap onto a second line";
        let (lines, _) = render_bullets(text, 30, &cfg);
        eprintln!("DEBUG lines: {:#?}", lines);
        assert!(
            lines.len() >= 3,
            "continuation should wrap into multiple lines: {:?}",
            lines
        );
        // Every continuation line must start with the bullet's content-column indent ("  ").
        for line in &lines[1..] {
            assert!(
                line.starts_with("  "),
                "continuation/wrap line must align to bullet content column: {:?}",
                line
            );
            assert!(
                !line.contains('●'),
                "continuation lines must not have a bullet glyph: {:?}",
                line
            );
        }
    }

    // ── parse_reveal_step ─────────────────────────────────

    #[test]
    fn parse_reveal_step_no_prefix_returns_step_1() {
        let (step, rest) = parse_reveal_step("- Normal bullet");
        assert_eq!(step, 1);
        assert_eq!(rest, "- Normal bullet");
    }

    #[test]
    fn parse_reveal_step_strips_bracket_prefix() {
        let (step, rest) = parse_reveal_step("[2] - Second step");
        assert_eq!(step, 2);
        assert_eq!(rest, "- Second step");
    }

    #[test]
    fn parse_reveal_step_handles_step_10() {
        let (step, rest) = parse_reveal_step("[10] - Tenth step");
        assert_eq!(step, 10);
        assert_eq!(rest, "- Tenth step");
    }

    #[test]
    fn parse_reveal_step_preserves_leading_indent() {
        // Nested reveal bullet: "  [2] - nested"
        let (step, rest) = parse_reveal_step("  [2] - nested");
        assert_eq!(step, 2);
        assert_eq!(rest, "- nested");
    }

    #[test]
    fn parse_reveal_step_non_numeric_bracket_is_step_1() {
        let (step, rest) = parse_reveal_step("[abc] - bullet");
        assert_eq!(step, 1);
        assert_eq!(rest, "[abc] - bullet");
    }

    // ── has_reveal_markers ────────────────────────────────

    #[test]
    fn has_reveal_markers_false_when_no_markers() {
        assert!(!has_reveal_markers("- A\n- B\n- C"));
    }

    #[test]
    fn has_reveal_markers_true_when_step_2_present() {
        assert!(has_reveal_markers("- A\n[2] - B"));
    }

    #[test]
    fn has_reveal_markers_false_for_step_1_only() {
        // [1] is valid syntax but step 1 is the default — not a true "reveal marker"
        assert!(!has_reveal_markers("[1] - A\n[1] - B"));
    }

    // ── render_bullets_pages ──────────────────────────────

    #[test]
    fn reveal_no_markers_returns_single_page() {
        let cfg = BulletConfig::default();
        let text = "- A\n- B\n- C";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        assert_eq!(pages.len(), 1, "no reveal markers → single page");
    }

    #[test]
    fn reveal_two_steps_returns_two_pages() {
        let cfg = BulletConfig::default();
        let text = "- Always\n[2] - Step 2";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        assert_eq!(pages.len(), 2, "steps 1 and 2 → 2 pages");
    }

    #[test]
    fn reveal_page_1_excludes_step_2_bullet() {
        let cfg = BulletConfig::default();
        let text = "- Always\n[2] - Step 2";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        let p1 = pages[0].join("\n");
        assert!(p1.contains("Always"), "page 1 should have step-1 bullet");
        assert!(
            !p1.contains("Step 2"),
            "page 1 should NOT have step-2 bullet"
        );
    }

    #[test]
    fn reveal_page_2_includes_both_bullets() {
        let cfg = BulletConfig::default();
        let text = "- Always\n[2] - Step 2";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        let p2 = pages[1].join("\n");
        assert!(p2.contains("Always"), "page 2 should have step-1 bullet");
        assert!(p2.contains("Step 2"), "page 2 should have step-2 bullet");
    }

    #[test]
    fn reveal_gap_steps_compacted() {
        // Steps 1 and 3 only — should produce 2 pages, not 3
        let cfg = BulletConfig::default();
        let text = "- Step 1\n[3] - Step 3";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        assert_eq!(pages.len(), 2, "gap steps 1,3 → 2 compacted pages");
        // Last page should have both
        let last = pages.last().unwrap().join("\n");
        assert!(last.contains("Step 1") && last.contains("Step 3"));
    }

    #[test]
    fn reveal_blank_lines_appear_on_every_page() {
        let cfg = BulletConfig::default();
        let text = "- A\n\n[2] - B";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        assert_eq!(pages.len(), 2);
        // Page 1 should have the blank line (spacing preserved)
        assert!(
            pages[0].iter().any(|l| l.trim().is_empty()),
            "blank lines should be present on all pages"
        );
    }

    #[test]
    fn reveal_three_steps_incremental() {
        let cfg = BulletConfig {
            max_bullets: 10,
            ..BulletConfig::default()
        };
        let text = "- One\n[2] - Two\n[3] - Three";
        let (pages, _) = render_bullets_pages(text, 80, &cfg);
        assert_eq!(pages.len(), 3);
        let p1 = pages[0].join("\n");
        let p2 = pages[1].join("\n");
        let p3 = pages[2].join("\n");
        assert!(p1.contains("One") && !p1.contains("Two") && !p1.contains("Three"));
        assert!(p2.contains("One") && p2.contains("Two") && !p2.contains("Three"));
        assert!(p3.contains("One") && p3.contains("Two") && p3.contains("Three"));
    }
}
