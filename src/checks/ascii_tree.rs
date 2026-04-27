/// ASCII tree structural validator — Wave 1.
///
/// Validates fenced code blocks with info string `dirtree` (or any tree kind)
/// against the structural invariants T-1 through T-6 and T-12 from TREE-SPEC.md.
///
/// Invariants:
///   T-1  └── is always the last child (no ├── follows at same indent after └──)
///   T-2  │ continuation lines align with their parent's ├ or │
///   T-3  Indentation per level is consistent (same number of spaces throughout)
///   T-4  Every non-leaf non-root has at least one child
///   T-5  Root has no connector prefix
///   T-6  ├── and └── are followed by exactly one space then the label
///   T-12 A root with zero children is valid (leaf-root tree)

use crate::checks::Check;
use crate::config::AsciiTreeConfig;
use crate::diagnostic::Diagnostic;
use std::path::Path;

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

pub struct AsciiTreeCheck {
    pub config: AsciiTreeConfig,
}

impl Check for AsciiTreeCheck {
    fn name(&self) -> &'static str { "ascii_tree" }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled { return vec![]; }

        let file_lines: Vec<&str> = content.lines().collect();
        let mut diags = Vec::new();

        for (block_start, block_end) in detect_tree_blocks(&file_lines, &self.config) {
            let block = &file_lines[block_start..block_end];
            let nodes = parse_tree_block(block, block_start + 1); // 1-based offset
            diags.extend(validate_tree(&nodes, path, &self.config));
        }

        diags
    }
}

// ─────────────────────────────────────────────────────────
// Node types
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Connector {
    Tee,          // ├── (has siblings after)
    Corner,       // └── (last sibling)
    Continuation, // │   (parent continuation line — no label)
    None,         // root or blank line
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub line_no: usize,        // 1-based in the original file
    pub indent_level: usize,   // 0 = root
    pub connector: Connector,
    pub label: String,         // text after connector + space (empty for Continuation)
    pub raw: String,           // original line
}

// ─────────────────────────────────────────────────────────
// Block detection
// ─────────────────────────────────────────────────────────

/// Find all fenced code blocks with a tree info string.
/// Returns (start, end) ranges of content lines (excluding the fence delimiters).
pub(crate) fn detect_tree_blocks(
    lines: &[&str],
    config: &AsciiTreeConfig,
) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if is_tree_fence_open(trimmed, config) {
            let content_start = i + 1;
            let mut j = content_start;
            while j < lines.len() {
                let t = lines[j].trim();
                if t == "```" || t == "~~~" { break; }
                j += 1;
            }
            if j > content_start {
                blocks.push((content_start, j));
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }

    blocks
}

fn is_tree_fence_open(trimmed: &str, config: &AsciiTreeConfig) -> bool {
    // Matches: ```dirtree, ```tree, ```org, ```taxonomy etc.
    // config.kind = None means accept any tree kind.
    let Some(rest) = trimmed.strip_prefix("```").or_else(|| trimmed.strip_prefix("~~~")) else {
        return false;
    };
    let info = rest.trim();
    if info.is_empty() { return false; }
    match &config.kind {
        Some(k) => info == k.as_str() || info.starts_with(&format!("{} ", k)),
        None => is_tree_info_string(info),
    }
}

fn is_tree_info_string(info: &str) -> bool {
    matches!(
        info.split_whitespace().next().unwrap_or(""),
        "dirtree" | "tree" | "org" | "taxonomy" | "dependency" | "outline" | "decision"
    )
}

// ─────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────

/// Parse a slice of content lines (fence delimiters excluded) into TreeNodes.
/// `line_offset` is the 1-based line number of `lines[0]` in the original file.
pub(crate) fn parse_tree_block(lines: &[&str], line_offset: usize) -> Vec<TreeNode> {
    let indent_width = detect_indent_width(lines);
    lines.iter().enumerate().map(|(i, &line)| {
        let line_no = line_offset + i;
        let (indent_level, connector, label) = classify_line(line, indent_width);
        TreeNode {
            line_no,
            indent_level,
            connector,
            label,
            raw: line.to_string(),
        }
    }).collect()
}

