/// Flat character grid for composing dashboard output.
///
/// D-6: every row is exactly `width` chars. `render()` joins rows with '\n'
/// and appends a final '\n'.
pub struct Canvas {
    width: usize,
    height: usize,
    buf: Vec<char>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buf: vec![' '; width * height],
        }
    }

    /// Paste `lines` at (x, y). Clips at canvas boundaries — no bleed.
    /// Uses visual column position (visual_width per char) so wide chars (emoji,
    /// CJK) do not cause column drift.
    pub fn paste(&mut self, x: usize, y: usize, lines: &[&str]) -> &mut Self {
        use crate::layout::visual_width;
        for (dy, line) in lines.iter().enumerate() {
            let row = y + dy;
            if row >= self.height { break; }
            let mut col = x; // visual column position
            for ch in line.chars() {
                if col >= self.width { break; }
                self.buf[row * self.width + col] = ch;
                let ch_w = if (0x2500..=0x28FF).contains(&(ch as u32))
                    || (0x25A0..=0x25FF).contains(&(ch as u32)) { 1 }
                    else { unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1) };
                // Wide char: fill second column with space to avoid overlap
                if ch_w >= 2 && col + 1 < self.width {
                    self.buf[row * self.width + col + 1] = ' ';
                }
                col += ch_w;
            }
        }
        self
    }

    /// Render to a newline-terminated string. Each row is exactly `width` chars (D-6).
    pub fn render(&self) -> String {
        let mut out = String::with_capacity((self.width + 1) * self.height);
        for row in 0..self.height {
            let start = row * self.width;
            let row_chars: String = self.buf[start..start + self.width].iter().collect();
            out.push_str(&row_chars);
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_line_count() {
        let c = Canvas::new(10, 3);
        let rendered = c.render();
        assert_eq!(rendered.lines().count(), 3);
    }

    #[test]
    fn render_row_width_d6() {
        let c = Canvas::new(10, 3);
        for line in c.render().lines() {
            assert_eq!(line.chars().count(), 10);
        }
    }

    #[test]
    fn paste_basic() {
        let mut c = Canvas::new(80, 24);
        c.paste(0, 0, &["hello"]);
        let rendered = c.render();
        let first_row = rendered.lines().next().unwrap();
        assert!(first_row.starts_with("hello"));
        // rest of row is spaces
        assert_eq!(&first_row[5..], &" ".repeat(75));
        // row 1 is all spaces
        let second_row = rendered.lines().nth(1).unwrap();
        assert_eq!(second_row, " ".repeat(80));
    }

    #[test]
    fn paste_clips_overflow_x() {
        let mut c = Canvas::new(5, 1);
        c.paste(3, 0, &["hello"]); // only "he" fits
        let rendered = c.render();
        assert_eq!(rendered.trim_end_matches('\n'), "   he");
    }

    #[test]
    fn paste_clips_overflow_y() {
        let mut c = Canvas::new(5, 2);
        // three lines starting at row 1 — only the first fits
        c.paste(0, 1, &["AAAAA", "BBBBB", "CCCCC"]);
        let rendered = c.render();
        let rows: Vec<&str> = rendered.lines().collect();
        assert_eq!(rows[0], "     ");
        assert_eq!(rows[1], "AAAAA");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn two_regions_no_bleed() {
        let mut c = Canvas::new(10, 2);
        c.paste(0, 0, &["AAAAA"]);
        c.paste(0, 1, &["BBBBB"]);
        let rendered = c.render();
        let rows: Vec<&str> = rendered.lines().collect();
        // row 0: AAAAA followed by spaces — no B
        assert!(rows[0].starts_with("AAAAA"));
        assert!(!rows[0].contains('B'));
        // row 1: BBBBB followed by spaces — no A
        assert!(rows[1].starts_with("BBBBB"));
        assert!(!rows[1].contains('A'));
    }

    #[test]
    fn render_terminates_with_newline() {
        let c = Canvas::new(4, 2);
        assert!(c.render().ends_with('\n'));
    }
}
