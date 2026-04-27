use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GlintConfig {
    /// Explicit parent config to inherit from (overrides auto-cascade).
    /// Path is relative to this config file's directory.
    pub extends: Option<String>,

    #[serde(default)]
    pub meta: MetaConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub ascii_box: AsciiBoxConfig,
    #[serde(default)]
    pub ascii_barchart: AsciiBarchartConfig,
    #[serde(default)]
    pub ascii_char: AsciiCharConfig,
    #[serde(default)]
    pub ascii_tree: AsciiTreeConfig,
    #[serde(default)]
    pub ascii_flow: AsciiFlowConfig,
    #[serde(default)]
    pub markdown: MarkdownConfig,
    #[serde(default)]
    pub markdown_table: MarkdownTableConfig,
    /// Per-directory schema overrides. Each entry applies to files matching `paths`.
    #[serde(default)]
    pub section_schemas: Vec<SectionSchema>,
    #[serde(default)]
    pub custom_rules: Vec<CustomRule>,
    /// Pinned figures with invariant protection (DaVinci tier).
    #[serde(default)]
    pub davinci: Vec<DaVinciEntry>,
}

/// A schema applied to files matching a glob pattern.
/// Merged additively on top of the root markdown config.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct SectionSchema {
    /// Glob patterns — files matching ANY of these are candidates.
    /// In a directory-level proof.toml, paths are relative to that directory.
    /// In the root proof.toml, paths are relative to the root.
    /// Example: `["*.md"]` matches all markdown files in the directory.
    pub paths: Vec<String>,

    /// Exclusion globs — files matching any of these are skipped even if
    /// they match `paths`. Use this to carve out special cases without
    /// listing every other file explicitly.
    /// Example: `paths_exclude = ["00-OVERVIEW.md"]` skips the overview
    /// while `paths = ["*.md"]` catches everything else.
    #[serde(default)]
    pub paths_exclude: Vec<String>,

    /// Additional required H2 headings (all must be present)
    #[serde(default)]
    pub required_h2_all: Vec<String>,
    /// Additional required H2 headings (at least one must be present)
    #[serde(default)]
    pub required_h2: Vec<String>,
    /// Additional required content patterns
    #[serde(default)]
    pub required_patterns: Vec<RequiredPattern>,
    /// Override max_lines for matching files
    pub max_lines: Option<usize>,
}

/// GFM pipe table validator configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct MarkdownTableConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Minimum number of dashes in each separator cell (GFM requires ≥ 3)
    #[serde(default = "default_sep_dashes")]
    pub min_separator_dashes: usize,
    /// Check cell padding
    #[serde(default = "bool_true")]
    pub check_cell_padding: bool,
    #[serde(default = "default_min_padding")]
    pub min_cell_padding: usize,
    /// Minimum number of pipe tables per file
    pub required_tables: Option<usize>,
    /// Named table schemas with structural requirements
    #[serde(default)]
    pub table_schemas: Vec<TableSchema>,
    /// Warn when a column header cell is empty
    #[serde(default = "bool_true")]
    pub check_empty_headers: bool,
    /// Warn when a table has more than this many columns (0 = no limit)
    #[serde(default)]
    pub max_columns: usize,
    /// Don't flag body rows that have MORE columns than the header.
    /// Use this when guide content includes pipe characters in math/code
    /// (e.g. |G| group notation, regex, bitwise ops) that are parsed as
    /// extra column separators. Rows with FEWER columns than the header
    /// are still flagged (those are genuine missing-column errors).
    #[serde(default)]
    pub ignore_extra_body_cols: bool,
}

impl Default for MarkdownTableConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_separator_dashes: 3,
            check_cell_padding: true,
            min_cell_padding: 1,
            required_tables: None,
            table_schemas: Vec::new(),
            check_empty_headers: true,
            max_columns: 0,
            ignore_extra_body_cols: false,
        }
    }
}

