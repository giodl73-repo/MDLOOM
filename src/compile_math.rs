use crate::math::{render_display_math, MathAlign, MathDiag};

pub(crate) struct RenderedMath {
    pub(crate) block: String,
    pub(crate) diagnostics: Vec<MathDiag>,
}

pub(crate) fn render_math_compiled(
    expr: &str,
    width: usize,
    align: MathAlign,
    no_chrome: bool,
) -> RenderedMath {
    let (math_lines, diagnostics) = render_display_math(expr, width, align);
    let rendered = math_lines.join("\n");
    let block = if no_chrome {
        format!("```\n{}\n```", rendered)
    } else {
        format!(
            "<!-- proof:compiled from=\"proof:math\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
            rendered
        )
    };
    RenderedMath { block, diagnostics }
}

pub(crate) fn render_math_inline(expr: &str, width: usize, align: MathAlign) -> RenderedMath {
    let (math_lines, diagnostics) = render_display_math(expr, width, align);
    RenderedMath {
        block: math_lines.join("\n"),
        diagnostics,
    }
}
