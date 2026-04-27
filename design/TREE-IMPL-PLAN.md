# proof tree — Implementation Plan

> **Status**: Planning — Wave 1 ready to start.
> **Spec**: [TREE-SPEC.md](./TREE-SPEC.md)

---

## Summary

Four waves. Each wave delivers a working, tested slice. Waves build on each other:
Wave 1 is the structural core; Wave 2 extends it to dirtree-specific rules and
generation; Wave 3 adds schema-driven kinds; Wave 4 delivers the full CLI surface
and compile directive.

**Estimated total LOC**: ~3,500–4,200 Rust, ~600 test lines.

---

## Wave 1 — Core structural validator

**Goal**: Parse a fenced `dirtree` code block into a `Vec<TreeNode>` and validate
T-1 through T-6 and T-12. Wire into `runner.rs` and `config.rs`. No generation yet.

**Estimated LOC**: ~650 Rust + ~200 test lines

### Files

| File | Action |
|------|--------|
| `src/checks/ascii_tree.rs` | New — parser + T-1/T-6/T-12 validator |
| `src/checks/mod.rs` | Add `pub mod ascii_tree;` |
| `src/config.rs` | Add `AsciiTreeConfig` struct + field on `GlintConfig` |
| `src/runner.rs` | Add `AsciiTreeCheck` to `build_checks()` |

### Key structs and functions

```rust
// src/checks/ascii_tree.rs

pub struct AsciiTreeConfig {
    pub enabled: bool,
    pub indent_width: usize,   // default: 4
    pub kind: Option<String>,  // None = any dirtree fence, Some("dirtree") = explicit
}

pub struct TreeNode {
    pub line_no: usize,        // 1-based within file
    pub indent_level: usize,   // 0 = root
    pub connector: Connector,  // Tee | Corner | Continuation | None
    pub label: String,         // text after connector + space
    pub raw: String,           // original line for auto-fix
}

pub enum Connector {
    Tee,          // ├──  (Unicode or ASCII + fallback)
    Corner,       // └──
    Continuation, // │ (vertical pipe only — continuation prefix)
    None,         // root or blank prefix
}

pub struct AsciiTreeCheck {
    pub config: AsciiTreeConfig,
}

impl Check for AsciiTreeCheck { ... }

// Internal API (pub within crate for Wave 2/4 reuse):
pub(crate) fn parse_tree_block(lines: &[&str], line_offset: usize) -> Vec<TreeNode>
pub(crate) fn validate_tree(nodes: &[TreeNode], path: &Path, config: &AsciiTreeConfig) -> Vec<Diagnostic>
fn detect_tree_blocks(lines: &[&str]) -> Vec<(usize, usize)>  // (start, end) of dirtree fenced blocks
fn classify_connector(line: &str, indent_width: usize) -> (usize, Connector, String)
fn validate_t1(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>  // 1-token lookahead
fn validate_t2(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
fn validate_t3(nodes: &[TreeNode], path: &Path, indent_width: usize) -> Vec<Diagnostic>
fn validate_t4(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
fn validate_t5(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
fn validate_t6(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
fn validate_t12(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
```

**T-1 lookahead strategy**: `validate_t1` iterates `nodes` with a peekable iterator.
For each `Corner` node at indent level N, it checks whether any subsequent node
(before the next node at level < N) has a `Tee` connector at the same level. That
would be a T-1 violation. One-line buffer: collect nodes into a `Vec` first so
peek is free; no streaming required.

**Fence detection**: `detect_tree_blocks` scans for ` ```dirtree ` info strings
(case-insensitive, trimmed). This mirrors `detect_code_blocks` in `ascii_box.rs`
but filters by info string. Only blocks tagged `dirtree` are checked, preventing
false positives in other code blocks.

**ASCII fallbacks**: `classify_connector` maps `+--` → `Tee`, `\--` or `L--` → `Corner`,
`|` → `Continuation` alongside the Unicode forms. Matching is by prefix scan, not
character class, so single-char `+` at start of label does not false-trigger.

### Diagnostic codes emitted in Wave 1

| Code | Invariant |
|------|-----------|
| `tree_connector` | T-1 (wrong ├ vs └), T-5 (root has connector) |
| `tree_indent` | T-3 (inconsistent indent width) |
| `tree_orphan` | T-2 (│ continuation with no parent column) |

### Config struct in `config.rs`

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct AsciiTreeConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default = "default_indent_width")]
    pub indent_width: usize,
    #[serde(default)]
    pub kind: Option<String>,
}

impl Default for AsciiTreeConfig {
    fn default() -> Self {
        Self { enabled: true, indent_width: 4, kind: None }
    }
}
fn default_indent_width() -> usize { 4 }
```

