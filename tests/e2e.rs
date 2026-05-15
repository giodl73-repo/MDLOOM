/// End-to-end pipeline tests.
///
/// Full cycle: copy fixtures → glint check --format rich → verify rich.json →
///             apply plan.json → glint check → verify zero errors → cleanup.
///
/// The plan.json is pre-authored (we know the exact fixes for the fixtures).
/// In production, Stage 2 (plan generation) is done by the fix-guide AI skill
/// reading rich.json. These tests verify Stages 1 and 3 are mechanically correct
/// and that the rich output contains what the AI would need.
use std::path::{Path, PathBuf};
use std::process::Command;

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

fn proof_bin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/proof")
}

fn e2e_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e_fixtures")
        .join(name)
}

/// Copy all e2e fixtures into a fresh temp directory. Returns the temp dir
/// (held alive — drop at end of test to clean up).
fn setup_workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for name in &["width_error.md", "col_error.md", "clean.md"] {
        let src = e2e_fixture(name);
        let dst = dir.path().join(name);
        std::fs::copy(&src, &dst)
            .unwrap_or_else(|e| panic!("copy {} → {}: {}", src.display(), dst.display(), e));
    }
    dir
}

fn run_glint_in(dir: &Path, args: &[&str]) -> std::process::Output {
    let bin = proof_bin();
    if !bin.exists() {
        panic!("debug binary not found — run `cargo build` first");
    }
    Command::new(&bin)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run proof {args:?}: {e}"))
}

// ─────────────────────────────────────────────────────────
// Stage 1: glint check --format rich
// ─────────────────────────────────────────────────────────

