use crate::element::row::RowElement;
use crate::element::{ElementAlign, ElementKind};
use crate::layout::{Align, Direction, LayoutConfig};
use crate::math::MathAlign;

#[derive(Debug, Clone)]
pub struct DirectiveSpan<'a> {
    pub line_start: usize,
    pub line_end: usize,
    pub kind: &'static str,
    pub info_after_backticks: String,
    pub attrs: String,
    pub body: Vec<&'a str>,
}

#[derive(Debug)]
pub(crate) enum Directive {
    Include {
        uri: String,
        /// Optional DaVinci pin ID declared inline. Compile warns if no matching
        /// [[davinci]] entry with this ID exists in proof.toml.
        pin: Option<String>,
        line_start: usize,
        line_end: usize,
    },
    Layout {
        uris: Vec<String>,
        attrs: LayoutAttrs,
        line_start: usize,
        line_end: usize,
    },
    Table {
        uri: String,
        line_start: usize,
        line_end: usize,
    },
    Tree {
        kind: String,
        source: Option<String>,
        inline_body: Vec<String>,
        attrs: TreeAttrs,
        line_start: usize,
        line_end: usize,
    },
    Element {
        kind: String,
        source: Option<String>,
        field: Option<String>,
        inline_value: Option<String>,
        attrs: ElementAttrs,
        line_start: usize,
        line_end: usize,
    },
    Row {
        source_uri: String,
        #[allow(dead_code)]
        var_name: String,
        separator: String,
        declared_width: Option<usize>,
        elements: Vec<RowElement>,
        no_chrome: bool,
        line_start: usize,
        line_end: usize,
    },
    Symbol {
        name: String,
        size: usize,
        #[allow(dead_code)]
        align: String,
        line_start: usize,
        line_end: usize,
    },
    Shape {
        attrs: crate::symbol::shape::ShapeAttrs,
        line_start: usize,
        line_end: usize,
    },
    Region {
        name: String,
        body: Vec<String>,
        line_start: usize,
        line_end: usize,
    },
    Math {
        expr: String,
        width: usize,
        align: crate::math::MathAlign,
        no_chrome: bool,
        line_start: usize,
        line_end: usize,
    },
    Toc {
        source: Option<String>,
        max_depth: usize,
        style: String,
        /// Restrict TOC to headings nested under the heading with this text.
        /// `None` lists every heading in the document.
        section: Option<String>,
        line_start: usize,
        line_end: usize,
    },
    /// proof:xref — cross-reference to a heading in another document.
    /// Renders as "See: [Heading Text](relative-path.md#slug)".
    Xref {
        /// Target URI: `md://path.md#heading-slug` or `md://path.md`
        uri: String,
        /// Optional override label; defaults to the resolved heading text
        label: Option<String>,
        /// Render format: "inline" | "note" | "callout"
        format: String,
        line_start: usize,
        line_end: usize,
    },
    /// proof:blockquote — prose-document block quote.
    ///
    /// Distinct from `proof:quote`, which is slide-only (centered, curly-quoted).
    /// `proof:blockquote` is for prose documents: left-aligned, indented, with
    /// optional attribution on its own trailing line.
    Blockquote {
        /// Body text — multi-line. Blank lines separate paragraphs within the quote.
        text: String,
        /// Optional attribution (rendered as `— Name` on a trailing line).
        attribution: Option<String>,
        /// Render style: "indent" (markdown `> ` lines, default) or "boxed" (ASCII frame).
        style: String,
        line_start: usize,
        line_end: usize,
    },
    Backlinks {
        target: String,
        source: Option<String>,
        format: String,
        line_start: usize,
        line_end: usize,
    },
    Links {
        source_doc: Option<String>,
        status: String,
        source: Option<String>,
        format: String,
        line_start: usize,
        line_end: usize,
    },
    Headings {
        source_doc: String,
        source: Option<String>,
        format: String,
        line_start: usize,
        line_end: usize,
    },
    Frontmatter {
        field: Option<String>,
        value: Option<String>,
        op: String,
        source: Option<String>,
        format: String,
        line_start: usize,
        line_end: usize,
    },
    /// proof:chart — full bar or line chart (distinct from sparkline elements).
    Chart {
        attrs: crate::chart::ChartAttrs,
        /// md:// URI of a data table when source-driven; None for inline body data.
        source: Option<String>,
        /// Column name for category labels when source is set.
        label_field: Option<String>,
        /// Column name for numeric values when source is set.
        value_field: Option<String>,
        /// Inline body text (used when `source` is None). Lines are `label: value` pairs.
        inline_body: String,
        line_start: usize,
        line_end: usize,
    },
}

impl Directive {
    pub(crate) fn line_start(&self) -> usize {
        match self {
            Directive::Include { line_start, .. } => *line_start,
            Directive::Layout { line_start, .. } => *line_start,
            Directive::Table { line_start, .. } => *line_start,
            Directive::Tree { line_start, .. } => *line_start,
            Directive::Element { line_start, .. } => *line_start,
            Directive::Row { line_start, .. } => *line_start,
            Directive::Symbol { line_start, .. } => *line_start,
            Directive::Shape { line_start, .. } => *line_start,
            Directive::Region { line_start, .. } => *line_start,
            Directive::Math { line_start, .. } => *line_start,
            Directive::Toc { line_start, .. } => *line_start,
            Directive::Xref { line_start, .. } => *line_start,
            Directive::Blockquote { line_start, .. } => *line_start,
            Directive::Backlinks { line_start, .. } => *line_start,
            Directive::Links { line_start, .. } => *line_start,
            Directive::Headings { line_start, .. } => *line_start,
            Directive::Frontmatter { line_start, .. } => *line_start,
            Directive::Chart { line_start, .. } => *line_start,
        }
    }

    pub(crate) fn line_end(&self) -> usize {
        match self {
            Directive::Include { line_end, .. } => *line_end,
            Directive::Layout { line_end, .. } => *line_end,
            Directive::Table { line_end, .. } => *line_end,
            Directive::Tree { line_end, .. } => *line_end,
            Directive::Element { line_end, .. } => *line_end,
            Directive::Row { line_end, .. } => *line_end,
            Directive::Symbol { line_end, .. } => *line_end,
            Directive::Shape { line_end, .. } => *line_end,
            Directive::Region { line_end, .. } => *line_end,
            Directive::Math { line_end, .. } => *line_end,
            Directive::Toc { line_end, .. } => *line_end,
            Directive::Xref { line_end, .. } => *line_end,
            Directive::Blockquote { line_end, .. } => *line_end,
            Directive::Backlinks { line_end, .. } => *line_end,
            Directive::Links { line_end, .. } => *line_end,
            Directive::Headings { line_end, .. } => *line_end,
            Directive::Frontmatter { line_end, .. } => *line_end,
            Directive::Chart { line_end, .. } => *line_end,
        }
    }
}

