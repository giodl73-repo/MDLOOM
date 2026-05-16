use crate::symbol::shape::{render_shape, ShapeAttrs};

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
