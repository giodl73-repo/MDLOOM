/// Tree source schema parser — Wave 3.
///
/// Parses source data (markdown tables or JSON arrays) into Vec<TreeNode>
/// for all non-dirtree tree kinds: org, taxonomy, dependency, outline, decision.
///
/// Uses field mapping (explicit or auto-detected) rather than rigid column names.
use crate::checks::ascii_tree::{Connector, TreeNode};
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

// ─────────────────────────────────────────────────────────
// Field mapping
// ─────────────────────────────────────────────────────────

/// Field mapping for tree source data.
#[derive(Debug, Clone, Default)]
pub struct FieldMap {
    pub name: Option<String>,        // column/field that holds the node label
    pub parent: Option<String>,      // column/field that holds the parent reference
    pub label: Option<String>,       // optional display text (defaults to name)
    pub level: Option<String>,       // for taxonomy: the classification level
    pub yes_branch: Option<String>,  // for decision: the "yes" target column
    pub no_branch: Option<String>,   // for decision: the "no" target column
    pub version: Option<String>,     // for dependency: optional version column
    pub root_marker: Option<String>, // value that marks root (default: —, -, null, empty)
}

/// Root markers recognized as "this node has no parent".
const DEFAULT_ROOT_MARKERS: &[&str] = &["—", "-", "none", "null", "", "0", "root"];

impl FieldMap {
    fn is_root_marker(&self, val: &str) -> bool {
        let trimmed = val.trim();
        if let Some(ref marker) = self.root_marker {
            return trimmed.eq_ignore_ascii_case(marker);
        }
        DEFAULT_ROOT_MARKERS
            .iter()
            .any(|m| trimmed.eq_ignore_ascii_case(m))
    }
}

// ─────────────────────────────────────────────────────────
// Auto-detection of field names
// ─────────────────────────────────────────────────────────

const ORG_NAME_CANDIDATES: &[&str] = &["name", "employee", "person", "member", "who"];
const ORG_PARENT_CANDIDATES: &[&str] = &["parent", "manager", "reports_to", "superior", "boss"];
const ORG_LABEL_CANDIDATES: &[&str] = &["label", "title", "role", "position"];

const TAX_NAME_CANDIDATES: &[&str] = &["label", "name", "taxon", "term", "taxon_name"];
const TAX_PARENT_CANDIDATES: &[&str] = &["parent", "parent_taxon", "belongs_to", "parent_name"];
const TAX_LEVEL_CANDIDATES: &[&str] = &["level", "rank", "classification", "tier"];

const DEP_NAME_CANDIDATES: &[&str] = &["package", "name", "module", "crate", "lib"];
const DEP_PARENT_CANDIDATES: &[&str] = &["depends_on", "requires", "dependency", "uses", "parent"];
const DEP_VER_CANDIDATES: &[&str] = &["version", "ver", "semver"];

// Decision tree column candidates. A decision row has four roles:
// * a node identity (used as a target reference from yes/no branches),
// * a condition/question (rendered as the label above the two branches),
// * a yes-branch target (another node name or a leaf label),
// * a no-branch target (another node name or a leaf label).
const DEC_NODE_CANDIDATES: &[&str] = &["node", "id", "step", "name"];
const DEC_CONDITION_CANDIDATES: &[&str] = &["condition", "question", "label"];
const DEC_YES_CANDIDATES: &[&str] = &["yes", "yes →", "true", "yes_branch", "then", "if_yes"];
const DEC_NO_CANDIDATES: &[&str] = &["no", "no →", "false", "no_branch", "else", "if_no"];

fn find_column<'a>(headers: &'a [String], candidates: &[&str]) -> Option<&'a str> {
    for candidate in candidates {
        if let Some(h) = headers.iter().find(|h| h.to_lowercase() == *candidate) {
            return Some(h.as_str());
        }
    }
    None
}

fn auto_detect_org(headers: &[String], map: &mut FieldMap) {
    if map.name.is_none() {
        map.name = find_column(headers, ORG_NAME_CANDIDATES).map(|s| s.to_string());
    }
    if map.parent.is_none() {
        map.parent = find_column(headers, ORG_PARENT_CANDIDATES).map(|s| s.to_string());
    }
    if map.label.is_none() {
        map.label = find_column(headers, ORG_LABEL_CANDIDATES).map(|s| s.to_string());
    }
}

