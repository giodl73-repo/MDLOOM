use crate::checks::ascii_box::AsciiBoxCheck;
use crate::checks::ascii_char::AsciiCharCheck;
use crate::checks::ascii_flow::AsciiFlowCheck;
use crate::checks::markdown::MarkdownCheck;
use crate::checks::Check;
use crate::config::{AsciiBoxConfig, AsciiCharConfig, AsciiFlowConfig, GlintConfig, MarkdownConfig, SectionSchema};
use crate::diagnostic::Diagnostic;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

pub struct Runner {
    root: PathBuf,
    #[allow(dead_code)]
    root_config: GlintConfig,
    /// Cache of per-directory resolved configs (dir path → resolved config)
    config_cache: Arc<Mutex<HashMap<PathBuf, Arc<GlintConfig>>>>,
    include: GlobSet,
    exclude: GlobSet,
}

impl Runner {
    pub fn new(root: &Path, config: GlintConfig) -> anyhow::Result<Self> {
        let include = build_globset(&config.files.include)?;
        let exclude = build_globset(&config.files.exclude)?;
        Ok(Self {
            root: root.to_path_buf(),
            root_config: config,
            config_cache: Arc::new(Mutex::new(HashMap::new())),
            include,
            exclude,
        })
    }

    /// Lint all matching files under root. Returns all diagnostics.
    pub fn run(&self) -> Vec<Diagnostic> {
        let files = self.collect_files();
        files.par_iter()
            .flat_map(|path| self.lint_file(path))
            .collect()
    }

    /// Lint a single file using the cascaded config for its directory.
    pub fn lint_file(&self, path: &Path) -> Vec<Diagnostic> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return vec![Diagnostic::error(
                    path.to_path_buf(), 1, 1, "io_error",
                    format!("cannot read file: {}", e),
                )];
            }
        };

        let config = self.resolve_config_for(path);
        let checks = build_checks(&config, path, &self.root);

        checks.iter()
            .flat_map(|check| check.check(path, &content))
            .collect()
    }

    /// Resolve the effective config for a file by cascading from its directory up to root.
    /// Results are cached by directory path.
    fn resolve_config_for(&self, file: &Path) -> Arc<GlintConfig> {
        let dir = file.parent().unwrap_or(&self.root).to_path_buf();

        // Check cache first
        {
            let cache = self.config_cache.lock().unwrap();
            if let Some(cfg) = cache.get(&dir) {
                return Arc::clone(cfg);
            }
        }

        // Resolve by cascading up to root
        let resolved = GlintConfig::resolve_for(file, &self.root);
        let arc = Arc::new(resolved);

        let mut cache = self.config_cache.lock().unwrap();
        cache.insert(dir, Arc::clone(&arc));
        arc
    }

    fn collect_files(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|p| self.matches(p))
            .collect()
    }

    fn matches(&self, path: &Path) -> bool {
        let rel = path.strip_prefix(&self.root).unwrap_or(path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let included = if self.include.is_empty() { true } else { self.include.is_match(&*rel_str) };
        let excluded = self.exclude.is_match(&*rel_str);
        included && !excluded
    }
}

/// Build the set of checks for a file given its resolved config.
/// Applies section_schemas additively for files that match their path globs.
fn build_checks(config: &GlintConfig, file: &Path, root: &Path) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    if config.ascii_box.enabled {
        checks.push(Box::new(AsciiBoxCheck {
            config: config.ascii_box.clone(),
        }));
    }

    if config.ascii_char.enabled {
        checks.push(Box::new(AsciiCharCheck {
            config: config.ascii_char.clone(),
        }));
    }

    if config.ascii_flow.enabled {
        checks.push(Box::new(AsciiFlowCheck {
            config: config.ascii_flow.clone(),
        }));
    }

    // Build the effective markdown config for this specific file
    let effective_md = effective_markdown(config, file, root);
    if effective_md.enabled {
        checks.push(Box::new(MarkdownCheck { config: effective_md }));
    }

    checks
}

/// Compute the effective MarkdownConfig for a file by applying any matching
/// section_schemas additively on top of the base markdown config.
fn effective_markdown(config: &GlintConfig, file: &Path, root: &Path) -> MarkdownConfig {
    let rel = file.strip_prefix(root).unwrap_or(file);
    // Normalize to forward slashes so glob patterns work on Windows too.
    // Glob patterns are always written as "languages/**" not "languages\\**".
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let mut md = config.markdown.clone();

    for schema in &config.section_schemas {
        let include = match build_globset(&schema.paths) {
            Ok(gs) => gs,
            Err(e) => { eprintln!("glint: invalid glob in section_schema paths: {}", e); continue; }
        };
        let exclude = match build_globset(&schema.paths_exclude) {
            Ok(gs) => gs,
            Err(e) => { eprintln!("glint: invalid glob in section_schema paths_exclude: {}", e); continue; }
        };
        if include.is_match(&*rel_str) && !exclude.is_match(&*rel_str) {
            apply_section_schema(&mut md, schema);
        }
    }

    md
}

fn apply_section_schema(md: &mut MarkdownConfig, schema: &SectionSchema) {
    md.enabled = true; // section schemas implicitly enable markdown checks
    md.required_h2_all.extend(schema.required_h2_all.clone());
    md.required_h2_all.dedup();
    md.required_h2.extend(schema.required_h2.clone());
    md.required_patterns.extend(schema.required_patterns.clone());
    if let Some(max) = schema.max_lines {
        md.max_lines = Some(max);
    }
}

fn build_globset(patterns: &[String]) -> anyhow::Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}
