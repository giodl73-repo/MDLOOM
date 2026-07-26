//! Tier 2: single-line ASCII art constructs for display math.
//! All constructs here produce a single String (no multi-line output).

use super::tokenizer::MathDiag;
use super::tokenizer::Token;
use super::{consume_group, expand_tokens, tokens_to_plain};

/// Render `\sqrt[n]{arg}` as a single-line string.
pub fn render_sqrt(root: Option<&str>, arg: &str) -> String {
    let inner = if arg.len() > 1 && !arg.starts_with('(') {
        format!("({})", arg)
    } else {
        arg.to_string()
    };
    match root {
        Some(r) => {
            let sup = crate::superscript::to_superscript(r);
            format!("{}√{}", sup, inner)
        }
        None => format!("√{}", inner),
    }
}

/// Inline \frac{num}{den}: renders as "num/den".
pub fn render_frac_inline(num: &str, den: &str) -> String {
    format!("{}/{}", num, den)
}

/// Render \lim, \max, \min with optional subscript/superscript.
pub fn render_limit_op(op: &str, sub: Option<&str>, sup: Option<&str>) -> String {
    let mut s = op.to_string();
    if let Some(sub) = sub {
        s.push_str(&format!("_({})", sub));
    }
    if let Some(sup) = sup {
        s.push_str(&format!("^({})", sup));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_simple() {
        assert_eq!(render_sqrt(None, "x"), "√x");
    }

    #[test]
    fn sqrt_parens_for_complex() {
        assert_eq!(render_sqrt(None, "x+1"), "√(x+1)");
    }

    #[test]
    fn sqrt_nth() {
        assert_eq!(render_sqrt(Some("3"), "x"), "³√x");
    }

    #[test]
    fn frac_inline() {
        assert_eq!(render_frac_inline("a", "b"), "a/b");
        assert_eq!(render_frac_inline("n(n+1)", "2"), "n(n+1)/2");
    }

    #[test]
    fn lim_with_sub() {
        assert_eq!(render_limit_op("lim", Some("x→0"), None), "lim_(x→0)");
    }

    #[test]
    fn max_no_args() {
        assert_eq!(render_limit_op("max", None, None), "max");
    }
}