fn auto_detect_taxonomy(headers: &[String], map: &mut FieldMap) {
    if map.name.is_none() {
        map.name = find_column(headers, TAX_NAME_CANDIDATES).map(|s| s.to_string());
    }
    if map.parent.is_none() {
        map.parent = find_column(headers, TAX_PARENT_CANDIDATES).map(|s| s.to_string());
    }
    if map.level.is_none() {
        map.level = find_column(headers, TAX_LEVEL_CANDIDATES).map(|s| s.to_string());
    }
}

fn auto_detect_dependency(headers: &[String], map: &mut FieldMap) {
    if map.name.is_none() {
        map.name = find_column(headers, DEP_NAME_CANDIDATES).map(|s| s.to_string());
    }
    if map.parent.is_none() {
        map.parent = find_column(headers, DEP_PARENT_CANDIDATES).map(|s| s.to_string());
    }
    if map.version.is_none() {
        map.version = find_column(headers, DEP_VER_CANDIDATES).map(|s| s.to_string());
    }
}

fn auto_detect_decision(headers: &[String], map: &mut FieldMap) {
    if map.name.is_none() {
        map.name = find_column(headers, DEC_NODE_CANDIDATES).map(|s| s.to_string());
    }
    if map.label.is_none() {
        map.label = find_column(headers, DEC_CONDITION_CANDIDATES).map(|s| s.to_string());
    }
    if map.yes_branch.is_none() {
        map.yes_branch = find_column(headers, DEC_YES_CANDIDATES).map(|s| s.to_string());
    }
    if map.no_branch.is_none() {
        map.no_branch = find_column(headers, DEC_NO_CANDIDATES).map(|s| s.to_string());
    }
}

// ─────────────────────────────────────────────────────────
// Parsed source row (intermediate)
// ─────────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SourceRow {
    name: String,
    parent: String,     // empty if root
    label: String,      // may equal name if no label column
    version: String,    // for dependency
    level: String,      // for taxonomy
    yes_target: String, // for decision
    no_target: String,  // for decision
}

// ─────────────────────────────────────────────────────────
// Markdown table parsing
// ─────────────────────────────────────────────────────────

/// Parse a GFM markdown table string into (headers, rows) where each row
/// is a HashMap<header, cell_value>.
///
/// Accepts both fully bounded pipe tables (`| a | b |`) and the unbounded
/// form mdpath returns when extracting an addressed table (`a | b`). A line
/// is treated as a table row if it contains at least one `|` character; the
/// separator row (`---|---|---` or `------ | -----`) is identified by being
/// the second non-prose row and consisting only of dashes/pipes/whitespace.
pub fn parse_md_table(content: &str) -> Result<(Vec<String>, Vec<HashMap<String, String>>)> {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.contains('|'))
        .collect();

    if lines.len() < 2 {
        bail!("source table must have at least a header row and a separator row");
    }

    let headers: Vec<String> = parse_table_row(lines[0])
        .into_iter()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .collect();

    if headers.is_empty() {
        bail!("source table header row is empty");
    }

    // Verify line 1 is a separator row (`---|---|---` or similar). If not,
    // treat lines[1..] as data rows (some authors omit the separator).
    let is_separator = lines[1].chars().all(|c| matches!(c, '-' | ':' | '|' | ' '));
    let body_start = if is_separator { 2 } else { 1 };

    let mut rows = Vec::new();
    for &line in &lines[body_start..] {
        let cells: Vec<String> = parse_table_row(line)
            .into_iter()
            .map(|c| c.trim().to_string())
            .collect();

        let mut row = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            row.insert(header.clone(), cells.get(i).cloned().unwrap_or_default());
        }
        rows.push(row);
    }

    Ok((headers, rows))
}

fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim_start_matches('|').trim_end_matches('|');
    trimmed.split('|').map(|s| s.to_string()).collect()
}

// ─────────────────────────────────────────────────────────
// JSON parsing (simple — no serde dependency beyond what proof already has)
// ─────────────────────────────────────────────────────────

/// Parse a JSON array of objects into rows for tree generation.
/// Uses serde_json which is already a proof dependency.
pub fn parse_json_source(content: &str) -> Result<(Vec<String>, Vec<HashMap<String, String>>)> {
    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;

    let arr = value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("JSON source must be an array of objects"))?;

    if arr.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Collect all keys from the first object as headers
    let first = arr[0]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("JSON array elements must be objects"))?;
    let headers: Vec<String> = first.keys().cloned().collect();

    let mut rows = Vec::new();
    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("JSON array element is not an object"))?;
        let mut row = HashMap::new();
        for header in &headers {
            let val = obj
                .get(header)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string().trim_matches('"').to_string(),
                })
                .unwrap_or_default();
            row.insert(header.clone(), val);
        }
        rows.push(row);
    }

    Ok((headers, rows))
}

