//! mdloom-canvas — Fixed-width ASCII character grid for terminal and TUI composition.
//!
//! A `Canvas` is a rectangular buffer of characters at exact width × height.
//! Content is placed by `paste(x, y, lines)` and rendered to a newline-terminated
//! string via `render()`. Wide Unicode characters (CJK, emoji) are handled correctly:
//! they consume two columns and the trailing column is filled with a space to prevent
//! drift in subsequent content.
//!
//! # Example
//!
//! ```rust
//! use mdloom_canvas::Canvas;
//!
//! let mut canvas = Canvas::new(40, 10);
//! canvas.paste(0, 0, &["╔══════════════════╗"]);
//! canvas.paste(0, 1, &["║  Header content  ║"]);
//! canvas.paste(0, 9, &["╚══════════════════╝"]);
//! canvas.paste(0, 3, &["Status: OK"]);
//! canvas.paste(20, 3, &["Score: 99.9%"]);
//! println!("{}", canvas.render());
//! ```

use unicode_width::UnicodeWidthChar;

/// A fixed-width, fixed-height character grid.
///
/// Every cell is a `char`. `new()` fills the grid with spaces.
/// `paste()` writes content at a given (column, row) origin.
/// `render()` produces a newline-terminated string — every row is
/// exactly `width` visual columns wide (invariant D-6).
pub struct Canvas {
    width: usize,
    height: usize,
    buf: Vec<char>,
}

impl Canvas {
    /// Create a new canvas of `width` × `height` filled with spaces.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buf: vec![' '; width * height],
        }
    }

    /// Width in columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height in rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Paste `lines` onto the canvas at visual position (`x`, `y`).
    ///
    /// - Lines that exceed the canvas width are clipped at the right edge.
    /// - Lines below the canvas height are silently ignored.
    /// - Wide characters (CJK, emoji) occupy two columns; the second column
    ///   is filled with a space to prevent the next character from drifting.
    /// - Box-drawing characters (U+2500–U+28FF) are always treated as 1 column
    ///   regardless of terminal font metrics.
    pub fn paste(&mut self, x: usize, y: usize, lines: &[&str]) -> &mut Self {
        for (dy, line) in lines.iter().enumerate() {
            let row = y + dy;
            if row >= self.height {
                break;
            }
            let mut col = x;
            for ch in line.chars() {
                if col >= self.width {
                    break;
                }
                self.buf[row * self.width + col] = ch;
                let ch_w = char_width(ch);
                if ch_w >= 2 && col + 1 < self.width {
                    self.buf[row * self.width + col + 1] = ' ';
                }
                col += ch_w;
            }
        }
        self
    }

    /// Render the canvas to a newline-terminated string.
    ///
    /// Every row is exactly `width` characters wide (invariant D-6).
    /// The final character of the string is always `\n`.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in 0..self.height {
            let start = row * self.width;
            for &ch in &self.buf[start..start + self.width] {
                out.push(ch);
            }
            out.push('\n');
        }
        out
    }

    /// Return a reference to the raw character buffer (row-major order).
    pub fn buf(&self) -> &[char] {
        &self.buf
    }

    /// Clear the canvas — reset all cells to space.
    pub fn clear(&mut self) {
        self.buf.fill(' ');
    }
}

/// Visual width of a character in terminal columns.
///
/// Box-drawing and Braille block characters are always 1 column.
/// Everything else uses Unicode East Asian Width rules.
pub fn char_width(ch: char) -> usize {
    let cp = ch as u32;
    // Box-drawing (U+2500–U+257F), block elements (U+2580–U+259F),
    // geometric shapes (U+25A0–U+25FF), Braille patterns (U+2800–U+28FF)
    // — always 1 column regardless of terminal font metrics.
    if (0x2500..=0x28FF).contains(&cp) || (0x25A0..=0x25FF).contains(&cp) {
        return 1;
    }
    UnicodeWidthChar::width(ch).unwrap_or(1)
}

impl Canvas {
    /// Same as `paste` but takes owned `String` lines.
    pub fn paste_owned(&mut self, x: usize, y: usize, lines: &[String]) -> &mut Self {
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        self.paste(x, y, &refs)
    }

