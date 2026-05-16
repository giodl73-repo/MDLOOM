use crate::layout::extract_content_lines;

pub(crate) fn include_block(uri: &str, content: &str) -> String {
    let lines = extract_content_lines(content);
    let body = lines.join("\n");
    format!(
        "<!-- proof:compiled from=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uri, body
    )
}

pub(crate) fn layout_block(uris: &[String], composed_inner: &str) -> String {
    let uris_str = uris.join(",");
    format!(
        "<!-- proof:compiled from=\"proof:layout\"\n     uris=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uris_str, composed_inner
    )
}
