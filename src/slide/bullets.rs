/// proof:bullets renderer — hierarchical bullet list with configurable chars.
///
/// Bullet chars by level (configurable via slide front-matter):
///   Level 1: ● (default), Level 2: ◦, Level 3: ▸, Level 4+: –
///
/// SLIDE-001: max_bullets exceeded per slide (advisory)
/// SLIDE-007: bullet depth exceeds max_depth (advisory)

#[derive(Debug, Clone)]
pub struct BulletConfig {
    pub level_chars: [char; 4],   // chars for levels 1, 2, 3, 4+
    pub indent_width: usize,      // spaces per level (default: 2)
    pub max_bullets: usize,       // SLIDE-001 threshold (default: 6)
    pub max_depth: usize,         // SLIDE-007 threshold (default: 4)
}

impl Default for BulletConfig {
    fn default() -> Self {
        BulletConfig {
            level_chars: ['●', '◦', '▸', '–'],
            indent_width: 2,
            max_bullets: 6,
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
/// Returns (rendered_lines, warnings).
pub fn render_bullets(
    text: &str,
    width: usize,
    config: &BulletConfig,
) -> (Vec<String>, Vec<BulletWarning>) {
    let mut lines: Vec<String> = Vec::new();
    let mut warnings: Vec<BulletWarning> = Vec::new();
    let mut bullet_count = 0usize;

    for raw_line in text.lines() {
        if raw_line.trim().is_empty() {
            lines.push(String::new());
            continue;
        }

        // Detect indent level: count leading spaces / indent_width
        let leading = raw_line.len() - raw_line.trim_start().len();
        let level = (leading / config.indent_width).min(3) + 1; // 1-indexed, max 4

        if level > config.max_depth {
            warnings.push(BulletWarning {
                code: "SLIDE-007",
                message: format!("bullet depth {} exceeds max_depth {}", level, config.max_depth),
            });
        }

        let trimmed = raw_line.trim();
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
                message: format!("bullet {} exceeds max_bullets {}", bullet_count, config.max_bullets),
            });
        }

        let bullet_char = config.level_chars[level.min(4) - 1];
        let indent = " ".repeat((level - 1) * config.indent_width);
        let prefix = format!("{}{} ", indent, bullet_char);
        let prefix_w = prefix.chars().count();
        let first_line = format!("{}{}", prefix, content);

        // Word-wrap with hanging indent — continuation lines align past the bullet char
        let wrapped = word_wrap_hanging(&first_line, prefix_w, width);
        lines.extend(wrapped);
    }

    (lines, warnings)
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
        let max_w = if first && result.is_empty() { width } else { cont_width };
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
        assert!(lines[1].contains('◦'), "level 2 should use ◦: {:?}", lines[1]);
    }

    #[test]
    fn level_3_uses_right_arrow() {
        let cfg = BulletConfig::default();
        let (lines, _) = render_bullets("- Top\n  - Mid\n    - Deep", 80, &cfg);
        assert!(lines[2].contains('▸'), "level 3 should use ▸: {:?}", lines[2]);
    }

    #[test]
    fn max_bullets_warning() {
        let cfg = BulletConfig { max_bullets: 2, ..Default::default() };
        let text = "- A\n- B\n- C";
        let (_, warns) = render_bullets(text, 80, &cfg);
        assert!(warns.iter().any(|w| w.code == "SLIDE-001"));
    }

    #[test]
    fn max_depth_warning() {
        let cfg = BulletConfig { max_depth: 2, ..Default::default() };
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
            assert!(line.chars().count() <= 40,
                "line {:?} exceeds width 40", line);
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
            assert!(lines[1].starts_with("  "),
                "continuation line should have hanging indent: {:?}", lines[1]);
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
        assert!(lines.iter().any(|l| l.contains('◦')),
            "nested bullet should use ◦ char");
    }
}
