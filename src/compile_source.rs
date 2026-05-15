use anyhow::Result;
use std::path::Path;

pub(crate) fn resolve_source_for_compile(src: &str, root: &Path) -> Result<String> {
    if src.starts_with("md://") {
        if let Ok(parsed) = mdpath::parse(src) {
            if let Ok(element) =
                mdpath::resolve_with_classifier(&parsed, root, &mdpath::DefaultClassifier)
            {
                return Ok(element.content);
            }
        }

        let path_part = src.strip_prefix("md://").unwrap_or(src);
        let path = root.join(path_part);
        if path.exists() {
            return std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e));
        }
        anyhow::bail!(
            "cannot resolve md:// URI {:?} — file not found and no addressed element",
            src
        )
    } else {
        let path = root.join(src);
        std::fs::read_to_string(&path).map_err(|e| anyhow::anyhow!("reading {:?}: {}", path, e))
    }
}
