//! Tier 3: integral, sum, and product renderers.

use super::fraction::{center_in, RenderedExpr};

/// Render \int with optional lower/upper bounds and an integrand.
///
/// Layout (with both limits):
///   upper
///   ⌠
///   ⌡ integrand
///   lower
pub fn render_int(
    lower: Option<RenderedExpr>,
    upper: Option<RenderedExpr>,
    integrand: RenderedExpr,
) -> RenderedExpr {
    let body_prefix = "⌡ ";
    let body_prefix_width = crate::visual_width(body_prefix);

    // Width of the full expression
    let integrand_width = integrand.width;
    let total_width = body_prefix_width + integrand_width;

    let mut lines: Vec<String> = Vec::new();

    // Upper limit line (if present)
    let upper_line_idx = if let Some(upper) = &upper {
        let idx = lines.len();
        for l in &upper.lines {
            lines.push(l.clone());
        }
        Some(idx)
    } else {
        None
    };

    // ⌠ line
    let top_line_idx = lines.len();
    lines.push("⌠".to_string());

    // ⌡ + integrand baseline line
    let base_idx = lines.len();
    let integrand_base = integrand.baseline;
    // If integrand is multi-line, emit all lines with ⌡ on the first
    for (i, l) in integrand.lines.iter().enumerate() {
        if i == 0 {
            lines.push(format!("{}{}", body_prefix, l));
        } else {
            lines.push(format!("{}{}", " ".repeat(body_prefix_width), l));
        }
    }

    // Lower limit line (if present)
    if let Some(lower) = lower {
        for l in &lower.lines {
            lines.push(l.clone());
        }
    }

    let baseline = base_idx + integrand_base;

    RenderedExpr {
        width: total_width,
        baseline,
        lines,
    }
}

