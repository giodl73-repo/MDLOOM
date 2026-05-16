use pebble::PebbleDocument;
use pulldown_cmark::{html, Event, Options, Parser};
use std::path::{Path, PathBuf};

pub fn markdown_to_html_document(markdown: &str, title: &str) -> String {
    let title = pebble::document_title(markdown, title);
    let body = markdown_to_html_fragment(markdown);

    format!(
        concat!(
            "<!doctype html>\n",
            "<html lang=\"en\">\n",
            "<head>\n",
            "<meta charset=\"utf-8\">\n",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "<title>{}</title>\n",
            "<style>\n",
            ":root {{ color-scheme: light dark; }}\n",
            "body {{ font-family: system-ui, sans-serif; line-height: 1.55; max-width: 72ch; margin: 2rem auto; padding: 0 1rem; }}\n",
            "pre, code {{ font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }}\n",
            "pre {{ overflow-x: auto; padding: 1rem; border: 1px solid color-mix(in srgb, currentColor 20%, transparent); border-radius: .5rem; }}\n",
            "table {{ border-collapse: collapse; width: 100%; }}\n",
            "th, td {{ border: 1px solid color-mix(in srgb, currentColor 20%, transparent); padding: .35rem .5rem; }}\n",
            "img {{ max-width: 100%; }}\n",
            "</style>\n",
            "</head>\n",
            "<body>\n",
            "{}",
            "</body>\n",
            "</html>\n"
        ),
        escape_html(&title),
        body
    )
}

pub fn markdown_to_pebble_document(
    markdown: &str,
    fallback_title: &str,
    source_path: &Path,
    resolved_files: &[PathBuf],
) -> String {
    let refs = resolved_files.iter().map(|path| path_string(path));
    PebbleDocument::from_markdown(markdown, fallback_title, path_string(source_path), refs)
        .to_json()
        .expect("serializing Pebble document cannot fail")
}

fn markdown_to_html_fragment(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, markdown_options()).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    });
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options
}

fn path_string(path: &Path) -> String {
    path.display().to_string()
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_backend_renders_common_markdown_blocks() {
        let html = markdown_to_html_document(
            "# Guide\n\n- one\n- two\n\n| A | B |\n|---|---|\n| x | y |\n\n[home](README.md)\n",
            "fallback",
        );

        assert!(html.contains("<title>Guide</title>"), "got:\n{}", html);
        assert!(html.contains("<ul>"), "got:\n{}", html);
        assert!(html.contains("<li>one</li>"), "got:\n{}", html);
        assert!(html.contains("<table>"), "got:\n{}", html);
        assert!(
            html.contains("<a href=\"README.md\">home</a>"),
            "got:\n{}",
            html
        );
    }

    #[test]
    fn html_backend_escapes_raw_html() {
        let html = markdown_to_html_document("# Safe\n\n<script>alert(1)</script>\n", "fallback");

        assert!(!html.contains("<script>"), "got:\n{}", html);
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn html_backend_escapes_title_fallback() {
        let html = markdown_to_html_document("Body only\n", "A < B");

        assert!(html.contains("<title>A &lt; B</title>"), "got:\n{}", html);
    }

    #[test]
    fn pebble_backend_chunks_markdown_for_transfer() {
        let pebble = markdown_to_pebble_document(
            "---\ntags: [proof, guide]\nstatus: ready\n---\n\n# Guide\n\nIntro.\n\n## Steps\n\n- one\n- two\n",
            "fallback",
            Path::new("guide.source.md"),
            &[PathBuf::from(".proof\\side-info\\links.json")],
        );
        let json: serde_json::Value = serde_json::from_str(&pebble).unwrap();

        assert_eq!(json["schema"], "pebble.v1");
        assert_eq!(json["title"], "Guide");
        assert_eq!(json["source"], "guide.source.md");
        assert_eq!(json["metadata"]["status"], "ready");
        assert_eq!(json["refs"].as_array().unwrap().len(), 1);
        assert_eq!(json["sections"][0]["id"], "guide");
        assert_eq!(json["sections"][0]["metadata"]["tags"], "[proof, guide]");
        assert_eq!(json["sections"][1]["path"][1], "Steps");
        assert!(json["sections"][1]["text"]
            .as_str()
            .unwrap()
            .contains("- one"));
        assert!(!json["sections"][0]["text"]
            .as_str()
            .unwrap()
            .contains("status: ready"));
    }
}
