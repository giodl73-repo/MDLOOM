use crate::config::GlintConfig;
use crate::runner::{RunSummary, Runner};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn load_config_for_path(path: &Path, override_path: &Option<PathBuf>) -> Result<GlintConfig> {
    if let Some(ref cfg_path) = override_path {
        return GlintConfig::load(cfg_path)
            .with_context(|| format!("loading explicit config: {}", cfg_path.display()));
    }
    let dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };
    Ok(GlintConfig::load_or_default(&dir))
}

pub fn lint_paths(paths: &[PathBuf], config_override: &Option<PathBuf>) -> Result<RunSummary> {
    let mut aggregate = RunSummary {
        diagnostics: Vec::new(),
        files_checked: 0,
        files: Vec::new(),
    };

    for path in paths {
        let cfg = load_config_for_path(path, config_override)?;
        let dir = if path.is_dir() {
            path.clone()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };
        let runner = runner_for(&dir, cfg, config_override)?;
        let summary = runner.run_path_summary(path);
        aggregate.files_checked += summary.files_checked;
        aggregate.files.extend(summary.files);
        aggregate.diagnostics.extend(summary.diagnostics);
    }

    Ok(aggregate)
}

fn runner_for(root: &Path, config: GlintConfig, override_path: &Option<PathBuf>) -> Result<Runner> {
    if override_path.is_some() {
        Runner::new_with_config(root, config)
    } else {
        Runner::new(root, config)
    }
}
