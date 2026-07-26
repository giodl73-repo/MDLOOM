//! Tier 3: stacked fraction renderer.

/// A multi-line rendered expression with a baseline for vertical alignment.
#[derive(Debug, Clone)]
pub struct RenderedExpr {
    pub lines: Vec<String>,
    pub width: usize,
    pub baseline: usize, // index of the primary alignment line
}

impl RenderedExpr {
    pub fn leaf(s: &str) -> Self {
        let w = crate::visual_width(s);
        RenderedExpr {
            lines: vec![s.to_string()],
            width: w,
            baseline: 0,
        }
    }

    pub fn empty() -> Self {
        RenderedExpr {
            lines: vec![String::new()],
            width: 0,
            baseline: 0,
        }
    }
}

/// Render a stacked fraction.
///
/// bar_width = max(num.width, den.width)
/// Numerator and denominator are centered over/under the bar.
/// Odd-width asymmetries: right padding gets the extra space.
pub fn render_frac(num: RenderedExpr, den: RenderedExpr) -> RenderedExpr {
    let bar_width = num.width.max(den.width).max(1);
    let bar = "─".repeat(bar_width);

    let mut lines = Vec::new();

    // Numerator lines — center each line within bar_width
    for line in &num.lines {
        lines.push(center_in(line, bar_width));
    }

    let bar_line_idx = lines.len();
    lines.push(bar.clone());

    // Denominator lines
    for line in &den.lines {
        lines.push(center_in(line, bar_width));
    }

    RenderedExpr {
        width: bar_width,
        baseline: bar_line_idx,
        lines,
    }
}

/// Center a string within a field of `width` visual columns.
/// Left padding = (width - crate::visual_width(s)) / 2 (integer division)
/// Right padding = width - crate::visual_width(s) - left_pad
pub fn center_in(s: &str, width: usize) -> String {
    let w = crate::visual_width(s);
    if w >= width {
        return s.to_string();
    }
    let total_pad = width - w;
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
}

/// Left-pad a string to `width` visual columns.
pub fn right_align_in(s: &str, width: usize) -> String {
    let w = crate::visual_width(s);
    if w >= width {
        return s.to_string();
    }
    format!("{}{}", " ".repeat(width - w), s)
}

/// Left-align a string to `width` visual columns.
pub fn left_align_in(s: &str, width: usize) -> String {
    let w = crate::visual_width(s);
    if w >= width {
        return s.to_string();
    }
    format!("{}{}", s, " ".repeat(width - w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_frac() {
        let num = RenderedExpr::leaf("d");
        let den = RenderedExpr::leaf("dx");
        let r = render_frac(num, den);
        assert_eq!(r.lines.len(), 3);
        assert_eq!(r.width, 2);
        assert_eq!(r.baseline, 1); // bar line
        assert_eq!(r.lines[1], "──");
    }

    #[test]
    fn wide_numerator_centered() {
        let num = RenderedExpr::leaf("n(n+1)");
        let den = RenderedExpr::leaf("2");
        let r = render_frac(num, den);
        assert_eq!(r.width, 6);
        assert_eq!(r.lines[0], "n(n+1)");
        // denominator "2" centered in 6 cols: 2 left + "2" + 3 right
        assert_eq!(r.lines[2], "  2   ");
    }

    #[test]
    fn odd_width_centering() {
        // 3-char "abc" over 4-char "wxyz"
        let num = RenderedExpr::leaf("abc");
        let den = RenderedExpr::leaf("wxyz");
        let r = render_frac(num, den);
        assert_eq!(r.width, 4);
        // left=0, right=1 for numerator
        assert_eq!(r.lines[0], "abc ");
    }

    #[test]
    fn nested_fraction() {
        // \frac{\frac{a}{b}}{c}
        let inner = render_frac(RenderedExpr::leaf("a"), RenderedExpr::leaf("b"));
        assert_eq!(inner.width, 1); // bar = "─", width=1
        assert_eq!(inner.lines.len(), 3); // a, ─, b
        let outer = render_frac(inner, RenderedExpr::leaf("c"));
        assert_eq!(outer.width, 1);
        // outer lines: ["a", "─", "b", "─", "c"] — 5 lines
        // baseline = num_lines.len() = 3 (index of outer bar)
        assert_eq!(outer.baseline, 3);
        assert_eq!(outer.lines.len(), 5);
    }

    #[test]
    fn baseline_is_bar_line() {
        let num = RenderedExpr::leaf("a");
        let den = RenderedExpr::leaf("b");
        let r = render_frac(num, den);
        assert_eq!(r.baseline, 1);
        assert_eq!(r.lines[r.baseline], "─");
    }

    #[test]
    fn center_in_even_padding() {
        assert_eq!(center_in("ab", 6), "  ab  ");
    }

    #[test]
    fn center_in_odd_padding_right_gets_extra() {
        // "a" (width 1) in 4 cols: left=1, right=2
        assert_eq!(center_in("a", 4), " a  ");
    }
}