// ─────────────────────────────────────────────────────────
// Hierarchical tree builder (shared across kinds)
// ─────────────────────────────────────────────────────────

/// Build a list of TreeNodes in depth-first order from a flat list of (name, parent, label).
/// Handles cycles (nodes whose parent doesn't exist are treated as orphans).
pub(crate) fn build_dfs_tree(rows: &[SourceRow], map: &FieldMap) -> Result<Vec<TreeNode>> {
    // Build adjacency: parent_name → [child_names]
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        if row.parent.is_empty() || map.is_root_marker(&row.parent) {
            roots.push(i);
        } else {
            children.entry(row.parent.clone()).or_default().push(i);
        }
    }

    // If no explicit root rows exist but there are parent references (e.g. a flat table where
    // the parent column holds category names that aren't themselves rows), synthesize parent
    // nodes from the unique parent values. This supports the common pattern:
    //   | name | category | ...  where category = "math", "elements", etc.
    let synthetic_roots: Vec<String> = if roots.is_empty() {
        let named: std::collections::HashSet<_> = rows.iter().map(|r| &r.name).collect();
        let mut seen = std::collections::HashSet::new();
        let mut synth = Vec::new();
        for row in rows {
            if !row.parent.is_empty()
                && !map.is_root_marker(&row.parent)
                && !named.contains(&row.parent)
                && seen.insert(row.parent.clone())
            {
                synth.push(row.parent.clone());
            }
        }
        synth
    } else {
        vec![]
    };

    if roots.is_empty() && synthetic_roots.is_empty() {
        bail!("no root node found — ensure one row has an empty or '—' parent field");
    }

    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut line_no = 1usize;

    // Emit explicit root rows
    for root_idx in &roots {
        let row = &rows[*root_idx];
        let label = if row.label.is_empty() {
            row.name.clone()
        } else {
            row.label.clone()
        };
        nodes.push(TreeNode {
            line_no,
            indent_level: 0,
            connector: Connector::None,
            label,
            raw: String::new(),
        });
        line_no += 1;
        dfs_children(
            &row.name,
            &children,
            rows,
            1,
            &mut nodes,
            &mut line_no,
            &mut HashSet::new(),
        );
    }

    // Emit synthesized root nodes (category labels not present as named rows)
    for parent_name in &synthetic_roots {
        nodes.push(TreeNode {
            line_no,
            indent_level: 0,
            connector: Connector::None,
            label: parent_name.clone(),
            raw: String::new(),
        });
        line_no += 1;
        dfs_children(
            parent_name,
            &children,
            rows,
            1,
            &mut nodes,
            &mut line_no,
            &mut HashSet::new(),
        );
    }

    Ok(nodes)
}

fn dfs_children(
    parent_name: &str,
    children_map: &HashMap<String, Vec<usize>>,
    rows: &[SourceRow],
    level: usize,
    nodes: &mut Vec<TreeNode>,
    line_no: &mut usize,
    visited: &mut HashSet<String>,
) {
    if visited.contains(parent_name) {
        return;
    } // cycle guard
    visited.insert(parent_name.to_string());

    let Some(child_indices) = children_map.get(parent_name) else {
        return;
    };
    let n = child_indices.len();

    for (i, &idx) in child_indices.iter().enumerate() {
        let row = &rows[idx];
        let is_last = i == n - 1;
        let label = if row.label.is_empty() {
            row.name.clone()
        } else {
            if row.version.is_empty() {
                row.label.clone()
            } else {
                format!("{} {}", row.label, row.version)
            }
        };
        // For dependency, show version
        let display = if !row.version.is_empty() && row.label == row.name {
            format!("{} {}", row.name, row.version)
        } else {
            label
        };

        nodes.push(TreeNode {
            line_no: *line_no,
            indent_level: level,
            connector: if is_last {
                Connector::Corner
            } else {
                Connector::Tee
            },
            label: display,
            raw: String::new(),
        });
        *line_no += 1;

        dfs_children(
            &row.name,
            children_map,
            rows,
            level + 1,
            nodes,
            line_no,
            visited,
        );
    }

    visited.remove(parent_name);
}

// ─────────────────────────────────────────────────────────
// Kind-specific generators
// ─────────────────────────────────────────────────────────