    /// Draw a box-drawing border around the rectangle (x, y, w, h).
    ///
    /// The border occupies the outermost row/column of the rectangle.
    /// Interior content must be pasted at (x+1, y+1) with size (w-2, h-2).
    /// Silently does nothing if w < 2 or h < 2.
    ///
    /// Uses single-line box-drawing: ┌ ─ ┐ │ └ ┘
    pub fn draw_border(&mut self, x: usize, y: usize, w: usize, h: usize) -> &mut Self {
        if w < 2 || h < 2 {
            return self;
        }
        let right = x + w - 1;
        let bottom = y + h - 1;

        // top and bottom edges
        let top_row: String = std::iter::once('┌')
            .chain(std::iter::repeat_n('─', w - 2))
            .chain(std::iter::once('┐'))
            .collect();
        let bot_row: String = std::iter::once('└')
            .chain(std::iter::repeat_n('─', w - 2))
            .chain(std::iter::once('┘'))
            .collect();

        self.paste(x, y, &[top_row.as_str()]);
        self.paste(x, bottom, &[bot_row.as_str()]);

        // side edges
        for row in (y + 1)..bottom {
            if row >= self.height {
                break;
            }
            if x < self.width {
                self.buf[row * self.width + x] = '│';
            }
            if right < self.width {
                self.buf[row * self.width + right] = '│';
            }
        }

        self
    }
}