Add to `GlintConfig`:
```rust
#[serde(default)]
pub ascii_tree: AsciiTreeConfig,
```

Add to `merge()` in `config.rs` (child wins, same as `ascii_box`):
```rust
ascii_tree: child.ascii_tree,
```

### Wiring in `runner.rs`

In `build_checks()`, after the `ascii_flow` block:
```rust
if config.ascii_tree.enabled {
    checks.push(Box::new(AsciiTreeCheck {
        config: config.ascii_tree.clone(),
    }));
}
```

Import: `use crate::checks::ascii_tree::AsciiTreeCheck;`

### Tests (`src/checks/ascii_tree.rs` — `#[cfg(test)]`)

Test file: `src/checks/ascii_tree.rs` (inline tests, same pattern as `ascii_box.rs`)

| Test | Covers |
|------|--------|
| `perfect_tree_no_errors` | Happy path — valid 3-level dirtree |
| `t1_corner_not_last_child` | T-1: ├ follows └ at same level |
| `t1_corner_after_corner_ok` | T-1: ├ after ├ is fine |
| `t1_lookahead_nested_scope` | T-1: └ in nested subtree doesn't affect parent level |
| `t2_orphan_continuation` | T-2: │ with no parent column |
| `t2_continuation_aligned` | T-2: │ aligned to parent ├ — no error |
| `t3_inconsistent_indent_2vs4` | T-3: first level = 2, second = 4 |
| `t3_consistent_indent_2` | T-3: indent_width=2 throughout — no error |
| `t4_internal_node_no_children` | T-4: non-leaf has no children |
| `t5_root_has_connector` | T-5: root line starts with ├ |
| `t5_root_no_connector_ok` | T-5: root is bare text — no error |
| `t6_connector_no_space` | T-6: ├──label (no space before label) |
| `t6_connector_with_space_ok` | T-6: ├── label — no error |
| `t12_single_line_root_ok` | T-12: single line with no connectors is valid |
| `t12_empty_block_ok` | T-12: empty dirtree fence is valid |
| `non_dirtree_fence_ignored` | ` ```rust ` block not checked |
| `ascii_fallback_connectors` | `+--`, `\--`, `|` forms accepted |
| `unicode_and_ascii_mixed` | Mixed connectors in same tree |
| `deep_nesting_5_levels` | 5-level tree with correct connectors |
| `multiple_tree_blocks_in_file` | Two dirtree blocks — each checked independently |

**Exit criterion**: `cargo test checks::ascii_tree` — 20+ tests pass, zero failures.
`proof check` on a file with a valid dirtree emits no diagnostics. A broken tree
(└ followed by ├ at same level) emits `tree_connector` on the correct line.

---

## Wave 2 — dirtree kind + auto-fix

**Goal**: dirtree-specific checks (T-7, T-8), filesystem path validation
(`--verify-paths`), tree generation from filesystem, and auto-fix for connector
and indent errors via the draft/fix pipeline.

**Estimated LOC**: ~900 Rust + ~150 test lines

**Depends on**: Wave 1 (`parse_tree_block`, `TreeNode`, `AsciiTreeConfig`)

### Files

| File | Action |
|------|--------|
| `src/tree/mod.rs` | New — declares `pub mod dirtree;` |
| `src/tree/dirtree.rs` | New — filesystem walk, generation, path validation |
| `src/checks/ascii_tree.rs` | Extend — add T-7 / T-8 validators |
| `src/config.rs` | Extend `AsciiTreeConfig` with dirtree options |
| `src/draft.rs` | Add `tree_connector` auto-fix case in `compute_auto_fix()` |
| `src/lib.rs` | Add `pub mod tree;` |

### Key structs and functions

```rust
// src/tree/dirtree.rs

