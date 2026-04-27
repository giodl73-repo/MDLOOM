use anyhow::{bail, Result};
use unicode_width::UnicodeWidthChar;

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Top,
    Center,
    Bottom,
}

impl Align {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "top" => Ok(Align::Top),
            "center" => Ok(Align::Center),
            "bottom" => Ok(Align::Bottom),
            other => bail!("unknown align value {:?} — use top, center, or bottom", other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "horizontal" | "h" => Ok(Direction::Horizontal),
            "vertical" | "v" => Ok(Direction::Vertical),
            other => bail!("unknown direction {:?} — use horizontal or vertical", other),
        }
    }
}

pub struct LayoutConfig {
    pub gap: usize,
    pub align: Align,
    pub labels: Vec<String>,
    pub cols: Option<usize>,
    pub width: usize,
    pub direction: Direction,
    pub border: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            gap: 3,
            align: Align::Top,
            labels: Vec::new(),
            cols: None,
            width: 120,
            direction: Direction::Horizontal,
            border: false,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Visual width
// ─────────────────────────────────────────────────────────

/// Visual column width of a string. Box-drawing characters (U+2500–U+259F),
/// arrow characters (U+2190–U+21FF), and Braille patterns (U+2800–U+28FF)
/// are measured at 1 column per L-5, regardless of what some terminals render.
/// CJK ideographs remain at 2.
pub fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (0x2190..=0x21FF).contains(&cp)  // arrows
                || (0x2500..=0x259F).contains(&cp)  // box-drawing + block elements
                || (0x25A0..=0x25FF).contains(&cp)  // geometric shapes
                || (0x2800..=0x28FF).contains(&cp)  // Braille patterns
            {
                1
            } else {
                UnicodeWidthChar::width(c).unwrap_or(1)
            }
        })
        .sum()
}

// ─────────────────────────────────────────────────────────
// Content extraction
// ─────────────────────────────────────────────────────────