/// Clip `lines` to a scrollable window of `max_height` rows starting at `offset`.
///
/// Returns a `Vec<&str>` slice — always at most `max_height` elements.
/// If `offset` exceeds the number of lines, returns an empty vec.
pub fn scroll_clip<'a>(lines: &'a [&'a str], max_height: usize, offset: usize) -> Vec<&'a str> {
    if offset >= lines.len() || max_height == 0 {
        return Vec::new();
    }
    lines[offset..].iter().copied().take(max_height).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fills_with_spaces() {
        let c = Canvas::new(5, 2);
        assert!(c.render().chars().all(|ch| ch == ' ' || ch == '\n'));
    }

    #[test]
    fn render_row_count() {
        let c = Canvas::new(10, 3);
        assert_eq!(c.render().lines().count(), 3);
    }

    #[test]
    fn render_row_width_d6() {
        let c = Canvas::new(10, 3);
        for line in c.render().lines() {
            assert_eq!(
                line.chars().count(),
                10,
                "every row must be exactly width chars"
            );
        }
    }

    #[test]
    fn render_terminates_with_newline() {
        let c = Canvas::new(4, 2);
        assert!(c.render().ends_with('\n'));
    }

    #[test]
    fn paste_basic() {
        let mut c = Canvas::new(10, 3);
        c.paste(0, 0, &["hello"]);
        let first = c.render().lines().next().unwrap().to_string();
        assert!(first.starts_with("hello"));
        assert_eq!(&first[5..], "     ");
    }

    #[test]
    fn paste_at_offset() {
        let mut c = Canvas::new(10, 3);
        c.paste(3, 1, &["AB"]);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        assert_eq!(&rows[1][3..5], "AB");
        assert_eq!(&rows[0], "          "); // untouched
    }

    #[test]
    fn paste_clips_right() {
        let mut c = Canvas::new(5, 1);
        c.paste(3, 0, &["hello"]); // only "he" fits
        let row = c.render().lines().next().unwrap().to_string();
        assert_eq!(row, "   he");
    }

    #[test]
    fn paste_clips_bottom() {
        let mut c = Canvas::new(5, 2);
        c.paste(0, 1, &["AAAAA", "BBBBB", "CCCCC"]);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        assert_eq!(rows[1], "AAAAA");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn two_regions_no_bleed() {
        let mut c = Canvas::new(10, 2);
        c.paste(0, 0, &["AAAAA"]);
        c.paste(0, 1, &["BBBBB"]);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        assert!(rows[0].starts_with("AAAAA") && !rows[0].contains('B'));
        assert!(rows[1].starts_with("BBBBB") && !rows[1].contains('A'));
    }

    #[test]
    fn clear_resets_to_spaces() {
        let mut c = Canvas::new(5, 2);
        c.paste(0, 0, &["hello"]);
        c.clear();
        assert!(c.render().chars().all(|ch| ch == ' ' || ch == '\n'));
    }

    #[test]
    fn paste_owned_works() {
        let mut c = Canvas::new(10, 2);
        c.paste_owned(0, 0, &["hello".to_string()]);
        assert!(c.render().starts_with("hello"));
    }

    #[test]
    fn box_drawing_chars_are_one_column() {
        assert_eq!(char_width('┌'), 1);
        assert_eq!(char_width('─'), 1);
        assert_eq!(char_width('└'), 1);
        assert_eq!(char_width('⣿'), 1); // Braille
    }

    // ── draw_border ──────────────────────────────────────────

    #[test]
    fn draw_border_corners() {
        let mut c = Canvas::new(5, 4);
        c.draw_border(0, 0, 5, 4);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        assert!(rows[0].starts_with('┌'), "top-left corner: {:?}", rows[0]);
        assert!(rows[0].ends_with('┐'), "top-right corner: {:?}", rows[0]);
        assert!(rows[3].starts_with('└'), "bot-left corner: {:?}", rows[3]);
        assert!(rows[3].ends_with('┘'), "bot-right corner: {:?}", rows[3]);
    }

    #[test]
    fn draw_border_top_edge_is_horizontal() {
        let mut c = Canvas::new(6, 3);
        c.draw_border(0, 0, 6, 3);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        // interior of top row (positions 1..5) should be '─'
        let top_interior: String = rows[0].chars().skip(1).take(4).collect();
        assert!(
            top_interior.chars().all(|ch| ch == '─'),
            "top interior: {:?}",
            top_interior
        );
    }

    #[test]
    fn draw_border_side_edges_are_vertical() {
        let mut c = Canvas::new(5, 5);
        c.draw_border(0, 0, 5, 5);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        for row in &rows[1..4] {
            assert_eq!(row.chars().next().unwrap(), '│', "left edge: {:?}", row);
            assert_eq!(row.chars().last().unwrap(), '│', "right edge: {:?}", row);
        }
    }

    #[test]
    fn draw_border_too_small_is_noop() {
        let mut c = Canvas::new(3, 3);
        c.draw_border(0, 0, 1, 1); // w < 2 — no-op
        assert!(c.render().chars().all(|ch| ch == ' ' || ch == '\n'));
    }

    #[test]
    fn draw_border_offset_inside_canvas() {
        let mut c = Canvas::new(10, 6);
        c.draw_border(2, 1, 6, 4);
        let rows: Vec<String> = c.render().lines().map(|l| l.to_string()).collect();
        // row 0 entirely untouched
        assert!(
            rows[0].chars().all(|ch| ch == ' '),
            "row 0 untouched: {:?}",
            rows[0]
        );
        // row 1 col 2 = '┌'
        let ch = rows[1].chars().nth(2).unwrap();
        assert_eq!(ch, '┌', "top-left of inner border: {:?}", ch);
    }

    // ── scroll_clip ──────────────────────────────────────────

    #[test]
    fn scroll_clip_zero_offset_takes_first_n() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let result = scroll_clip(&lines, 3, 0);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn scroll_clip_offset_skips_lines() {
        let lines = vec!["a", "b", "c", "d", "e"];
        let result = scroll_clip(&lines, 3, 2);
        assert_eq!(result, vec!["c", "d", "e"]);
    }

    #[test]
    fn scroll_clip_offset_at_last_line() {
        let lines = vec!["a", "b", "c"];
        let result = scroll_clip(&lines, 5, 2);
        assert_eq!(result, vec!["c"]);
    }

    #[test]
    fn scroll_clip_offset_beyond_end_is_empty() {
        let lines = vec!["a", "b"];
        assert!(scroll_clip(&lines, 3, 5).is_empty());
    }

    #[test]
    fn scroll_clip_max_height_zero_is_empty() {
        let lines = vec!["a", "b", "c"];
        assert!(scroll_clip(&lines, 0, 0).is_empty());
    }

    #[test]
    fn scroll_clip_empty_input_is_empty() {
        assert!(scroll_clip(&[], 5, 0).is_empty());
    }
}
