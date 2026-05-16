use std::path::PathBuf;

use proof_lib::artifact::{select_artifacts, ArtifactDiagnostic, ArtifactRecord, ArtifactStatus};

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

    let selected = select_artifacts(
        &artifacts,
        "target eq 'html' and status eq 'written' and diagnostics_count eq 0",
    )
    .unwrap()
    .into_iter()
    .map(|artifact| artifact.output_path.display().to_string())
    .collect::<Vec<_>>();

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

    let selected = select_artifacts(
        &artifacts,
        "target eq 'markdown' and has_diagnostics eq true",
    )
    .unwrap()
    .into_iter()
    .map(|artifact| artifact.output_path.display().to_string())
    .collect::<Vec<_>>();

    assert_eq!(selected, ["docs/bad.md"]);
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