/// Schema for a required table — structural constraints that a table must satisfy.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TableSchema {
    /// Table must appear under this exact ## heading text (without the `##`).
    /// If None, applies to any table in the file.
    pub heading: Option<String>,
    /// All of these column headers must be present (exact match)
    #[serde(default)]
    pub required_columns: Vec<String>,
    /// At least one of these column headers must be present
    #[serde(default)]
    pub required_columns_any: Vec<String>,
    /// Minimum body rows (excluding header + separator)
    pub min_body_rows: Option<usize>,
    /// Values that must appear in the first (key) column of body rows
    #[serde(default)]
    pub required_row_keys: Vec<String>,
    /// Allowed values per column: { "ColumnName": ["allowed1", "allowed2"] }
    #[serde(default)]
    pub column_allowed_values: std::collections::HashMap<String, Vec<String>>,

    // ── Link validation ────────────────────────────────────────────────────

    /// Columns where every body cell MUST contain at least one markdown link.
    /// Pattern: `[text](url)` — bare text is flagged as `md_table_missing_link`.
    /// Example: `link_columns = ["Directory", "Entry Point"]`
    #[serde(default)]
    pub link_columns: Vec<String>,

    /// Auto-fix strategy for link_columns cells that have bare text:
    /// "directory" — `computing/` → `[computing/](../computing/00-OVERVIEW.md)`
    /// "file"      — `01-PKG.md` → `[01-PKG.md](../dirname/01-PKG.md)`
    /// ""          — no auto-fix
    #[serde(default)]
    pub link_auto_fix: String,

    /// Check that link targets (relative file paths) exist on disk.
    #[serde(default)]
    pub verify_link_targets: bool,
}

/// ASCII bar chart validator.
#[derive(Debug, Deserialize, Clone)]
pub struct AsciiBarchartConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Minimum consecutive bar characters to count as a bar (default 3)
    #[serde(default = "default_min_bar_width")]
    pub min_bar_width: usize,
    /// Minimum number of consecutive bar rows to count as a chart (default 2)
    #[serde(default = "default_min_chart_rows")]
    pub min_chart_rows: usize,
    /// Characters that form the bar body. Empty = use defaults (█▓▒░#=)
    #[serde(default)]
    pub bar_chars: Vec<String>,
    /// Minimum spaces between label text and bar start
    #[serde(default = "default_one")]
    pub min_label_padding: usize,
    /// Minimum spaces between bar end and value
    #[serde(default = "default_one")]
    pub min_value_padding: usize,
    /// Warn when value formats differ across rows (% vs integer vs float)
    #[serde(default = "bool_true")]
    pub check_value_format: bool,
    /// Warn when value column is not aligned across rows
    #[serde(default = "bool_true")]
    pub require_value_alignment: bool,
    /// Tolerance in columns for value alignment (default 1)
    #[serde(default = "default_one")]
    pub alignment_tolerance: usize,
    /// Warn when bar widths are not proportional to their numeric values.
    /// e.g. a bar at 78% that fills 100% of the max bar width is disproportionate.
    #[serde(default = "bool_true")]
    pub check_proportionality: bool,
    /// Tolerance in bar characters for proportionality (default 2 — rounding errors)
    #[serde(default = "default_prop_tolerance")]
    pub proportionality_tolerance: usize,
}

impl Default for AsciiBarchartConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_bar_width: 3,
            min_chart_rows: 2,
            bar_chars: Vec::new(),
            min_label_padding: 1,
            min_value_padding: 1,
            check_value_format: true,
            require_value_alignment: true,
            alignment_tolerance: 1,
            check_proportionality: true,
            proportionality_tolerance: 2,
        }
    }
}

fn default_min_bar_width() -> usize { 3 }
fn default_min_chart_rows() -> usize { 2 }
fn default_one() -> usize { 1 }
fn default_prop_tolerance() -> usize { 2 }
fn default_sep_dashes() -> usize { 3 }

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MetaConfig {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilesConfig {
    #[serde(default = "default_include")]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Stop cascading up past this directory (like tsconfig's `root = true`)
    #[serde(default)]
    pub root: bool,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            include: default_include(),
            exclude: Vec::new(),
            root: false,
        }
    }
}

fn default_include() -> Vec<String> {
    vec!["**/*.md".to_string()]
}

#[derive(Debug, Deserialize, Clone)]
pub struct AsciiBoxConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Columns of tolerance for misalignment (0 = exact match required)
    #[serde(default)]
    pub tolerance: usize,
    /// Only check inside fenced code blocks (recommended)
    #[serde(default = "bool_true")]
    pub code_blocks_only: bool,
    /// Also validate Unicode box-drawing character boxes
    #[serde(default = "bool_true")]
    pub check_unicode: bool,
    /// Tab width for visual column calculation (CommonMark default: 4)
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    /// Check that column separators (│ positions) are consistent row-to-row.
    /// Disable for diagrams with multiple independent side-by-side boxes
    /// (spatial layouts) where different rows legitimately have │ at different columns.
    #[serde(default = "bool_true")]
    pub check_col_separators: bool,
}

impl Default for AsciiBoxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance: 0,
            code_blocks_only: true,
            check_unicode: true,
            tab_width: 4,
            check_col_separators: true,
        }
    }
}

