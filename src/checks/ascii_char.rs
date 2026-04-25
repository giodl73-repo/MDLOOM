/// ASCII art character range validator (Style Guide Rule S-01).
///
/// Warns when characters outside the alignment-safe Unicode ranges appear
/// inside code blocks, with escalated severity for wide/fullwidth characters
/// that will definitely break horizontal alignment in monospace diagrams.
///
/// Safe ranges (all render at exactly 1 column in any CommonMark renderer):
///   U+0020–U+007E  Basic Latin (printable ASCII)
///   U+2500–U+257F  Box Drawing
///   U+2580–U+259F  Block Elements
///   U+25A0–U+25FF  Geometric Shapes
///   U+2190–U+21FF  Arrows
///
/// Prohibited (break alignment):
///   Wide (W): CJK ideographs, Hiragana, Katakana — 2 columns
///   Fullwidth (F): Fullwidth ASCII variants (Ａ, Ｂ, …) — 2 columns
///
/// See: specs/unicode-east-asian-width.md, specs/gfm-code-blocks.md

use crate::checks::Check;
use crate::config::AsciiCharConfig;
use crate::diagnostic::Diagnostic;
use std::path::Path;
use unicode_width::UnicodeWidthChar;

pub struct AsciiCharCheck {
    pub config: AsciiCharConfig,
}

impl Check for AsciiCharCheck {
    fn name(&self) -> &'static str { "ascii_char" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut diags = Vec::new();

        // Only check inside code blocks
        let regions = code_block_regions(&lines);
        for (start, end) in regions {
            for (rel_idx, &line) in lines[start..end].iter().enumerate() {
                let abs_line = rel_idx + start + 1; // 1-based

                let mut col_0 = 0usize;
                for c in line.chars() {
                    let width = c.width().unwrap_or(0);
                    let col_1 = col_0 + 1; // 1-based

                    if !is_alignment_safe(c) {
                        if width >= 2 {
                            // Wide or Fullwidth — ERROR: will definitely break alignment
                            diags.push(Diagnostic::error(
                                path.to_path_buf(),
                                abs_line,
                                col_1,
                                "ascii_char_range",
                                format!(
                                    "wide character {:?} (U+{:04X}) at col {} — \
                                     occupies {} display columns, breaks box alignment",
                                    c, c as u32, col_1, width
                                ),
                            ));
                        } else if self.config.warn_unusual {
                            // Unusual but narrow — WARNING: may render differently
                            diags.push(Diagnostic::warning(
                                path.to_path_buf(),
                                abs_line,
                                col_1,
                                "ascii_char_range",
                                format!(
                                    "unusual character {:?} (U+{:04X}) at col {} — \
                                     outside alignment-safe Unicode ranges; \
                                     renderer-dependent display width",
                                    c, c as u32, col_1
                                ),
                            ));
                        }
                    }

                    // Tab advance (consistent with ascii_box.rs)
                    col_0 += if c == '\t' {
                        let next = ((col_0 / 4) + 1) * 4;
                        next - col_0
                    } else {
                        width.max(1)
                    };
                }
            }
        }

        diags
    }
}

/// Returns true if `c` is in one of the alignment-safe Unicode ranges.
/// Characters in these ranges are guaranteed to render at exactly 1 display
/// column in every supported markdown renderer (CommonMark, GFM, MkDocs).
pub fn is_alignment_safe(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        // Basic Latin printable ASCII
        0x0020..=0x007E |
        // Box Drawing (─│┌┐└┘├┤┬┴┼═║╔╗╚╝╠╣╦╩╬ etc.)
        0x2500..=0x257F |
        // Block Elements (▀▁▂…▓█)
        0x2580..=0x259F |
        // Geometric Shapes (■□▲△▼▽◆◇ etc.)
        0x25A0..=0x25FF |
        // Arrows (←↑→↓↔↕↖↗↘↙▶◀▲▼ etc.)
        0x2190..=0x21FF
    )
}