#[test]
fn e2e_stage1_rich_output_contains_errors_with_context() {
    if !proof_bin().exists() {
        return;
    }
    let ws = setup_workspace();
    let rich_path = ws.path().join("rich.json");

    // Run check --format rich, write to rich.json
    let out = run_glint_in(
        ws.path(),
        &[
            "check",
            "--format",
            "rich",
            "--no-fail",
            "-o",
            rich_path.to_str().unwrap(),
            ".",
        ],
    );
    assert!(
        out.status.success(),
        "check --format rich failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // rich.json must exist and be valid JSON
    let raw = std::fs::read_to_string(&rich_path).expect("rich.json not written");
    let diags: serde_json::Value = serde_json::from_str(&raw).expect("rich.json is not valid JSON");
    let arr = diags.as_array().expect("rich.json must be a JSON array");

    // Must have detected errors in the two broken files
    let codes: Vec<&str> = arr.iter().filter_map(|d| d["code"].as_str()).collect();
    assert!(
        codes.contains(&"ascii_box_width"),
        "must detect ascii_box_width in rich output"
    );
    assert!(
        codes.contains(&"ascii_box_col"),
        "must detect ascii_box_col in rich output"
    );

    // Every ascii_box_* diagnostic must have a rich context block
    for diag in arr.iter().filter(|d| {
        d["code"]
            .as_str()
            .map_or(false, |c| c.starts_with("ascii_box"))
    }) {
        let rich = &diag["rich"];
        assert!(
            !rich.is_null(),
            "ascii_box diagnostic missing rich context: {}",
            diag["message"]
        );
        assert!(
            rich["box_opens_at"].is_number(),
            "rich.box_opens_at must be a number"
        );
        assert!(
            rich["border_line"].is_string(),
            "rich.border_line must be a string"
        );
        assert!(
            rich["expected_cols"].is_array(),
            "rich.expected_cols must be an array"
        );
        assert!(
            rich["actual_cols"].is_array(),
            "rich.actual_cols must be an array"
        );
        assert!(!rich["lines"].is_null(), "rich.lines must be present");

        // The failing line must appear in the context lines
        let span_line = diag["span"]["line"].as_u64().expect("span.line");
        let line_key = span_line.to_string();
        assert!(
            rich["lines"][&line_key].is_string(),
            "context.lines must contain the failing line ({})",
            span_line
        );
    }

    // clean.md must produce zero diagnostics
    let clean_errors: Vec<_> = arr
        .iter()
        .filter(|d| d["file"].as_str().map_or(false, |f| f.contains("clean")))
        .collect();
    assert!(
        clean_errors.is_empty(),
        "clean.md should have zero diagnostics, got: {:?}",
        clean_errors
    );
}

// ─────────────────────────────────────────────────────────
// Stage 2 → 3: apply plan.json, verify zero errors
// ─────────────────────────────────────────────────────────

#[test]
fn e2e_stage3_fix_plan_resolves_all_errors() {
    if !proof_bin().exists() {
        return;
    }
    let ws = setup_workspace();

    // Verify errors exist before fixing
    let before = run_glint_in(ws.path(), &["check", "--no-fail", "."]);
    let before_stderr = String::from_utf8_lossy(&before.stderr);
    assert!(
        !before.status.success() || before_stderr.contains("error"),
        "should have errors before fixing"
    );

    // Write the plan.json — these edits exactly match the fixture errors
    // (In production, Stage 2 is the AI skill; here we hand-author the plan
    //  because we know the exact fix for each fixture)
    let width_file = ws.path().join("width_error.md");
    let col_file = ws.path().join("col_error.md");
    let plan = serde_json::json!({
        "schema_version": "1",
        "generated_by": "e2e-test",
        "source_report": "rich.json",
        "summary": {
            "total_fixes": 2,
            "high_confidence": 2,
            "medium_confidence": 0,
            "low_confidence": 0,
            "files_affected": 2
        },
        "fixes": [
            {
                "id": "fix-001",
                "file": width_file,
                "description": "Remove extra trailing + from bottom border (width 9 → 8)",
                "confidence": "high",
                "reasoning": "Top border is '+------+' (width 8). Bottom '+------++' is width 9. Extra '+' at end.",
                "diagnostic": { "code": "ascii_box_width", "line": 8, "col": 1 },
                "edit": {
                    "line": 8,
                    "old_string": "+------++",
                    "new_string": "+------+"
                }
            },
            {
                "id": "fix-002",
                "file": col_file,
                "description": "Add space to first cell — shift | from col 8 to col 9",
                "confidence": "high",
                "reasoning": "Border expects | at col 9. Row has | at col 8 (one space short in first cell). Fix: '| bad  |' → '|  bad  |', adjust second cell to keep width.",
                "diagnostic": { "code": "ascii_box_col", "line": 8, "col": 9 },
                "edit": {
                    "line": 8,
                    "old_string": "| bad  |  bad   |",
                    "new_string": "|  bad  |  bad  |"
                }
            }
        ]
    });

    let plan_path = ws.path().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap())
        .expect("write plan.json");

    // Stage 3a: dry-run — no files written
    let dry = run_glint_in(
        ws.path(),
        &[
            "fix",
            "--plan",
            plan_path.to_str().unwrap(),
            "--dry-run",
            "--no-verify",
        ],
    );
    assert!(
        dry.status.success(),
        "dry-run failed:\n{}",
        String::from_utf8_lossy(&dry.stderr)
    );

    // Verify dry-run didn't touch the files
    let width_content_before = std::fs::read_to_string(&width_file).unwrap();
    assert!(
        width_content_before.contains("+------++"),
        "dry-run must not modify files"
    );

    // Stage 3b: apply the plan
    let apply = run_glint_in(
        ws.path(),
        &["fix", "--plan", plan_path.to_str().unwrap(), "--no-verify"],
    );
    assert!(
        apply.status.success(),
        "fix failed:\n{}",
        String::from_utf8_lossy(&apply.stderr)
    );

    // Verify the edits were applied correctly
    let width_after = std::fs::read_to_string(&width_file).unwrap();
    assert!(
        width_after.contains("+------+\n"),
        "width_error.md bottom border should be fixed"
    );
    assert!(
        !width_after.contains("+------++"),
        "extra + must be gone from width_error.md"
    );

    let col_after = std::fs::read_to_string(&col_file).unwrap();
    assert!(
        col_after.contains("|  bad  |  bad  |"),
        "col_error.md inner | should be shifted to col 9"
    );
    assert!(
        !col_after.contains("| bad  |  bad   |"),
        "original misaligned row must be gone"
    );

    // Verify clean.md was not touched
    let clean_content = std::fs::read_to_string(ws.path().join("clean.md")).unwrap();
    let original_clean = std::fs::read_to_string(e2e_fixture("clean.md")).unwrap();
    assert_eq!(clean_content, original_clean, "clean.md must be unchanged");

    // Stage 4: re-run check — must be clean (exit 0, zero errors)
    let after = run_glint_in(ws.path(), &["check", "."]);
    assert!(
        after.status.success(),
        "check after fix should exit 0 (zero errors), stderr:\n{}",
        String::from_utf8_lossy(&after.stderr)
    );
}

