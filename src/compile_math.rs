use crate::math::{render_display_math, MathAlign, MathDiag};

use crate::compile_types::{CompileViolation, ViolationSeverity};

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

pub(crate) fn compile_math(
    expr: &str,
    width: usize,
    align: MathAlign,
    no_chrome: bool,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    let rendered = render_math_compiled(expr, width, align, no_chrome);
    *resolved_count += 1;
    for diagnostic in &rendered.diagnostics {
        violations.push(CompileViolation {
            code: diagnostic.code,
            severity: ViolationSeverity::Warning,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message: diagnostic.message.clone(),
            source_line: line_start + 1 + source_line_offset,
        });
    }
    rendered.block
}
