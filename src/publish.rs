use pebble::PebbleDocument;
use pulldown_cmark::{html, Event, Options, Parser};
use serde::Serialize;
use std::{
    fmt::Write as _,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};
use zip::{write::FileOptions, ZipWriter};

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

pub fn markdown_to_docx_document(markdown: &str, fallback_title: &str) -> Vec<u8> {
    let title = pebble::document_title(markdown, fallback_title);
    let blocks = markdown_to_docx_blocks(markdown);
    let document = docx_document_xml(&blocks);
    let core = docx_core_xml(&title);
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for (path, content) in [
        ("[Content_Types].xml", docx_content_types_xml()),
        ("_rels/.rels", docx_root_relationships_xml()),
        ("docProps/core.xml", core),
        ("docProps/app.xml", docx_app_xml()),
        (
            "word/_rels/document.xml.rels",
            docx_document_relationships_xml(),
        ),
        ("word/document.xml", document),
        ("word/styles.xml", docx_styles_xml()),
        ("word/numbering.xml", docx_numbering_xml()),
    ] {
        writer
            .start_file(path, options)
            .expect("writing DOCX package part cannot fail");
        writer
            .write_all(content.as_bytes())
            .expect("writing DOCX package part cannot fail");
    }

    writer
        .finish()
        .expect("finalizing DOCX package cannot fail")
        .into_inner()
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum DocxBlock {
    Heading { level: usize, text: String },
    Paragraph(String),
    Bullet { level: usize, text: String },
    Numbered { level: usize, text: String },
    Code(Vec<String>),
    Table(Vec<Vec<String>>),
}

fn markdown_to_docx_blocks(markdown: &str) -> Vec<DocxBlock> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed.starts_with("```") {
            index += 1;
            let mut code = Vec::new();
            while index < lines.len() && !lines[index].trim_start().starts_with("```") {
                code.push(lines[index].to_string());
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            blocks.push(DocxBlock::Code(code));
            continue;
        }
        if let Some((level, title)) = markdown_heading(line) {
            blocks.push(DocxBlock::Heading {
                level,
                text: inline_markdown_text(title),
            });
            index += 1;
            continue;
        }
        if is_markdown_table_start(&lines, index) {
            let mut rows = vec![table_cells(lines[index])];
            index += 2;
            while index < lines.len()
                && lines[index].contains('|')
                && !lines[index].trim().is_empty()
            {
                rows.push(table_cells(lines[index]));
                index += 1;
            }
            blocks.push(DocxBlock::Table(rows));
            continue;
        }
        if let Some((level, text)) = markdown_bullet(line) {
            blocks.push(DocxBlock::Bullet {
                level,
                text: inline_markdown_text(text),
            });
            index += 1;
            continue;
        }
        if let Some((level, text)) = markdown_numbered(line) {
            blocks.push(DocxBlock::Numbered {
                level,
                text: inline_markdown_text(text),
            });
            index += 1;
            continue;
        }

        let mut paragraph = vec![trimmed.to_string()];
        index += 1;
        while index < lines.len() {
            let next = lines[index];
            if next.trim().is_empty()
                || next.trim_start().starts_with("```")
                || markdown_heading(next).is_some()
                || markdown_bullet(next).is_some()
                || markdown_numbered(next).is_some()
                || is_markdown_table_start(&lines, index)
            {
                break;
            }
            paragraph.push(next.trim().to_string());
            index += 1;
        }
        blocks.push(DocxBlock::Paragraph(inline_markdown_text(
            &paragraph.join(" "),
        )));
    }

    blocks
}