pub(crate) fn collect_directives(source: &str) -> Vec<Directive> {
    let mut directives = Vec::new();
    for span in scan_directive_spans(source) {
        let line_start = span.line_start;
        let line_end = span.line_end;
        let attrs_str = span.attrs;
        let body = span.body;
        match span.kind {
            "include" => {
                let include = parse_include_directive(&attrs_str, &body);
                if let Some(uri) = include.uri {
                    directives.push(Directive::Include {
                        uri,
                        pin: include.pin,
                        line_start,
                        line_end,
                    });
                }
            }
            "layout" => {
                let layout = parse_layout_directive(&attrs_str, &body);
                directives.push(Directive::Layout {
                    uris: layout.uris,
                    attrs: layout.attrs,
                    line_start,
                    line_end,
                });
            }
            "table" => {
                if let Some(uri) = parse_table_directive(&body) {
                    directives.push(Directive::Table {
                        uri,
                        line_start,
                        line_end,
                    });
                }
            }
            "tree" => {
                let tree = parse_tree_directive(&attrs_str, &body);

                directives.push(Directive::Tree {
                    kind: tree.kind,
                    source: tree.source,
                    inline_body: tree.inline_body,
                    attrs: tree.attrs,
                    line_start,
                    line_end,
                });
            }
            "element" => {
                let element = parse_element_directive(&attrs_str, &body);

                directives.push(Directive::Element {
                    kind: element.kind,
                    source: element.source,
                    field: element.field,
                    inline_value: element.inline_value,
                    attrs: element.attrs,
                    line_start,
                    line_end,
                });
            }
            "row" => {
                let row = parse_row_directive(&attrs_str, &body);

                if !row.source_uri.is_empty() {
                    directives.push(Directive::Row {
                        source_uri: row.source_uri,
                        var_name: row.var_name,
                        separator: row.separator,
                        declared_width: row.declared_width,
                        elements: row.elements,
                        no_chrome: row.no_chrome,
                        line_start,
                        line_end,
                    });
                }
            }
            "symbol" => {
                let attrs = parse_symbol_directive(&attrs_str);
                if !attrs.name.is_empty() {
                    directives.push(Directive::Symbol {
                        name: attrs.name,
                        size: attrs.size,
                        align: attrs.align,
                        line_start,
                        line_end,
                    });
                }
            }
            "shape" => {
                let attrs = parse_shape_directive(&attrs_str);
                if !attrs.name.is_empty() {
                    directives.push(Directive::Shape {
                        attrs,
                        line_start,
                        line_end,
                    });
                }
            }
            "region" => {
                let region = parse_region_directive(&attrs_str, &body);
                directives.push(Directive::Region {
                    name: region.name,
                    body: region.body,
                    line_start,
                    line_end,
                });
            }
            "math" => {
                let math = parse_math_directive(&attrs_str, &body);
                directives.push(Directive::Math {
                    expr: math.expr,
                    width: math.width,
                    align: math.align,
                    no_chrome: math.no_chrome,
                    line_start,
                    line_end,
                });
            }
            "toc" => {
                let attrs = parse_toc_directive(&attrs_str, &body);
                directives.push(Directive::Toc {
                    source: attrs.source,
                    max_depth: attrs.max_depth,
                    style: attrs.style,
                    section: attrs.section,
                    line_start,
                    line_end,
                });
            }
            "xref" => {
                let attrs = parse_xref_directive(&attrs_str, &body);
                directives.push(Directive::Xref {
                    uri: attrs.uri,
                    label: attrs.label,
                    format: attrs.format,
                    line_start,
                    line_end,
                });
            }
            "blockquote" => {
                let blockquote = parse_blockquote_directive(&attrs_str, &body);
                directives.push(Directive::Blockquote {
                    text: blockquote.text,
                    attribution: blockquote.attribution,
                    style: blockquote.style,
                    line_start,
                    line_end,
                });
            }
            "backlinks" => {
                let backlinks = parse_backlinks_directive(&attrs_str, &body);
                directives.push(Directive::Backlinks {
                    target: backlinks.target,
                    source: backlinks.source,
                    format: backlinks.format,
                    line_start,
                    line_end,
                });
            }
            "links" => {
                let links = parse_links_directive(&attrs_str, &body);
                directives.push(Directive::Links {
                    source_doc: links.source_doc,
                    status: links.status,
                    source: links.source,
                    format: links.format,
                    line_start,
                    line_end,
                });
            }
            "headings" => {
                let headings = parse_headings_directive(&attrs_str, &body);
                directives.push(Directive::Headings {
                    source_doc: headings.source_doc,
                    source: headings.source,
                    format: headings.format,
                    line_start,
                    line_end,
                });
            }
            "frontmatter" => {
                let frontmatter = parse_frontmatter_directive(&attrs_str);
                directives.push(Directive::Frontmatter {
                    field: frontmatter.field,
                    value: frontmatter.value,
                    op: frontmatter.op,
                    source: frontmatter.source,
                    format: frontmatter.format,
                    line_start,
                    line_end,
                });
            }
            "chart" => {
                let chart = parse_chart_directive(&attrs_str, &body);
                directives.push(Directive::Chart {
                    attrs: chart.attrs,
                    source: chart.source,
                    label_field: chart.label_field,
                    value_field: chart.value_field,
                    inline_body: chart.inline_body,
                    line_start,
                    line_end,
                });
            }
            _ => {}
        }
    }
    directives
}

pub fn parse_directives(source: &str) -> Vec<(usize, usize, String, String)> {
    scan_directive_spans(source)
        .into_iter()
        .map(|span| {
            (
                span.line_start,
                span.line_end,
                span.kind.to_string(),
                span.body.join("\n"),
            )
        })
        .collect()
}

pub(crate) fn directive_header_attrs<'a>(info_after_backticks: &'a str, kind: &str) -> &'a str {
    info_after_backticks
        .strip_prefix("proof:")
        .and_then(|rest| rest.strip_prefix(kind))
        .unwrap_or("")
        .trim()
}

/// Parsed attributes from a proof:element directive.
#[derive(Debug, Default)]
pub struct ElementAttrs {
    pub width: Option<usize>,
    pub align: String,
    pub format: String,
    pub no_chrome: bool,
    pub max: Option<f64>,
    pub fill: char,
    pub empty: char,
}

impl ElementAttrs {
    pub(crate) fn parse(attrs_str: &str) -> Self {
        let mut out = ElementAttrs {
            align: "left".to_string(),
            format: "{}".to_string(),
            fill: '█',
            empty: '░',
            ..Default::default()
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            let tok_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let tok = &rest[..tok_end];

            if let Some(eq) = tok.find('=') {
                let key = tok[..eq].trim();
                let after_eq = &tok[eq + 1..];
                let val_start = &rest[eq + 1..];
                let (val, consumed) = if val_start.starts_with('"') {
                    if let Some(close) = val_start[1..].find('"') {
                        (&val_start[1..close + 1], eq + 1 + close + 2)
                    } else {
                        (after_eq, tok_end)
                    }
                } else {
                    (after_eq, tok_end)
                };
                match key {
                    "width" => out.width = val.parse().ok(),
                    "align" => out.align = val.to_string(),
                    "format" => out.format = val.to_string(),
                    "max" => out.max = val.parse().ok(),
                    "fill" => out.fill = val.chars().next().unwrap_or('█'),
                    "empty" => out.empty = val.chars().next().unwrap_or('░'),
                    "no-chrome" => out.no_chrome = matches!(val, "true" | "1" | ""),
                    _ => {}
                }
                rest = rest[consumed..].trim_start();
            } else {
                if tok == "no-chrome" {
                    out.no_chrome = true;
                }
                rest = rest[tok_end..].trim_start();
            }
        }
        out
    }
}

pub(crate) struct ElementDirective {
    pub(crate) kind: String,
    pub(crate) source: Option<String>,
    pub(crate) field: Option<String>,
    pub(crate) inline_value: Option<String>,
    pub(crate) attrs: ElementAttrs,
}

pub(crate) fn parse_element_directive(attrs_str: &str, body: &[&str]) -> ElementDirective {
    let kind = extract_attr_value(attrs_str, "kind").unwrap_or_else(|| "value".to_string());
    let field = extract_attr_value(attrs_str, "field");
    let inline_value = extract_attr_value(attrs_str, "value");
    let attrs = ElementAttrs::parse(attrs_str);
    let source = body.iter().find_map(|l| {
        let t = l.trim();
        if t.starts_with("md://") {
            Some(t.to_string())
        } else {
            None
        }
    });

    ElementDirective {
        kind,
        source,
        field,
        inline_value,
        attrs,
    }
}