// ─────────────────────────────────────────────────────────
// Full pipeline in one test: rich → plan → fix → verify
// ─────────────────────────────────────────────────────────

#[test]
fn e2e_full_pipeline_check_rich_fix_verify() {
    if !proof_bin().exists() {
        return;
    }
    let ws = setup_workspace();

    // Stage 1: generate rich.json
    let rich_path = ws.path().join("rich.json");
    let s1 = run_glint_in(
        ws.path(),
        &[
            "check",
            "--format",
            "rich",
            "--no-fail",
            "-o",
            rich_path.to_str().unwrap(),
            ".",
        ],
    );
    assert!(
        s1.status.success(),
        "Stage 1 failed: {}",
        String::from_utf8_lossy(&s1.stderr)
    );
    assert!(rich_path.exists(), "rich.json must be written");

    // Parse rich.json and confirm it has context blocks
    let raw = std::fs::read_to_string(&rich_path).unwrap();
    let diags: serde_json::Value = serde_json::from_str(&raw).expect("rich.json invalid JSON");
    let errors: Vec<_> = diags
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .collect();
    assert!(
        !errors.is_empty(),
        "Stage 1: must detect at least one error"
    );
    assert!(
        errors.iter().all(|d| !d["rich"].is_null()),
        "Stage 1: every error must have a rich context block"
    );

    // Stage 2 (simulated): write plan.json from what rich.json told us
    let plan = serde_json::json!({
        "schema_version": "1",
        "generated_by": "e2e-simulated-stage2",
        "source_report": rich_path,
        "summary": { "total_fixes": 2, "high_confidence": 2,
                      "medium_confidence": 0, "low_confidence": 0, "files_affected": 2 },
        "fixes": [
            {
                "id": "fix-001",
                "file": ws.path().join("width_error.md"),
                "description": "Remove extra trailing + (col 9 of bottom border)",
                "confidence": "high",
                "reasoning": "rich context shows border_line='+------+' (width 8), actual bottom '+------++' (width 9)",
                "diagnostic": { "code": "ascii_box_width", "line": 8, "col": 1 },
                "edit": { "line": 8, "old_string": "+------++", "new_string": "+------+" }
            },
            {
                "id": "fix-002",
                "file": ws.path().join("col_error.md"),
                "description": "Shift inner | from col 8 to col 9 by adding space to first cell",
                "confidence": "high",
                "reasoning": "rich context: expected_cols=[1,9,17], actual_cols=[1,8,17] — first cell needs one more char",
                "diagnostic": { "code": "ascii_box_col", "line": 8, "col": 9 },
                "edit": { "line": 8, "old_string": "| bad  |  bad   |", "new_string": "|  bad  |  bad  |" }
            }
        ]
    });
    let plan_path = ws.path().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    // Stage 3: apply plan
    let s3 = run_glint_in(
        ws.path(),
        &["fix", "--plan", plan_path.to_str().unwrap(), "--no-verify"],
    );
    assert!(
        s3.status.success(),
        "Stage 3 failed: {}",
        String::from_utf8_lossy(&s3.stderr)
    );

    // Stage 4: verify
    let s4 = run_glint_in(ws.path(), &["check", "."]);
    assert!(
        s4.status.success(),
        "Stage 4: check after fix must exit 0\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&s4.stdout),
        String::from_utf8_lossy(&s4.stderr),
    );

    // Explicitly confirm clean.md was never touched (it had no errors)
    let clean_original = std::fs::read_to_string(e2e_fixture("clean.md")).unwrap();
    let clean_after = std::fs::read_to_string(ws.path().join("clean.md")).unwrap();
    assert_eq!(
        clean_original, clean_after,
        "clean.md must be byte-identical after pipeline"
    );

    // temp dir drops here → files deleted automatically
}