/// Extract content lines from a string, stripping fence delimiters if present.
/// The layout engine operates on content lines — never on raw fence ``` lines.
pub fn extract_content_lines(content: &str) -> Vec<String> {
    let raw: Vec<&str> = content.lines().collect();
    if raw.is_empty() {
        return vec![];
    }
    let first = raw[0].trim();
    if (first.starts_with("```") || first.starts_with("~~~")) && raw.len() >= 2 {
        let last = raw[raw.len() - 1].trim();
        if last.starts_with("```") || last.starts_with("~~~") {
            return raw[1..raw.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
    }
    raw.iter().map(|s| s.to_string()).collect()
}

// ─────────────────────────────────────────────────────────
// Core layout algorithm
// ─────────────────────────────────────────────────────────

/// Lay out N figures according to config. Returns the composed output as a
/// fenced code block string (includes the opening and closing ``` lines).
pub fn layout(figures: Vec<Vec<String>>, config: &LayoutConfig) -> String {
    if figures.is_empty() {
        return "```\n```".to_string();
    }

    match config.direction {
        Direction::Horizontal => layout_horizontal(figures, config),
        Direction::Vertical => layout_vertical(figures, config),
    }
}

fn layout_horizontal(figures: Vec<Vec<String>>, config: &LayoutConfig) -> String {
    let n = figures.len();
    let cols = config.cols.unwrap_or(n).max(1);

    // Step 2: normalize each figure into a Frame (measure width, pad, add label, add border)
    let frames: Vec<Frame> = figures
        .into_iter()
        .enumerate()
        .map(|(i, lines)| normalize_frame(lines, config, i))
        .collect();

    // Chunk into rows of `cols`
    let row_chunks: Vec<&[Frame]> = frames.chunks(cols).collect();
    let mut composed_rows: Vec<String> = Vec::new();

    for row in row_chunks {
        // Step 3: equalize heights within this row
        let max_height = row.iter().map(|f| f.lines.len()).max().unwrap_or(0);

        let equalized: Vec<Vec<String>> = row
            .iter()
            .map(|f| equalize_height(&f.lines, f.width, max_height, config.align))
            .collect();

        // Step 5: join each line across frames with gap
        let mut row_lines: Vec<String> = Vec::with_capacity(max_height);
        for line_idx in 0..max_height {
            let mut line = String::new();
            for (fi, frame_lines) in equalized.iter().enumerate() {
                if fi > 0 {
                    line.push_str(&" ".repeat(config.gap));
                }
                if let Some(l) = frame_lines.get(line_idx) {
                    line.push_str(l);
                } else {
                    // shouldn't happen after equalize, but guard anyway
                    line.push_str(&" ".repeat(row[fi].width));
                }
            }
            // L-9: strip trailing spaces on emit
            row_lines.push(line.trim_end().to_string());
        }

        composed_rows.push(row_lines.join("\n"));
    }

    // L-8: rows separated by exactly one blank line
    let body = composed_rows.join("\n\n");
    format!("```\n{}\n```", body)
}

fn layout_vertical(figures: Vec<Vec<String>>, config: &LayoutConfig) -> String {
    let mut body_lines: Vec<String> = Vec::new();
    for (i, lines) in figures.iter().enumerate() {
        if i > 0 {
            body_lines.push(String::new()); // blank line separator
        }
        for line in lines {
            body_lines.push(line.trim_end().to_string()); // L-9
        }
    }
    let body = body_lines.join("\n");
    format!("```\n{}\n```", body)
}

// ─────────────────────────────────────────────────────────
// Frame
// ─────────────────────────────────────────────────────────

struct Frame {
    lines: Vec<String>,
    width: usize,
}

fn normalize_frame(mut lines: Vec<String>, config: &LayoutConfig, frame_idx: usize) -> Frame {
    // L-6: empty figure → single blank line, min width 1
    if lines.is_empty() {
        lines.push(String::new());
    }

    // Step 2: frame width = max visual width of all lines
    let frame_width = lines.iter().map(|l| visual_width(l)).max().unwrap_or(0).max(1);

    // Pad every line to frame_width (right-pad with spaces)
    let padded: Vec<String> = lines
        .into_iter()
        .map(|line| {
            let w = visual_width(&line);
            if w < frame_width {
                format!("{}{}", line, " ".repeat(frame_width - w))
            } else {
                line
            }
        })
        .collect();

    // Step 4: prepend label line if provided
    let label_str = config.labels.get(frame_idx).map(|s| s.as_str()).unwrap_or("");
    let mut final_lines = if !label_str.is_empty() {
        let mut v = vec![center_label(label_str, frame_width)];
        v.extend(padded);
        v
    } else {
        padded
    };

    // Border option: wrap each frame in box-drawing characters
    let effective_width = if config.border {
        apply_border(&mut final_lines, frame_width);
        frame_width + 4 // ┌─ + ─┐ adds 2 on each side
    } else {
        frame_width
    };

    Frame {
        lines: final_lines,
        width: effective_width,
    }
}

// ─────────────────────────────────────────────────────────
// Label centering
// ─────────────────────────────────────────────────────────

/// Center a label over frame_width columns.
/// Truncates if label is wider than frame. Tie-break: extra space on right (L-7).
fn center_label(label: &str, frame_width: usize) -> String {
    let label_w = visual_width(label);
    if label_w >= frame_width {
        // truncate to frame_width
        let mut result = String::new();
        let mut w = 0;
        for c in label.chars() {
            let cw = UnicodeWidthChar::width(c).unwrap_or(1);
            if w + cw > frame_width {
                break;
            }
            result.push(c);
            w += cw;
        }
        // right-pad to frame_width if truncated short
        if w < frame_width {
            result.push_str(&" ".repeat(frame_width - w));
        }
        return result;
    }
    let total_pad = frame_width - label_w;
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad; // L-7: extra space on right
    format!("{}{}{}", " ".repeat(left_pad), label, " ".repeat(right_pad))
}

// ─────────────────────────────────────────────────────────
// Height equalization
// ─────────────────────────────────────────────────────────

/// Pad frame lines to max_height with blank lines (spaces × width), per align mode.
fn equalize_height(
    lines: &[String],
    frame_width: usize,
    max_height: usize,
    align: Align,
) -> Vec<String> {
    let height = lines.len();
    if height >= max_height {
        return lines.to_vec();
    }
    let pad_count = max_height - height;
    let blank = " ".repeat(frame_width);

    let (top_pad, bottom_pad) = match align {
        Align::Top => (0, pad_count),
        Align::Bottom => (pad_count, 0),
        Align::Center => {
            let top = pad_count / 2;
            (top, pad_count - top)
        }
    };

    let mut result = Vec::with_capacity(max_height);
    for _ in 0..top_pad {
        result.push(blank.clone());
    }
    result.extend_from_slice(lines);
    for _ in 0..bottom_pad {
        result.push(blank.clone());
    }
    result
}

// ─────────────────────────────────────────────────────────
// Border
// ─────────────────────────────────────────────────────────

fn apply_border(lines: &mut Vec<String>, inner_width: usize) {
    let top = format!("┌{}┐", "─".repeat(inner_width + 2));
    let bottom = format!("└{}┘", "─".repeat(inner_width + 2));
    let bordered: Vec<String> = lines
        .iter()
        .map(|l| format!("│ {} │", l))
        .collect();
    *lines = std::iter::once(top)
        .chain(bordered)
        .chain(std::iter::once(bottom))
        .collect();
}

// ─────────────────────────────────────────────────────────
// Invariant checks (used by tests)
// ─────────────────────────────────────────────────────────

/// Check L-1: every output line ≤ width columns.
pub fn check_l1(output: &str, width: usize) -> Vec<usize> {
    output
        .lines()
        .enumerate()
        .filter(|(_, line)| visual_width(line) > width)
        .map(|(i, _)| i + 1)
        .collect()
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> LayoutConfig {
        LayoutConfig::default()
    }

    // ── visual_width ──────────────────────────────────────

    #[test]
    fn test_visual_width_ascii() {
        assert_eq!(visual_width("hello"), 5);
    }

    #[test]
    fn test_visual_width_box_drawing_at_1() {
        // L-5: box-drawing characters measured at 1 column
        assert_eq!(visual_width("┌──┐"), 4);
        assert_eq!(visual_width("│AB│"), 4);
        assert_eq!(visual_width("└──┘"), 4);
    }

    #[test]
    fn test_visual_width_cjk_at_2() {
        // CJK characters are 2 columns wide
        assert_eq!(visual_width("我"), 2);
        assert_eq!(visual_width("AB我"), 4);
    }

    #[test]
    fn test_visual_width_arrows_at_1() {
        assert_eq!(visual_width("→"), 1);
        assert_eq!(visual_width("A→B"), 3);
    }

    #[test]
    fn test_visual_width_braille_at_1() {
        assert_eq!(visual_width("⠿"), 1);
    }

    // ── extract_content_lines ─────────────────────────────

    #[test]
    fn test_extract_strips_fence() {
        let content = "```\nfoo\nbar\n```";
        let lines = extract_content_lines(content);
        assert_eq!(lines, vec!["foo", "bar"]);
    }

    #[test]
    fn test_extract_strips_info_fence() {
        let content = "```rust\nfn main() {}\n```";
        let lines = extract_content_lines(content);
        assert_eq!(lines, vec!["fn main() {}"]);
    }

    #[test]
    fn test_extract_no_fence() {
        let content = "just text\nno fence";
        let lines = extract_content_lines(content);
        assert_eq!(lines, vec!["just text", "no fence"]);
    }

    #[test]
    fn test_extract_empty() {
        let lines = extract_content_lines("");
        assert!(lines.is_empty());
    }

    // ── center_label ──────────────────────────────────────

    #[test]
    fn test_center_label_even_even() {
        // "Go" (2) in width 10: 4 left, 4 right
        assert_eq!(center_label("Go", 10), "    Go    ");
    }

    #[test]
    fn test_center_label_odd_extra_on_right() {
        // "Go" (2) in width 9: total_pad=7, left=3, right=4 (L-7)
        assert_eq!(center_label("Go", 9), "   Go    ");
    }

    #[test]
    fn test_center_label_exact_fit() {
        assert_eq!(center_label("hello", 5), "hello");
    }

    #[test]
    fn test_center_label_truncates() {
        let result = center_label("hello world", 5);
        assert_eq!(result.len(), 5);
        assert!(result.starts_with("hello"));
    }

    #[test]
    fn test_center_label_empty() {
        assert_eq!(center_label("", 5), "     ");
    }

    // ── equalize_height ───────────────────────────────────

    #[test]
    fn test_equalize_top() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let result = equalize_height(&lines, 3, 4, Align::Top);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "a");
        assert_eq!(result[1], "b");
        assert_eq!(result[2], "   "); // blank
        assert_eq!(result[3], "   ");
    }

    #[test]
    fn test_equalize_bottom() {
        let lines = vec!["a".to_string()];
        let result = equalize_height(&lines, 2, 3, Align::Bottom);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "  ");
        assert_eq!(result[1], "  ");
        assert_eq!(result[2], "a");
    }

    #[test]
    fn test_equalize_center_even() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let result = equalize_height(&lines, 1, 4, Align::Center);
        // pad_count=2, top=1, bottom=1
        assert_eq!(result, vec![" ", "a", "b", " "]);
    }

    #[test]
    fn test_equalize_center_odd() {
        let lines = vec!["x".to_string()];
        let result = equalize_height(&lines, 1, 4, Align::Center);
        // pad_count=3, top=1, bottom=2
        assert_eq!(result, vec![" ", "x", " ", " "]);
    }

    #[test]
    fn test_equalize_already_max() {
        let lines = vec!["a".to_string(), "b".to_string()];
        let result = equalize_height(&lines, 1, 2, Align::Top);
        assert_eq!(result, lines);
    }

    // ── layout ────────────────────────────────────────────

    #[test]
    fn test_layout_single_figure() {
        let figures = vec![vec!["hello".to_string(), "world".to_string()]];
        let result = layout(figures, &cfg());
        assert!(result.starts_with("```\n"));
        assert!(result.ends_with("\n```"));
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_layout_two_figures_side_by_side() {
        let fig_a = vec!["AAA".to_string(), "AAA".to_string()];
        let fig_b = vec!["BB".to_string(), "BB".to_string()];
        let config = LayoutConfig { gap: 2, ..Default::default() };
        let result = layout(vec![fig_a, fig_b], &config);
        let inner = result
            .trim_start_matches("```\n")
            .trim_end_matches("\n```");
        let lines: Vec<&str> = inner.lines().collect();
        assert_eq!(lines.len(), 2);
        // Each line: "AAA" + "  " + "BB" = "AAA  BB"
        assert_eq!(lines[0], "AAA  BB");
        assert_eq!(lines[1], "AAA  BB");
    }

    #[test]
    fn test_layout_gap_respected() {
        let fig_a = vec!["A".to_string()];
        let fig_b = vec!["B".to_string()];
        let config = LayoutConfig { gap: 5, ..Default::default() };
        let result = layout(vec![fig_a, fig_b], &config);
        let inner = result
            .trim_start_matches("```\n")
            .trim_end_matches("\n```");
        assert_eq!(inner, "A     B");
    }

    #[test]
    fn test_layout_height_equalized_top() {
        let tall = vec!["T".to_string(), "T".to_string(), "T".to_string()];
        let short = vec!["S".to_string()];
        let config = LayoutConfig { gap: 1, align: Align::Top, ..Default::default() };
        let result = layout(vec![tall, short], &config);
        let inner = result
            .trim_start_matches("```\n")
            .trim_end_matches("\n```");
        let lines: Vec<&str> = inner.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "T S"); // first line: both have content
        assert_eq!(lines[1], "T");   // short frame padded (blank, trailing stripped)
        assert_eq!(lines[2], "T");
    }

    #[test]
    fn test_layout_cols_wrapping() {
        // 3 figures, cols=2 → row1: [A,B], row2: [C]
        let a = vec!["A".to_string()];
        let b = vec!["B".to_string()];
        let c = vec!["C".to_string()];
        let config = LayoutConfig { gap: 1, cols: Some(2), ..Default::default() };
        let result = layout(vec![a, b, c], &config);
        let inner = result
            .trim_start_matches("```\n")
            .trim_end_matches("\n```");
        // Should have row1 "A B", blank line, row2 "C"
        let sections: Vec<&str> = inner.split("\n\n").collect();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0], "A B");
        assert_eq!(sections[1], "C");
    }

    #[test]
    fn test_layout_trailing_spaces_stripped() {
        // Short frames padded to frame_width for alignment, but trailing spaces stripped on emit
        let tall = vec!["LONG".to_string(), "LONG".to_string()];
        let short = vec!["X".to_string()];
        let config = LayoutConfig { gap: 1, ..Default::default() };
        let result = layout(vec![tall, short], &config);
        for line in result.lines() {
            assert!(
                !line.ends_with(' '),
                "line has trailing space: {:?}",
                line
            );
        }
    }

    #[test]
    fn test_layout_empty_figure_l6() {
        // Empty figure → single blank line frame, min width 1
        let figures = vec![vec![], vec!["content".to_string()]];
        let result = layout(figures, &cfg());
        assert!(result.contains("content"));
    }

    #[test]
    fn test_layout_with_labels() {
        // Frames must be wide enough to hold the labels without truncation
        let fig_a = vec!["AAAAAAA".to_string()]; // 7 wide — holds "Go" fine
        let fig_b = vec!["BBBBBBB".to_string()]; // 7 wide — holds "Rust" fine
        let config = LayoutConfig {
            gap: 2,
            labels: vec!["Go".to_string(), "Rust".to_string()],
            ..Default::default()
        };
        let result = layout(vec![fig_a, fig_b], &config);
        // Labels appear before content
        assert!(result.contains("Go"));
        assert!(result.contains("Rust"));
        assert!(result.contains("AAAAAAA"));
        assert!(result.contains("BBBBBBB"));
        // Content lines appear below labels — total height = 2 (label + content)
        let inner = result.trim_start_matches("```\n").trim_end_matches("\n```");
        let lines: Vec<&str> = inner.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_label_truncation() {
        // Label wider than frame gets truncated to frame_width
        let fig = vec!["AB".to_string()]; // frame_width = 2
        let config = LayoutConfig {
            labels: vec!["Toolong".to_string()],
            ..Default::default()
        };
        let result = layout(vec![fig], &config);
        let inner = result.trim_start_matches("```\n").trim_end_matches("\n```");
        let first_line = inner.lines().next().unwrap();
        // Label truncated to 2 chars
        assert_eq!(visual_width(first_line), 2);
    }

    #[test]
    fn test_layout_vertical() {
        let fig_a = vec!["A".to_string()];
        let fig_b = vec!["B".to_string()];
        let config = LayoutConfig { direction: Direction::Vertical, ..Default::default() };
        let result = layout(vec![fig_a, fig_b], &config);
        let inner = result
            .trim_start_matches("```\n")
            .trim_end_matches("\n```");
        assert_eq!(inner, "A\n\nB");
    }

    #[test]
    fn test_layout_border() {
        let figures = vec![vec!["hi".to_string()]];
        let config = LayoutConfig { border: true, ..Default::default() };
        let result = layout(figures, &config);
        assert!(result.contains('┌'));
        assert!(result.contains('┐'));
        assert!(result.contains('└'));
        assert!(result.contains('┘'));
        assert!(result.contains("│ hi │"));
    }

    #[test]
    fn test_extract_then_layout_roundtrip() {
        // Simulate: figure file has a fenced block, layout strips fence and composes
        let fig_content = "```\nFOO\nBAR\n```";
        let lines = extract_content_lines(fig_content);
        assert_eq!(lines, vec!["FOO", "BAR"]);
        let result = layout(vec![lines], &cfg());
        assert!(result.contains("FOO"));
        assert!(result.contains("BAR"));
    }

    #[test]
    fn test_check_l1() {
        let output = "```\nshort line\na very long line that exceeds the limit here\n```";
        let violations = check_l1(output, 20);
        assert!(!violations.is_empty());
    }
}