/// Parsed attributes from a proof:tree directive.
#[derive(Debug, Default)]
pub struct TreeAttrs {
    pub name: Option<String>,
    pub parent: Option<String>,
    pub label: Option<String>,
    pub format: String,
    pub indent_width: usize,
    pub root: Option<String>,
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>,
    pub stub: bool,
}

impl TreeAttrs {
    pub(crate) fn parse(attrs_str: &str) -> Self {
        let mut out = TreeAttrs {
            format: "table".to_string(),
            indent_width: 4,
            ..Default::default()
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            if let Some(eq) = rest.find('=') {
                let key = rest[..eq].trim();
                rest = &rest[eq + 1..];
                let (val, next) = if rest.starts_with('"') {
                    if let Some(close) = rest[1..].find('"') {
                        (&rest[1..close + 1], &rest[close + 2..])
                    } else {
                        ("", "")
                    }
                } else {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (&rest[..end], &rest[end..])
                };
                match key {
                    "name" => out.name = Some(val.to_string()),
                    "parent" => out.parent = Some(val.to_string()),
                    "label" => out.label = Some(val.to_string()),
                    "format" => out.format = val.to_string(),
                    "indent-width" => out.indent_width = val.parse().unwrap_or(4),
                    "root" => out.root = Some(val.to_string()),
                    "max-depth" => out.max_depth = val.parse().ok(),
                    "exclude" => {
                        out.exclude = val.split(',').map(|s| s.trim().to_string()).collect()
                    }
                    "stub" => out.stub = val == "true" || val == "1",
                    _ => {}
                }
                rest = next.trim_start();
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                rest = rest[end..].trim_start();
            }
        }
        out
    }
}

pub(crate) struct TreeDirective {
    pub(crate) kind: String,
    pub(crate) source: Option<String>,
    pub(crate) inline_body: Vec<String>,
    pub(crate) attrs: TreeAttrs,
}

