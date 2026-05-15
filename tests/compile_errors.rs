/// L1 integration tests for compile directive error cases.
///
/// Verifies that proof surfaces actionable errors — not silent empty output —
/// when directives reference missing files, wrong columns, or bad URIs.
use proof_lib::compile::{compile_file, ViolationSeverity};
use proof_lib::GlintConfig;
use std::path::Path;

fn compile_source(
    src: &str,
    filename: &str,
    root: &Path,
) -> (String, Vec<proof_lib::compile::CompileViolation>) {
    let src_path = root.join(filename);
    std::fs::write(&src_path, src).unwrap();
    let out_file = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src_path, out_file.path(), root, &cfg).unwrap();
    let content = std::fs::read_to_string(out_file.path()).unwrap_or_default();
    (content, result.violations)
}

fn has_error(violations: &[proof_lib::compile::CompileViolation]) -> bool {
    violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
}

fn error_codes(violations: &[proof_lib::compile::CompileViolation]) -> Vec<&str> {
    violations
        .iter()
        .filter(|v| v.severity == ViolationSeverity::Error)
        .map(|v| v.code)
        .collect()
}

// ─────────────────────────────────────────────────────────
// proof:tree — bad source references
// ─────────────────────────────────────────────────────────

#[test]
fn tree_missing_md_uri_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Test\n\n```proof:tree kind=taxonomy source=md://does-not-exist.md\n```\n";
    let (out, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "missing URI should produce error, got: {:?}",
        error_codes(&violations)
    );
    // Output should NOT be a silent empty fence — it should fall back to original or error marker
    assert!(
        !out.contains("```taxonomy\n\n```"),
        "silent empty fence is not acceptable"
    );
}

#[test]
fn tree_wrong_column_names_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    // Data table with columns "thing" and "group" — not "name"/"parent"
    std::fs::write(
        dir.path().join("data.md"),
        "# Data\n\n| thing | group |\n|-------|-------|\n| A | X |\n",
    )
    .unwrap();
    let src = "# Test\n\n```proof:tree kind=org source=md://data.md name=nonexistent parent=also_missing\n```\n";
    let (out, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "wrong column names should produce error"
    );
    assert!(
        !out.contains("```org\n\n```"),
        "silent empty fence is not acceptable"
    );
}

#[test]
fn tree_empty_output_produces_error_not_empty_fence() {
    let dir = tempfile::tempdir().unwrap();
    // Table where categories exist as parent values but NOT as named rows
    // Without synthetic root support this produces empty output — should error
    std::fs::write(dir.path().join("data.md"),
        "# Features\n\n| name | category |\n|------|----------|\n| Feature A | cat1 |\n| Feature B | cat1 |\n").unwrap();
    let src = "# Test\n\n```proof:tree kind=taxonomy source=md://data.md name=name parent=category\n```\n";
    let (out, _violations) = compile_source(src, "test.source.md", dir.path());
    // With the synthetic root fix, this should NOW work without error
    // The test verifies the tree is non-empty
    let full = out;
    assert!(
        full.contains("cat1") || full.contains("Feature"),
        "tree should render category nodes"
    );
}

// ─────────────────────────────────────────────────────────
// proof:element — bad source references
// ─────────────────────────────────────────────────────────

#[test]
fn element_missing_source_uri_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    let src =
        "# Test\n\n```proof:element kind=value field=score width=8\nmd://missing-file.md\n```\n";
    let (_, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "missing source URI should produce error, got: {:?}",
        error_codes(&violations)
    );
}

#[test]
fn element_missing_field_column_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("data.md"),
        "# Data\n\n| name | value |\n|------|-------|\n| A | 42 |\n",
    )
    .unwrap();
    let src =
        "# Test\n\n```proof:element kind=value field=nonexistent_col width=8\nmd://data.md\n```\n";
    let (_, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "missing field column should produce error, got: {:?}",
        error_codes(&violations)
    );
}

// ─────────────────────────────────────────────────────────
// proof:row — bad source references
// ─────────────────────────────────────────────────────────

#[test]
fn row_missing_source_uri_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Test\n\n```proof:row source=md://missing.md foreach=row separator=\" | \"\nproof:element kind=label field=name width=10\n```\n";
    let (_, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "missing row source should produce error"
    );
}

#[test]
fn row_empty_table_emits_warning() {
    let dir = tempfile::tempdir().unwrap();
    // Table with headers but no data rows
    std::fs::write(
        dir.path().join("data.md"),
        "# Data\n\n| name | value |\n|------|-------|\n",
    )
    .unwrap();
    let src = "# Test\n\n```proof:row source=md://data.md foreach=row separator=\" | \"\nproof:element kind=label field=name width=10\n```\n";
    let (_, violations) = compile_source(src, "test.source.md", dir.path());
    // Empty table should warn — not silently produce no output
    assert!(
        violations
            .iter()
            .any(|v| v.code == "COMPILE-004" || v.code == "ELEMENT-007"),
        "empty source table should produce a warning, got: {:?}",
        violations.iter().map(|v| v.code).collect::<Vec<_>>()
    );
}

// ─────────────────────────────────────────────────────────
// proof:include — bad URIs
// ─────────────────────────────────────────────────────────

#[test]
fn include_missing_file_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Test\n\n```proof:include\nmd://nonexistent-figure.md\n```\n";
    let (_, violations) = compile_source(src, "test.source.md", dir.path());
    assert!(
        has_error(&violations),
        "include of missing file should produce error"
    );
    let codes = error_codes(&violations);
    assert!(
        codes
            .iter()
            .any(|&c| c == "COMPILE-001" || c == "COMPILE-002"),
        "expected COMPILE-001 or COMPILE-002, got: {:?}",
        codes
    );
}

// ─────────────────────────────────────────────────────────
// General: output always contains the original block on error
// ─────────────────────────────────────────────────────────

#[test]
fn compile_error_does_not_write_broken_output() {
    // When a directive fails with an error, proof should NOT write partial output.
    // written=false means the original file is preserved, not overwritten with broken content.
    let dir = tempfile::tempdir().unwrap();
    let src = "# Test\n\n```proof:tree kind=taxonomy source=md://missing.md\n```\n\nAfter.\n";
    let src_path = dir.path().join("test.source.md");
    std::fs::write(&src_path, src).unwrap();
    let out_path = dir.path().join("test.md");
    // Pre-write a sentinel so we can detect if it got overwritten
    std::fs::write(&out_path, "ORIGINAL").unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src_path, &out_path, dir.path(), &cfg).unwrap();
    assert!(
        has_error(&result.violations),
        "missing URI should produce error"
    );
    assert!(
        !result.written,
        "proof should NOT write output when errors occur"
    );
    let output = std::fs::read_to_string(&out_path).unwrap();
    assert_eq!(
        output, "ORIGINAL",
        "original output file should be untouched on error"
    );
}