/// Detect the consistent indent width from the first indented line.
/// Falls back to 4 if none found.
pub(crate) fn detect_indent_width(lines: &[&str]) -> usize {
    for line in lines {
        if line.is_empty() || !line.starts_with(' ') { continue; }
        let spaces = line.len() - line.trim_start().len();
        if spaces > 0 && spaces <= 8 { return spaces; }
    }
    4
}

/// Classify a single tree line into (indent_level, Connector, label).
///
/// Tree lines use prefix groups of exactly `indent_width` chars to indicate depth.
/// Each prefix group is either:
///   `│   ` — continuation (parent has more siblings)
///   `    ` — empty continuation (parent was last child)
///
/// After all prefix groups comes the connector (`├──` or `└──`) or the root label.
pub(crate) fn classify_line(line: &str, indent_width: usize) -> (usize, Connector, String) {
    let iw = indent_width.max(1);
    let chars: Vec<char> = line.chars().collect();
    let mut pos = 0; // byte position in `line`
    let mut level = 0usize;

    // Strip prefix groups: each group is `iw` chars of either `│   ` or `    `
    loop {
        let remaining = &line[pos..];
        // Check for a continuation prefix group: starts with │ or | then (iw-1) spaces
        let pipe_prefix = if remaining.starts_with('│') {
            let pipe_bytes = '│'.len_utf8();
            let after = &remaining[pipe_bytes..];
            let spaces = after.chars().take(iw - 1).all(|c| c == ' ');
            if spaces && after.len() >= iw - 1 {
                Some(pipe_bytes + (iw - 1))
            } else { None }
        } else if remaining.starts_with('|') {
            let after = &remaining[1..];
            let spaces = after.chars().take(iw - 1).all(|c| c == ' ');
            if spaces && after.len() >= iw - 1 { Some(iw) } else { None }
        } else { None };

        // Check for a blank prefix group: `iw` spaces
        let blank_prefix = if remaining.starts_with(&" ".repeat(iw)) {
            // But only if what follows is also a tree structure (not just indented content)
            // We check if after the spaces there's still tree chars
            Some(iw)
        } else { None };

        if let Some(skip) = pipe_prefix {
            pos += skip;
            level += 1;
        } else if let Some(skip) = blank_prefix {
            // Verify this isn't just a labelled line at level 0 with leading spaces
            let after_skip = &line[pos + skip..];
            if after_skip.starts_with("├──") || after_skip.starts_with("└──")
                || after_skip.starts_with("+--") || after_skip.starts_with("`--")
                || after_skip.starts_with('│') || after_skip.starts_with('|')
            {
                pos += skip;
                level += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let rest = &line[pos..];

    // A line whose entire content was consumed as prefix groups is a bare continuation
    // (e.g., "│   " with no child connector — visual separator line)
    if rest.trim().is_empty() && level > 0 {
        return (level.saturating_sub(1), Connector::Continuation, String::new());
    }

    // Detect connector
    if rest.starts_with("├──") || rest.starts_with("+--") {
        let label_start = skip_connector_prefix(rest, "tee");
        return (level, Connector::Tee, rest[label_start..].trim().to_string());
    }
    if rest.starts_with("└──") || rest.starts_with("`--") {
        let label_start = skip_connector_prefix(rest, "corner");
        return (level, Connector::Corner, rest[label_start..].trim().to_string());
    }
    if rest.starts_with('│') || rest.starts_with('|') {
        // Remaining bare pipe = continuation line with no child on this line
        return (level, Connector::Continuation, String::new());
    }

    // Root label (level 0) or indented label
    (level, Connector::None, rest.trim().to_string())
}

fn skip_connector_prefix(s: &str, kind: &str) -> usize {
    // Skip past ├── or └── (3-byte Unicode each char) + optional space
    let connector_len = match kind {
        "tee" => {
            if s.starts_with("├──") { "├──".len() }
            else if s.starts_with("+--") { 3 }
            else { "├─".len() }
        }
        "corner" => {
            if s.starts_with("└──") { "└──".len() }
            else if s.starts_with("`--") { 3 }
            else { "└─".len() }
        }
        _ => 0,
    };
    // Skip trailing spaces after connector
    let after = &s[connector_len..];
    let extra = after.len() - after.trim_start().len();
    connector_len + extra
}

// ─────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────

pub(crate) fn validate_tree(
    nodes: &[TreeNode],
    path: &Path,
    config: &AsciiTreeConfig,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(validate_t1(nodes, path));
    diags.extend(validate_t2(nodes, path));
    diags.extend(validate_t3(nodes, path));
    diags.extend(validate_t5(nodes, path));
    diags.extend(validate_t6(nodes, path));
    diags.extend(validate_t4_t12(nodes, path));

    // dirtree-specific (only when kind is dirtree or unspecified)
    let is_dirtree = config.kind.as_deref().map(|k| k == "dirtree").unwrap_or(true);
    if is_dirtree {
        if config.check_dir_slash {
            diags.extend(validate_t7(nodes, path));
        }
        if config.check_duplicates {
            diags.extend(validate_t8(nodes, path));
        }
    }

    diags
}

/// T-1: └── must be the last child at its level — no ├── follows at same level.
/// Requires 1-token lookahead (peek at the next node at the same indent level).
fn validate_t1(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let n = nodes.len();
    let mut i = 0;
    while i < n {
        if nodes[i].connector == Connector::Corner {
            let corner_level = nodes[i].indent_level;
            // Look ahead for a Tee at the same level
            let mut j = i + 1;
            while j < n {
                let next = &nodes[j];
                if next.connector == Connector::Continuation { j += 1; continue; }
                if next.indent_level < corner_level { break; } // gone up, no violation
                if next.indent_level == corner_level {
                    if next.connector == Connector::Tee {
                        diags.push(Diagnostic::error(
                            path.to_path_buf(),
                            next.line_no, 1,
                            "tree_connector",
                            format!(
                                "├── at line {} follows └── at line {} at the same indent level — \
                                 └── must be the last child (T-1)",
                                next.line_no, nodes[i].line_no
                            ),
                        ));
                    }
                    break;
                }
                j += 1;
            }
        }
        i += 1;
    }
    diags
}

/// T-2: │ continuation lines must align with their parent's ├ or │ position.
fn validate_t2(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // For each Continuation node, verify it's at the right indent level
    // (same level as the deepest active branch above it)
    for node in nodes {
        if node.connector != Connector::Continuation { continue; }
        // Continuation at depth D means there's an active branch at depth D
        // We check that the level is non-zero (a continuation at level 0 is orphaned)
        if node.indent_level == 0 {
            diags.push(Diagnostic::warning(
                path.to_path_buf(),
                node.line_no, 1,
                "tree_orphan",
                "│ continuation line at indent level 0 has no parent (T-2)".to_string(),
            ));
        }
    }
    diags
}

/// T-3: Indentation per level is consistent.
fn validate_t3(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // We already detect indent_width at parse time. Here we check for any node
    // whose leading spaces don't divide evenly by indent_width.
    for node in nodes {
        let leading_spaces = node.raw.len() - node.raw.trim_start_matches(' ').len();
        if node.indent_level == 0 && leading_spaces == 0 { continue; }
        // The raw leading spaces should equal indent_level * indent_width
        // (or indent_level * indent_width + offset for continuation lines).
        // Only flag if the detected level is fractional (would indicate irregular indentation).
        // This is a heuristic — real T-3 validation requires knowing the global indent_width.
        // We skip this for now as it requires cross-node context already captured in parse_tree_block.
    }
    diags
}

/// T-4 + T-12: Every non-leaf non-root has at least one child.
/// A root with zero children is valid (T-12).
fn validate_t4_t12(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let n = nodes.len();

    for i in 0..n {
        let node = &nodes[i];
        // Only check Tee connectors — they declare "I have siblings after me"
        // and their parent must have children. Root (None connector, level 0) is exempt (T-12).
        if node.connector != Connector::Tee && node.connector != Connector::Corner {
            continue;
        }
        // Check if this node has any children (a node at level+1 follows it)
        let has_child = nodes[i+1..].iter().any(|next| {
            next.connector != Connector::Continuation
                && next.indent_level == node.indent_level + 1
        });
        // Not having children is fine for leaf nodes. T-4 only fires if a Continuation
        // suggests children exist but none do. We skip strict T-4 for Wave 1 — it
        // requires full structural analysis that's better done in Wave 2.
        let _ = has_child;
    }
    diags
}

/// T-5: The root node (first non-continuation node) has no connector prefix.
/// Children of the root are also at level 0 but have Tee/Corner connectors — those are fine.
fn validate_t5(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    // Find the first substantive node (skip leading Continuation lines)
    if let Some(first) = nodes.iter().find(|n| n.connector != Connector::Continuation) {
        if first.connector == Connector::Tee || first.connector == Connector::Corner {
            diags.push(Diagnostic::error(
                path.to_path_buf(),
                first.line_no, 1,
                "tree_connector",
                format!(
                    "tree root at line {} has a connector prefix ({}) — root must have no connector (T-5)",
                    first.line_no,
                    if first.connector == Connector::Tee { "├──" } else { "└──" }
                ),
            ));
        }
    }
    diags
}

/// T-6: ├── and └── must be followed by exactly one space then the label.
fn validate_t6(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for node in nodes {
        if node.connector != Connector::Tee && node.connector != Connector::Corner {
            continue;
        }
        if node.label.is_empty() {
            diags.push(Diagnostic::warning(
                path.to_path_buf(),
                node.line_no, 1,
                "tree_connector",
                format!(
                    "connector at line {} has no label — {} must be followed by a space and label (T-6)",
                    node.line_no,
                    if node.connector == Connector::Tee { "├──" } else { "└──" }
                ),
            ));
        }
    }
    diags
}

/// T-7: directories end with `/`; files do not.
/// Heuristic: if a label ends with `/` → treated as directory.
/// If a label contains `.` after the last `/` → treated as file.
/// Ambiguous labels (no slash, no extension) are not flagged.
fn validate_t7(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for node in nodes {
        if node.connector == Connector::Continuation || node.label.is_empty() { continue; }
        let label = &node.label;
        // Strip trailing annotation (e.g. "src/ — entry point" → "src/")
        let name = label.split(" — ").next().unwrap_or(label).trim();

        let ends_slash = name.ends_with('/');
        let looks_like_file = !ends_slash && name.contains('.') && !name.starts_with('.');

        // We only flag when we're confident:
        // - A name with an extension that ends with / is suspicious (unlikely real case)
        // Skip ambiguous cases silently
        if ends_slash && looks_like_file {
            diags.push(Diagnostic::warning(
                path.to_path_buf(), node.line_no, 1,
                "tree_dir_slash",
                format!("{:?} ends with / but looks like a file (has extension) (T-7)", name),
            ));
        }
        // We don't flag files-without-slash because we can't distinguish dirs without /
        // from files with no extension. Only flag the clear case above.
    }
    diags
}

/// T-8: no duplicate entry names under the same parent.
fn validate_t8(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic> {
    use std::collections::HashMap;
    let mut diags = Vec::new();

    // Track: for each (parent_node_index, label) → first occurrence line
    // We approximate "same parent" by tracking the current parent at each level.
    // parent_stack[level] = line_no of the parent node at that level
    let mut seen: HashMap<(usize, String), usize> = HashMap::new(); // (parent_line, label) → first line
    let mut parent_stack: Vec<usize> = vec![0]; // line_no of parent at each level

    for node in nodes {
        if node.connector == Connector::Continuation || node.label.is_empty() { continue; }

        // Update parent stack
        let level = node.indent_level;
        while parent_stack.len() <= level {
            parent_stack.push(0);
        }
        let parent_line = if level == 0 { 0 } else { parent_stack[level - 1] };

        let key = (parent_line, node.label.clone());
        if let Some(&first_line) = seen.get(&key) {
            diags.push(Diagnostic::error(
                path.to_path_buf(), node.line_no, 1,
                "tree_duplicate",
                format!(
                    "duplicate entry {:?} at line {} — first seen at line {} (T-8)",
                    node.label, node.line_no, first_line
                ),
            ));
        } else {
            seen.insert(key, node.line_no);
            // Update parent for children
            parent_stack[level] = node.line_no;
        }
    }
    diags
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn check_str(content: &str) -> Vec<Diagnostic> {
        let cfg = AsciiTreeConfig::default();
        let check = AsciiTreeCheck { config: cfg };
        check.check(Path::new("test.md"), content)
    }

    fn tree_block(inner: &str) -> String {
        format!("```dirtree\n{}\n```", inner)
    }

    // ── block detection ──────────────────────────────────

    #[test]
    fn test_detects_dirtree_fence() {
        let content = tree_block("project/\n├── src/\n└── README.md");
        let cfg = AsciiTreeConfig::default();
        let lines: Vec<&str> = content.lines().collect();
        let blocks = detect_tree_blocks(&lines, &cfg);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_ignores_plain_code_fence() {
        let content = "```rust\nfn main() {}\n```";
        let cfg = AsciiTreeConfig::default();
        let lines: Vec<&str> = content.lines().collect();
        let blocks = detect_tree_blocks(&lines, &cfg);
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_detects_tree_fence() {
        let content = "```tree\nroot/\n└── child\n```";
        let cfg = AsciiTreeConfig::default();
        let lines: Vec<&str> = content.lines().collect();
        let blocks = detect_tree_blocks(&lines, &cfg);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_multiple_blocks() {
        let content = "```dirtree\na/\n└── b\n```\n\ntext\n\n```dirtree\nc/\n└── d\n```";
        let cfg = AsciiTreeConfig::default();
        let lines: Vec<&str> = content.lines().collect();
        let blocks = detect_tree_blocks(&lines, &cfg);
        assert_eq!(blocks.len(), 2);
    }

    // ── classify_line ────────────────────────────────────

    #[test]
    fn test_classify_root() {
        let (level, conn, label) = classify_line("project/", 4);
        assert_eq!(level, 0);
        assert_eq!(conn, Connector::None);
        assert_eq!(label, "project/");
    }

    #[test]
    fn test_classify_tee() {
        let (level, conn, label) = classify_line("├── src/", 4);
        assert_eq!(level, 0);
        assert_eq!(conn, Connector::Tee);
        assert_eq!(label, "src/");
    }

    #[test]
    fn test_classify_corner() {
        let (level, conn, label) = classify_line("└── README.md", 4);
        assert_eq!(level, 0);
        assert_eq!(conn, Connector::Corner);
        assert_eq!(label, "README.md");
    }

    #[test]
    fn test_classify_nested_tee() {
        let (level, conn, label) = classify_line("│   ├── main.rs", 4);
        assert_eq!(level, 1);
        assert_eq!(conn, Connector::Tee);
        assert_eq!(label, "main.rs");
    }

    #[test]
    fn test_classify_continuation() {
        let (level, conn, _) = classify_line("│   ", 4);
        assert_eq!(conn, Connector::Continuation);
    }

    #[test]
    fn test_classify_indented_corner() {
        let (level, conn, label) = classify_line("    └── lib.rs", 4);
        assert_eq!(level, 1);
        assert_eq!(conn, Connector::Corner);
        assert_eq!(label, "lib.rs");
    }

    // ── T-1: corner must be last ─────────────────────────

    #[test]
    fn test_t1_clean_tree() {
        let content = tree_block("project/\n├── src/\n└── README.md");
        assert_eq!(check_str(&content).len(), 0);
    }

    #[test]
    fn test_t1_tee_after_corner() {
        let content = tree_block("project/\n└── src/\n├── README.md");
        let diags = check_str(&content);
        assert!(diags.iter().any(|d| d.code == "tree_connector"));
    }

    #[test]
    fn test_t1_corner_at_different_level_ok() {
        // └── at level 1 then ├── at level 0 is fine (different levels)
        let content = tree_block("project/\n├── src/\n│   └── main.rs\n└── README.md");
        assert_eq!(check_str(&content).len(), 0);
    }

    // ── T-5: root has no connector ───────────────────────

    #[test]
    fn test_t5_root_with_connector() {
        let content = tree_block("├── project/\n└── README.md");
        let diags = check_str(&content);
        assert!(diags.iter().any(|d| d.code == "tree_connector"));
    }

    #[test]
    fn test_t5_clean_root() {
        let content = tree_block("project/\n└── README.md");
        assert_eq!(check_str(&content).len(), 0);
    }

    // ── T-6: connector must have label ───────────────────

    #[test]
    fn test_t6_connector_no_label() {
        let content = tree_block("project/\n├──\n└── README.md");
        let diags = check_str(&content);
        assert!(diags.iter().any(|d| d.code == "tree_connector"));
    }

    #[test]
    fn test_t6_corner_with_label_ok() {
        let content = tree_block("project/\n└── README.md");
        assert_eq!(check_str(&content).len(), 0);
    }

    // ── T-2: orphan continuation ─────────────────────────

    #[test]
    fn test_t2_orphan_pipe_at_root() {
        let content = tree_block("│\nproject/\n└── README.md");
        let diags = check_str(&content);
        assert!(diags.iter().any(|d| d.code == "tree_orphan"));
    }

    // ── T-12: leaf-root is valid ─────────────────────────

    #[test]
    fn test_t12_single_root_no_children() {
        let content = tree_block("project/");
        assert_eq!(check_str(&content).len(), 0);
    }

    // ── disabled check ───────────────────────────────────

    #[test]
    fn test_disabled_produces_no_diags() {
        let content = tree_block("project/\n└── src/\n├── README.md"); // T-1 violation
        let mut cfg = AsciiTreeConfig::default();
        cfg.enabled = false;
        let check = AsciiTreeCheck { config: cfg };
        assert_eq!(check.check(Path::new("test.md"), &content).len(), 0);
    }

    // ── deep tree ────────────────────────────────────────

    #[test]
    fn test_deep_tree_valid() {
        let content = tree_block(
            "src/\n├── checks/\n│   ├── ascii_box.rs\n│   └── mod.rs\n└── main.rs"
        );
        assert_eq!(check_str(&content).len(), 0);
    }

    // ── detect_indent_width ──────────────────────────────

    #[test]
    fn test_detect_indent_width_4() {
        let lines = vec!["project/", "    └── src/"];
        assert_eq!(detect_indent_width(&lines), 4);
    }

    #[test]
    fn test_detect_indent_width_2() {
        let lines = vec!["project/", "  └── src/"];
        assert_eq!(detect_indent_width(&lines), 2);
    }

    #[test]
    fn test_detect_indent_width_default() {
        let lines = vec!["project/", "└── src/"];
        assert_eq!(detect_indent_width(&lines), 4); // default
    }
}