pub(crate) fn parse_tree_directive(attrs_str: &str, body: &[&str]) -> TreeDirective {
    let kind = attrs_str
        .split_whitespace()
        .find_map(|tok| {
            if tok.starts_with("kind=") {
                Some(
                    tok.strip_prefix("kind=")
                        .unwrap_or("dirtree")
                        .trim_matches('"')
                        .to_string(),
                )
            } else if !tok.contains('=') {
                Some(tok.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "dirtree".to_string());

    let attrs = TreeAttrs::parse(attrs_str);
    let source_from_attrs = extract_attr_value(attrs_str, "source")
        .filter(|s| s.starts_with("md://") || s.contains('/'));
    let source = source_from_attrs.or_else(|| {
        body.iter().find_map(|l| {
            let t = l.trim();
            if t.starts_with("md://") {
                Some(t.to_string())
            } else {
                None
            }
        })
    });
    let inline_body = body
        .iter()
        .filter(|l| !l.trim().starts_with("md://") && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    TreeDirective {
        kind,
        source,
        inline_body,
        attrs,
    }
}

#[derive(Debug, Default)]
pub(crate) struct LayoutAttrs {
    pub(crate) gap: usize,
    pub(crate) align: String,
    pub(crate) labels: Vec<String>,
    pub(crate) cols: Option<usize>,
    pub(crate) width: usize,
    pub(crate) direction: String,
    pub(crate) border: bool,
}

impl LayoutAttrs {
    pub(crate) fn parse(attrs_str: &str) -> Self {
        let mut out = LayoutAttrs {
            gap: 3,
            align: "top".to_string(),
            labels: Vec::new(),
            cols: None,
            width: 120,
            direction: "horizontal".to_string(),
            border: false,
        };
        let mut rest = attrs_str.trim();
        while !rest.is_empty() {
            if let Some(eq_pos) = rest.find('=') {
                let key = rest[..eq_pos].trim();
                rest = &rest[eq_pos + 1..];
                let (val, next) = if rest.starts_with('"') {
                    if let Some(close) = rest[1..].find('"') {
                        (&rest[1..close + 1], &rest[close + 2..])
                    } else {
                        ("", "")
                    }
                } else {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (&rest[..end], &rest[end..])
                };
                match key {
                    "gap" => out.gap = val.parse().unwrap_or(3),
                    "align" => out.align = val.to_string(),
                    "labels" => out.labels = val.split(',').map(|s| s.to_string()).collect(),
                    "cols" => out.cols = val.parse().ok(),
                    "width" => out.width = val.parse().unwrap_or(120),
                    "direction" => out.direction = val.to_string(),
                    "border" => out.border = matches!(val, "true" | "1" | ""),
                    _ => {}
                }
                rest = next.trim_start();
            } else {
                let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                let key = rest[..end].trim();
                if key == "border" {
                    out.border = true;
                }
                rest = rest[end..].trim_start();
            }
        }
        out
    }

    pub(crate) fn to_layout_config(&self) -> LayoutConfig {
        LayoutConfig {
            gap: self.gap,
            align: Align::parse(&self.align).unwrap_or(Align::Top),
            labels: self.labels.clone(),
            cols: self.cols,
            width: self.width,
            direction: Direction::parse(&self.direction).unwrap_or(Direction::Horizontal),
            border: self.border,
        }
    }
}

pub(crate) struct LayoutDirective {
    pub(crate) uris: Vec<String>,
    pub(crate) attrs: LayoutAttrs,
}

pub(crate) fn parse_layout_directive(attrs_str: &str, body: &[&str]) -> LayoutDirective {
    let attrs = LayoutAttrs::parse(attrs_str);
    let uris = body
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    LayoutDirective { uris, attrs }
}

pub(crate) fn parse_chart_attrs(attrs_str: &str) -> crate::chart::ChartAttrs {
    let kind = extract_attr_value(attrs_str, "kind")
        .as_deref()
        .and_then(crate::chart::ChartKind::parse)
        .unwrap_or(crate::chart::ChartKind::Bar);
    let width = extract_attr_value(attrs_str, "width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let height = extract_attr_value(attrs_str, "height")
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let title = extract_attr_value(attrs_str, "title");
    let x_label = extract_attr_value(attrs_str, "x-label")
        .or_else(|| extract_attr_value(attrs_str, "xlabel"));
    let y_label = extract_attr_value(attrs_str, "y-label")
        .or_else(|| extract_attr_value(attrs_str, "ylabel"));
    let max = extract_attr_value(attrs_str, "max").and_then(|s| s.parse().ok());
    let no_chrome = extract_attr_value(attrs_str, "no-chrome")
        .map(|s| s == "true")
        .unwrap_or(false);

    crate::chart::ChartAttrs {
        kind,
        width,
        height,
        title,
        x_label,
        y_label,
        max,
        no_chrome,
    }
}

pub(crate) struct ChartDirective {
    pub(crate) attrs: crate::chart::ChartAttrs,
    pub(crate) source: Option<String>,
    pub(crate) label_field: Option<String>,
    pub(crate) value_field: Option<String>,
    pub(crate) inline_body: String,
}

pub(crate) fn parse_chart_directive(attrs_str: &str, body: &[&str]) -> ChartDirective {
    let attrs = parse_chart_attrs(attrs_str);
    let source = extract_attr_value(attrs_str, "source");
    let label_field = extract_attr_value(attrs_str, "label-field")
        .or_else(|| extract_attr_value(attrs_str, "label_field"));
    let value_field = extract_attr_value(attrs_str, "value-field")
        .or_else(|| extract_attr_value(attrs_str, "value_field"));
    let inline_body = body.join("\n");

    ChartDirective {
        attrs,
        source,
        label_field,
        value_field,
        inline_body,
    }
}

pub(crate) struct BacklinksDirective {
    pub(crate) target: String,
    pub(crate) source: Option<String>,
    pub(crate) format: String,
}

pub(crate) fn parse_backlinks_directive(attrs_str: &str, body: &[&str]) -> BacklinksDirective {
    let target = extract_attr_value(attrs_str, "target")
        .or_else(|| extract_attr_value(attrs_str, "uri"))
        .or_else(|| extract_attr_value(attrs_str, "source"))
        .or_else(|| first_non_empty_body_line(body))
        .unwrap_or_default();
    let source = extract_attr_value(attrs_str, "side-info")
        .or_else(|| extract_attr_value(attrs_str, "side_info"));
    let format = extract_attr_value(attrs_str, "format").unwrap_or_else(|| "list".to_string());

    BacklinksDirective {
        target,
        source,
        format,
    }
}

pub(crate) struct LinksDirective {
    pub(crate) source_doc: Option<String>,
    pub(crate) status: String,
    pub(crate) source: Option<String>,
    pub(crate) format: String,
}

pub(crate) fn parse_links_directive(attrs_str: &str, body: &[&str]) -> LinksDirective {
    let source_doc =
        extract_attr_value(attrs_str, "source").or_else(|| first_non_empty_body_line(body));
    let source = extract_attr_value(attrs_str, "side-info")
        .or_else(|| extract_attr_value(attrs_str, "side_info"));
    let status = extract_attr_value(attrs_str, "status").unwrap_or_else(|| "all".to_string());
    let format = extract_attr_value(attrs_str, "format").unwrap_or_else(|| "list".to_string());

    LinksDirective {
        source_doc,
        status,
        source,
        format,
    }
}

pub(crate) struct HeadingsDirective {
    pub(crate) source_doc: String,
    pub(crate) source: Option<String>,
    pub(crate) format: String,
}

pub(crate) fn parse_headings_directive(attrs_str: &str, body: &[&str]) -> HeadingsDirective {
    let source_doc = extract_attr_value(attrs_str, "source")
        .or_else(|| extract_attr_value(attrs_str, "target"))
        .or_else(|| extract_attr_value(attrs_str, "uri"))
        .or_else(|| first_non_empty_body_line(body))
        .unwrap_or_default();
    let source = extract_attr_value(attrs_str, "side-info")
        .or_else(|| extract_attr_value(attrs_str, "side_info"));
    let format = extract_attr_value(attrs_str, "format").unwrap_or_else(|| "list".to_string());

    HeadingsDirective {
        source_doc,
        source,
        format,
    }
}

pub(crate) struct FrontmatterDirective {
    pub(crate) field: Option<String>,
    pub(crate) value: Option<String>,
    pub(crate) op: String,
    pub(crate) source: Option<String>,
    pub(crate) format: String,
}

pub(crate) fn parse_frontmatter_directive(attrs_str: &str) -> FrontmatterDirective {
    let field =
        extract_attr_value(attrs_str, "field").or_else(|| extract_attr_value(attrs_str, "key"));
    let value = extract_attr_value(attrs_str, "value");
    let op = extract_attr_value(attrs_str, "op").unwrap_or_else(|| "has".to_string());
    let source = extract_attr_value(attrs_str, "side-info")
        .or_else(|| extract_attr_value(attrs_str, "side_info"));
    let format = extract_attr_value(attrs_str, "format").unwrap_or_else(|| "list".to_string());

    FrontmatterDirective {
        field,
        value,
        op,
        source,
        format,
    }
}

fn first_non_empty_body_line(body: &[&str]) -> Option<String> {
    body.iter().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) struct MathAttrs {
    pub(crate) width: usize,
    pub(crate) align: MathAlign,
    pub(crate) no_chrome: bool,
}

pub(crate) fn parse_math_attrs(attrs_str: &str) -> MathAttrs {
    let width = extract_attr_value(attrs_str, "width")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let align = match extract_attr_value(attrs_str, "align").as_deref() {
        Some("left") => MathAlign::Left,
        Some("right") => MathAlign::Right,
        _ => MathAlign::Center,
    };
    let no_chrome = extract_attr_value(attrs_str, "no-chrome")
        .map(|s| s == "true")
        .unwrap_or(false);

    MathAttrs {
        width,
        align,
        no_chrome,
    }
}

pub(crate) struct MathDirective {
    pub(crate) expr: String,
    pub(crate) width: usize,
    pub(crate) align: MathAlign,
    pub(crate) no_chrome: bool,
}

pub(crate) fn parse_math_directive(attrs_str: &str, body: &[&str]) -> MathDirective {
    let attrs = parse_math_attrs(attrs_str);
    let expr = body.join("\n");

    MathDirective {
        expr,
        width: attrs.width,
        align: attrs.align,
        no_chrome: attrs.no_chrome,
    }
}

pub(crate) struct TocAttrs {
    pub(crate) source: Option<String>,
    pub(crate) max_depth: usize,
    pub(crate) style: String,
    pub(crate) section: Option<String>,
}

pub(crate) fn parse_toc_attrs(attrs_str: &str, body: &[&str]) -> TocAttrs {
    let source = extract_attr_value(attrs_str, "source").or_else(|| {
        body.iter().find_map(|l| {
            let t = l.trim();
            if t.starts_with("md://") {
                Some(t.to_string())
            } else {
                None
            }
        })
    });
    let max_depth = extract_attr_value(attrs_str, "max-depth")
        .or_else(|| extract_attr_value(attrs_str, "max_depth"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let style = extract_attr_value(attrs_str, "style").unwrap_or_else(|| "list".to_string());
    let section = extract_attr_value(attrs_str, "section");

    TocAttrs {
        source,
        max_depth,
        style,
        section,
    }
}

pub(crate) fn parse_toc_directive(attrs_str: &str, body: &[&str]) -> TocAttrs {
    parse_toc_attrs(attrs_str, body)
}

pub(crate) struct XrefAttrs {
    pub(crate) uri: String,
    pub(crate) label: Option<String>,
    pub(crate) format: String,
}

pub(crate) fn parse_xref_attrs(attrs_str: &str, body: &[&str]) -> XrefAttrs {
    let uri = extract_attr_value(attrs_str, "uri")
        .or_else(|| extract_attr_value(attrs_str, "source"))
        .or_else(|| {
            body.iter().find_map(|l| {
                let t = l.trim();
                if t.starts_with("md://") {
                    Some(t.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();
    let label = extract_attr_value(attrs_str, "label");
    let format = extract_attr_value(attrs_str, "format").unwrap_or_else(|| "inline".to_string());

    XrefAttrs { uri, label, format }
}

pub(crate) fn parse_xref_directive(attrs_str: &str, body: &[&str]) -> XrefAttrs {
    parse_xref_attrs(attrs_str, body)
}

pub(crate) struct BlockquoteAttrs {
    pub(crate) attribution: Option<String>,
    pub(crate) style: String,
}

pub(crate) fn parse_blockquote_attrs(attrs_str: &str) -> BlockquoteAttrs {
    let attribution = extract_attr_value(attrs_str, "attribution")
        .or_else(|| extract_attr_value(attrs_str, "by"));
    let style = extract_attr_value(attrs_str, "style").unwrap_or_else(|| "indent".to_string());

    BlockquoteAttrs { attribution, style }
}

pub(crate) struct BlockquoteDirective {
    pub(crate) text: String,
    pub(crate) attribution: Option<String>,
    pub(crate) style: String,
}

pub(crate) fn parse_blockquote_directive(attrs_str: &str, body: &[&str]) -> BlockquoteDirective {
    let attrs = parse_blockquote_attrs(attrs_str);
    let text = body.join("\n");

    BlockquoteDirective {
        text,
        attribution: attrs.attribution,
        style: attrs.style,
    }
}

pub(crate) struct SymbolAttrs {
    pub(crate) name: String,
    pub(crate) size: usize,
    pub(crate) align: String,
}

pub(crate) fn parse_symbol_attrs(attrs_str: &str) -> SymbolAttrs {
    let name = extract_attr_value(attrs_str, "name").unwrap_or_default();
    let size = extract_attr_value(attrs_str, "size")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);
    let align = extract_attr_value(attrs_str, "align").unwrap_or_else(|| "left".to_string());

    SymbolAttrs { name, size, align }
}

pub(crate) fn parse_symbol_directive(attrs_str: &str) -> SymbolAttrs {
    parse_symbol_attrs(attrs_str)
}

pub(crate) fn parse_shape_attrs(attrs_str: &str) -> crate::symbol::shape::ShapeAttrs {
    crate::symbol::shape::ShapeAttrs::parse(attrs_str)
}

pub(crate) fn parse_shape_directive(attrs_str: &str) -> crate::symbol::shape::ShapeAttrs {
    parse_shape_attrs(attrs_str)
}

pub(crate) struct RegionDirective {
    pub(crate) name: String,
    pub(crate) body: Vec<String>,
}

pub(crate) fn parse_region_directive(attrs_str: &str, body: &[&str]) -> RegionDirective {
    let name = extract_attr_value(attrs_str, "name").unwrap_or_default();
    let body = body.iter().map(|s| s.to_string()).collect();

    RegionDirective { name, body }
}

pub(crate) struct IncludeDirective {
    pub(crate) uri: Option<String>,
    pub(crate) pin: Option<String>,
}

pub(crate) fn parse_include_directive(attrs_str: &str, body: &[&str]) -> IncludeDirective {
    let pin = extract_attr_value(attrs_str, "pin");
    let uri = body.iter().find_map(|l| {
        let t = l.trim();
        if !t.is_empty() && !t.starts_with("pin=") {
            Some(t.to_string())
        } else {
            None
        }
    });

    IncludeDirective { uri, pin }
}

pub(crate) fn parse_table_uri(body: &[&str]) -> Option<String> {
    body.iter().find_map(|l| {
        let t = l.trim();
        if t.starts_with("md://") {
            Some(t.to_string())
        } else {
            None
        }
    })
}

pub(crate) fn parse_table_directive(body: &[&str]) -> Option<String> {
    parse_table_uri(body)
}

pub(crate) struct RowDirective {
    pub(crate) source_uri: String,
    pub(crate) var_name: String,
    pub(crate) separator: String,
    pub(crate) declared_width: Option<usize>,
    pub(crate) elements: Vec<RowElement>,
    pub(crate) no_chrome: bool,
}

pub(crate) fn parse_row_directive(attrs_str: &str, body: &[&str]) -> RowDirective {
    let (var_name, source_uri) = parse_foreach(attrs_str);
    let separator = extract_attr_value(attrs_str, "separator").unwrap_or_else(|| " ".to_string());
    let declared_width = extract_attr_value(attrs_str, "width").and_then(|v| v.parse().ok());
    let no_chrome = attrs_str
        .split_whitespace()
        .any(|t| t == "no-chrome" || t == "no-chrome=true" || t == "no-chrome=1");
    let elements = body
        .iter()
        .filter_map(|l| parse_row_element_line(l.trim()))
        .collect();

    RowDirective {
        source_uri,
        var_name,
        separator,
        declared_width,
        elements,
        no_chrome,
    }
}

pub fn scan_directive_spans(source: &str) -> Vec<DirectiveSpan<'_>> {
    let lines: Vec<&str> = source.lines().collect();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if let Some(kind) = proof_directive_kind(trimmed) {
            let line_start = i;
            let info_after_backticks = trimmed[3..].to_string();
            let attrs = directive_header_attrs(&info_after_backticks, kind).to_string();
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() {
                let line = lines[i].trim();
                if line == "```" || line == "~~~" {
                    break;
                }
                body.push(lines[i]);
                i += 1;
            }
            spans.push(DirectiveSpan {
                line_start,
                line_end: i,
                kind,
                info_after_backticks,
                attrs,
                body,
            });
        }
        i += 1;
    }
    spans
}

pub fn proof_directive_kind(line: &str) -> Option<&'static str> {
    let line = line.trim_start();
    if !line.starts_with("```proof:") {
        return None;
    }
    let rest = &line[9..]; // after "```proof:"
    if rest.starts_with("include") {
        Some("include")
    } else if rest.starts_with("layout") {
        Some("layout")
    } else if rest.starts_with("table") {
        Some("table")
    } else if rest.starts_with("tree") {
        Some("tree")
    } else if rest.starts_with("element") {
        Some("element")
    } else if rest.starts_with("row") {
        Some("row")
    } else if rest.starts_with("symbol") {
        Some("symbol")
    } else if rest.starts_with("shape") {
        Some("shape")
    } else if rest.starts_with("region") {
        Some("region")
    } else if rest.starts_with("math") {
        Some("math")
    } else if rest.starts_with("toc") {
        Some("toc")
    } else if rest.starts_with("xref") {
        Some("xref")
    } else if rest.starts_with("blockquote") {
        Some("blockquote")
    } else if rest.starts_with("backlinks") {
        Some("backlinks")
    } else if rest.starts_with("links") {
        Some("links")
    } else if rest.starts_with("headings") {
        Some("headings")
    } else if rest.starts_with("frontmatter") {
        Some("frontmatter")
    } else if rest.starts_with("chart") {
        Some("chart")
    } else if rest.starts_with("numbered-list") || rest.starts_with("ol") {
        Some("ol")
    } else {
        None
    }
}

/// Extract a quoted or unquoted value for `key=` from a directive attribute string.
pub fn extract_attr_value(attrs: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    let mut rest = attrs;
    while !rest.is_empty() {
        if let Some(pos) = rest.find(&prefix) {
            // Ensure it's a word boundary (not mid-identifier)
            if pos > 0 {
                let prev = rest.as_bytes()[pos - 1] as char;
                if prev.is_alphanumeric() || prev == '-' || prev == '_' {
                    rest = &rest[pos + 1..];
                    continue;
                }
            }
            let after = &rest[pos + prefix.len()..];
            let val = if let Some(after_quote) = after.strip_prefix('"') {
                after_quote
                    .find('"')
                    .map(|end| after_quote[..end].to_string())
            } else {
                let end = after.find(char::is_whitespace).unwrap_or(after.len());
                if end > 0 {
                    Some(after[..end].to_string())
                } else {
                    None
                }
            };
            return val;
        } else {
            break;
        }
    }
    None
}

/// Parse `foreach=VAR in URI` from the info string after `proof:row`.
/// Returns (var_name, source_uri). Both empty strings on parse failure.
pub(crate) fn parse_foreach(info: &str) -> (String, String) {
    let mut var_name = String::new();
    let mut source_uri = String::new();

    if let Some(s) = extract_attr_value(info, "source") {
        if s.starts_with("md://") || s.contains('/') {
            source_uri = s;
        }
    }

    for tok in info.split_whitespace() {
        if let Some(var) = tok.strip_prefix("foreach=") {
            var_name = var.to_string();
        } else if tok.starts_with("md://") && source_uri.is_empty() {
            source_uri = tok.to_string();
        }
    }
    (var_name, source_uri)
}

/// Parse a body line of the form `proof:element kind=X field=Y width=N ...`.
pub(crate) fn parse_row_element_line(line: &str) -> Option<RowElement> {
    let rest = line.strip_prefix("proof:element")?.trim();
    let attrs = ElementAttrs::parse(rest);
    let kind_str = extract_attr_value(rest, "kind").unwrap_or_else(|| "value".to_string());
    let kind = ElementKind::parse(&kind_str)?;
    let field = extract_attr_value(rest, "field").unwrap_or_default();
    let width = attrs.width.unwrap_or(0);
    if field.is_empty() || width == 0 {
        return None;
    }
    Some(RowElement {
        kind,
        field,
        width,
        align: ElementAlign::parse(&attrs.align),
        format: attrs.format,
        max: attrs.max,
        fill_char: attrs.fill,
        empty_char: attrs.empty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_directive_kinds() {
        assert_eq!(proof_directive_kind("```proof:include"), Some("include"));
        assert_eq!(
            proof_directive_kind("```proof:layout gap=4"),
            Some("layout")
        );
        assert_eq!(proof_directive_kind("```proof:numbered-list"), Some("ol"));
        assert_eq!(proof_directive_kind("```proof:ol"), Some("ol"));
        assert_eq!(
            proof_directive_kind("  ```proof:chart kind=bar"),
            Some("chart")
        );
        assert_eq!(proof_directive_kind("```proof:links"), Some("links"));
        assert_eq!(
            proof_directive_kind("```proof:backlinks target=README.md"),
            Some("backlinks")
        );
        assert_eq!(
            proof_directive_kind("```proof:headings source=README.md"),
            Some("headings")
        );
        assert_eq!(
            proof_directive_kind("```proof:frontmatter field=tags"),
            Some("frontmatter")
        );
    }

    #[test]
    fn ignores_unknown_or_non_proof_fences() {
        assert_eq!(proof_directive_kind("```rust"), None);
        assert_eq!(proof_directive_kind("```proof:unknown"), None);
        assert_eq!(proof_directive_kind("~~~proof:include"), None);
    }

    #[test]
    fn scans_directive_spans_with_body_and_closing_line() {
        let source = "# Doc\n\n```proof:include pin=arch\nmd://figures/arch.md\n```\n\nAfter\n";

        let spans = scan_directive_spans(source);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].line_start, 2);
        assert_eq!(spans[0].line_end, 4);
        assert_eq!(spans[0].kind, "include");
        assert_eq!(spans[0].info_after_backticks, "proof:include pin=arch");
        assert_eq!(spans[0].attrs, "pin=arch");
        assert_eq!(spans[0].body, vec!["md://figures/arch.md"]);
    }

    #[test]
    fn scans_multiple_directive_spans() {
        let source = "```proof:math\nx\n```\n\n```proof:chart kind=bar\nA: 1\n```\n";

        let spans = scan_directive_spans(source);

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].kind, "math");
        assert_eq!(spans[0].attrs, "");
        assert_eq!(spans[1].kind, "chart");
        assert_eq!(spans[1].attrs, "kind=bar");
    }

    #[test]
    fn extracts_quoted_and_unquoted_attr_values() {
        let attrs = "kind=label value=\"Hello world\" width=12";

        assert_eq!(extract_attr_value(attrs, "kind"), Some("label".to_string()));
        assert_eq!(
            extract_attr_value(attrs, "value"),
            Some("Hello world".to_string())
        );
        assert_eq!(extract_attr_value(attrs, "width"), Some("12".to_string()));
    }

    #[test]
    fn slices_directive_header_attrs() {
        assert_eq!(
            directive_header_attrs("proof:element kind=value width=4", "element"),
            "kind=value width=4"
        );
        assert_eq!(
            directive_header_attrs("proof:blockquote attribution=\"Ada\"", "blockquote"),
            "attribution=\"Ada\""
        );
        assert_eq!(directive_header_attrs("proof:table", "table"), "");
        assert_eq!(directive_header_attrs("proof:table", "toc"), "");
        assert_eq!(directive_header_attrs("not-proof:table", "table"), "");
    }

    #[test]
    fn extract_attr_value_respects_word_boundaries() {
        let attrs = "label-field=name field=value";

        assert_eq!(
            extract_attr_value(attrs, "field"),
            Some("value".to_string())
        );
        assert_eq!(
            extract_attr_value(attrs, "label-field"),
            Some("name".to_string())
        );
        assert_eq!(extract_attr_value(attrs, "missing"), None);
    }

    #[test]
    fn parses_crop_side_info_directives() {
        let source = r#"```proof:links source="README.md" status=broken format=count side-info="reports/links.json"
```

```proof:backlinks target="README.md" format=table
```

```proof:headings format=count
README.md
```

```proof:frontmatter field=tags value=guide op=has format=table side_info="reports/frontmatter.json"
```"#;

        let dirs = collect_directives(source);

        assert_eq!(dirs.len(), 4);
        match &dirs[0] {
            Directive::Links {
                source_doc,
                status,
                source,
                format,
                ..
            } => {
                assert_eq!(source_doc.as_deref(), Some("README.md"));
                assert_eq!(status, "broken");
                assert_eq!(source.as_deref(), Some("reports/links.json"));
                assert_eq!(format, "count");
            }
            other => panic!("expected links directive, got {other:?}"),
        }
        match &dirs[1] {
            Directive::Backlinks { target, format, .. } => {
                assert_eq!(target, "README.md");
                assert_eq!(format, "table");
            }
            other => panic!("expected backlinks directive, got {other:?}"),
        }
        match &dirs[2] {
            Directive::Headings {
                source_doc, format, ..
            } => {
                assert_eq!(source_doc, "README.md");
                assert_eq!(format, "count");
            }
            other => panic!("expected headings directive, got {other:?}"),
        }
        match &dirs[3] {
            Directive::Frontmatter {
                field,
                value,
                op,
                source,
                format,
                ..
            } => {
                assert_eq!(field.as_deref(), Some("tags"));
                assert_eq!(value.as_deref(), Some("guide"));
                assert_eq!(op, "has");
                assert_eq!(source.as_deref(), Some("reports/frontmatter.json"));
                assert_eq!(format, "table");
            }
            other => panic!("expected frontmatter directive, got {other:?}"),
        }
    }

    #[test]
    fn parses_element_attrs_with_defaults_and_flags() {
        let attrs = ElementAttrs::parse(
            "width=12 align=right format=\"{:.1}\" max=99 fill=# empty=. no-chrome",
        );

        assert_eq!(attrs.width, Some(12));
        assert_eq!(attrs.align, "right");
        assert_eq!(attrs.format, "{:.1}");
        assert_eq!(attrs.max, Some(99.0));
        assert_eq!(attrs.fill, '#');
        assert_eq!(attrs.empty, '.');
        assert!(attrs.no_chrome);

        let defaults = ElementAttrs::parse("");
        assert_eq!(defaults.width, None);
        assert_eq!(defaults.align, "left");
        assert_eq!(defaults.format, "{}");
        assert_eq!(defaults.fill, '█');
        assert_eq!(defaults.empty, '░');
        assert!(!defaults.no_chrome);
    }

    #[test]
    fn parses_element_directive_kind_source_field_and_inline_value() {
        let sourced = parse_element_directive(
            "kind=sparkline field=trend width=10 no-chrome",
            &["md://stats.md#table:0"],
        );

        assert_eq!(sourced.kind, "sparkline");
        assert_eq!(sourced.source.as_deref(), Some("md://stats.md#table:0"));
        assert_eq!(sourced.field.as_deref(), Some("trend"));
        assert_eq!(sourced.inline_value, None);
        assert_eq!(sourced.attrs.width, Some(10));
        assert!(sourced.attrs.no_chrome);

        let inline = parse_element_directive("value=\"42\" width=4", &[]);
        assert_eq!(inline.kind, "value");
        assert_eq!(inline.source, None);
        assert_eq!(inline.inline_value.as_deref(), Some("42"));
    }

    #[test]
    fn parses_tree_attrs_with_defaults_and_lists() {
        let attrs = TreeAttrs::parse(
            "name=\"Employee\" parent=Manager label=Name format=json indent-width=2 root=src max-depth=3 exclude=target,.git stub=true",
        );

        assert_eq!(attrs.name.as_deref(), Some("Employee"));
        assert_eq!(attrs.parent.as_deref(), Some("Manager"));
        assert_eq!(attrs.label.as_deref(), Some("Name"));
        assert_eq!(attrs.format, "json");
        assert_eq!(attrs.indent_width, 2);
        assert_eq!(attrs.root.as_deref(), Some("src"));
        assert_eq!(attrs.max_depth, Some(3));
        assert_eq!(
            attrs.exclude,
            vec!["target".to_string(), ".git".to_string()]
        );
        assert!(attrs.stub);

        let defaults = TreeAttrs::parse("");
        assert_eq!(defaults.format, "table");
        assert_eq!(defaults.indent_width, 4);
        assert_eq!(defaults.max_depth, None);
        assert!(!defaults.stub);
    }

    #[test]
    fn parses_tree_directive_kind_source_and_inline_body() {
        let attrs_source = parse_tree_directive(
            "kind=org source=md://people.md#table:0 name=\"Employee\"",
            &["CEO", "  CTO"],
        );
        assert_eq!(attrs_source.kind, "org");
        assert_eq!(
            attrs_source.source.as_deref(),
            Some("md://people.md#table:0")
        );
        assert_eq!(attrs_source.attrs.name.as_deref(), Some("Employee"));
        assert_eq!(attrs_source.inline_body, vec!["CEO", "  CTO"]);

        let bare_kind = parse_tree_directive("taxonomy", &["md://terms.md", "ignored inline"]);
        assert_eq!(bare_kind.kind, "taxonomy");
        assert_eq!(bare_kind.source.as_deref(), Some("md://terms.md"));
        assert_eq!(bare_kind.inline_body, vec!["ignored inline"]);

        let defaults = parse_tree_directive("", &["Root", "  Leaf"]);
        assert_eq!(defaults.kind, "dirtree");
        assert_eq!(defaults.source, None);
        assert_eq!(defaults.inline_body, vec!["Root", "  Leaf"]);
    }

    #[test]
    fn parses_layout_attrs_with_defaults_and_flags() {
        let attrs =
            LayoutAttrs::parse("gap=2 align=center labels=\"Go,Rust,C#\" cols=3 width=200 border");

        assert_eq!(attrs.gap, 2);
        assert_eq!(attrs.align, "center");
        assert_eq!(attrs.labels, vec!["Go", "Rust", "C#"]);
        assert_eq!(attrs.cols, Some(3));
        assert_eq!(attrs.width, 200);
        assert!(attrs.border);

        let defaults = LayoutAttrs::parse("");
        assert_eq!(defaults.gap, 3);
        assert_eq!(defaults.align, "top");
        assert_eq!(defaults.labels, Vec::<String>::new());
        assert_eq!(defaults.cols, None);
        assert_eq!(defaults.width, 120);
        assert_eq!(defaults.direction, "horizontal");
        assert!(!defaults.border);
    }

    #[test]
    fn parses_layout_directive_attrs_and_body_uris() {
        let layout = parse_layout_directive(
            "gap=2 align=center labels=\"A,B\" cols=2 border",
            &["md://a.md", "", "  md://b.md  "],
        );

        assert_eq!(layout.uris, vec!["md://a.md", "md://b.md"]);
        assert_eq!(layout.attrs.gap, 2);
        assert_eq!(layout.attrs.align, "center");
        assert_eq!(layout.attrs.labels, vec!["A", "B"]);
        assert_eq!(layout.attrs.cols, Some(2));
        assert!(layout.attrs.border);
    }

    #[test]
    fn parses_chart_attrs_with_defaults_and_aliases() {
        let attrs = parse_chart_attrs(
            "kind=line width=72 height=12 title=\"Velocity\" xlabel=Time y-label=Speed max=100 no-chrome=true",
        );

        assert!(matches!(attrs.kind, crate::chart::ChartKind::Line));
        assert_eq!(attrs.width, 72);
        assert_eq!(attrs.height, 12);
        assert_eq!(attrs.title.as_deref(), Some("Velocity"));
        assert_eq!(attrs.x_label.as_deref(), Some("Time"));
        assert_eq!(attrs.y_label.as_deref(), Some("Speed"));
        assert_eq!(attrs.max, Some(100.0));
        assert!(attrs.no_chrome);

        let defaults = parse_chart_attrs("");
        assert!(matches!(defaults.kind, crate::chart::ChartKind::Bar));
        assert_eq!(defaults.width, 60);
        assert_eq!(defaults.height, 8);
        assert_eq!(defaults.title, None);
        assert!(!defaults.no_chrome);
    }

    #[test]
    fn parses_chart_directive_source_fields_and_inline_body() {
        let chart = parse_chart_directive(
            "kind=line source=md://metrics.md#table:0 label_field=month value-field=revenue",
            &["Jan: 12", "Feb: 18"],
        );

        assert!(matches!(chart.attrs.kind, crate::chart::ChartKind::Line));
        assert_eq!(chart.source.as_deref(), Some("md://metrics.md#table:0"));
        assert_eq!(chart.label_field.as_deref(), Some("month"));
        assert_eq!(chart.value_field.as_deref(), Some("revenue"));
        assert_eq!(chart.inline_body, "Jan: 12\nFeb: 18");
    }

    #[test]
    fn parses_math_attrs_with_defaults() {
        let attrs = parse_math_attrs("width=72 align=right no-chrome=true");

        assert_eq!(attrs.width, 72);
        assert!(matches!(attrs.align, MathAlign::Right));
        assert!(attrs.no_chrome);

        let defaults = parse_math_attrs("");
        assert_eq!(defaults.width, 0);
        assert!(matches!(defaults.align, MathAlign::Center));
        assert!(!defaults.no_chrome);
    }

    #[test]
    fn parses_math_directive_attrs_and_expression_body() {
        let math = parse_math_directive("width=72 align=left no-chrome=true", &["a^2", "+ b^2"]);

        assert_eq!(math.expr, "a^2\n+ b^2");
        assert_eq!(math.width, 72);
        assert!(matches!(math.align, MathAlign::Left));
        assert!(math.no_chrome);
    }

    #[test]
    fn parses_toc_attrs_with_body_source_and_aliases() {
        let body = ["md://guide.md"];
        let attrs = parse_toc_attrs("max_depth=4 style=flat section=\"API Reference\"", &body);

        assert_eq!(attrs.source.as_deref(), Some("md://guide.md"));
        assert_eq!(attrs.max_depth, 4);
        assert_eq!(attrs.style, "flat");
        assert_eq!(attrs.section.as_deref(), Some("API Reference"));

        let defaults = parse_toc_attrs("", &[]);
        assert_eq!(defaults.source, None);
        assert_eq!(defaults.max_depth, 3);
        assert_eq!(defaults.style, "list");
        assert_eq!(defaults.section, None);

        let source_attr = parse_toc_attrs("source=md://api.md max-depth=2", &body);
        assert_eq!(source_attr.source.as_deref(), Some("md://api.md"));
        assert_eq!(source_attr.max_depth, 2);
    }

    #[test]
    fn parses_toc_directive_payload() {
        let body = ["md://guide.md"];
        let toc = parse_toc_directive("max-depth=2 style=numbered section=\"API\"", &body);

        assert_eq!(toc.source.as_deref(), Some("md://guide.md"));
        assert_eq!(toc.max_depth, 2);
        assert_eq!(toc.style, "numbered");
        assert_eq!(toc.section.as_deref(), Some("API"));
    }

    #[test]
    fn parses_xref_attrs_with_uri_source_and_body_fallback() {
        let uri_attrs = parse_xref_attrs("uri=md://api.md#auth label=\"Auth\" format=note", &[]);
        assert_eq!(uri_attrs.uri, "md://api.md#auth");
        assert_eq!(uri_attrs.label.as_deref(), Some("Auth"));
        assert_eq!(uri_attrs.format, "note");

        let source_attrs = parse_xref_attrs("source=md://guide.md", &[]);
        assert_eq!(source_attrs.uri, "md://guide.md");
        assert_eq!(source_attrs.label, None);
        assert_eq!(source_attrs.format, "inline");

        let body = ["md://body.md#section"];
        let body_attrs = parse_xref_attrs("", &body);
        assert_eq!(body_attrs.uri, "md://body.md#section");
    }

    #[test]
    fn parses_xref_directive_payload() {
        let xref = parse_xref_directive("uri=md://api.md#auth label=\"Auth\" format=note", &[]);

        assert_eq!(xref.uri, "md://api.md#auth");
        assert_eq!(xref.label.as_deref(), Some("Auth"));
        assert_eq!(xref.format, "note");
    }

    #[test]
    fn parses_blockquote_attrs_with_defaults_and_aliases() {
        let attrs = parse_blockquote_attrs("attribution=\"Ada Lovelace\" style=boxed");
        assert_eq!(attrs.attribution.as_deref(), Some("Ada Lovelace"));
        assert_eq!(attrs.style, "boxed");

        let by_attrs = parse_blockquote_attrs("by=Hamilton");
        assert_eq!(by_attrs.attribution.as_deref(), Some("Hamilton"));
        assert_eq!(by_attrs.style, "indent");

        let defaults = parse_blockquote_attrs("");
        assert_eq!(defaults.attribution, None);
        assert_eq!(defaults.style, "indent");
    }

    #[test]
    fn parses_blockquote_directive_attrs_and_text_body() {
        let blockquote = parse_blockquote_directive(
            "attribution=\"Ada Lovelace\" style=boxed",
            &["First paragraph.", "", "Second paragraph."],
        );

        assert_eq!(blockquote.text, "First paragraph.\n\nSecond paragraph.");
        assert_eq!(blockquote.attribution.as_deref(), Some("Ada Lovelace"));
        assert_eq!(blockquote.style, "boxed");
    }

    #[test]
    fn parses_symbol_attrs_with_defaults() {
        let attrs = parse_symbol_attrs("name=warning size=3 align=center");

        assert_eq!(attrs.name, "warning");
        assert_eq!(attrs.size, 3);
        assert_eq!(attrs.align, "center");

        let defaults = parse_symbol_attrs("");
        assert_eq!(defaults.name, "");
        assert_eq!(defaults.size, 1);
        assert_eq!(defaults.align, "left");
    }

    #[test]
    fn parses_symbol_directive_payload() {
        let symbol = parse_symbol_directive("name=warning size=3 align=center");

        assert_eq!(symbol.name, "warning");
        assert_eq!(symbol.size, 3);
        assert_eq!(symbol.align, "center");
    }

    #[test]
    fn parses_shape_attrs_with_symbol_defaults() {
        let attrs = parse_shape_attrs("name=arrow title=\"Flow\" direction=left size=2 width=40");

        assert_eq!(attrs.name, "arrow");
        assert_eq!(attrs.title.as_deref(), Some("Flow"));
        assert_eq!(attrs.direction, "left");
        assert_eq!(attrs.size, 2);
        assert_eq!(attrs.width, Some(40));

        let defaults = parse_shape_attrs("");
        assert_eq!(defaults.style, "double");
        assert_eq!(defaults.direction, "right");
        assert_eq!(defaults.size, 1);
    }

    #[test]
    fn parses_shape_directive_payload() {
        let shape = parse_shape_directive("name=arrow title=\"Flow\" direction=left size=2");

        assert_eq!(shape.name, "arrow");
        assert_eq!(shape.title.as_deref(), Some("Flow"));
        assert_eq!(shape.direction, "left");
        assert_eq!(shape.size, 2);
    }

    #[test]
    fn parses_region_directive_name_and_body() {
        let body = ["Header", "proof:element kind=label value=\"X\" width=5"];
        let region = parse_region_directive("name=\"top row\"", &body);

        assert_eq!(region.name, "top row");
        assert_eq!(region.body, vec![body[0].to_string(), body[1].to_string()]);

        let defaults = parse_region_directive("", &[]);
        assert_eq!(defaults.name, "");
        assert!(defaults.body.is_empty());
    }

    #[test]
    fn collect_directives_collects_region_body() {
        let source = "```proof:region name=header\nHello world\nproof:element kind=label value=\"X\" width=5\n```";
        let dirs = collect_directives(source);

        assert_eq!(dirs.len(), 1);
        match &dirs[0] {
            Directive::Region { name, body, .. } => {
                assert_eq!(name, "header");
                assert_eq!(
                    body,
                    &vec![
                        "Hello world".to_string(),
                        "proof:element kind=label value=\"X\" width=5".to_string(),
                    ]
                );
            }
            other => panic!("expected Region, got {other:?}"),
        }
    }

    #[test]
    fn parses_include_directive_uri_and_pin() {
        let body = ["pin=body-pin", "md://figures/arch.md#:0"];
        let include = parse_include_directive("pin=arch-diagram", &body);

        assert_eq!(include.uri.as_deref(), Some("md://figures/arch.md#:0"));
        assert_eq!(include.pin.as_deref(), Some("arch-diagram"));

        let empty = parse_include_directive("", &["", "pin=only-pin"]);
        assert_eq!(empty.uri, None);
        assert_eq!(empty.pin, None);
    }

    #[test]
    fn parses_table_uri_from_body() {
        assert_eq!(
            parse_table_uri(&["not a uri", "md://tables/stats.md#table:0"]).as_deref(),
            Some("md://tables/stats.md#table:0")
        );
        assert_eq!(parse_table_uri(&["", "https://example.com"]), None);
    }

    #[test]
    fn parses_table_directive_payload() {
        assert_eq!(
            parse_table_directive(&["not a uri", "md://tables/stats.md#table:0"]).as_deref(),
            Some("md://tables/stats.md#table:0")
        );
        assert_eq!(parse_table_directive(&["", "https://example.com"]), None);
    }

    #[test]
    fn parses_foreach_positional_and_source_attr_forms() {
        assert_eq!(
            parse_foreach("foreach=player in md://stats.md#edm:table:0 separator=\" \""),
            (
                "player".to_string(),
                "md://stats.md#edm:table:0".to_string()
            )
        );
        assert_eq!(
            parse_foreach("source=md://stats.md foreach=row"),
            ("row".to_string(), "md://stats.md".to_string())
        );
    }

    #[test]
    fn parses_row_element_lines() {
        let label = parse_row_element_line(
            "proof:element kind=label field=name width=12 align=left fill=# empty=.",
        )
        .unwrap();

        assert_eq!(label.field, "name");
        assert_eq!(label.width, 12);
        assert!(matches!(label.kind, ElementKind::Label));
        assert_eq!(label.fill_char, '#');
        assert_eq!(label.empty_char, '.');

        let mini_bar =
            parse_row_element_line("proof:element kind=mini-bar field=pts width=10 max=200")
                .unwrap();
        assert_eq!(mini_bar.max, Some(200.0));
        assert!(matches!(mini_bar.kind, ElementKind::MiniBar));

        assert!(parse_row_element_line("# Comment").is_none());
        assert!(parse_row_element_line("proof:element kind=label width=12").is_none());
    }

    #[test]
    fn parses_row_directive_attrs_and_elements() {
        let row = parse_row_directive(
            "foreach=player in md://stats.md#edm:table:0 separator=\",\" width=80 no-chrome",
            &[
                "proof:element kind=label field=name width=12",
                "proof:element kind=mini-bar field=pts width=10 max=200",
            ],
        );

        assert_eq!(row.var_name, "player");
        assert_eq!(row.source_uri, "md://stats.md#edm:table:0");
        assert_eq!(row.separator, ",");
        assert_eq!(row.declared_width, Some(80));
        assert!(row.no_chrome);
        assert_eq!(row.elements.len(), 2);
        assert!(matches!(row.elements[0].kind, ElementKind::Label));
        assert!(matches!(row.elements[1].kind, ElementKind::MiniBar));
    }
}
