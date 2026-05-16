use pulldown_cmark::{html, Event, Options, Parser};

pub fn markdown_to_html_document(markdown: &str, title: &str) -> String {
    let title = document_title(markdown, title);
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

fn document_title(markdown: &str, fallback: &str) -> String {
    markdown
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("# ")
                .map(str::trim)
                .filter(|title| !title.is_empty())
        })
        .unwrap_or(fallback)
        .to_string()
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
}