/// Generate an org tree from source data.
pub fn generate_org(
    content: &str,
    format: &str,
    map: &mut FieldMap,
    indent_width: usize,
) -> Result<String> {
    let (headers, table_rows) = parse_source(content, format)?;
    auto_detect_org(&headers, map);

    let name_col = map.name.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect name column — specify with name=\"ColName\"")
    })?;
    let parent_col = map.parent.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect parent column — specify with parent=\"ColName\"")
    })?;
    let label_col = map.label.as_deref();

    let rows: Vec<SourceRow> = table_rows
        .iter()
        .map(|row| {
            let name = row.get(name_col).cloned().unwrap_or_default();
            let parent = row.get(parent_col).cloned().unwrap_or_default();
            let title = label_col
                .and_then(|c| row.get(c))
                .cloned()
                .filter(|s| !s.is_empty());
            // Display as "Title: Name" when both title and name are available
            let label = match &title {
                Some(t) if t != &name => format!("{}: {}", t, name),
                _ => name.clone(),
            };
            SourceRow {
                name,
                parent,
                label,
                version: String::new(),
                level: String::new(),
                yes_target: String::new(),
                no_target: String::new(),
            }
        })
        .collect();

    let nodes = build_dfs_tree(&rows, map)?;
    Ok(render_nodes(&nodes, indent_width))
}

/// Generate a taxonomy tree from source data.
pub fn generate_taxonomy(
    content: &str,
    format: &str,
    map: &mut FieldMap,
    indent_width: usize,
) -> Result<String> {
    let (headers, table_rows) = parse_source(content, format)?;
    auto_detect_taxonomy(&headers, map);

    let name_col = map.name.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect name column — specify with name=\"ColName\"")
    })?;
    let parent_col = map.parent.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect parent column — specify with parent=\"ColName\"")
    })?;
    let level_col = map.level.as_deref();

    let rows: Vec<SourceRow> = table_rows
        .iter()
        .map(|row| {
            let name = row.get(name_col).cloned().unwrap_or_default();
            let parent = row.get(parent_col).cloned().unwrap_or_default();
            let level = level_col
                .and_then(|c| row.get(c))
                .cloned()
                .unwrap_or_default();
            SourceRow {
                name: name.clone(),
                parent,
                label: if level.is_empty() {
                    name
                } else {
                    format!("{}: {}", level, name)
                },
                version: String::new(),
                level,
                yes_target: String::new(),
                no_target: String::new(),
            }
        })
        .collect();

    let nodes = build_dfs_tree(&rows, map)?;
    Ok(render_nodes(&nodes, indent_width))
}

/// Generate a dependency tree from source data.
pub fn generate_dependency(
    content: &str,
    format: &str,
    map: &mut FieldMap,
    indent_width: usize,
) -> Result<String> {
    let (headers, table_rows) = parse_source(content, format)?;
    auto_detect_dependency(&headers, map);

    let name_col = map
        .name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("cannot detect package name column"))?;
    let parent_col = map
        .parent
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("cannot detect dependency column"))?;
    let ver_col = map.version.as_deref();

    let rows: Vec<SourceRow> = table_rows
        .iter()
        .map(|row| {
            let name = row.get(name_col).cloned().unwrap_or_default();
            let parent = row.get(parent_col).cloned().unwrap_or_default();
            let version = ver_col
                .and_then(|c| row.get(c))
                .cloned()
                .unwrap_or_default();
            SourceRow {
                name: name.clone(),
                parent,
                label: name,
                version,
                level: String::new(),
                yes_target: String::new(),
                no_target: String::new(),
            }
        })
        .collect();

    // DFS with deduplication tracking
    let nodes = build_dfs_tree_with_dedup(&rows, map)?;
    Ok(render_nodes(&nodes, indent_width))
}

/// Like build_dfs_tree but tracks first-seen line numbers for dedup markers.
fn build_dfs_tree_with_dedup(rows: &[SourceRow], map: &FieldMap) -> Result<Vec<TreeNode>> {
    // For dependency: track which packages have been rendered fully
    // On second occurrence, show "(deduped ↑ N)" where N is the first line number
    let mut first_seen: HashMap<String, usize> = HashMap::new();
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();

    for (i, row) in rows.iter().enumerate() {
        if row.parent.is_empty() || map.is_root_marker(&row.parent) {
            roots.push(i);
        } else {
            children.entry(row.parent.clone()).or_default().push(i);
        }
    }

    if roots.is_empty() {
        bail!("no root node found");
    }

    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut line_no = 1usize;

    for root_idx in &roots {
        let row = &rows[*root_idx];
        let display = if row.version.is_empty() {
            row.name.clone()
        } else {
            format!("{} {}", row.name, row.version)
        };
        first_seen.insert(row.name.clone(), line_no);
        nodes.push(TreeNode {
            line_no,
            indent_level: 0,
            connector: Connector::None,
            label: display,
            raw: String::new(),
        });
        line_no += 1;
        dfs_dedup(
            &row.name,
            &children,
            rows,
            1,
            &mut nodes,
            &mut line_no,
            &mut first_seen,
            &mut HashSet::new(),
        );
    }

    Ok(nodes)
}

