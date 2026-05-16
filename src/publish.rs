use pebble::PebbleDocument;
use pulldown_cmark::{html, Event, Options, Parser};
use serde::Serialize;
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use crate::frontmatter::SourceFrontmatter;

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

pub fn markdown_to_json_report_bundle(
    markdown: &str,
    fallback_title: &str,
    source_path: &Path,
    output_path: &Path,
    resolved_files: &[PathBuf],
    frontmatter: SourceFrontmatter,
    compile: JsonReportCompile,
) -> String {
    let title = pebble::document_title(markdown, fallback_title);
    let sections = json_report_sections(markdown);
    let report = JsonReportBundle {
        schema: "proof.publish.json_report.v1".to_string(),
        kind: "compile_report".to_string(),
        source_path: path_string(source_path),
        title,
        format: "markdown".to_string(),
        artifact: JsonReportArtifact {
            target: "json-report".to_string(),
            output_path: path_string(output_path),
        },
        frontmatter,
        refs: resolved_files
            .iter()
            .map(|path| path_string(path))
            .collect(),
        document: JsonReportDocument {
            markdown: markdown.to_string(),
            section_count: sections.len(),
            sections,
        },
        compile,
    };
    serde_json::to_string_pretty(&report).expect("serializing JSON report cannot fail")
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportBundle {
    pub schema: String,
    pub kind: String,
    pub source_path: String,
    pub title: String,
    pub format: String,
    pub artifact: JsonReportArtifact,
    pub frontmatter: SourceFrontmatter,
    pub refs: Vec<String>,
    pub document: JsonReportDocument,
    pub compile: JsonReportCompile,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportArtifact {
    pub target: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportDocument {
    pub markdown: String,
    pub section_count: usize,
    pub sections: Vec<JsonReportSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportSection {
    pub id: String,
    pub level: usize,
    pub title: String,
    pub path: Vec<String>,
    pub line_start: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportCompile {
    pub directives_resolved: usize,
    pub diagnostics_count: usize,
    pub diagnostics: Vec<JsonReportDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonReportDiagnostic {
    pub code: String,
    pub severity: String,
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteManifest {
    pub schema: String,
    pub generated_by: String,
    pub page_count: usize,
    pub pages: Vec<SitePage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SitePage {
    pub title: String,
    pub source_path: String,
    pub output_path: String,
    pub href: String,
    pub status: String,
    pub diagnostics_count: usize,
}

pub fn html_document_title(html: &str) -> Option<String> {
    let start = html.find("<title>")? + "<title>".len();
    let end = html[start..].find("</title>")? + start;
    Some(html[start..end].to_string())
}

pub fn html_to_pdf_document(html: &str, fallback_title: &str) -> Vec<u8> {
    let title =
        decode_html_text(&html_document_title(html).unwrap_or_else(|| fallback_title.to_string()));
    let text = html_to_plain_text(html);
    let lines = wrapped_pdf_lines(&text);
    build_simple_pdf(&title, &lines)
}

pub fn write_static_site(site_root: &Path, mut pages: Vec<SitePage>) -> std::io::Result<()> {
    std::fs::create_dir_all(site_root)?;
    pages.sort_by(|left, right| left.href.cmp(&right.href));
    let manifest = SiteManifest {
        schema: "proof.publish.site.v1".to_string(),
        generated_by: "proof compile --target site".to_string(),
        page_count: pages.len(),
        pages,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(std::io::Error::other)?;
    std::fs::write(site_root.join("proof-site.json"), manifest_json)?;
    std::fs::write(site_root.join("index.html"), static_site_index(&manifest))?;
    Ok(())
}

fn static_site_index(manifest: &SiteManifest) -> String {
    let pages = manifest
        .pages
        .iter()
        .map(|page| {
            format!(
                "<li><a href=\"{}\">{}</a> <span>{}</span></li>\n",
                escape_html(&page.href),
                escape_html(&page.title),
                escape_html(&page.status),
            )
        })
        .collect::<String>();
    format!(
        concat!(
            "<!doctype html>\n",
            "<html lang=\"en\">\n",
            "<head>\n",
            "<meta charset=\"utf-8\">\n",
            "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
            "<title>PROOF Site</title>\n",
            "<style>body {{ font-family: system-ui, sans-serif; line-height: 1.55; max-width: 72ch; margin: 2rem auto; padding: 0 1rem; }} span {{ color: #666; }}</style>\n",
            "</head>\n",
            "<body>\n",
            "<h1>PROOF Site</h1>\n",
            "<nav aria-label=\"Site pages\">\n",
            "<ul>\n",
            "{}",
            "</ul>\n",
            "</nav>\n",
            "</body>\n",
            "</html>\n",
        ),
        pages
    )
}

fn html_to_plain_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut last_was_space = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let tag_name = tag
                    .trim()
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if matches!(
                    tag_name,
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "li" | "tr" | "pre" | "br"
                ) {
                    text.push('\n');
                    last_was_space = true;
                }
            }
            _ if in_tag => tag.push(c),
            c if c.is_whitespace() => {
                if !last_was_space {
                    text.push(' ');
                    last_was_space = true;
                }
            }
            c => {
                text.push(c);
                last_was_space = false;
            }
        }
    }

    decode_html_text(&text)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_html_text(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn wrapped_pdf_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut current = String::new();
        for word in source_line.split_whitespace() {
            if current.len() + word.len() + usize::from(!current.is_empty()) > 82 {
                lines.push(current);
                current = String::new();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn build_simple_pdf(title: &str, lines: &[String]) -> Vec<u8> {
    let mut content = String::from("BT\n/F1 12 Tf\n72 760 Td\n14 TL\n");
    let page_lines = lines.iter().take(48).collect::<Vec<_>>();
    for (index, line) in page_lines.iter().enumerate() {
        if index > 0 {
            content.push_str("T*\n");
        }
        let _ = writeln!(content, "({}) Tj", escape_pdf_text(line));
    }
    content.push_str("ET\n");

    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", content.len(), content),
        format!(
            "<< /Title ({}) /Producer (PROOF) >>",
            escape_pdf_text(title)
        ),
    ];

    let mut pdf = Vec::<u8>::from(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".as_slice());
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }
    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    pdf
}

fn escape_pdf_text(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '(' => "\\(".to_string(),
            ')' => "\\)".to_string(),
            c if c.is_ascii() && !c.is_control() => c.to_string(),
            _ => "?".to_string(),
        })
        .collect()
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

fn json_report_sections(markdown: &str) -> Vec<JsonReportSection> {
    let mut sections = Vec::new();
    let mut heading_path: Vec<String> = Vec::new();

    for (index, line) in markdown.lines().enumerate() {
        let Some((level, title)) = markdown_heading(line) else {
            continue;
        };
        heading_path.truncate(level.saturating_sub(1));
        heading_path.push(title.to_string());
        sections.push(JsonReportSection {
            id: slugify(title),
            level,
            title: title.to_string(),
            path: heading_path.clone(),
            line_start: index + 1,
        });
    }

    sections
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = trimmed.get(hashes..)?;
    if !rest.starts_with(' ') {
        return None;
    }
    let title = rest.trim();
    (!title.is_empty()).then_some((hashes, title))
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
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

    #[test]
    fn json_report_backend_serializes_compile_bundle() {
        let report = markdown_to_json_report_bundle(
            "# Guide\n\nIntro.\n\n## Steps\n\n- one\n",
            "fallback",
            Path::new("guide.source.md"),
            Path::new("guide.proof-report.json"),
            &[PathBuf::from("figures\\flow.md")],
            SourceFrontmatter {
                tags: vec!["publish".to_string()],
                ops: Vec::new(),
                content: vec!["guide".to_string()],
            },
            JsonReportCompile {
                directives_resolved: 2,
                diagnostics_count: 0,
                diagnostics: Vec::new(),
            },
        );
        let json: serde_json::Value = serde_json::from_str(&report).unwrap();

        assert_eq!(json["schema"], "proof.publish.json_report.v1");
        assert_eq!(json["kind"], "compile_report");
        assert_eq!(json["title"], "Guide");
        assert_eq!(json["artifact"]["target"], "json-report");
        assert_eq!(json["frontmatter"]["tags"][0], "publish");
        assert_eq!(json["document"]["section_count"], 2);
        assert_eq!(json["document"]["sections"][1]["path"][1], "Steps");
        assert_eq!(json["compile"]["directives_resolved"], 2);
        assert_eq!(json["refs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn pdf_backend_writes_valid_pdf_bytes_from_html() {
        let html = markdown_to_html_document("# Guide\n\nBody with <angle> text.\n", "fallback");
        let pdf = html_to_pdf_document(&html, "fallback");

        assert!(pdf.starts_with(b"%PDF-1.4"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Producer (PROOF)"), "got:\n{}", text);
        assert!(text.contains("(Guide) Tj"), "got:\n{}", text);
        assert!(text.contains("Body with <angle> text"), "got:\n{}", text);
    }
}