fn docx_document_xml(blocks: &[DocxBlock]) -> String {
    let mut body = String::new();
    for block in blocks {
        match block {
            DocxBlock::Heading { level, text } => {
                body.push_str(&docx_paragraph(
                    Some(&format!("Heading{}", (*level).min(6))),
                    None,
                    text,
                ));
            }
            DocxBlock::Paragraph(text) => body.push_str(&docx_paragraph(None, None, text)),
            DocxBlock::Bullet { level, text } => {
                body.push_str(&docx_paragraph(None, Some((1, *level)), text));
            }
            DocxBlock::Numbered { level, text } => {
                body.push_str(&docx_paragraph(None, Some((2, *level)), text));
            }
            DocxBlock::Code(lines) => {
                for line in lines {
                    body.push_str(&docx_paragraph(Some("Code"), None, line));
                }
            }
            DocxBlock::Table(rows) => body.push_str(&docx_table(rows)),
        }
    }

    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
            "<w:body>{}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr></w:body></w:document>"
        ),
        body
    )
}

fn docx_paragraph(style: Option<&str>, numbering: Option<(usize, usize)>, text: &str) -> String {
    let mut props = String::new();
    if let Some(style) = style {
        let _ = write!(props, r#"<w:pStyle w:val="{}"/>"#, escape_xml(style));
    }
    if let Some((num_id, level)) = numbering {
        let _ = write!(
            props,
            r#"<w:numPr><w:ilvl w:val="{}"/><w:numId w:val="{}"/></w:numPr>"#,
            level.min(8),
            num_id
        );
    }
    let props = if props.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{props}</w:pPr>")
    };
    format!(
        "<w:p>{}<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        props,
        escape_xml(text)
    )
}

fn docx_table(rows: &[Vec<String>]) -> String {
    let mut xml = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>");
    for row in rows {
        xml.push_str("<w:tr>");
        for cell in row {
            xml.push_str("<w:tc>");
            xml.push_str(&docx_paragraph(None, None, cell));
            xml.push_str("</w:tc>");
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
    xml
}

fn docx_content_types_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
        r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#,
        r#"<Default Extension="xml" ContentType="application/xml"/>"#,
        r#"<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>"#,
        r#"<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#,
        r#"<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>"#,
        r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#,
        r#"<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>"#,
        "</Types>"
    )
    .to_string()
}

fn docx_root_relationships_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>"#,
        r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>"#,
        r#"<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>"#,
        "</Relationships>"
    )
    .to_string()
}

fn docx_document_relationships_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#,
        "</Relationships>"
    )
    .to_string()
}

fn docx_core_xml(title: &str) -> String {
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">"#,
            "<dc:title>{}</dc:title><dc:creator>PROOF</dc:creator></cp:coreProperties>"
        ),
        escape_xml(title)
    )
}

fn docx_app_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">"#,
        "<Application>PROOF</Application></Properties>"
    )
    .to_string()
}

fn docx_styles_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
        r#"<w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#,
        r#"<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:pPr><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style>"#,
        r#"<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:pPr><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style>"#,
        r#"<w:style w:type="paragraph" w:styleId="Code"><w:name w:val="Code"/><w:rPr><w:rFonts w:ascii="Consolas" w:hAnsi="Consolas"/></w:rPr></w:style>"#,
        "</w:styles>"
    )
    .to_string()
}

fn docx_numbering_xml() -> String {
    concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
        r#"<w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:numFmt w:val="bullet"/><w:lvlText w:val="&#8226;"/></w:lvl></w:abstractNum>"#,
        r#"<w:abstractNum w:abstractNumId="2"><w:lvl w:ilvl="0"><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum>"#,
        r#"<w:num w:numId="1"><w:abstractNumId w:val="1"/></w:num>"#,
        r#"<w:num w:numId="2"><w:abstractNumId w:val="2"/></w:num>"#,
        "</w:numbering>"
    )
    .to_string()
}

fn is_markdown_table_start(lines: &[&str], index: usize) -> bool {
    index + 1 < lines.len()
        && lines[index].contains('|')
        && lines[index + 1]
            .chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' '))
        && lines[index + 1].contains('-')
}

fn table_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| inline_markdown_text(cell.trim()))
        .collect()
}

fn markdown_bullet(line: &str) -> Option<(usize, &str)> {
    let leading = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .map(|text| (leading / 2, text.trim()))
}