fn dfs_dedup(
    parent_name: &str,
    children_map: &HashMap<String, Vec<usize>>,
    rows: &[SourceRow],
    level: usize,
    nodes: &mut Vec<TreeNode>,
    line_no: &mut usize,
    first_seen: &mut HashMap<String, usize>,
    visiting: &mut HashSet<String>,
) {
    if visiting.contains(parent_name) {
        return;
    }
    visiting.insert(parent_name.to_string());

    let Some(child_indices) = children_map.get(parent_name) else {
        return;
    };
    let n = child_indices.len();

    for (i, &idx) in child_indices.iter().enumerate() {
        let row = &rows[idx];
        let is_last = i == n - 1;
        let connector = if is_last {
            Connector::Corner
        } else {
            Connector::Tee
        };

        let label = if let Some(&first_line) = first_seen.get(&row.name) {
            format!("{} (deduped ↑ {})", row.name, first_line)
        } else {
            first_seen.insert(row.name.clone(), *line_no);
            if row.version.is_empty() {
                row.name.clone()
            } else {
                format!("{} {}", row.name, row.version)
            }
        };

        nodes.push(TreeNode {
            line_no: *line_no,
            indent_level: level,
            connector,
            label: label.clone(),
            raw: String::new(),
        });
        *line_no += 1;

        // Only recurse into non-deduped nodes
        if !label.contains("deduped") {
            dfs_dedup(
                &row.name,
                children_map,
                rows,
                level + 1,
                nodes,
                line_no,
                first_seen,
                visiting,
            );
        }
    }

    visiting.remove(parent_name);
}

/// Generate an outline tree from the heading structure of a markdown file.
pub fn generate_outline(content: &str, indent_width: usize) -> Result<String> {
    // Parse headings from the markdown content
    let mut headings: Vec<(usize, String)> = Vec::new(); // (level, text)
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            let text = trimmed[level..].trim().to_string();
            if !text.is_empty() {
                headings.push((level, text));
            }
        }
    }

    if headings.is_empty() {
        bail!("no headings found in source document for outline generation");
    }

    let mut nodes: Vec<TreeNode> = Vec::new();
    let min_level = headings.iter().map(|(l, _)| *l).min().unwrap_or(1);
    let mut line_no = 1usize;

    for (i, (level, text)) in headings.iter().enumerate() {
        let depth = level - min_level;
        // Determine if this is the last sibling at this level
        let is_last = !headings[i + 1..].iter().any(|(l, _)| l <= level);
        let connector = if depth == 0 {
            Connector::None
        } else if is_last {
            Connector::Corner
        } else {
            Connector::Tee
        };
        nodes.push(TreeNode {
            line_no,
            indent_level: depth,
            connector,
            label: text.clone(),
            raw: String::new(),
        });
        line_no += 1;
    }

    Ok(render_nodes(&nodes, indent_width))
}