pub struct DirtreeOptions {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub exclude: Vec<String>,        // glob patterns
    pub dirs_first: bool,            // default: true
    pub sort: SortOrder,             // Name | Ext | Size | Mtime
    pub annotate: bool,              // add " — description" from YAML
    pub wrap_fence: bool,            // default: true
    pub indent_width: usize,         // default: 4
}

pub enum SortOrder { Name, Ext, Size, Mtime }

pub fn generate(opts: &DirtreeOptions) -> anyhow::Result<String>
pub fn verify_paths(nodes: &[TreeNode], root: &Path) -> Vec<(usize, String)>
// returns (line_no, missing_path) pairs

fn walk_dir(root: &Path, opts: &DirtreeOptions) -> anyhow::Result<Vec<DirEntry>>
fn render_tree(entries: &[DirEntry], opts: &DirtreeOptions) -> String
fn render_node(entry: &DirEntry, prefix: &str, is_last: bool, opts: &DirtreeOptions) -> String
```

```rust
// src/checks/ascii_tree.rs additions

fn validate_t7(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
// Directories must end with /; files must not.
// Heuristic: treat a node ending with / as directory. A node NOT ending with /
// whose label contains a "." in a filename-like position is treated as a file.
// Ambiguous cases (no extension, no slash) are not flagged — only clear violations.

fn validate_t8(nodes: &[TreeNode], path: &Path) -> Vec<Diagnostic>
// Duplicate entry names under same parent.
// Build a HashMap<(indent_level, parent_key), HashSet<label>> during iteration.
// Flag second occurrence of any label under the same parent.
```

**Config additions** (in `AsciiTreeConfig`):

```rust
#[serde(default = "bool_true")]
pub check_dir_slash: bool,        // T-7 — default: true
#[serde(default = "bool_true")]
pub check_duplicates: bool,       // T-8 — default: true
#[serde(default)]
pub verify_paths: bool,           // filesystem existence check — default: false
#[serde(default)]
pub verify_root: Option<String>,  // root dir for verify_paths
```

### Auto-fix in `draft.rs`

In `compute_auto_fix()`, add a `tree_connector` case:

```rust
if codes_on_line.iter().any(|&c| c == "tree_connector") {
    if let Some(fixed) = fix_tree_connector(old_string) {
        return (fixed, true);
    }
}
```

`fix_tree_connector(line: &str) -> Option<String>`:
- Scans the line for `├` or `└` at the correct indent level
- Recomputes whether this node is the last sibling by looking at the next sibling
  line in the diagnostic's rich context (the rich context must carry the sibling info)
- This requires the `tree_connector` diagnostic to carry enough context: the diagnostic
  message should encode whether the fix is "swap ├ → └" or "swap └ → ├", which
  `fix_tree_connector` can parse deterministically

For indent normalization, `tree_indent` diagnostics encode the expected vs actual
indent, making `fix_tree_indent` a deterministic prefix-replacement.

### Tests

Location: `src/tree/dirtree.rs` + `src/checks/ascii_tree.rs`

| Test | Covers |
|------|--------|
| `t7_dir_missing_slash` | T-7: directory entry without trailing `/` |
| `t7_file_has_slash` | T-7: file entry ending with `/` |
| `t7_ambiguous_no_flag` | T-7: bare name without extension not flagged |
| `t8_duplicate_under_same_parent` | T-8: same name twice under same parent |
| `t8_same_name_different_parents_ok` | T-8: `lib.rs` under two different dirs |
| `generate_simple_tree` | Walk a temp dir, check output format |
| `generate_max_depth` | `--max-depth 1` stops at one level |
| `generate_exclude_glob` | `--exclude target/**` skips matched dirs |
| `generate_dirs_first` | Directories appear before files |
| `generate_sorts_by_name` | Files sorted alphabetically |
| `generate_wraps_fence` | Output has ` ```dirtree ` and ` ``` ` wrapper |
| `verify_paths_missing` | Missing path emits `tree_path_missing` |
| `verify_paths_all_exist` | All paths present — no diagnostic |
| `auto_fix_corner_to_tee` | `fix_tree_connector` swaps └ → ├ correctly |
| `auto_fix_tee_to_corner` | `fix_tree_connector` swaps ├ → └ correctly |

**Exit criterion**: `proof tree generate --kind dirtree --root src/` produces a
valid tree (zero `tree_connector` or `tree_indent` errors when re-checked).
`proof tree check --verify-paths --root .` validates an existing tree against disk.

---

## Wave 3 — Source schema parsing + other kinds

**Goal**: Parse markdown tables into `Vec<TreeNode>` for each non-dirtree kind
(org, taxonomy, dependency, outline, decision). Each schema parser validates its
kind-specific invariants (T-9 through T-11) and emits kind-specific diagnostic
codes. Generation from `md://` URIs.

**Estimated LOC**: ~1,100 Rust + ~200 test lines

**Depends on**: Wave 1 (TreeNode, render primitives), Wave 2 (DirtreeOptions pattern)

### Files

| File | Action |
|------|--------|
| `src/tree/schema.rs` | New — all schema parsers, one per kind |
| `src/tree/generate.rs` | New — shared tree renderer from `Vec<TreeNode>` |
| `src/tree/dirtree.rs` | Minor — export `render_tree` for reuse |

### Key structs and functions

```rust
// src/tree/schema.rs

pub enum TreeKind { Org, Taxonomy, Dependency, Outline, Decision }

pub struct SchemaParseResult {
    pub nodes: Vec<TreeNode>,
    pub errors: Vec<SchemaError>,
}

pub struct SchemaError {
    pub code: &'static str,
    pub message: String,
    pub source_row: Option<usize>,  // row in source table, 1-based
}

// One parser function per kind — uniform signature:
pub fn parse_org(table: &[Vec<String>]) -> SchemaParseResult
pub fn parse_taxonomy(table: &[Vec<String>], levels: &[String]) -> SchemaParseResult
pub fn parse_dependency(table: &[Vec<String>]) -> SchemaParseResult
pub fn parse_outline(markdown_content: &str) -> SchemaParseResult
pub fn parse_decision(table: &[Vec<String>]) -> SchemaParseResult

// Shared table reader — parses GFM table into rows of trimmed cell strings.
// Skips the separator row (|---|---| line). Returns header row + body rows.
pub fn read_gfm_table(content: &str) -> Option<(Vec<String>, Vec<Vec<String>>)>

// Shared tree renderer — converts Vec<TreeNode> back to ASCII tree string.
// Recomputes ├/└ from structure; does not trust existing connectors.
// Used by all generation paths.
pub fn render_nodes_to_string(nodes: &[TreeNode], indent_width: usize) -> String
```

**`parse_org` internals**:
- Expect columns: `Name`, `Parent`, `Label` (Label optional)
- Build `HashMap<name, Vec<children>>` from rows
- Find root: row where Parent cell is `—` or `-` or empty
- T-9: zero roots → `tree_no_root`; multiple roots → `tree_connector`
- T-10: cycle detection via DFS with a `visiting` set → `tree_cycle`
- DFS traversal produces ordered `Vec<TreeNode>` with correct indent levels

**`parse_taxonomy` internals**:
- Columns: `Label`, `Parent`, `Level`
- Validate levels are in the declared `levels` array order
- Each node's level must be exactly one step below parent's level → `tree_level_skip`
- Same root/cycle checks as `parse_org`

**`parse_dependency` internals**:
- Columns: `Package`, `Depends On`, `Version`
- Root = package that appears in `Package` but never in `Depends On`
- DFS traversal with deduplication: first DFS occurrence is rendered fully;
  subsequent occurrences → `(deduped ↑ N)` where N is the `line_no` of the
  first occurrence in the rendered output
- Cycle detection → `tree_cycle`

**`parse_outline` internals**:
- No table — directly parse heading structure of the `md://` target
- Scan lines for `# ... ` patterns; extract level (1–6) and text
- T-3 equivalent: heading levels must not skip (H1 → H3 → error)
- Build `Vec<TreeNode>` where indent_level = heading_level - 1

**`parse_decision` internals**:
- Columns: `Node`, `Condition`, `Yes →`, `No →`
- Build a map of Node → (Yes-target, No-target)
- T-11: every non-leaf must have exactly 2 children → `tree_child_count`
- A leaf is a node whose name never appears as a Yes/No target from another node
  AND whose own Yes/No columns are empty
- Orphan: any Yes/No target that does not appear in the Node column → `tree_orphan`

### `src/tree/generate.rs`

```rust
pub struct GenerateOptions {
    pub kind: TreeKind,
    pub source_uri: Option<String>,  // md:// URI for schema-driven kinds
    pub dirtree_opts: Option<DirtreeOptions>,  // for kind=dirtree
    pub indent_width: usize,
    pub wrap_fence: bool,
}

pub fn generate(opts: &GenerateOptions, root: &Path) -> anyhow::Result<String>
// Dispatches to the right parser/renderer based on opts.kind.
// For dirtree: delegates to dirtree::generate().
// For schema kinds: reads source via mdpath::resolve(), calls the schema parser,
//   validates, then renders via render_nodes_to_string().
```

### Diagnostic codes added in Wave 3

| Code | Kind | Invariant |
|------|------|-----------|
| `tree_no_root` | org, taxonomy | T-9 |
| `tree_cycle` | org, taxonomy, dependency | T-10 |
| `tree_level_skip` | taxonomy | Level ordering |
| `tree_child_count` | decision | T-11 |
| `tree_duplicate` | dirtree (T-8, already in W2), org/taxonomy | duplicate Name |

### Tests

Location: `src/tree/schema.rs`

| Test | Kind | Covers |
|------|------|--------|
| `org_parse_simple` | org | 5-node org chart, no errors |
| `org_no_root_error` | org | T-9: no row with Parent=— |
| `org_multiple_roots_error` | org | T-9: two rows with Parent=— |
| `org_cycle_detected` | org | T-10: A→B→A cycle |
| `org_orphan_parent` | org | Parent references non-existent name |
| `taxonomy_linear_chain` | taxonomy | Valid phyla chain |
| `taxonomy_level_skip` | taxonomy | Kingdom → Class (skips Phylum) |
| `taxonomy_wrong_parent_level` | taxonomy | Parent at non-adjacent level |
| `dependency_simple` | dependency | 3-package tree, correct dedup |
| `dependency_cycle` | dependency | tree_cycle emitted |
| `dependency_dedup_annotation` | dependency | (deduped ↑ N) on second occurrence |
| `outline_simple` | outline | H1/H2/H3 parse |
| `outline_heading_skip` | outline | H1 → H3 (no H2) |
| `decision_valid_binary` | decision | 3-node decision tree |
| `decision_non_leaf_missing_branch` | decision | tree_child_count: only Yes, no No |
| `decision_orphan_target` | decision | Yes → references undeclared node |
| `render_nodes_recomputes_connectors` | shared | Corrupted ├/└ recomputed correctly |
| `read_gfm_table_parses_header` | shared | Header row extracted |
| `read_gfm_table_skips_separator` | shared | `|---|---| ` row excluded |

**Exit criterion**: each schema parser has 10+ tests passing. `proof tree generate
--kind org md://docs/team.md#engineering-org:table:0` produces a valid tree (zero
structural errors when re-checked). All schema error codes appear in at least one
test.

---

## Wave 4 — CLI + proof:tree compile directive

**Goal**: Full `proof tree` subcommand surface, `proof:tree` compile directive in
`compile.rs`, and mtime-based cache for dirtree generation.

**Estimated LOC**: ~750 Rust + ~100 test lines

**Depends on**: Waves 1–3

### Files

| File | Action |
|------|--------|
| `src/commands/mod.rs` | New — declares `pub mod tree;` |
| `src/commands/tree.rs` | New — `cmd_tree()` dispatcher + subcommand handlers |
| `src/main.rs` | Add `Tree` variant to `Command` enum; add `pub mod commands;` to `lib.rs` |
| `src/compile.rs` | Add `Tree` variant to `Directive` enum; extend `proof_directive_kind()`, `collect_directives()`, and `compile_file()` |
| `src/tree/cache.rs` | New — mtime-based cache key for dirtree, Tier-2 key for schema |

### CLI surface in `src/commands/tree.rs`

```rust
// Three subcommands under `proof tree`:

pub fn cmd_tree_check(
    uri: Option<String>,
    kind: Option<String>,
    verify_paths: bool,
    root: Option<PathBuf>,
    config: &GlintConfig,
) -> anyhow::Result<()>
// Lints the target file or URI for tree errors.
// If kind is given, restrict to that kind's rules.
// If verify_paths, enable filesystem existence checks.

pub fn cmd_tree_fix(
    uri: String,
    config: &GlintConfig,
) -> anyhow::Result<()>
// Run check, build draft plan, apply all auto-fixable groups.
// Same flow as cmd_fix but scoped to tree_* diagnostic codes.

pub fn cmd_tree_generate(
    kind: String,
    source_uri: Option<String>,
    root: Option<PathBuf>,
    max_depth: Option<usize>,
    exclude: Vec<String>,
    dirs_first: bool,
    sort: String,
    annotate: bool,
    wrap_fence: bool,
    indent_width: usize,
    output: Option<PathBuf>,
    config: &GlintConfig,
) -> anyhow::Result<()>
// Dispatch to generate::generate() and write to output or stdout.
```

**`Command` enum addition** in `main.rs`:

```rust
/// Validate, fix, and generate ASCII trees
Tree {
    #[command(subcommand)]
    subcommand: TreeCommand,
},
```

```rust
#[derive(Subcommand)]
enum TreeCommand {
    Check {
        uri: Option<String>,
        #[arg(long)] kind: Option<String>,
        #[arg(long)] verify_paths: bool,
        #[arg(long)] root: Option<PathBuf>,
    },
    Fix {
        uri: String,
    },
    Generate {
        #[arg(long, required = true)] kind: String,
        source_uri: Option<String>,
        #[arg(long)] root: Option<PathBuf>,
        #[arg(long)] max_depth: Option<usize>,
        #[arg(long, num_args = 0..)] exclude: Vec<String>,
        #[arg(long, default_value = "true")] dirs_first: bool,
        #[arg(long, default_value = "name")] sort: String,
        #[arg(long)] annotate: bool,
        #[arg(long, default_value = "true")] wrap_fence: bool,
        #[arg(long, default_value = "4")] indent_width: usize,
        #[arg(short = 'o', long)] output: Option<PathBuf>,
    },
}
```

### `proof:tree` directive in `compile.rs`

**`proof_directive_kind()` extension**:
```rust
else if rest.starts_with("tree") { Some("tree") }
```

**`Directive` enum addition**:
```rust
Tree {
    kind: String,
    attrs: TreeAttrs,
    source_uri: Option<String>,  // body line starting with md://
    line_start: usize,
    line_end: usize,
},
```

**`TreeAttrs` struct** (mirrors `LayoutAttrs` pattern):
```rust
struct TreeAttrs {
    kind: String,          // dirtree | org | taxonomy | dependency | outline | decision
    root: Option<String>,  // filesystem root for dirtree
    max_depth: Option<usize>,
    exclude: Vec<String>,
    verify_paths: bool,
    indent_width: usize,   // default: 4
    sort: String,          // default: name
    dirs_first: bool,      // default: true
}

impl TreeAttrs {
    fn parse(attrs_str: &str) -> Self { ... }  // same key=value scanner as LayoutAttrs::parse
}
```

**`compile_file()` extension** — new `Directive::Tree` match arm:
```rust
Directive::Tree { kind, attrs, source_uri, .. } => {
    let cache_key = tree_cache_key(kind, attrs, source_uri, root);
    let opts = GenerateOptions { ... };
    match generate(&opts, root) {
        Ok(tree_content) => format_tree_block(source_uri.as_deref(), &tree_content, attrs),
        Err(e) => { /* emit COMPILE-002 violation */ ... }
    }
}
```

**Output formatting**:
```rust
fn format_tree_block(source: Option<&str>, content: &str, attrs: &TreeAttrs) -> String {
    let from = source.unwrap_or(&format!("proof:tree kind={}", attrs.kind));
    format!(
        "<!-- proof:compiled from=\"{}\" -->\n```dirtree\n{}\n```\n<!-- /proof:compiled -->",
        from, content
    )
}
```

### `src/tree/cache.rs`

```rust
pub fn dirtree_cache_key(root: &Path, opts: &DirtreeOptions) -> String
// Walks root directory collecting (path, mtime) pairs.
// Sorts by path for stability, then hashes with FxHasher or std DefaultHasher.
// Returns hex string. Used as part of a larger cache key in compile.rs.
// NOTE: This is a cache *key* computation, not a full caching system.
// The caller (compile.rs) checks if the output file is newer than the key changed.