fn markdown_numbered(line: &str) -> Option<(usize, &str)> {
    let leading = line.len() - line.trim_start().len();
    let trimmed = line.trim_start();
    let dot = trimmed.find('.')?;
    if dot == 0 || !trimmed[..dot].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    trimmed[dot + 1..]
        .strip_prefix(' ')
        .map(|text| (leading / 2, text.trim()))
}

fn inline_markdown_text(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        let Some(close) = rest[start + 1..]
            .find("](")
            .map(|offset| start + 1 + offset)
        else {
            break;
        };
        let Some(end) = rest[close + 2..].find(')').map(|offset| close + 2 + offset) else {
            break;
        };
        output.push_str(&rest[..start]);
        output.push_str(&rest[start + 1..close]);
        output.push_str(" (");
        output.push_str(&rest[close + 2..end]);
        output.push(')');
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    output.replace('`', "")
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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

    #[test]
    fn docx_backend_writes_native_ooxml_package_parts() {
        let docx = markdown_to_docx_document(
            "# Guide\n\nBody with [home](README.md).\n\n- one\n\n1. first\n\n| A | B |\n|---|---|\n| x | y |\n\n```text\nlet x = 1;\n```\n",
            "fallback",
        );
        let mut archive = zip::ZipArchive::new(Cursor::new(docx)).expect("valid DOCX ZIP archive");

        for part in [
            "[Content_Types].xml",
            "_rels/.rels",
            "docProps/core.xml",
            "docProps/app.xml",
            "word/_rels/document.xml.rels",
            "word/document.xml",
            "word/styles.xml",
            "word/numbering.xml",
        ] {
            assert!(archive.by_name(part).is_ok(), "missing DOCX part {part}");
        }

        let mut document = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("word/document.xml").unwrap(),
            &mut document,
        )
        .unwrap();
        assert!(document.contains(r#"<w:pStyle w:val="Heading1"/>"#));
        assert!(document.contains(">Guide<"));
        assert!(document.contains("home (README.md)"));
        assert!(document.contains(r#"<w:numId w:val="1"/>"#));
        assert!(document.contains(r#"<w:numId w:val="2"/>"#));
        assert!(document.contains("<w:tbl>"));
        assert!(document.contains("let x = 1;"));

        let mut numbering = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("word/numbering.xml").unwrap(),
            &mut numbering,
        )
        .unwrap();
        assert!(numbering.contains(r#"<w:numFmt w:val="bullet"/>"#));
        assert!(numbering.contains(r#"<w:numFmt w:val="decimal"/>"#));
    }

    #[test]
    fn static_site_helper_sorts_pages_and_writes_index_manifest() {
        let dir = tempfile::tempdir().unwrap();
        write_static_site(
            dir.path(),
            vec![
                SitePage {
                    title: "Beta & Co".to_string(),
                    source_path: "src\\beta.source.md".to_string(),
                    output_path: "site\\beta.html".to_string(),
                    href: "beta.html".to_string(),
                    status: "written".to_string(),
                    diagnostics_count: 0,
                },
                SitePage {
                    title: "Alpha <One>".to_string(),
                    source_path: "src\\alpha.source.md".to_string(),
                    output_path: "site\\alpha.html".to_string(),
                    href: "alpha.html".to_string(),
                    status: "cached".to_string(),
                    diagnostics_count: 1,
                },
            ],
        )
        .unwrap();

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join("proof-site.json")).unwrap(),
        )
        .unwrap();
        let index = std::fs::read_to_string(dir.path().join("index.html")).unwrap();

        assert_eq!(manifest["schema"], "proof.publish.site.v1");
        assert_eq!(manifest["page_count"], 2);
        assert_eq!(manifest["pages"][0]["href"], "alpha.html");
        assert_eq!(manifest["pages"][1]["href"], "beta.html");
        assert!(index.contains("<a href=\"alpha.html\">Alpha &lt;One&gt;</a>"));
        assert!(index.contains("<a href=\"beta.html\">Beta &amp; Co</a>"));
    }
}