/// Generate a decision tree from source data.
///
/// Each row defines a decision node with:
/// - **name** — the node's identity (referenced from yes/no columns)
/// - **condition** (label column) — the question text rendered above the branches
/// - **yes** — the branch target if the condition is true (another node name or a leaf label)
/// - **no** — the branch target if the condition is false
///
/// The tree's root is the row whose name equals "root" (case-insensitive)
/// when present, otherwise the first row. Targets that resolve to another
/// declared node recurse; targets that don't are rendered as leaf labels
/// prefixed with "Yes → " / "No  → ".
///
/// Cycles are detected and broken (a re-entry to a visited node renders as a
/// "(cycle ↑)" leaf marker so the tree is bounded and the issue visible).
pub fn generate_decision(
    content: &str,
    format: &str,
    map: &mut FieldMap,
    indent_width: usize,
) -> Result<String> {
    let (headers, table_rows) = parse_source(content, format)?;
    auto_detect_decision(&headers, map);

    let name_col = map.name.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect node column — specify with name=\"ColName\"")
    })?;
    let cond_col = map.label.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect condition column — specify with label=\"ColName\"")
    })?;
    let yes_col = map.yes_branch.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect yes-branch column — specify with yes_branch=\"ColName\"")
    })?;
    let no_col = map.no_branch.as_deref().ok_or_else(|| {
        anyhow::anyhow!("cannot detect no-branch column — specify with no_branch=\"ColName\"")
    })?;

    // Index rows by node name for O(1) lookup during traversal.
    let mut by_name: HashMap<String, usize> = HashMap::new();
    for (i, row) in table_rows.iter().enumerate() {
        let name = row.get(name_col).cloned().unwrap_or_default();
        if !name.is_empty() {
            by_name.insert(name, i);
        }
    }

    // Pick the root: prefer a row literally named "root", else the first row.
    let root_idx = by_name
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("root"))
        .map(|(_, v)| *v)
        .unwrap_or(0);

    if table_rows.is_empty() {
        bail!("decision tree source has no rows");
    }

    // DFS-build TreeNodes with branch labels and cycle guard.
    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut line_no = 1usize;
    let mut visiting: HashSet<String> = HashSet::new();
    decision_walk(
        root_idx,
        &table_rows,
        &by_name,
        name_col,
        cond_col,
        yes_col,
        no_col,
        0,
        /* root has no branch label */ None,
        &mut nodes,
        &mut line_no,
        &mut visiting,
        /* is_last_sibling */ true,
    );

    if nodes.is_empty() {
        bail!("decision tree generation produced no output — check the source table");
    }
    Ok(render_nodes(&nodes, indent_width))
}

#[allow(clippy::too_many_arguments)]
fn decision_walk(
    idx: usize,
    rows: &[HashMap<String, String>],
    by_name: &HashMap<String, usize>,
    name_col: &str,
    cond_col: &str,
    yes_col: &str,
    no_col: &str,
    level: usize,
    branch_prefix: Option<&str>, // "Yes → " or "No  → ", or None for root
    nodes: &mut Vec<TreeNode>,
    line_no: &mut usize,
    visiting: &mut HashSet<String>,
    is_last_sibling: bool,
) {
    let row = &rows[idx];
    let name = row.get(name_col).cloned().unwrap_or_default();

    // Cycle guard: a re-entry to a node we're already expanding becomes a leaf.
    if visiting.contains(&name) {
        let label = format!("{}(cycle ↑ {})", branch_prefix.unwrap_or(""), name);
        nodes.push(TreeNode {
            line_no: *line_no,
            indent_level: level,
            connector: if is_last_sibling {
                Connector::Corner
            } else {
                Connector::Tee
            },
            label,
            raw: String::new(),
        });
        *line_no += 1;
        return;
    }
    visiting.insert(name.clone());

    let condition = row.get(cond_col).cloned().unwrap_or_default();
    let yes_target = row.get(yes_col).cloned().unwrap_or_default();
    let no_target = row.get(no_col).cloned().unwrap_or_default();

    // Emit this node's condition. Root has no branch prefix; children prepend "Yes → " / "No  → ".
    let node_label = match branch_prefix {
        Some(p) => format!("{}{}", p, condition),
        None => condition.clone(),
    };
    let connector = if level == 0 {
        Connector::None
    } else if is_last_sibling {
        Connector::Corner
    } else {
        Connector::Tee
    };
    nodes.push(TreeNode {
        line_no: *line_no,
        indent_level: level,
        connector,
        label: node_label,
        raw: String::new(),
    });
    *line_no += 1;

    // Yes branch (sibling-1 of two): not last.
    emit_branch(
        &yes_target,
        "Yes → ",
        rows,
        by_name,
        name_col,
        cond_col,
        yes_col,
        no_col,
        level + 1,
        nodes,
        line_no,
        visiting,
        /* is_last_sibling */ false,
    );
    // No branch (sibling-2 of two): last.
    emit_branch(
        &no_target,
        "No  → ",
        rows,
        by_name,
        name_col,
        cond_col,
        yes_col,
        no_col,
        level + 1,
        nodes,
        line_no,
        visiting,
        /* is_last_sibling */ true,
    );

    visiting.remove(&name);
}