/// Render \sum or \prod with optional lower/upper bounds.
///
/// Layout:
///    upper
///    ∑  body
///   lower
pub fn render_sum_prod(
    op: char,
    lower: Option<RenderedExpr>,
    upper: Option<RenderedExpr>,
    body: RenderedExpr,
) -> RenderedExpr {
    let op_str = op.to_string();
    let op_width = crate::visual_width(&op_str);
    let separator = " ";
    let sep_width = crate::visual_width(separator);

    // Total width = max(op_limit_width, op_width) + sep + body_width
    let limit_width = {
        let uw = upper.as_ref().map(|e| e.width).unwrap_or(0);
        let lw = lower.as_ref().map(|e| e.width).unwrap_or(0);
        uw.max(lw).max(op_width)
    };
    let total_width = limit_width + sep_width + body.width;

    let mut lines: Vec<String> = Vec::new();

    // Upper limit line(s)
    if let Some(ref upper) = upper {
        for l in &upper.lines {
            lines.push(format!(
                "{}{}",
                center_in(l, limit_width),
                " ".repeat(sep_width + body.width)
            ));
        }
    }

    // Operator + body baseline
    let op_line_idx = lines.len();
    let body_base = body.baseline;
    let body_lines = body.lines.len();

    // Emit body lines, with operator on the line that matches body.baseline
    for (i, bl) in body.lines.iter().enumerate() {
        let left = if i == body_base {
            center_in(&op_str, limit_width)
        } else {
            " ".repeat(limit_width)
        };
        lines.push(format!("{}{}{}", left, separator, bl));
    }

    // If operator line wasn't emitted (body has 0 lines), emit it alone
    if body.lines.is_empty() {
        lines.push(format!("{}{}", center_in(&op_str, limit_width), separator));
    }

    // Lower limit line(s)
    if let Some(ref lower) = lower {
        for l in &lower.lines {
            lines.push(format!(
                "{}{}",
                center_in(l, limit_width),
                " ".repeat(sep_width + body.width)
            ));
        }
    }

    let baseline = op_line_idx + body_base;

    RenderedExpr {
        width: total_width,
        baseline,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(s: &str) -> RenderedExpr {
        RenderedExpr::leaf(s)
    }

    #[test]
    fn int_both_limits() {
        let r = render_int(Some(leaf("0")), Some(leaf("∞")), leaf("f(x) dx"));
        // Lines: [∞, ⌠, ⌡ f(x) dx, 0]
        assert_eq!(r.lines.len(), 4);
        assert_eq!(r.lines[0], "∞");
        assert_eq!(r.lines[1], "⌠");
        assert!(r.lines[2].starts_with("⌡ "));
        assert_eq!(r.lines[3], "0");
        assert_eq!(r.baseline, 2);
    }

    #[test]
    fn int_no_limits() {
        let r = render_int(None, None, leaf("f(x) dx"));
        assert_eq!(r.lines.len(), 2); // ⌠ and ⌡ integrand
        assert_eq!(r.lines[0], "⌠");
        assert!(r.lines[1].starts_with("⌡ "));
        assert_eq!(r.baseline, 1);
    }

    #[test]
    fn int_lower_only() {
        let r = render_int(Some(leaf("0")), None, leaf("e^x dx"));
        assert_eq!(r.lines.len(), 3); // ⌠, ⌡ body, 0
        assert_eq!(r.lines[2], "0");
    }

    #[test]
    fn int_upper_only() {
        let r = render_int(None, Some(leaf("n")), leaf("dx"));
        assert_eq!(r.lines.len(), 3); // n, ⌠, ⌡ dx
        assert_eq!(r.lines[0], "n");
    }

    #[test]
    fn int_baseline_is_bottom_integral_line() {
        let r = render_int(Some(leaf("0")), Some(leaf("∞")), leaf("f(x) dx"));
        assert_eq!(r.baseline, 2); // ⌡ line
    }

    #[test]
    fn sum_both_limits() {
        let r = render_sum_prod('∑', Some(leaf("i=1")), Some(leaf("n")), leaf("i"));
        // Lines: [n_centered, ∑ i, i=1_centered]
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[0].contains('n'));
        assert!(r.lines[1].contains('∑'));
        assert!(r.lines[1].contains('i'));
        assert!(r.lines[2].contains("i=1"));
    }

    #[test]
    fn sum_no_limits() {
        let r = render_sum_prod('∑', None, None, leaf("f(i)"));
        assert_eq!(r.lines.len(), 1);
        assert!(r.lines[0].contains('∑'));
        assert!(r.lines[0].contains("f(i)"));
    }

    #[test]
    fn prod_operator() {
        let r = render_sum_prod('∏', Some(leaf("i=1")), Some(leaf("n")), leaf("a_i"));
        assert!(r.lines[1].contains('∏'));
    }

    #[test]
    fn sum_baseline_on_op_line() {
        let r = render_sum_prod('∑', Some(leaf("i=1")), Some(leaf("n")), leaf("f(i)"));
        assert_eq!(r.baseline, 1); // upper at 0, op at 1
    }

    #[test]
    fn prod_with_both_limits() {
        let r = render_sum_prod('∏', Some(leaf("k=0")), Some(leaf("n")), leaf("a_k"));
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[0].contains('n'));
        assert!(r.lines[1].contains('∏'));
        assert!(r.lines[2].contains("k=0"));
    }

    #[test]
    fn prod_no_limits() {
        let r = render_sum_prod('∏', None, None, leaf("x_i"));
        assert_eq!(r.lines.len(), 1);
        assert!(r.lines[0].contains('∏'));
    }

    #[test]
    fn int_no_body() {
        // Empty integrand — no panic
        let r = render_int(Some(leaf("0")), Some(leaf("1")), leaf(""));
        assert!(r.lines.len() >= 2);
    }

    #[test]
    fn sum_upper_limit_only() {
        let r = render_sum_prod('∑', None, Some(leaf("N")), leaf("a_i"));
        let full = r.lines.join("\n");
        assert!(full.contains('N'));
        assert!(full.contains('∑'));
    }
}
