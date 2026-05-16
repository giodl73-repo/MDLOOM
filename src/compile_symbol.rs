use crate::symbol::shape::{render_shape, ShapeAttrs};

use crate::compile_output;
use crate::compile_types::{CompileViolation, ViolationSeverity};

pub(crate) struct SymbolRenderError {
    pub(crate) code: &'static str,
    pub(crate) is_warning: bool,
    pub(crate) message: String,
}

pub(crate) fn render_symbol(name: &str, size: usize) -> Result<String, SymbolRenderError> {
    let lib = crate::symbol::SymbolLibrary::new();
    match crate::symbol::resolve(name, &lib) {
        Some(sym) => Ok(crate::symbol::render_symbol_block(&sym, size)),
        None => {
            let hint = crate::symbol::suggest_symbol(name, &lib)
                .map(|s| format!(" — did you mean '{}'?", s))
                .unwrap_or_default();
            Err(SymbolRenderError {
                code: "SYMBOL-001",
                is_warning: true,
                message: format!("Unknown symbol '{}'{}", name, hint),
            })
        }
    }
}

pub(crate) fn render_symbol_compiled(name: &str, size: usize) -> Result<String, SymbolRenderError> {
    let rendered = render_symbol(name, size)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:symbol\" name=\"{}\" size=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        name, size, rendered
    ))
}

pub(crate) fn render_shape_inline(attrs: &ShapeAttrs) -> Result<String, SymbolRenderError> {
    render_shape(attrs).map_err(|e| SymbolRenderError {
        code: e.code,
        is_warning: false,
        message: e.message,
    })
}

pub(crate) fn render_shape_compiled(attrs: &ShapeAttrs) -> Result<String, SymbolRenderError> {
    let rendered = render_shape_inline(attrs)?;
    Ok(format!(
        "<!-- proof:compiled from=\"proof:shape\" name=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        attrs.name, rendered
    ))
}

pub(crate) fn compile_symbol(
    name: &str,
    size: usize,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_symbol_compiled(name, size) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_symbol_violation(e, line_start, source_line_offset, violations);
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

pub(crate) fn compile_shape(
    attrs: &ShapeAttrs,
    line_start: usize,
    line_end: usize,
    source_line_offset: usize,
    source_lines: &[&str],
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> String {
    match render_shape_compiled(attrs) {
        Ok(rendered) => {
            *resolved_count += 1;
            rendered
        }
        Err(e) => {
            push_symbol_violation(e, line_start, source_line_offset, violations);
            compile_output::source_fallback(source_lines, line_start, line_end)
        }
    }
}

fn push_symbol_violation(
    error: SymbolRenderError,
    line_start: usize,
    source_line_offset: usize,
    violations: &mut Vec<CompileViolation>,
) {
    violations.push(CompileViolation {
        code: error.code,
        severity: if error.is_warning {
            ViolationSeverity::Warning
        } else {
            ViolationSeverity::Error
        },
        uri: String::new(),
        figure_id: None,
        invariant: String::new(),
        message: error.message,
        source_line: line_start + 1 + source_line_offset,
    });
}
