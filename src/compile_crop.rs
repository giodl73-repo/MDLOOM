use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::crop_side_info;

#[derive(Clone, Copy)]
pub(crate) enum SideInfoKind {
    Links,
    Backlinks,
    Headings,
    Frontmatter,
}

impl SideInfoKind {
    fn filename(self) -> &'static str {
        match self {
            SideInfoKind::Links => "links.json",
            SideInfoKind::Backlinks => "backlinks.json",
            SideInfoKind::Headings => "headings.json",
            SideInfoKind::Frontmatter => "frontmatter.json",
        }
    }
}

pub(crate) fn side_info_path(root: &Path, explicit: Option<&str>, kind: SideInfoKind) -> PathBuf {
    explicit
        .map(|p| root.join(p))
        .unwrap_or_else(|| root.join(".proof").join("side-info").join(kind.filename()))
}

pub(crate) fn frontmatter_filter(
    field: &Option<String>,
    value: &Option<String>,
    op: &str,
) -> Result<crop_side_info::FrontmatterFilter> {
    let op = match op {
        "has" => crop_side_info::FrontmatterMatch::Has,
        "eq" => crop_side_info::FrontmatterMatch::Eq,
        _ => bail!("frontmatter match op must be 'has' or 'eq'"),
    };
    Ok(crop_side_info::FrontmatterFilter {
        field: field.clone(),
        value: value.clone(),
        op,
    })
}

pub(crate) fn link_filter(
    source: &Option<String>,
    status: &str,
) -> Result<crop_side_info::LinkFilter> {
    let status = match status {
        "all" => Some("all".to_string()),
        "ok" | "broken" => Some(status.to_string()),
        _ => bail!("link status must be 'all', 'ok', or 'broken'"),
    };
    Ok(crop_side_info::LinkFilter {
        source: source.clone(),
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_info_path_uses_explicit_or_default_report_path() {
        let root = Path::new("repo");

        assert_eq!(
            side_info_path(root, Some("reports/links.json"), SideInfoKind::Links),
            PathBuf::from("repo").join("reports").join("links.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Backlinks),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("backlinks.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Headings),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("headings.json")
        );
        assert_eq!(
            side_info_path(root, None, SideInfoKind::Frontmatter),
            PathBuf::from("repo")
                .join(".proof")
                .join("side-info")
                .join("frontmatter.json")
        );
    }

    #[test]
    fn validates_crop_side_info_filters() {
        assert!(matches!(
            frontmatter_filter(&Some("tags".to_string()), &Some("guide".to_string()), "has")
                .unwrap()
                .op,
            crop_side_info::FrontmatterMatch::Has
        ));
        assert!(frontmatter_filter(&None, &None, "approx").is_err());

        assert_eq!(
            link_filter(&Some("README.md".to_string()), "broken")
                .unwrap()
                .status,
            Some("broken".to_string())
        );
        assert!(link_filter(&None, "unknown").is_err());
    }
}
