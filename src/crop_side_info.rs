use anyhow::Result;
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

pub fn render_backlinks(target: &str, report_path: &Path, format: &str) -> Result<String> {
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let report: BacklinksReport = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_backlinks_report(target, &report, format)
}

pub fn render_headings(source: &str, report_path: &Path, format: &str) -> Result<String> {
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let inventory: HeadingInventory = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_headings_inventory(source, &inventory, format)
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
}