pub fn schema_cache_key(uri: &str, root: &Path) -> anyhow::Result<String>
// Delegates to mdpath::resolve to get the source file path,
// then returns its mtime as the cache key (Tier 2 pattern from the spec).
```

### Tests

Location: `src/commands/tree.rs` + `src/compile.rs` (existing test module)

| Test | Covers |
|------|--------|
| `parse_tree_directive_dirtree` | `proof:tree kind=dirtree root=src/` parsed correctly |
| `parse_tree_directive_org` | `proof:tree kind=org` + body URI parsed |
| `tree_attrs_parse_max_depth` | `max-depth=3` → `max_depth: Some(3)` |
| `tree_attrs_parse_exclude` | `exclude=target/**,*.log` splits correctly |
| `tree_attrs_parse_verify_paths` | `verify-paths` flag parsed |
| `tree_attrs_parse_indent_width` | `indent-width=2` → `indent_width: 2` |
| `compile_tree_directive_dirtree_e2e` | Write temp source.md → compile → check output |
| `compile_tree_directive_org_e2e` | `proof:tree kind=org md://...` → resolved tree in output |
| `compile_tree_missing_source_error` | Bad URI → COMPILE-002 violation, no output written |
| `dirtree_cache_key_stable` | Same dir twice → same key |
| `dirtree_cache_key_changes_on_mtime` | After touching a file, key differs |

**Exit criterion**: Full E2E test:
1. Write a `.source.md` with a `proof:tree kind=dirtree root=src/` directive
2. Run `proof compile`
3. Output file contains a `<!-- proof:compiled -->` block with a valid dirtree
4. `proof check` on the output emits zero `tree_*` errors

---

## Wave dependencies

```
Wave 1  ──────────────────────────────────►  structural parser, T-1/T-6/T-12, config, runner wiring
  │
  ▼
Wave 2  ──────────────────────────────────►  T-7/T-8, dirtree generation, auto-fix, verify-paths
  │
  ▼
Wave 3  ──────────────────────────────────►  schema parsers (org/taxonomy/dependency/outline/decision)
  │                                           generate.rs, kind-specific diagnostics
  ▼
Wave 4  ──────────────────────────────────►  proof tree CLI, proof:tree compile directive, cache
```

---

## LOC estimate summary

| Wave | File(s) | Est. Rust LOC | Est. Test LOC |
|------|---------|--------------|--------------|
| 1 | `ascii_tree.rs`, config/runner wiring | ~500 | ~200 |
| 2 | `tree/dirtree.rs`, draft.rs additions | ~700 | ~150 |
| 3 | `tree/schema.rs`, `tree/generate.rs` | ~850 | ~200 |
| 4 | `commands/tree.rs`, compile.rs additions, cache.rs | ~600 | ~100 |
| **Total** | | **~2,650** | **~650** |

Total including comments and blank lines: ~3,500–4,200 lines.

---

## Cross-cutting notes

**ASCII fallback acceptance**: All parsers accept `+--`/`\--`/`|` in addition to
Unicode `├──`/`└──`/`│`. This is structural, not stylistic — the validator never
emits a diagnostic for using ASCII fallbacks.

**False positive guard**: `AsciiTreeCheck` only activates inside code blocks with
an explicit `dirtree` info string (` ```dirtree `). Code blocks tagged `rust`,
`python`, `ascii_box`, etc. are not scanned — this prevents `│` inside flowcharts
or box diagrams from triggering `tree_orphan`.

**Render vs validate**: `render_nodes_to_string()` (Wave 3) always recomputes
connectors from structure. It does not trust the `connector` field on input nodes.
This means it is safe to use for auto-fix (Wave 2) and generation (Waves 2–3)
without the caller caring about stale connector values.

**`proof.toml` naming convention**: All `AsciiTreeConfig` fields use `snake_case`
in TOML. CLI flags use `--kebab-case`. Directive attributes use `kebab-case`.
This matches the convention established for all other proof directives.

**`md://` URI handling**: All Wave 3/4 source resolution goes through
`mdpath::resolve()`, exactly as `compile.rs` does for `proof:include`. Wave 4's
`compile.rs` additions reuse the existing `resolve_uri()` helper unchanged.
