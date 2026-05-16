use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct BacklinksReport {
    pages: Vec<BacklinksPage>,
}

#[derive(Debug, serde::Deserialize)]
struct BacklinksPage {
    source: String,
    #[serde(default)]
    inbound_links: Vec<BacklinkEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct BacklinkEntry {
    source: String,
    #[serde(default)]
    target: String,
}

#[derive(Debug, serde::Deserialize)]
struct HeadingInventory {
    #[serde(default)]
    headings: Vec<HeadingEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct HeadingEntry {
    source: String,
    level: usize,
    text: String,
    #[serde(default)]
    md_uri: String,
}

#[derive(Debug, serde::Deserialize)]
struct FrontmatterInventory {
    #[serde(default)]
    pages: Vec<FrontmatterPage>,
}

#[derive(Debug, serde::Deserialize)]
struct FrontmatterPage {
    source: String,
    #[serde(default)]
    keys: Vec<String>,
    #[serde(default)]
    fields: BTreeMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
struct LinkAudit {
    #[serde(default)]
    links: Vec<LinkEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct LinkEntry {
    source: String,
    target: String,
    status: String,
    #[serde(default)]
    resolved_source: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterFilter {
    pub field: Option<String>,
    pub value: Option<String>,
    pub op: FrontmatterMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterMatch {
    Has,
    Eq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkFilter {
    pub source: Option<String>,
    pub status: Option<String>,
}

pub fn render_backlinks(target: &str, report_path: &Path, format: &str) -> Result<String> {
    validate_snippet_format(format)?;
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let report: BacklinksReport = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_backlinks_report(target, &report, format)
}

pub fn render_headings(source: &str, report_path: &Path, format: &str) -> Result<String> {
    validate_snippet_format(format)?;
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let inventory: HeadingInventory = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_headings_inventory(source, &inventory, format)
}

pub fn render_frontmatter(
    report_path: &Path,
    filter: &FrontmatterFilter,
    format: &str,
) -> Result<String> {
    validate_snippet_format(format)?;
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let inventory: FrontmatterInventory = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_frontmatter_inventory(&inventory, filter, format)
}

pub fn render_links(report_path: &Path, filter: &LinkFilter, format: &str) -> Result<String> {
    validate_snippet_format(format)?;
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let audit: LinkAudit = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_link_audit(&audit, filter, format)
}

fn validate_snippet_format(format: &str) -> Result<()> {
    match format {
        "list" | "table" | "count" => Ok(()),
        other => anyhow::bail!(
            "CROP side-info snippet format must be list, table, or count, got {:?}",
            other
        ),
    }
}

fn render_backlinks_report(target: &str, report: &BacklinksReport, format: &str) -> Result<String> {
    let target = normalize_backlink_target(target);
    let page = report
        .pages
        .iter()
        .find(|page| normalize_backlink_target(&page.source) == target)
        .ok_or_else(|| {
            anyhow::anyhow!("target {:?} not found in CROP backlinks side-info", target)
        })?;

    if page.inbound_links.is_empty() {
        return Ok("_No backlinks._".to_string());
    }

    let mut lines = Vec::new();
    match format {
        "count" => Ok(page.inbound_links.len().to_string()),
        "table" => {
            lines.push("| Source | Target |".to_string());
            lines.push("|--------|--------|".to_string());
            for link in &page.inbound_links {
                lines.push(format!(
                    "| [{}]({}) | `{}` |",
                    backlink_label(&link.source),
                    link.source,
                    link.target
                ));
            }
            Ok(lines.join("\n"))
        }
        _ => {
            for link in &page.inbound_links {
                lines.push(format!(
                    "- [{}]({})",
                    backlink_label(&link.source),
                    link.source
                ));
            }
            Ok(lines.join("\n"))
        }
    }
}

fn render_headings_inventory(
    source: &str,
    inventory: &HeadingInventory,
    format: &str,
) -> Result<String> {
    let source = normalize_source(source);
    let headings: Vec<_> = inventory
        .headings
        .iter()
        .filter(|heading| normalize_source(&heading.source) == source)
        .collect();
    if headings.is_empty() {
        return Ok("_No headings._".to_string());
    }

    let mut lines = Vec::new();
    match format {
        "count" => Ok(headings.len().to_string()),
        "table" => {
            lines.push("| Level | Heading | URI |".to_string());
            lines.push("|------:|---------|-----|".to_string());
            for heading in headings {
                lines.push(format!(
                    "| {} | {} | `{}` |",
                    heading.level, heading.text, heading.md_uri
                ));
            }
            Ok(lines.join("\n"))
        }
        _ => {
            let min_level = headings
                .iter()
                .map(|heading| heading.level)
                .min()
                .unwrap_or(1);
            for heading in headings {
                let depth = heading.level.saturating_sub(min_level);
                let indent = "  ".repeat(depth);
                let uri = if heading.md_uri.is_empty() {
                    heading.source.clone()
                } else {
                    heading.md_uri.clone()
                };
                lines.push(format!("{}- [{}]({})", indent, heading.text, uri));
            }
            Ok(lines.join("\n"))
        }
    }
}

fn render_frontmatter_inventory(
    inventory: &FrontmatterInventory,
    filter: &FrontmatterFilter,
    format: &str,
) -> Result<String> {
    let pages: Vec<_> = inventory
        .pages
        .iter()
        .filter(|page| frontmatter_page_matches(page, filter))
        .collect();

    if pages.is_empty() {
        return Ok("_No frontmatter matches._".to_string());
    }

    match format {
        "count" => Ok(pages.len().to_string()),
        "table" => render_frontmatter_table(&pages, filter),
        _ => {
            let mut lines = Vec::new();
            for page in pages {
                let label = page
                    .fields
                    .get("title")
                    .filter(|title| !title.trim().is_empty())
                    .map(String::as_str)
                    .unwrap_or(&page.source);
                lines.push(format!("- [{}]({})", label, page.source));
            }
            Ok(lines.join("\n"))
        }
    }
}

fn render_frontmatter_table(
    pages: &[&FrontmatterPage],
    filter: &FrontmatterFilter,
) -> Result<String> {
    let mut columns = Vec::new();
    if let Some(field) = &filter.field {
        columns.push(field.clone());
    } else {
        for page in pages {
            for key in &page.keys {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        }
    }

    let mut lines = Vec::new();
    lines.push(format!(
        "| Source | {} |",
        if columns.is_empty() {
            "Keys".to_string()
        } else {
            columns.join(" | ")
        }
    ));
    lines.push(format!(
        "|--------|{}|",
        if columns.is_empty() {
            "------".to_string()
        } else {
            columns
                .iter()
                .map(|_| "------")
                .collect::<Vec<_>>()
                .join("|")
        }
    ));
    for page in pages {
        if columns.is_empty() {
            lines.push(format!(
                "| [{}]({}) | `{}` |",
                page.source,
                page.source,
                page.keys.join(", ")
            ));
        } else {
            let values = columns
                .iter()
                .map(|key| {
                    format!(
                        "`{}`",
                        page.fields
                            .get(key)
                            .map(String::as_str)
                            .unwrap_or("")
                            .replace('|', "\\|")
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
            lines.push(format!(
                "| [{}]({}) | {} |",
                page.source, page.source, values
            ));
        }
    }
    Ok(lines.join("\n"))
}

fn frontmatter_page_matches(page: &FrontmatterPage, filter: &FrontmatterFilter) -> bool {
    let Some(field) = &filter.field else {
        return true;
    };
    let Some(actual) = page.fields.get(field) else {
        return false;
    };
    let Some(value) = &filter.value else {
        return true;
    };
    match filter.op {
        FrontmatterMatch::Has => actual.contains(value),
        FrontmatterMatch::Eq => actual == value,
    }
}

fn render_link_audit(audit: &LinkAudit, filter: &LinkFilter, format: &str) -> Result<String> {
    let links: Vec<_> = audit
        .links
        .iter()
        .filter(|link| link_matches(link, filter))
        .collect();

    if links.is_empty() {
        return if format == "count" {
            Ok("0".to_string())
        } else {
            Ok("_No links._".to_string())
        };
    }

    match format {
        "count" => Ok(links.len().to_string()),
        "table" => {
            let mut lines = vec![
                "| Source | Target | Status | Resolved | Error |".to_string(),
                "|--------|--------|--------|----------|-------|".to_string(),
            ];
            for link in links {
                lines.push(format!(
                    "| `{}` | `{}` | `{}` | `{}` | {} |",
                    escape_table_cell(&link.source),
                    escape_table_cell(&link.target),
                    escape_table_cell(&link.status),
                    escape_table_cell(link.resolved_source.as_deref().unwrap_or("")),
                    escape_table_cell(link.error.as_deref().unwrap_or(""))
                ));
            }
            Ok(lines.join("\n"))
        }
        _ => {
            let mut lines = Vec::new();
            for link in links {
                let suffix = if link.status == "ok" {
                    link.resolved_source
                        .as_deref()
                        .map(|resolved| format!(" -> {}", resolved))
                        .unwrap_or_default()
                } else {
                    link.error
                        .as_deref()
                        .map(|error| format!(" ({})", error))
                        .unwrap_or_default()
                };
                lines.push(format!(
                    "- `{}` -> `{}` [{}]{}",
                    link.source, link.target, link.status, suffix
                ));
            }
            Ok(lines.join("\n"))
        }
    }
}

fn link_matches(link: &LinkEntry, filter: &LinkFilter) -> bool {
    if let Some(source) = &filter.source {
        if normalize_source(&link.source) != normalize_source(source) {
            return false;
        }
    }
    if let Some(status) = &filter.status {
        if status != "all" && link.status != *status {
            return false;
        }
    }
    true
}

fn escape_table_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

fn normalize_backlink_target(target: &str) -> String {
    let target = target.trim().trim_matches('"').trim_matches('\'');
    let target = target.strip_prefix("md://").unwrap_or(target);
    let target = target.split('#').next().unwrap_or(target);
    target.replace('\\', "/")
}

fn normalize_source(source: &str) -> String {
    let source = source.trim().trim_matches('"').trim_matches('\'');
    let source = source.strip_prefix("md://").unwrap_or(source);
    let source = source.split('#').next().unwrap_or(source);
    source.replace('\\', "/")
}

fn backlink_label(source: &str) -> String {
    let path = source.replace('\\', "/");
    path.rsplit('/').next().unwrap_or(source).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> BacklinksReport {
        serde_json::from_str(
            r#"{
  "pages": [
    {
      "source": "reference.source.md",
      "inbound_links": [
        { "source": "guide.source.md", "target": "reference.source.md#reference" },
        { "source": "nested/overview.source.md", "target": "reference.source.md" }
      ]
    },
    { "source": "empty.source.md", "inbound_links": [] }
  ]
}"#,
        )
        .unwrap()
    }

    fn heading_inventory() -> HeadingInventory {
        serde_json::from_str(
            r#"{
  "headings": [
    { "source": "guide.source.md", "level": 1, "text": "Guide", "md_uri": "md://guide.source.md#guide" },
    { "source": "guide.source.md", "level": 2, "text": "Install", "md_uri": "md://guide.source.md#install" },
    { "source": "other.source.md", "level": 1, "text": "Other", "md_uri": "md://other.source.md#other" }
  ]
}"#,
        )
        .unwrap()
    }

    fn frontmatter_inventory() -> FrontmatterInventory {
        serde_json::from_str(
            r#"{
  "pages": [
    {
      "source": "guide.source.md",
      "keys": ["status", "tags", "title"],
      "fields": { "status": "ready", "tags": "[proof, guide]", "title": "Guide" }
    },
    {
      "source": "draft.source.md",
      "keys": ["status", "tags", "title"],
      "fields": { "status": "draft", "tags": "[proof]", "title": "Draft" }
    }
  ]
}"#,
        )
        .unwrap()
    }

    fn link_audit() -> LinkAudit {
        serde_json::from_str(
            r#"{
  "links": [
    { "source": "guide.source.md", "target": "reference.source.md#reference", "status": "ok", "resolved_source": "reference.source.md" },
    { "source": "guide.source.md", "target": "missing.source.md", "status": "broken", "error": "missing target" },
    { "source": "other.source.md", "target": "guide.source.md", "status": "ok", "resolved_source": "guide.source.md" }
  ]
}"#,
        )
        .unwrap()
    }

    #[test]
    fn renders_backlink_list_for_normalized_target() {
        let rendered =
            render_backlinks_report("md://reference.source.md#reference", &report(), "list")
                .unwrap();

        assert!(rendered.contains("- [guide.source.md](guide.source.md)"));
        assert!(rendered.contains("- [overview.source.md](nested/overview.source.md)"));
    }

    #[test]
    fn renders_backlink_count_table_and_empty_state() {
        assert_eq!(
            render_backlinks_report("reference.source.md", &report(), "count").unwrap(),
            "2"
        );
        let table = render_backlinks_report("reference.source.md", &report(), "table").unwrap();
        assert!(table.contains("| Source | Target |"));
        assert!(table
            .contains("| [guide.source.md](guide.source.md) | `reference.source.md#reference` |"));
        assert_eq!(
            render_backlinks_report("empty.source.md", &report(), "list").unwrap(),
            "_No backlinks._"
        );
    }

    #[test]
    fn renders_source_heading_list_count_table_and_empty_state() {
        let list =
            render_headings_inventory("md://guide.source.md#install", &heading_inventory(), "list")
                .unwrap();
        assert!(list.contains("- [Guide](md://guide.source.md#guide)"));
        assert!(list.contains("  - [Install](md://guide.source.md#install)"));

        assert_eq!(
            render_headings_inventory("guide.source.md", &heading_inventory(), "count").unwrap(),
            "2"
        );

        let table =
            render_headings_inventory("guide.source.md", &heading_inventory(), "table").unwrap();
        assert!(table.contains("| Level | Heading | URI |"));
        assert!(table.contains("| 2 | Install | `md://guide.source.md#install` |"));

        assert_eq!(
            render_headings_inventory("missing.source.md", &heading_inventory(), "list").unwrap(),
            "_No headings._"
        );
    }

    #[test]
    fn renders_frontmatter_list_count_table_and_empty_state() {
        let filter = FrontmatterFilter {
            field: Some("tags".to_string()),
            value: Some("guide".to_string()),
            op: FrontmatterMatch::Has,
        };
        let list = render_frontmatter_inventory(&frontmatter_inventory(), &filter, "list").unwrap();
        assert!(list.contains("- [Guide](guide.source.md)"));
        assert!(!list.contains("Draft"));

        assert_eq!(
            render_frontmatter_inventory(&frontmatter_inventory(), &filter, "count").unwrap(),
            "1"
        );

        let table =
            render_frontmatter_inventory(&frontmatter_inventory(), &filter, "table").unwrap();
        assert!(table.contains("| Source | tags |"));
        assert!(table.contains("| [guide.source.md](guide.source.md) | `[proof, guide]` |"));

        let eq_filter = FrontmatterFilter {
            field: Some("status".to_string()),
            value: Some("ready".to_string()),
            op: FrontmatterMatch::Eq,
        };
        assert_eq!(
            render_frontmatter_inventory(&frontmatter_inventory(), &eq_filter, "count").unwrap(),
            "1"
        );

        let missing = FrontmatterFilter {
            field: Some("tags".to_string()),
            value: Some("missing".to_string()),
            op: FrontmatterMatch::Has,
        };
        assert_eq!(
            render_frontmatter_inventory(&frontmatter_inventory(), &missing, "list").unwrap(),
            "_No frontmatter matches._"
        );
    }

    #[test]
    fn renders_links_list_count_table_and_empty_state() {
        let filter = LinkFilter {
            source: Some("md://guide.source.md#guide".to_string()),
            status: Some("all".to_string()),
        };
        let list = render_link_audit(&link_audit(), &filter, "list").unwrap();
        assert!(list.contains(
            "- `guide.source.md` -> `reference.source.md#reference` [ok] -> reference.source.md"
        ));
        assert!(
            list.contains("- `guide.source.md` -> `missing.source.md` [broken] (missing target)")
        );
        assert!(!list.contains("other.source.md"));

        let broken = LinkFilter {
            source: None,
            status: Some("broken".to_string()),
        };
        assert_eq!(
            render_link_audit(&link_audit(), &broken, "count").unwrap(),
            "1"
        );

        let table = render_link_audit(&link_audit(), &broken, "table").unwrap();
        assert!(table.contains("| Source | Target | Status | Resolved | Error |"));
        assert!(table.contains(
            "| `guide.source.md` | `missing.source.md` | `broken` | `` | missing target |"
        ));

        let missing = LinkFilter {
            source: Some("missing.source.md".to_string()),
            status: None,
        };
        assert_eq!(
            render_link_audit(&link_audit(), &missing, "count").unwrap(),
            "0"
        );
        assert_eq!(
            render_link_audit(&link_audit(), &missing, "list").unwrap(),
            "_No links._"
        );
    }

    #[test]
    fn public_render_rejects_unknown_snippet_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("links.json");
        std::fs::write(
            &path,
            r#"{
  "links": [
    { "source": "guide.source.md", "target": "reference.source.md", "status": "ok" }
  ]
}"#,
        )
        .unwrap();

        let err = render_links(
            &path,
            &LinkFilter {
                source: None,
                status: None,
            },
            "markdown",
        )
        .unwrap_err();

        assert!(err.to_string().contains("list, table, or count"));
    }
}
