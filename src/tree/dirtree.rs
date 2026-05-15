/// dirtree — filesystem tree generation and path validation.
///
/// Generates formatted dirtree code blocks from a filesystem root,
/// and validates that paths declared in a dirtree block exist on disk.
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────
// Generation options
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DirtreeOptions {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>, // glob patterns relative to root
    pub dirs_first: bool,     // directories before files (default: true)
    pub sort: SortOrder,
    pub wrap_fence: bool,    // wrap output in ```dirtree fence (default: true)
    pub indent_width: usize, // spaces per level (default: 4)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Name,  // alphabetical (default)
    Ext,   // by file extension then name
    Size,  // by file size descending
    Mtime, // by modification time descending
}

impl Default for DirtreeOptions {
    fn default() -> Self {
        DirtreeOptions {
            root: PathBuf::from("."),
            max_depth: None,
            exclude: Vec::new(),
            dirs_first: true,
            sort: SortOrder::Name,
            wrap_fence: true,
            indent_width: 4,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Generation
// ─────────────────────────────────────────────────────────

/// Generate a dirtree from the filesystem.
pub fn generate(opts: &DirtreeOptions) -> Result<String> {
    let exclude_set = build_exclude_set(&opts.exclude)?;
    let root_name = opts
        .root
        .file_name()
        .map(|n| format!("{}/", n.to_string_lossy()))
        .unwrap_or_else(|| "./".to_string());

    let mut lines: Vec<String> = Vec::new();
    lines.push(root_name);

    walk_dir_inner(
        &opts.root,
        &opts.root,
        "",
        &exclude_set,
        opts,
        0,
        &mut lines,
    )?;

    let body = lines.join("\n");
    if opts.wrap_fence {
        Ok(format!("```dirtree\n{}\n```", body))
    } else {
        Ok(body)
    }
}

fn walk_dir_inner(
    dir: &Path,
    root: &Path,
    prefix: &str,
    exclude: &GlobSet,
    opts: &DirtreeOptions,
    depth: usize,
    lines: &mut Vec<String>,
) -> Result<()> {
    if let Some(max) = opts.max_depth {
        if depth >= max {
            return Ok(());
        }
    }

    let mut entries: Vec<(String, PathBuf, bool)> = Vec::new(); // (name, path, is_dir)

    for entry in
        std::fs::read_dir(dir).with_context(|| format!("reading directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let is_dir = path.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();

        // Check exclusion
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if exclude.is_match(&*rel_str) {
            continue;
        }

        entries.push((name, path, is_dir));
    }

    // Sort
    sort_entries(&mut entries, opts);

    let n = entries.len();
    for (i, (name, path, is_dir)) in entries.iter().enumerate() {
        let is_last = i == n - 1;
        let connector = if is_last { "└──" } else { "├──" };
        let display_name = if *is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };

        lines.push(format!("{}{} {}", prefix, connector, display_name));

        if *is_dir {
            let child_prefix = format!(
                "{}{}",
                prefix,
                if is_last {
                    " ".repeat(opts.indent_width)
                } else {
                    format!("│{}", " ".repeat(opts.indent_width - 1))
                }
            );
            walk_dir_inner(path, root, &child_prefix, exclude, opts, depth + 1, lines)?;
        }
    }

    Ok(())
}

fn sort_entries(entries: &mut Vec<(String, PathBuf, bool)>, opts: &DirtreeOptions) {
    entries.sort_by(|(name_a, path_a, is_dir_a), (name_b, path_b, is_dir_b)| {
        // dirs_first: directories before files at each level
        if opts.dirs_first && is_dir_a != is_dir_b {
            return is_dir_b.cmp(is_dir_a); // true > false: dirs first
        }
        match &opts.sort {
            SortOrder::Name => name_a.to_lowercase().cmp(&name_b.to_lowercase()),
            SortOrder::Ext => {
                let ext_a = path_a
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let ext_b = path_b
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                ext_a
                    .cmp(&ext_b)
                    .then(name_a.to_lowercase().cmp(&name_b.to_lowercase()))
            }
            SortOrder::Size => {
                let size_a = path_a.metadata().map(|m| m.len()).unwrap_or(0);
                let size_b = path_b.metadata().map(|m| m.len()).unwrap_or(0);
                size_b.cmp(&size_a) // descending
            }
            SortOrder::Mtime => {
                use std::time::SystemTime;
                let mtime_a = path_a
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let mtime_b = path_b
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                mtime_b.cmp(&mtime_a) // descending (newest first)
            }
        }
    });
}

fn build_exclude_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        builder.add(Glob::new(pat)?);
        // If pattern is "dir/**" or "dir/*", also exclude the directory "dir" itself
        // so that the whole directory is skipped during traversal.
        let stripped = pat
            .strip_suffix("/**")
            .or_else(|| pat.strip_suffix("/*"))
            .or_else(|| pat.strip_suffix("/"));
        if let Some(dir_pat) = stripped {
            if !dir_pat.is_empty() {
                if let Ok(g) = Glob::new(dir_pat) {
                    builder.add(g);
                }
            }
        }
    }
    Ok(builder.build()?)
}

// ─────────────────────────────────────────────────────────
// Path verification
// ─────────────────────────────────────────────────────────

/// Verify that paths declared in parsed tree nodes exist on disk.
/// Returns (line_no, missing_path) pairs for any path not found.
pub fn verify_paths(
    nodes: &[crate::checks::ascii_tree::TreeNode],
    root: &Path,
) -> Vec<(usize, String)> {
    let mut missing = Vec::new();
    let mut path_stack: Vec<String> = Vec::new(); // current path segments per level

    for node in nodes {
        use crate::checks::ascii_tree::Connector;
        if node.connector == Connector::Continuation {
            continue;
        }
        if node.label.is_empty() {
            continue;
        }

        let level = node.indent_level;

        // Maintain path stack at current depth
        path_stack.truncate(level);
        path_stack.push(node.label.clone());

        // Build the path
        let rel: PathBuf = path_stack.iter().collect();
        let abs = root.join(&rel);

        // For directories (label ends with /) check the directory exists
        // For files, check the file exists
        if !abs.exists() {
            missing.push((node.line_no, rel.to_string_lossy().replace('\\', "/")));
        }
    }

    missing
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_tree() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        std::fs::write(root.join("src/lib.rs"), "").unwrap();
        std::fs::create_dir(root.join("tests")).unwrap();
        std::fs::write(root.join("tests/e2e.rs"), "").unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]").unwrap();
        std::fs::write(root.join("README.md"), "# README").unwrap();
        dir
    }

    #[test]
    fn test_generate_basic() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            wrap_fence: false,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        assert!(result.contains("src/"));
        assert!(result.contains("├──") || result.contains("└──"));
        assert!(result.contains("Cargo.toml"));
    }

    #[test]
    fn test_generate_with_fence() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            wrap_fence: true,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        assert!(result.starts_with("```dirtree\n"));
        assert!(result.ends_with("\n```"));
    }

    #[test]
    fn test_generate_dirs_first() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            dirs_first: true,
            wrap_fence: false,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        // src/ and tests/ (dirs) should appear before Cargo.toml and README.md (files)
        let src_pos = lines
            .iter()
            .position(|l| l.contains("src/"))
            .unwrap_or(usize::MAX);
        let cargo_pos = lines
            .iter()
            .position(|l| l.contains("Cargo.toml"))
            .unwrap_or(usize::MAX);
        assert!(src_pos < cargo_pos, "directories should come before files");
    }

    #[test]
    fn test_generate_max_depth() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            max_depth: Some(1), // only show immediate children, not nested
            wrap_fence: false,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        // src/ should appear but src/main.rs should not (depth 2)
        assert!(result.contains("src/"));
        assert!(!result.contains("main.rs"));
    }

    #[test]
    fn test_generate_excludes() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            exclude: vec!["tests/**".to_string()],
            wrap_fence: false,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        assert!(!result.contains("tests/"));
        assert!(!result.contains("e2e.rs"));
        assert!(result.contains("src/"));
    }

    #[test]
    fn test_corner_is_last_entry() {
        let dir = make_test_tree();
        let opts = DirtreeOptions {
            root: dir.path().to_path_buf(),
            wrap_fence: false,
            ..Default::default()
        };
        let result = generate(&opts).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        // The last entry at each level must use └──
        let last_non_indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .last()
            .unwrap();
        assert!(
            last_non_indent.contains("└──"),
            "last entry should use └──: {:?}",
            last_non_indent
        );
    }

    #[test]
    fn test_verify_paths_all_exist() {
        // Build nodes from a real directory
        let dir = make_test_tree();
        // Manually create nodes that match the actual tree
        use crate::checks::ascii_tree::{Connector, TreeNode};
        let nodes = vec![TreeNode {
            line_no: 1,
            indent_level: 0,
            connector: Connector::None,
            label: "src/".to_string(),
            raw: "src/".to_string(),
        }];
        let missing = verify_paths(&nodes, dir.path());
        assert!(missing.is_empty(), "src/ should exist");
    }

    #[test]
    fn test_verify_paths_missing() {
        use crate::checks::ascii_tree::{Connector, TreeNode};
        let dir = tempfile::tempdir().unwrap();
        let nodes = vec![TreeNode {
            line_no: 1,
            indent_level: 0,
            connector: Connector::None,
            label: "nonexistent/".to_string(),
            raw: "nonexistent/".to_string(),
        }];
        let missing = verify_paths(&nodes, dir.path());
        assert_eq!(missing.len(), 1);
        assert!(missing[0].1.contains("nonexistent"));
    }
}