/// Detect code block content ranges (shared logic with ascii_box.rs).
fn code_block_regions(lines: &[&str]) -> Vec<(usize, usize)> {
    let mut regions = Vec::new();
    let mut in_block = false;
    let mut block_start = 0;
    let mut fence_char = '`';
    let mut fence_len = 3usize;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            let ch = trimmed.chars().next();
            if matches!(ch, Some('`') | Some('~')) {
                let c = ch.unwrap();
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if run >= 3 {
                    in_block = true;
                    fence_char = c;
                    fence_len = run;
                    block_start = i + 1;
                }
            }
        } else {
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len {
                    regions.push((block_start, i));
                    in_block = false;
                }
            }
        }
    }
    if in_block {
        regions.push((block_start, lines.len()));
    }
    regions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_default() -> AsciiCharCheck {
        AsciiCharCheck { config: AsciiCharConfig::default() }
    }

    #[test]
    fn cjk_in_code_block_is_error() {
        // 中 is Wide (W), U+4E2D, 2 columns
        let content = "```\n+------+\n| 中文 |\n+------+\n```";
        let diags = check_default().check(Path::new("test.md"), content);
        let errs: Vec<_> = diags.iter().filter(|d| d.severity == crate::diagnostic::Severity::Error).collect();
        assert!(!errs.is_empty(), "CJK chars must be reported as errors");
        assert!(errs.iter().all(|d| d.code == "ascii_char_range"),
            "code must be ascii_char_range");
    }

    #[test]
    fn fullwidth_latin_is_error() {
        // Ａ is Fullwidth (F), U+FF21, 2 columns
        let content = "```\n+------+\n| Ａ   |\n+------+\n```";
        let diags = check_default().check(Path::new("test.md"), content);
        let errs: Vec<_> = diags.iter().filter(|d| d.code == "ascii_char_range" && d.severity == crate::diagnostic::Severity::Error).collect();
        assert!(!errs.is_empty(), "fullwidth Latin must be error");
    }

    #[test]
    fn box_drawing_chars_are_safe() {
        // Box drawing (┌─┐│└) are in the safe range
        let content = "```\n┌──────┐\n│ text │\n└──────┘\n```";
        let diags = check_default().check(Path::new("test.md"), content);
        let char_errs: Vec<_> = diags.iter().filter(|d| d.code == "ascii_char_range").collect();
        assert!(char_errs.is_empty(), "box drawing chars must be safe, got: {:?}",
            char_errs.iter().map(|d| &d.message).collect::<Vec<_>>());
    }

    #[test]
    fn arrows_are_safe() {
        let content = "```\n→ ← ↑ ↓ ▶ ◀ ▲ ▼\n```";
        let diags = check_default().check(Path::new("test.md"), content);
        let char_errs: Vec<_> = diags.iter().filter(|d| d.code == "ascii_char_range").collect();
        assert!(char_errs.is_empty(), "arrow chars must be safe");
    }

    #[test]
    fn wide_chars_outside_code_block_not_checked() {
        // CJK outside a code block — should NOT fire (only checks inside fences)
        let content = "# Title with 中文\n\nProse with 日本語 text.\n";
        let diags = check_default().check(Path::new("test.md"), content);
        assert!(diags.is_empty(),
            "chars outside code blocks must not be checked");
    }

    #[test]
    fn is_alignment_safe_covers_expected_ranges() {
        // Basic Latin
        assert!(is_alignment_safe(' '));
        assert!(is_alignment_safe('A'));
        assert!(is_alignment_safe('+'));
        assert!(is_alignment_safe('|'));
        // Box drawing
        assert!(is_alignment_safe('─'));
        assert!(is_alignment_safe('│'));
        assert!(is_alignment_safe('┌'));
        assert!(is_alignment_safe('╔'));
        // Arrows
        assert!(is_alignment_safe('→'));
        assert!(is_alignment_safe('▶'));
        // NOT safe
        assert!(!is_alignment_safe('中'));  // CJK
        assert!(!is_alignment_safe('Ａ'));  // Fullwidth Latin
        assert!(!is_alignment_safe('α'));   // Greek (outside safe range)
    }
}