/// Character range check (Style Guide Rule S-01).
#[derive(Debug, Deserialize, Clone)]
pub struct AsciiCharConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Error on wide/fullwidth chars (2-col) that will break alignment (always recommended)
    #[serde(default = "bool_true")]
    pub error_on_wide: bool,
    /// Also warn on narrow chars outside the safe Unicode ranges
    #[serde(default)]
    pub warn_unusual: bool,
}

impl Default for AsciiCharConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            error_on_wide: true,
            warn_unusual: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AsciiTreeConfig {
    /// Master switch
    #[serde(default = "bool_true")]
    pub enabled: bool,
    /// Spaces per indentation level (default: 4)
    #[serde(default = "default_indent_width")]
    pub indent_width: usize,
    /// Only validate code blocks with this specific info string.
    /// None = validate all tree-kind fences (dirtree, tree, org, taxonomy, etc.)
    #[serde(default)]
    pub kind: Option<String>,
}

fn default_indent_width() -> usize { 4 }

impl Default for AsciiTreeConfig {
    fn default() -> Self {
        Self { enabled: true, indent_width: 4, kind: None }
    }
}

fn default_tab_width() -> usize { 4 }

#[derive(Debug, Deserialize, Clone)]
pub struct AsciiFlowConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default = "bool_true")]
    pub check_arrow_alignment: bool,
    #[serde(default = "bool_true")]
    pub check_cell_padding: bool,
    #[serde(default = "default_min_padding")]
    pub min_cell_padding: usize,
}

impl Default for AsciiFlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_arrow_alignment: true,
            check_cell_padding: true,
            min_cell_padding: 1,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarkdownConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Maximum number of H1 headings per file
    pub max_h1: Option<usize>,
    /// Required H2 headings — at least one must be present
    #[serde(default)]
    pub required_h2: Vec<String>,
    /// Required H2 headings — ALL must be present
    #[serde(default)]
    pub required_h2_all: Vec<String>,
    /// Content patterns that must appear
    #[serde(default)]
    pub required_patterns: Vec<RequiredPattern>,
    /// Max file length in lines
    pub max_lines: Option<usize>,

    // ── Heading quality checks ──────────────────────────────────────────────

    /// Warn on headings missing the required space after `#` (e.g. `##heading`)
    #[serde(default = "bool_true")]
    pub check_heading_format: bool,
    /// Warn on empty headings (`## ` with no content)
    #[serde(default = "bool_true")]
    pub check_empty_headings: bool,
    /// Warn when heading levels skip (H1 → H3 without H2)
    #[serde(default = "bool_true")]
    pub check_heading_hierarchy: bool,
    /// Warn on duplicate heading text at the same level within a file
    #[serde(default)]
    pub check_duplicate_headings: bool,

    // ── Document style checks ────────────────────────────────────────────────

    /// Enforce a consistent thematic break style: "---" | "***" | "___" | "" (any)
    #[serde(default)]
    pub thematic_break_style: Option<String>,
    /// Warn when `>` block quotes are missing the required space (`>text` vs `> text`)
    #[serde(default)]
    pub check_blockquote_spacing: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_h1: None,
            required_h2: Vec::new(),
            required_h2_all: Vec::new(),
            required_patterns: Vec::new(),
            max_lines: None,
            check_heading_format: true,
            check_empty_headings: true,
            check_heading_hierarchy: true,
            check_duplicate_headings: false,
            thematic_break_style: None,
            check_blockquote_spacing: false,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RequiredPattern {
    pub pattern: String,
    pub description: String,
    #[serde(default)]
    pub severity: PatternSeverity,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum PatternSeverity {
    #[default]
    Error,
    Warning,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomRule {
    pub name: String,
    pub description: String,
    pub pattern: String,
    /// Warn when pattern IS found (inverse match)
    #[serde(default)]
    pub negate: bool,
    #[serde(default = "default_custom_severity")]
    pub severity: String,
    /// Restrict to files matching these globs
    #[serde(default)]
    pub only_in: Vec<String>,
}

fn bool_true() -> bool { true }
fn default_min_padding() -> usize { 1 }
fn default_custom_severity() -> String { "warning".to_string() }

// ─────────────────────────────────────────────────────────
// Config resolution: cascade up the directory tree
// ─────────────────────────────────────────────────────────

impl GlintConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading config file: {}", path.display()))?;
        let config: GlintConfig = toml::from_str(&content)
            .with_context(|| format!("parsing config file: {}", path.display()))?;
        Ok(config)
    }

    /// Resolve the effective config for a file at `file_path` by cascading up
    /// the directory tree. Configs are merged: parent first, then child overrides.
    ///
    /// section_schemas paths are automatically prefixed with the config file's
    /// directory relative to root_dir. This means a `languages/proof.toml` can
    /// write `paths = ["02-*.md"]` instead of `paths = ["languages/02-*.md"]`.
    ///
    /// Cascade stops when a config has `files.root = true` or we hit root_dir.
    pub fn resolve_for(file_path: &Path, root_dir: &Path) -> Self {
        let dir = file_path.parent().unwrap_or(file_path);
        let mut configs_with_origin = collect_configs_up_with_origin(dir, root_dir);
        configs_with_origin.reverse(); // root first, nearest-to-file last

        // Prefix each config's section_schema paths with that config's
        // directory relative to root, so directory-level configs can use
        // simple names like "02-*.md" rather than "languages/02-*.md".
        let prefixed = configs_with_origin.into_iter().map(|(origin_dir, mut cfg)| {
            let prefix = origin_dir
                .strip_prefix(root_dir)
                .unwrap_or(Path::new(""))
                .to_string_lossy()
                .replace('\\', "/");

            if !prefix.is_empty() {
                for schema in &mut cfg.section_schemas {
                    let prefix_glob = |p: &String| -> String {
                        if p.starts_with('/') { p.clone() } // root-absolute — leave it
                        else { format!("{}/{}", prefix, p) }
                    };
                    schema.paths = schema.paths.iter().map(prefix_glob).collect();
                    schema.paths_exclude = schema.paths_exclude.iter().map(prefix_glob).collect();
                }
            }
            cfg
        });

        prefixed.fold(GlintConfig::default(), |acc, cfg| merge(acc, cfg))
    }

    pub fn load_or_default(dir: &Path) -> Self {
        for name in &["proof.toml", ".proof.toml", ".proof/config.toml"] {
            let path = dir.join(name);
            if path.exists() {
                match Self::load(&path) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("proof: warning: {}", e),
                }
            }
        }
        GlintConfig::default()
    }
}