#[allow(clippy::too_many_arguments)]
fn emit_branch(
    target: &str,
    prefix: &str,
    rows: &[HashMap<String, String>],
    by_name: &HashMap<String, usize>,
    name_col: &str,
    cond_col: &str,
    yes_col: &str,
    no_col: &str,
    level: usize,
    nodes: &mut Vec<TreeNode>,
    line_no: &mut usize,
    visiting: &mut HashSet<String>,
    is_last_sibling: bool,
) {
    if target.is_empty() {
        // Empty branch — emit a placeholder leaf so the tree shape stays balanced.
        nodes.push(TreeNode {
            line_no: *line_no,
            indent_level: level,
            connector: if is_last_sibling {
                Connector::Corner
            } else {
                Connector::Tee
            },
            label: format!("{}—", prefix),
            raw: String::new(),
        });
        *line_no += 1;
        return;
    }
    if let Some(&child_idx) = by_name.get(target) {
        decision_walk(
            child_idx,
            rows,
            by_name,
            name_col,
            cond_col,
            yes_col,
            no_col,
            level,
            Some(prefix),
            nodes,
            line_no,
            visiting,
            is_last_sibling,
        );
    } else {
        // Leaf label.
        nodes.push(TreeNode {
            line_no: *line_no,
            indent_level: level,
            connector: if is_last_sibling {
                Connector::Corner
            } else {
                Connector::Tee
            },
            label: format!("{}{}", prefix, target),
            raw: String::new(),
        });
        *line_no += 1;
    }
}

// ─────────────────────────────────────────────────────────
// Rendering
// ─────────────────────────────────────────────────────────

/// Render a Vec<TreeNode> to a formatted tree string (no fence).
pub fn render_nodes(nodes: &[TreeNode], indent_width: usize) -> String {
    let iw = indent_width.max(1);
    let n = nodes.len();
    let mut lines: Vec<String> = Vec::new();

    for i in 0..n {
        let node = &nodes[i];
        if node.connector == Connector::Continuation {
            continue;
        }

        let level = node.indent_level;

        // Build prefix for ancestor levels 0..level-1.
        // A level L is "open" at position i if a later node at level L exists
        // (as a sibling) without first leaving level L (i.e. a node at level < L appears).
        let prefix = if level == 0 {
            String::new()
        } else {
            let mut p = String::new();
            // Start from l=1: root (l=0) never needs a continuation │.
            // Each ancestor level from 1..level adds │   (open) or     (closed).
            for l in 1..level {
                let open = is_level_open(nodes, i, l);
                if open {
                    p.push('│');
                    for _ in 0..iw.saturating_sub(1) {
                        p.push(' ');
                    }
                } else {
                    for _ in 0..iw {
                        p.push(' ');
                    }
                }
            }
            p
        };

        let connector_str = match node.connector {
            Connector::None => "",
            Connector::Tee => "├── ",
            Connector::Corner => "└── ",
            Connector::Continuation => "",
        };

        lines.push(format!("{}{}{}", prefix, connector_str, node.label));
    }

    lines.join("\n")
}

/// Returns true if level `l` is still "open" at position `pos` — i.e. there
/// is a sibling at level `l` after `pos` without any node at level < `l` in between.
fn is_level_open(nodes: &[TreeNode], pos: usize, l: usize) -> bool {
    for node in &nodes[pos + 1..] {
        if node.connector == Connector::Continuation {
            continue;
        }
        if node.indent_level < l {
            return false;
        } // left the branch
        if node.indent_level == l {
            return node.connector == Connector::Tee || node.connector == Connector::Corner;
        }
    }
    false
}

// ─────────────────────────────────────────────────────────
// Helper: parse source by format
// ─────────────────────────────────────────────────────────

