use crate::layout::extract_content_lines;

pub(crate) fn include_block(uri: &str, content: &str) -> String {
    let lines = extract_content_lines(content);
    let body = lines.join("\n");
    format!(
        "<!-- mdloom:compiled from=\"{}\" -->\n```\n{}\n```\n<!-- /mdloom:compiled -->",
        uri, body
    )
}

pub(crate) fn layout_block(uris: &[String], composed_inner: &str) -> String {
    let uris_str = uris.join(",");
    format!(
        "<!-- mdloom:compiled from=\"mdloom:layout\"\n     uris=\"{}\" -->\n```\n{}\n```\n<!-- /mdloom:compiled -->",
        uris_str, composed_inner
    )
}

pub(crate) fn element_block(uri: &str, rendered: &str) -> String {
    format!(
        "<!-- mdloom:compiled from=\"mdloom:element\" uri=\"{}\" -->\n```\n{}\n```\n<!-- /mdloom:compiled -->",
        uri, rendered
    )
}

pub(crate) fn row_block(uri: &str, rendered: &str) -> String {
    format!(
        "<!-- mdloom:compiled from=\"mdloom:row\" uri=\"{}\" -->\n```\n{}\n```\n<!-- /mdloom:compiled -->",
        uri, rendered
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_block_has_traceability() {
        let out = include_block("md://figures/foo.md#:0", "CONTENT\nLINE2");
        assert!(out.contains("<!-- mdloom:compiled from=\"md://figures/foo.md#:0\" -->"));
        assert!(out.contains("<!-- /mdloom:compiled -->"));
        assert!(out.contains("CONTENT"));
        assert!(out.contains("LINE2"));
    }

    #[test]
    fn include_block_strips_fence() {
        let out = include_block("md://x.md#:0", "```\nFOO\nBAR\n```");
        assert!(out.contains("FOO"));
        assert!(out.contains("BAR"));
    }

    #[test]
    fn layout_block_has_uris() {
        let uris = vec!["md://a.md#:0".to_string(), "md://b.md#:0".to_string()];
        let out = layout_block(&uris, "COMPOSED");
        assert!(out.contains("mdloom:layout"));
        assert!(out.contains("md://a.md#:0"));
        assert!(out.contains("md://b.md#:0"));
        assert!(out.contains("COMPOSED"));
    }
}
