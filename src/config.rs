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
    pub ascii_flow: AsciiFlowConfig,
    #[serde(default)]
    pub markdown: MarkdownConfig,
    /// Per-directory schema overrides. Each entry applies to files matching `paths`.
    #[serde(default)]
    pub section_schemas: Vec<SectionSchema>,
    #[serde(default)]
    pub custom_rules: Vec<CustomRule>,
}

/// A schema applied to files matching a glob pattern.
/// Merged additively on top of the root markdown config.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct SectionSchema {
    /// Glob patterns relative to the root (e.g. `["languages/**"]`)
    pub paths: Vec<String>,
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
}

impl Default for AsciiBoxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance: 0,
            code_blocks_only: true,
            check_unicode: true,
        }
    }
}

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

#[derive(Debug, Deserialize, Default, Clone)]
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
    /// Cascade stops when:
    ///   - We reach `root_dir`
    ///   - A config has `files.root = true`
    ///   - We hit the filesystem root
    pub fn resolve_for(file_path: &Path, root_dir: &Path) -> Self {
        let dir = file_path.parent().unwrap_or(file_path);
        let mut configs = collect_configs_up(dir, root_dir);
        configs.reverse(); // root first, nearest-to-file last
        configs.into_iter().fold(GlintConfig::default(), |acc, cfg| merge(acc, cfg))
    }

    pub fn load_or_default(dir: &Path) -> Self {
        for name in &["glint.toml", ".glint.toml", ".glint/config.toml"] {
            let path = dir.join(name);
            if path.exists() {
                match Self::load(&path) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("glint: warning: {}", e),
                }
            }
        }
        GlintConfig::default()
    }
}

/// Walk from `dir` up to `root_dir`, collecting every glint.toml found.
/// Returns configs ordered nearest-first (dir's config first, root last).
///
/// If a config has `extends = "path/to/parent.toml"`, that file is loaded and
/// inserted immediately after the current config (lower priority, higher in chain).
/// This allows explicit parenting that overrides the auto-cascade direction.
fn collect_configs_up(dir: &Path, root_dir: &Path) -> Vec<GlintConfig> {
    let mut configs = Vec::new();
    let mut current = dir.to_path_buf();

    loop {
        if let Some(cfg) = try_load_config(&current) {
            let is_root = cfg.files.root;

            // Resolve `extends` before pushing — extends acts as an explicit parent,
            // inserted at lower priority (further from the file) than the current config.
            if let Some(ref parent_rel) = cfg.extends.clone() {
                let parent_abs = current.join(parent_rel);
                match GlintConfig::load(&parent_abs) {
                    Ok(parent_cfg) => {
                        // Push current first (nearest → highest priority after reversal)
                        configs.push(cfg);
                        // Then push extends (will end up at lower priority)
                        configs.push(parent_cfg);
                    }
                    Err(e) => {
                        eprintln!("glint: warning: extends {:?} failed: {}", parent_abs, e);
                        configs.push(cfg);
                    }
                }
            } else {
                configs.push(cfg);
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
    for name in &["glint.toml", ".glint.toml"] {
        let path = dir.join(name);
        if path.exists() {
            match GlintConfig::load(&path) {
                Ok(cfg) => return Some(cfg),
                Err(e) => eprintln!("glint: warning: {}", e),
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
        ascii_box: child.ascii_box, // scalars: child wins entirely
        ascii_flow: child.ascii_flow,
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
    }
}
