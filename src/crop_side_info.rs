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

pub fn render_backlinks(target: &str, report_path: &Path, format: &str) -> Result<String> {
    let content = std::fs::read_to_string(report_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", report_path.display(), e))?;
    let report: BacklinksReport = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("parsing {}: {}", report_path.display(), e))?;
    render_backlinks_report(target, &report, format)
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

fn normalize_backlink_target(target: &str) -> String {
    let target = target.trim().trim_matches('"').trim_matches('\'');
    let target = target.strip_prefix("md://").unwrap_or(target);
    let target = target.split('#').next().unwrap_or(target);
    target.replace('\\', "/")
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
}