fn parse_source(
    content: &str,
    format: &str,
) -> Result<(Vec<String>, Vec<HashMap<String, String>>)> {
    match format {
        "json" => parse_json_source(content),
        _ => parse_md_table(content), // default: markdown table
    }
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const ORG_TABLE: &str = "| Employee | Manager | Title |\n|----------|---------|-------|\n| Gio | — | CTO |\n| Alice | Gio | VP Eng |\n| Dave | Gio | VP Product |\n| Bob | Alice | Staff Eng |";

    #[test]
    fn test_parse_md_table() {
        let (headers, rows) = parse_md_table(ORG_TABLE).unwrap();
        assert_eq!(headers, vec!["Employee", "Manager", "Title"]);
        assert_eq!(rows.len(), 4); // Gio, Alice, Dave, Bob
        assert_eq!(rows[0]["Employee"], "Gio");
        assert_eq!(rows[0]["Manager"], "—");
    }

    #[test]
    fn test_auto_detect_org_columns() {
        let (headers, _) = parse_md_table(ORG_TABLE).unwrap();
        let mut map = FieldMap::default();
        auto_detect_org(&headers, &mut map);
        assert_eq!(map.name.as_deref(), Some("Employee"));
        assert_eq!(map.parent.as_deref(), Some("Manager"));
        assert_eq!(map.label.as_deref(), Some("Title"));
    }

    #[test]
    fn test_generate_org_auto_detect() {
        let result = generate_org(ORG_TABLE, "table", &mut FieldMap::default(), 4).unwrap();
        assert!(result.contains("CTO: Gio") || result.contains("Gio"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
        assert!(result.contains("└──"));
        assert!(result.contains("├──"));
    }

    #[test]
    fn test_root_marker_detection() {
        let map = FieldMap::default();
        assert!(map.is_root_marker("—"));
        assert!(map.is_root_marker("-"));
        assert!(map.is_root_marker(""));
        assert!(map.is_root_marker("null"));
        assert!(!map.is_root_marker("Alice"));
    }

    #[test]
    fn test_parse_json_source() {
        let json = r#"[{"name":"Alice","parent":null,"title":"CTO"},{"name":"Bob","parent":"Alice","title":"VP"}]"#;
        let (headers, rows) = parse_json_source(json).unwrap();
        assert!(headers.contains(&"name".to_string()));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], "Alice");
    }

    #[test]
    fn test_generate_outline() {
        let md = "# Root\n## Section A\n### Subsection\n## Section B";
        let result = generate_outline(md, 4).unwrap();
        assert!(result.contains("Root"));
        assert!(result.contains("Section A"));
        assert!(result.contains("Subsection"));
        assert!(result.contains("Section B"));
        assert!(result.contains("└──") || result.contains("├──"));
    }

    #[test]
    fn test_dedup_dependency() {
        let table = "| package | depends_on | version |\n|---------|------------|--------|\n| app | lib | |\n| lib | core | 1.0 |\n| tool | core | |\n| core | — | 2.0 |";
        let result = generate_dependency(table, "table", &mut FieldMap::default(), 4).unwrap();
        assert!(result.contains("core"));
    }

    const DECISION_TABLE: &str = "| Node | Condition | Yes | No |\n\
        |------|-----------|-----|-----|\n\
        | root | Is the file .md? | parse | skip |\n\
        | parse | Has proof: directive? | compile | check-only |\n\
        | compile | DaVinci pin exists? | validate | embed |\n";

    #[test]
    fn test_decision_basic_renders() {
        let out = generate_decision(DECISION_TABLE, "table", &mut FieldMap::default(), 4).unwrap();
        // Root condition appears unprefixed.
        assert!(
            out.starts_with("Is the file .md?"),
            "root condition first:\n{}",
            out
        );
        // Yes/No prefixes appear on branches.
        assert!(
            out.contains("Yes → Has proof: directive?"),
            "Yes branch into nested node:\n{}",
            out
        );
        assert!(out.contains("No  → skip"), "No leaf:\n{}", out);
        assert!(out.contains("Yes → validate"), "deeper Yes leaf:\n{}", out);
        assert!(out.contains("No  → embed"), "deeper No leaf:\n{}", out);
    }

    #[test]
    fn test_decision_uses_first_row_when_no_root_named() {
        let table = "| node | condition | yes | no |\n\
            |------|-----------|-----|-----|\n\
            | start | Begin? | a | b |\n";
        let out = generate_decision(table, "table", &mut FieldMap::default(), 4).unwrap();
        assert!(
            out.starts_with("Begin?"),
            "first row treated as root:\n{}",
            out
        );
    }

    #[test]
    fn test_decision_cycle_guarded() {
        // Cycle: a → yes:b, b → yes:a. The walker breaks on re-entry.
        let table = "| node | condition | yes | no |\n\
            |------|-----------|-----|-----|\n\
            | a | Q1? | b | leaf-a |\n\
            | b | Q2? | a | leaf-b |\n";
        let out = generate_decision(table, "table", &mut FieldMap::default(), 4).unwrap();
        assert!(out.contains("cycle ↑"), "cycle marker present:\n{}", out);
    }

    #[test]
    fn test_render_nodes_basic() {
        let nodes = vec![
            TreeNode {
                line_no: 1,
                indent_level: 0,
                connector: Connector::None,
                label: "root".into(),
                raw: String::new(),
            },
            TreeNode {
                line_no: 2,
                indent_level: 1,
                connector: Connector::Tee,
                label: "child-a".into(),
                raw: String::new(),
            },
            TreeNode {
                line_no: 3,
                indent_level: 1,
                connector: Connector::Corner,
                label: "child-b".into(),
                raw: String::new(),
            },
        ];
        let rendered = render_nodes(&nodes, 4);
        assert!(rendered.contains("root"));
        assert!(rendered.contains("├── child-a"));
        assert!(rendered.contains("└── child-b"));
    }
}
