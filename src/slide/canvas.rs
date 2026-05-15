/// Lightweight fixed-size canvas for slide rendering.
///
/// Internally a flat Vec<char> of `width × height` characters, initially all spaces.
/// `paste` writes lines into a rectangular region; excess lines are clipped silently.
/// `render` returns each row as a space-padded String of exactly `width` visual columns.
use crate::layout::visual_width;

pub struct SlideCanvas {
    pub width: usize,
    pub height: usize,
    cells: Vec<char>,
}

impl SlideCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        SlideCanvas {
            width,
            height,
            cells: vec![' '; width * height],
        }
    }

    /// Write `ch` at (col, row) — 0-based. Out-of-bounds writes are silently ignored.
    pub fn set(&mut self, col: usize, row: usize, ch: char) {
        if col < self.width && row < self.height {
            self.cells[row * self.width + col] = ch;
        }
    }

    /// Paste `lines` into the canvas starting at (start_col, start_row).
    /// Lines that exceed `max_col` are truncated. Rows beyond `max_row` are clipped.
    /// Visual width is used for truncation so box-drawing chars count as 1 column.
    pub fn paste(
        &mut self,
        start_col: usize,
        start_row: usize,
        max_col: usize, // exclusive right boundary (0-based col)
        max_row: usize, // exclusive bottom boundary (0-based row)
        lines: &[&str],
    ) {
        let col_limit = max_col.min(self.width);
        let row_limit = max_row.min(self.height);

        for (dy, line) in lines.iter().enumerate() {
            let row = start_row + dy;
            if row >= row_limit {
                break;
            }
            let mut col = start_col;
            for ch in line.chars() {
                let w = char_visual_width(ch);
                if col + w > col_limit {
                    break;
                }
                self.cells[row * self.width + col] = ch;
                // Wide chars leave a space in the second cell
                if w == 2 && col + 1 < col_limit {
                    self.cells[row * self.width + col + 1] = ' ';
                }
                col += w;
            }
        }
    }

    /// Write a single character at a specific cell position (0-based).
    pub fn draw_char(&mut self, col: usize, row: usize, ch: char) {
        self.set(col, row, ch);
    }

    /// Draw a horizontal run of `ch` from col_start..col_end (exclusive) on `row`.
    pub fn fill_h(&mut self, row: usize, col_start: usize, col_end: usize, ch: char) {
        for col in col_start..col_end.min(self.width) {
            self.set(col, row, ch);
        }
    }

    /// Draw a vertical run of `ch` from row_start..row_end (exclusive) on `col`.
    pub fn fill_v(&mut self, col: usize, row_start: usize, row_end: usize, ch: char) {
        for row in row_start..row_end.min(self.height) {
            self.set(col, row, ch);
        }
    }

    /// Render the canvas as a Vec of exactly `height` Strings.
    /// Each string contains exactly `width` visual columns (space-padded on the right).
    pub fn render(&self) -> Vec<String> {
        (0..self.height)
            .map(|row| {
                let start = row * self.width;
                let raw: String = self.cells[start..start + self.width].iter().collect();
                // Pad to width using visual_width so wide chars are counted correctly
                let vw = visual_width(&raw);
                if vw < self.width {
                    format!("{}{}", raw, " ".repeat(self.width - vw))
                } else {
                    raw
                }
            })
            .collect()
    }
}

/// Visual width of a single char (box-drawing = 1, CJK = 2, others = unicode-width).
fn char_visual_width(ch: char) -> usize {
    use unicode_width::UnicodeWidthChar;
    ch.width().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_canvas_all_spaces() {
        let c = SlideCanvas::new(10, 3);
        for row in c.render() {
            assert_eq!(row, " ".repeat(10));
        }
    }

    #[test]
    fn render_returns_correct_dimensions() {
        let c = SlideCanvas::new(80, 24);
        let rows = c.render();
        assert_eq!(rows.len(), 24);
        for row in &rows {
            assert_eq!(
                visual_width(row),
                80,
                "row has wrong visual width: {:?}",
                row
            );
        }
    }

    #[test]
    fn paste_writes_content() {
        let mut c = SlideCanvas::new(20, 5);
        c.paste(2, 1, 20, 5, &["hello"]);
        let rows = c.render();
        assert!(
            rows[1].contains("hello"),
            "pasted content not found: {:?}",
            rows[1]
        );
    }

    #[test]
    fn paste_clips_at_row_boundary() {
        let mut c = SlideCanvas::new(20, 3);
        // 10 lines into 3 rows starting at row 1 — only 2 should land
        let lines: Vec<&str> = (0..10).map(|_| "X").collect();
        c.paste(0, 1, 20, 3, &lines);
        let rows = c.render();
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn paste_clips_at_col_boundary() {
        let mut c = SlideCanvas::new(5, 3);
        c.paste(0, 0, 5, 3, &["ABCDEFGHI"]);
        let rows = c.render();
        assert_eq!(rows[0], "ABCDE");
    }

    #[test]
    fn fill_h_draws_horizontal_line() {
        let mut c = SlideCanvas::new(10, 3);
        c.fill_h(1, 2, 8, '─');
        let rows = c.render();
        let chars: Vec<char> = rows[1].chars().collect();
        // cols 2..8 should be '─'
        for i in 2..8 {
            assert_eq!(chars[i], '─', "col {} should be ─", i);
        }
    }

    #[test]
    fn fill_v_draws_vertical_line() {
        let mut c = SlideCanvas::new(10, 5);
        c.fill_v(3, 1, 4, '│');
        let rows = c.render();
        assert_eq!(rows[1].chars().nth(3).unwrap(), '│');
        assert_eq!(rows[2].chars().nth(3).unwrap(), '│');
        assert_eq!(rows[3].chars().nth(3).unwrap(), '│');
        // Row 0 and 4 untouched
        assert_eq!(rows[0].chars().nth(3).unwrap(), ' ');
        assert_eq!(rows[4].chars().nth(3).unwrap(), ' ');
    }

    #[test]
    fn set_single_char() {
        let mut c = SlideCanvas::new(5, 5);
        c.set(2, 2, 'X');
        let rows = c.render();
        assert_eq!(rows[2].chars().nth(2).unwrap(), 'X');
    }

    #[test]
    fn out_of_bounds_set_is_noop() {
        let mut c = SlideCanvas::new(5, 5);
        c.set(10, 10, 'X'); // should not panic
        let rows = c.render();
        for row in &rows {
            assert!(!row.contains('X'));
        }
    }
}