/// Walk from `dir` up to `root_dir`, collecting every proof.toml found.
/// Returns (origin_directory, config) pairs ordered nearest-first.
///
/// The origin_directory is the directory where each proof.toml was found.
/// It is used by resolve_for() to prefix section_schema paths so that a
/// `languages/proof.toml` can write `paths = ["02-*.md"]` not `["languages/02-*.md"]`.
fn collect_configs_up_with_origin(dir: &Path, root_dir: &Path) -> Vec<(PathBuf, GlintConfig)> {
    let mut configs: Vec<(PathBuf, GlintConfig)> = Vec::new();
    let mut current = dir.to_path_buf();

    loop {
        if let Some(cfg) = try_load_config(&current) {
            let is_root = cfg.files.root;

            // Handle explicit `extends` — load and insert at lower priority
            if let Some(ref parent_rel) = cfg.extends.clone() {
                let parent_abs = current.join(parent_rel);
                let parent_dir = parent_abs.parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf();
                match GlintConfig::load(&parent_abs) {
                    Ok(parent_cfg) => {
                        configs.push((current.clone(), cfg));
                        configs.push((parent_dir, parent_cfg));
                    }
                    Err(e) => {
                        eprintln!("proof: warning: extends {:?} failed: {}", parent_abs, e);
                        configs.push((current.clone(), cfg));
                    }
                }
            } else {
                configs.push((current.clone(), cfg));
            }

            if is_root { break; }
        }

        if current == root_dir { break; }

        match current.parent() {
            Some(p) => current = p.to_path_buf(),
            None => break,
        }
    }

    configs
}

fn try_load_config(dir: &Path) -> Option<GlintConfig> {
    for name in &["proof.toml", ".proof.toml"] {
        let path = dir.join(name);
        if path.exists() {
            match GlintConfig::load(&path) {
                Ok(cfg) => return Some(cfg),
                Err(e) => eprintln!("proof: warning: {}", e),
            }
        }
    }
    None
}

/// Merge two configs. `parent` is the ancestor; `child` is closer to the file.
///
/// Merge semantics:
///   - Lists (required sections, patterns, rules) → ADDITIVE (parent + child)
///   - Scalars (tolerance, max_h1, enabled) → child wins
///   - Absent optional scalars (None) → fall through to parent's value
pub fn merge(parent: GlintConfig, child: GlintConfig) -> GlintConfig {
    GlintConfig {
        extends: child.extends,
        meta: if child.meta.name.is_some() { child.meta } else { parent.meta },
        files: merge_files(parent.files, child.files),
        ascii_box: child.ascii_box,   // scalars: child wins entirely
        ascii_barchart: child.ascii_barchart,
        ascii_char: child.ascii_char,
        ascii_tree: child.ascii_tree,
        ascii_flow: child.ascii_flow,
        // markdown_table: child wins (schemas are per-directory, not additive)
        markdown_table: child.markdown_table,
        markdown: merge_markdown(parent.markdown, child.markdown),
        section_schemas: {
            let mut v = parent.section_schemas;
            v.extend(child.section_schemas);
            v
        },
        custom_rules: {
            let mut v = parent.custom_rules;
            v.extend(child.custom_rules);
            v
        },
        // DaVinci entries are additive — all levels contribute pins
        davinci: {
            let mut v = parent.davinci;
            v.extend(child.davinci);
            v
        },
    }
}

