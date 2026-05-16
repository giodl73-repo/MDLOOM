use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

pub fn select_artifacts<'a>(
    artifacts: &'a [ArtifactRecord],
    expr: &str,
) -> std::result::Result<Vec<&'a ArtifactRecord>, slice_core::SliceError> {
    let selector = slice_core::compile(expr, &artifact_catalog())?;
    Ok(artifacts
        .iter()
        .filter(|artifact| selector.matches(&artifact_row(artifact)))
        .collect())
}

fn artifact_catalog() -> slice_core::FieldCatalog {
    let mut catalog = slice_core::FieldCatalog::new();
    catalog
        .insert("source_path", slice_core::ValueType::String)
        .insert("output_path", slice_core::ValueType::String)
        .insert("target", slice_core::ValueType::String)
        .insert("status", slice_core::ValueType::String)
        .insert("directives_resolved", slice_core::ValueType::Number)
        .insert("from_cache", slice_core::ValueType::Bool)
        .insert("resolved_files_count", slice_core::ValueType::Number)
        .insert("diagnostics_count", slice_core::ValueType::Number)
        .insert("has_diagnostics", slice_core::ValueType::Bool);
    catalog
}

fn artifact_row(artifact: &ArtifactRecord) -> Value {
    json!({
        "source_path": artifact.source_path.display().to_string(),
        "output_path": artifact.output_path.display().to_string(),
        "target": artifact.target,
        "status": artifact_status(&artifact.status),
        "directives_resolved": artifact.directives_resolved,
        "from_cache": artifact.from_cache,
        "resolved_files_count": artifact.resolved_files.len(),
        "diagnostics_count": artifact.diagnostics.len(),
        "has_diagnostics": !artifact.diagnostics.is_empty(),
    })
}

fn artifact_status(status: &ArtifactStatus) -> &'static str {
    match status {
        ArtifactStatus::Written => "written",
        ArtifactStatus::Cached => "cached",
        ArtifactStatus::UpToDate => "up_to_date",
        ArtifactStatus::Error => "error",
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
