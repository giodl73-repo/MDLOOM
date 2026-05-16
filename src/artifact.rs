use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub schema_version: String,
    pub generated_by: String,
    pub config_root: PathBuf,
    pub generated_at_ms: u64,
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub target: String,
    pub status: ArtifactStatus,
    pub directives_resolved: usize,
    pub from_cache: bool,
    #[serde(default)]
    pub resolved_files: Vec<PathBuf>,
    pub diagnostics: Vec<ArtifactDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Written,
    Cached,
    UpToDate,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDiagnostic {
    pub code: String,
    pub severity: String,
    pub line: usize,
    pub message: String,
}

pub fn manifest_path(root: &Path) -> PathBuf {
    root.join(".proof").join("artifacts.json")
}

pub fn write_manifest(root: &Path, artifacts: Vec<ArtifactRecord>) -> Result<PathBuf> {
    let manifest = ArtifactManifest {
        schema_version: "1".to_string(),
        generated_by: "proof compile".to_string(),
        config_root: root.to_path_buf(),
        generated_at_ms: now_ms(),
        artifacts,
    };
    let path = manifest_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
