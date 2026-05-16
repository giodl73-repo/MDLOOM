use std::path::PathBuf;

use proof_lib::artifact::{ArtifactDiagnostic, ArtifactRecord, ArtifactStatus};
use serde_json::{json, Value};
use slice_core::{FieldCatalog, ValueType};

#[test]
fn slice_selects_proof_artifact_rows_after_compile() {
    let artifacts = vec![
        artifact(
            "src/guide.source.md",
            "docs/guide.html",
            "html",
            ArtifactStatus::Written,
            0,
        ),
        artifact(
            "src/guide.source.md",
            ".proof/pebbles/guide.json",
            "pebble",
            ArtifactStatus::Cached,
            0,
        ),
        artifact(
            "src/bad.source.md",
            "docs/bad.html",
            "html",
            ArtifactStatus::Error,
            2,
        ),
    ];

    let selected = select_artifact_outputs(
        &artifacts,
        "target eq 'html' and status eq 'written' and diagnostics_count eq 0",
    );

    assert_eq!(selected, ["docs/guide.html"]);
}

#[test]
fn slice_selects_proof_artifact_rows_with_diagnostics() {
    let artifacts = vec![
        artifact(
            "src/ok.source.md",
            "docs/ok.md",
            "markdown",
            ArtifactStatus::Written,
            0,
        ),
        artifact(
            "src/bad.source.md",
            "docs/bad.md",
            "markdown",
            ArtifactStatus::Error,
            1,
        ),
    ];

    let selected = select_artifact_outputs(
        &artifacts,
        "target eq 'markdown' and has_diagnostics eq true",
    );

    assert_eq!(selected, ["docs/bad.md"]);
}

fn select_artifact_outputs(artifacts: &[ArtifactRecord], expr: &str) -> Vec<String> {
    let mut catalog = FieldCatalog::new();
    catalog
        .insert("source_path", ValueType::String)
        .insert("output_path", ValueType::String)
        .insert("target", ValueType::String)
        .insert("status", ValueType::String)
        .insert("from_cache", ValueType::Bool)
        .insert("diagnostics_count", ValueType::Number)
        .insert("has_diagnostics", ValueType::Bool);
    let selector = slice_core::compile(expr, &catalog).unwrap();

    artifacts
        .iter()
        .filter(|artifact| selector.matches(&artifact_row(artifact)))
        .map(|artifact| artifact.output_path.display().to_string())
        .collect()
}

fn artifact(
    source_path: &str,
    output_path: &str,
    target: &str,
    status: ArtifactStatus,
    diagnostic_count: usize,
) -> ArtifactRecord {
    ArtifactRecord {
        source_path: PathBuf::from(source_path),
        output_path: PathBuf::from(output_path),
        target: target.to_string(),
        status,
        directives_resolved: 0,
        from_cache: false,
        resolved_files: Vec::new(),
        diagnostics: (0..diagnostic_count)
            .map(|index| ArtifactDiagnostic {
                code: "test".to_string(),
                severity: "error".to_string(),
                line: index + 1,
                message: "test diagnostic".to_string(),
            })
            .collect(),
    }
}

fn artifact_row(artifact: &ArtifactRecord) -> Value {
    json!({
        "source_path": artifact.source_path.display().to_string(),
        "output_path": artifact.output_path.display().to_string(),
        "target": artifact.target,
        "status": artifact_status(&artifact.status),
        "from_cache": artifact.from_cache,
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