/// Merge file selection configs.
/// - `include`: child wins if it differs from the default (non-empty overrides parent)
/// - `exclude`: additive — a child cannot un-exclude what the root excluded
/// - `root`: either can mark the stop point
fn merge_files(parent: FilesConfig, child: FilesConfig) -> FilesConfig {
    FilesConfig {
        // Child's include overrides parent (it knows what files are in its subtree)
        include: if !child.include.is_empty() { child.include } else { parent.include },
        // Exclude is additive: child adds more exclusions on top of parent's
        exclude: {
            let mut v = parent.exclude;
            for pat in child.exclude {
                if !v.contains(&pat) {
                    v.push(pat);
                }
            }
            v
        },
        root: child.root || parent.root,
    }
}

fn merge_markdown(parent: MarkdownConfig, child: MarkdownConfig) -> MarkdownConfig {
    MarkdownConfig {
        // Child's enabled state wins
        enabled: child.enabled || parent.enabled,
        // Scalar: child's explicit value wins; fall back to parent if child has None
        max_h1: child.max_h1.or(parent.max_h1),
        max_lines: child.max_lines.or(parent.max_lines),
        // Lists: additive (both parent and child requirements must hold)
        required_h2: {
            let mut v = parent.required_h2;
            v.extend(child.required_h2);
            v
        },
        required_h2_all: {
            let mut v = parent.required_h2_all;
            v.extend(child.required_h2_all);
            v.dedup();
            v
        },
        required_patterns: {
            let mut v = parent.required_patterns;
            v.extend(child.required_patterns);
            v
        },
        // Scalar style checks: child wins
        check_heading_format: child.check_heading_format,
        check_empty_headings: child.check_empty_headings,
        check_heading_hierarchy: child.check_heading_hierarchy,
        check_duplicate_headings: child.check_duplicate_headings,
        thematic_break_style: child.thematic_break_style.or(parent.thematic_break_style),
        check_blockquote_spacing: child.check_blockquote_spacing,
    }
}


// ─────────────────────────────────────────────────────────────────────────────
// DaVinci — pinned figures with invariant protection
// ─────────────────────────────────────────────────────────────────────────────

/// A pinned figure entry in [[davinci]] of proof.toml.
#[derive(Debug, Deserialize, Clone)]
pub struct DaVinciEntry {
    /// Stable identifier — used in diagnostics and reports
    pub id: String,
    /// md:// URI addressing the pinned element
    pub uri: String,
    /// Human description (shown in `proof pin list`)
    #[serde(default)]
    pub description: String,
    /// Optional template name — inherits its base invariants
    pub template: Option<String>,
    /// What happens when an invariant is violated
    #[serde(default)]
    pub protection: ProtectionTier,
    /// Invariants to enforce on the resolved element
    #[serde(default)]
    pub invariants: Vec<Invariant>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProtectionTier {
    #[default]
    Warn,
    Error,
    Lock,
}

impl std::fmt::Display for ProtectionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtectionTier::Warn => write!(f, "warn"),
            ProtectionTier::Error => write!(f, "error"),
            ProtectionTier::Lock => write!(f, "lock"),
        }
    }
}

/// A single invariant rule on a pinned element.
#[derive(Debug, Deserialize, Clone)]
pub struct Invariant {
    /// Rule name: box-width, contains-text, box-count, column-count,
    ///            row-count, equals, required-row-keys, all-boxes-same-width,
    ///            starts-with, ends-with, pattern, bar-proportional
    pub rule: String,
    /// String parameter (contains-text, equals, starts-with, ends-with, pattern)
    pub text: Option<String>,
    /// Minimum value (box-width.min, row-count.min, etc.)
    pub min: Option<usize>,
    /// Maximum value (box-width.max, row-count.max, etc.)
    pub max: Option<usize>,
    /// Exact value (box-count, column-count)
    pub value: Option<usize>,
    /// List parameter (required-row-keys)
    pub values: Option<Vec<String>>,
    /// Tolerance (bar-proportional, all-boxes-same-width)
    pub tolerance: Option<usize>,
}
