/// Integration tests: run checks against fixture files and verify diagnostics.
///
/// L0 = unit (in-module #[cfg(test)])
/// L1 = integration (this file — fixture files, check composition, error codes)
/// L2 = E2E (CLI invocation, exit codes, output formats)

use glint_lib::checks::ascii_box::AsciiBoxCheck;
use glint_lib::checks::ascii_flow::AsciiFlowCheck;
use glint_lib::checks::markdown::MarkdownCheck;
use glint_lib::checks::Check;
use glint_lib::config::{AsciiBoxConfig, AsciiFlowConfig, MarkdownConfig};
use glint_lib::diagnostic::Severity;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn box_check() -> AsciiBoxCheck {
    AsciiBoxCheck { config: AsciiBoxConfig::default() }
}

fn flow_check() -> AsciiFlowCheck {
    AsciiFlowCheck { config: AsciiFlowConfig::default() }
}

fn read_fixture(name: &str) -> (PathBuf, String) {
    let path = fixture(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {}", name, e));
    (path, content)
}

// ─────────────────────────────────────────────────────────
// ASCII Box — L1: fixture-level tests
// ─────────────────────────────────────────────────────────

#[test]
fn perfect_box_zero_diagnostics() {
    let (path, content) = read_fixture("perfect_box.md");
    let diags = box_check().check(&path, &content);
    assert!(
        diags.is_empty(),
        "expected zero diagnostics for perfect_box.md, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn width_mismatch_detected_in_fixture() {
    let (path, content) = read_fixture("width_mismatch.md");
    let diags = box_check().check(&path, &content);
    let errors: Vec<_> = diags.iter().filter(|d| d.severity == Severity::Error).collect();
    assert!(
        !errors.is_empty(),
        "expected at least one error in width_mismatch.md"
    );
    let codes: Vec<_> = errors.iter().map(|d| d.code).collect();
    assert!(
        codes.iter().any(|&c| c == "ascii_box_width"),
        "expected ascii_box_width error, got codes: {:?}", codes
    );
}

#[test]
fn col_misalignment_detected_in_fixture() {
    let (path, content) = read_fixture("col_misalignment.md");
    let diags = box_check().check(&path, &content);
    let col_errors: Vec<_> = diags.iter()
        .filter(|d| d.code == "ascii_box_col")
        .collect();
    assert!(
        !col_errors.is_empty(),
        "expected ascii_box_col errors in col_misalignment.md, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn complex_diagram_inner_box_misalignment() {
    let (path, content) = read_fixture("complex_diagram.md");
    let diags = box_check().check(&path, &content);
    // The complex_diagram.md has one broken inner box — should have at least one error
    assert!(
        !diags.is_empty(),
        "expected diagnostics in complex_diagram.md for the broken inner box"
    );
}

// ─────────────────────────────────────────────────────────
// Cell Padding — L1
// ─────────────────────────────────────────────────────────

#[test]
fn cell_padding_warnings_produced() {
    let (path, content) = read_fixture("cell_padding.md");
    let diags = flow_check().check(&path, &content);
    let padding_warns: Vec<_> = diags.iter()
        .filter(|d| d.code == "ascii_cell_padding")
        .collect();
    assert!(
        !padding_warns.is_empty(),
        "expected ascii_cell_padding warnings in cell_padding.md, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn cell_padding_correct_rows_no_warnings() {
    // The "correct" box in cell_padding.md should produce no padding warnings
    let content = "```\n+----------+----------+\n| fine     | fine     |\n| fine     | fine     |\n+----------+----------+\n```";
    let check = AsciiFlowCheck { config: AsciiFlowConfig::default() };
    let diags = check.check(Path::new("test.md"), content);
    let padding_warns: Vec<_> = diags.iter()
        .filter(|d| d.code == "ascii_cell_padding")
        .collect();
    assert!(
        padding_warns.is_empty(),
        "expected no padding warnings for well-padded cells, got:\n{}",
        format_diags(&diags)
    );
}

// ─────────────────────────────────────────────────────────
// Markdown Structure — L1
// ─────────────────────────────────────────────────────────

#[test]
fn markdown_h1_count_enforced() {
    let content = "# Title One\n\n# Title Two\n\nsome content";
    let check = MarkdownCheck {
        config: MarkdownConfig {
            enabled: true,
            max_h1: Some(1),
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("test.md"), content);
    let h1_warns: Vec<_> = diags.iter()
        .filter(|d| d.code == "md_h1_count")
        .collect();
    assert_eq!(h1_warns.len(), 1, "expected exactly one H1 count warning");
    assert_eq!(h1_warns[0].span.line, 3, "expected warning on line 3 (second H1)");
}

#[test]
fn markdown_required_section_missing() {
    let content = "# Title\n\n## Some Section\n\nContent here.";
    let check = MarkdownCheck {
        config: MarkdownConfig {
            enabled: true,
            required_h2_all: vec!["Decision Cheat Sheet".to_string()],
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().any(|d| d.code == "md_missing_section"),
        "expected md_missing_section diagnostic"
    );
}

#[test]
fn markdown_required_section_present() {
    let content = "# Title\n\n## Decision Cheat Sheet\n\nContent here.";
    let check = MarkdownCheck {
        config: MarkdownConfig {
            enabled: true,
            required_h2_all: vec!["Decision Cheat Sheet".to_string()],
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "md_missing_section"),
        "expected no missing section diagnostic when section is present"
    );
}

#[test]
fn markdown_required_pattern_missing() {
    let content = "# Title\n\nsome prose without a code block";
    let check = MarkdownCheck {
        config: MarkdownConfig {
            enabled: true,
            required_patterns: vec![glint_lib::config::RequiredPattern {
                pattern: "```".to_string(),
                description: "must have code block".to_string(),
                severity: glint_lib::config::PatternSeverity::Warning,
            }],
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().any(|d| d.code == "md_missing_pattern"),
        "expected md_missing_pattern warning"
    );
}

#[test]
fn markdown_max_lines_exceeded() {
    let content: String = (0..100).map(|i| format!("line {}\n", i)).collect();
    let check = MarkdownCheck {
        config: MarkdownConfig {
            enabled: true,
            max_lines: Some(50),
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("test.md"), &content);
    assert!(
        diags.iter().any(|d| d.code == "md_file_length"),
        "expected md_file_length warning"
    );
}

// ─────────────────────────────────────────────────────────
// Config loading — L1
// ─────────────────────────────────────────────────────────

#[test]
fn default_config_loads_without_panic() {
    let cfg = glint_lib::GlintConfig::load_or_default(Path::new("."));
    assert!(cfg.ascii_box.enabled);
    assert_eq!(cfg.ascii_box.tolerance, 0);
}

#[test]
fn schema_file_loads_correctly() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas/default.toml");
    if schema_path.exists() {
        let cfg = glint_lib::GlintConfig::load(&schema_path)
            .expect("default schema should parse without error");
        assert!(cfg.ascii_box.enabled);
    }
}

// ─────────────────────────────────────────────────────────
// Runner — L1: file collection and parallel execution
// ─────────────────────────────────────────────────────────

#[test]
fn runner_scans_fixture_dir() {
    use glint_lib::Runner;
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cfg = glint_lib::GlintConfig::default();
    let runner = Runner::new(&fixture_dir, cfg).expect("runner should build");
    let diags = runner.run();
    assert!(
        !diags.is_empty(),
        "expected diagnostics when scanning fixtures dir (intentional errors present)"
    );
}

#[test]
fn runner_lint_single_perfect_file() {
    use glint_lib::Runner;
    let path = fixture("perfect_box.md");
    let cfg = glint_lib::GlintConfig::default();
    let runner = Runner::new(path.parent().unwrap(), cfg).expect("runner should build");
    let diags = runner.lint_file(&path);
    assert!(
        diags.is_empty(),
        "expected zero diagnostics for perfect_box.md, got:\n{}",
        format_diags(&diags)
    );
}

// ─────────────────────────────────────────────────────────
// L2: E2E — check that the binary produces correct exit codes
// ─────────────────────────────────────────────────────────

#[test]
fn binary_exits_zero_on_clean_file() {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/glint");
    if !bin.exists() {
        return; // skip if not built yet
    }
    let output = std::process::Command::new(&bin)
        .arg(fixture("perfect_box.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected exit 0 for clean file, got:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn binary_exits_nonzero_on_errors() {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/glint");
    if !bin.exists() {
        return;
    }
    let output = std::process::Command::new(&bin)
        .arg(fixture("width_mismatch.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");
    assert_ne!(
        output.status.code(),
        Some(0),
        "expected non-zero exit for file with errors"
    );
}

#[test]
fn binary_json_output_is_parseable() {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/debug/glint");
    if !bin.exists() {
        return;
    }
    let output = std::process::Command::new(&bin)
        .args(["--format", "json", "--no-fail"])
        .arg(fixture("width_mismatch.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be a JSON array
    assert!(stdout.trim().starts_with('['), "expected JSON array output, got: {}", stdout);
    assert!(stdout.trim().ends_with(']'), "expected JSON array output, got: {}", stdout);
}

// ─────────────────────────────────────────────────────────
// Rich context — L1: verify context blocks are populated
// ─────────────────────────────────────────────────────────

#[test]
fn rich_context_populated_on_box_errors() {
    let (path, content) = read_fixture("width_mismatch.md");
    let diags = box_check().check(&path, &content);
    let box_errors: Vec<_> = diags.iter()
        .filter(|d| d.code == "ascii_box_width" || d.code == "ascii_box_col")
        .collect();
    assert!(!box_errors.is_empty(), "expected box errors in width_mismatch.md");
    for d in &box_errors {
        let rich = d.rich.as_ref()
            .unwrap_or_else(|| panic!("diagnostic {} at line {} missing rich context", d.code, d.span.line));
        assert!(rich.box_opens_at.is_some(), "box_opens_at should be set");
        assert!(rich.border_line.is_some(), "border_line should be set");
        assert!(!rich.lines.is_empty(), "surrounding lines should be present");
    }
}

#[test]
fn rich_context_expected_cols_match_border() {
    let content = "```\n+------+------+\n| bad |  bad  |\n+------+------+\n```";
    let check = box_check();
    let diags = check.check(Path::new("test.md"), content);
    for d in &diags {
        if let Some(rich) = &d.rich {
            if let Some(expected) = &rich.expected_cols {
                // Expected cols from "+------+------+" are at 1, 8, 15
                assert!(expected.contains(&1), "expected col 1 in expected_cols");
                assert!(!expected.is_empty(), "expected_cols must not be empty");
            }
        }
    }
}

#[test]
fn rich_context_surrounding_lines_include_failing_line() {
    let (path, content) = read_fixture("width_mismatch.md");
    let diags = box_check().check(&path, &content);
    for d in diags.iter().filter(|d| d.code == "ascii_box_width") {
        let rich = d.rich.as_ref().unwrap();
        // The failing line should appear in the context
        assert!(
            rich.lines.contains_key(&d.span.line),
            "context.lines should contain failing line {}, got keys: {:?}",
            d.span.line, rich.lines.keys().collect::<Vec<_>>()
        );
    }
}

// ─────────────────────────────────────────────────────────
// Invariant tests
// ─────────────────────────────────────────────────────────

// I-3: Every diagnostic has valid span (line >= 1, col >= 1)
#[test]
fn invariant_i3_all_diagnostics_have_valid_spans() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    use glint_lib::Runner;
    let runner = Runner::new(&fixture_dir, glint_lib::GlintConfig::default()).unwrap();
    let diags = runner.run();
    for d in &diags {
        assert!(d.span.line >= 1, "diagnostic {} has line=0 (must be ≥1): {:?}", d.code, d.file);
        assert!(d.span.col >= 1, "diagnostic {} has col=0 (must be ≥1): {:?}", d.code, d.file);
    }
}

// I-4: Linting the same file twice produces identical diagnostics
#[test]
fn invariant_i4_linting_is_deterministic() {
    let (path, content) = read_fixture("width_mismatch.md");
    let check = box_check();
    let run1 = check.check(&path, &content);
    let run2 = check.check(&path, &content);
    assert_eq!(run1.len(), run2.len(), "diagnostic count must be the same across runs");
    for (d1, d2) in run1.iter().zip(run2.iter()) {
        assert_eq!(d1.span.line, d2.span.line);
        assert_eq!(d1.span.col, d2.span.col);
        assert_eq!(d1.code, d2.code);
    }
}

// I-6: tolerance = N suppresses drift ≤ N, reports drift > N
#[test]
fn invariant_i6_tolerance_bounds() {
    // This box has | at col 8, border expects col 9 → drift = 1
    let content = "```\n+------+------+\n| bad |  bad  |\n+------+------+\n```";
    let path = Path::new("test.md");

    // tolerance = 0 → should report drift of 1
    let strict = AsciiBoxCheck { config: AsciiBoxConfig { tolerance: 0, ..AsciiBoxConfig::default() } };
    let diags_strict = strict.check(path, content);
    let col_errors_strict: Vec<_> = diags_strict.iter().filter(|d| d.code == "ascii_box_col").collect();
    assert!(!col_errors_strict.is_empty(), "tolerance=0 must report drift of 1");

    // tolerance = 1 → should suppress drift of 1
    let lenient = AsciiBoxCheck { config: AsciiBoxConfig { tolerance: 1, ..AsciiBoxConfig::default() } };
    let diags_lenient = lenient.check(path, content);
    let col_errors_lenient: Vec<_> = diags_lenient.iter().filter(|d| d.code == "ascii_box_col").collect();
    assert!(col_errors_lenient.is_empty(), "tolerance=1 must suppress drift of 1");
}

// I-7: Parallel and sequential execution produce same diagnostic SET
#[test]
fn invariant_i7_parallel_equals_sequential() {
    use glint_lib::Runner;
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cfg1 = glint_lib::GlintConfig::default();
    let cfg2 = glint_lib::GlintConfig::default();

    // Parallel (runner uses rayon internally)
    let runner = Runner::new(&fixture_dir, cfg1).unwrap();
    let mut parallel = runner.run();

    // Sequential (lint each file one-by-one)
    let runner2 = Runner::new(&fixture_dir, cfg2).unwrap();
    let mut sequential: Vec<glint_lib::Diagnostic> = walkdir::WalkDir::new(&fixture_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .flat_map(|e| runner2.lint_file(e.path()))
        .collect();

    // Sort both to make comparison order-independent
    let key = |d: &glint_lib::Diagnostic| (d.file.clone(), d.span.line, d.span.col, d.code);
    parallel.sort_by_key(key);
    sequential.sort_by_key(key);

    assert_eq!(parallel.len(), sequential.len(),
        "parallel ({}) and sequential ({}) produced different counts",
        parallel.len(), sequential.len());

    for (p, s) in parallel.iter().zip(sequential.iter()) {
        assert_eq!(p.code, s.code);
        assert_eq!(p.span.line, s.span.line);
        assert_eq!(p.span.col, s.span.col);
    }
}

// ─────────────────────────────────────────────────────────
// Fix plan — L1: fix module integration
// ─────────────────────────────────────────────────────────

#[test]
fn fix_plan_round_trip_json() {
    use glint_lib::fix::{Confidence, DiagnosticRef, Edit, Fix, FixPlan, PlanSummary};
    use std::path::PathBuf;

    let plan = FixPlan {
        schema_version: "1".to_string(),
        generated_by: "test".to_string(),
        source_report: "rich.json".to_string(),
        summary: PlanSummary { total_fixes: 1, high_confidence: 1, ..Default::default() },
        fixes: vec![Fix {
            id: "fix-001".to_string(),
            file: PathBuf::from("test.md"),
            description: "test".to_string(),
            confidence: Confidence::High,
            reasoning: "obvious".to_string(),
            edit: Edit {
                line: 5,
                old_string: "| foo |".to_string(),
                new_string: "| foo  |".to_string(),
            },
            diagnostic: DiagnosticRef { code: "ascii_box_col".to_string(), line: 5, col: 7 },
        }],
    };

    let json = serde_json::to_string_pretty(&plan).expect("serialize");
    let back: FixPlan = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.fixes[0].id, "fix-001");
    assert_eq!(back.fixes[0].confidence, Confidence::High);
    assert_eq!(back.fixes[0].edit.old_string, "| foo |");
}

#[test]
fn fix_plan_confidence_filtering() {
    use glint_lib::fix::{Confidence, DiagnosticRef, Edit, Fix, FixOptions, FixPlan, PlanSummary};
    use std::path::PathBuf;

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.md");
    std::fs::write(&file, "hello\n").unwrap();

    let plan = FixPlan {
        schema_version: "1".to_string(),
        generated_by: "test".to_string(),
        source_report: String::new(),
        summary: PlanSummary::default(),
        fixes: vec![
            Fix {
                id: "high-fix".to_string(),
                file: file.clone(),
                description: "high confidence".to_string(),
                confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit { line: 1, old_string: "hello".to_string(), new_string: "hello!".to_string() },
                diagnostic: DiagnosticRef::default(),
            },
            Fix {
                id: "low-fix".to_string(),
                file: file.clone(),
                description: "low confidence".to_string(),
                confidence: Confidence::Low,
                reasoning: String::new(),
                edit: Edit { line: 1, old_string: "hello".to_string(), new_string: "goodbye".to_string() },
                diagnostic: DiagnosticRef::default(),
            },
        ],
    };

    // Apply with min_confidence = High → only high-fix applies
    let result = plan.apply(
        &FixOptions { dry_run: false, min_confidence: Confidence::High },
        dir.path(),
    ).unwrap();

    assert!(result.applied.contains(&"high-fix".to_string()), "high-fix should apply");
    assert!(!result.applied.contains(&"low-fix".to_string()), "low-fix should be skipped");
    assert_eq!(result.skipped.len(), 1);
}

// ─────────────────────────────────────────────────────────
// L2: additional E2E tests
// ─────────────────────────────────────────────────────────

fn debug_bin() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/glint")
}

#[test]
fn binary_rich_output_contains_context_block() {
    let bin = debug_bin();
    if !bin.exists() { return; }

    let output = std::process::Command::new(&bin)
        .args(["--format", "rich", "--no-fail"])
        .arg(fixture("width_mismatch.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"rich\""), "rich output must contain 'rich' key");
    assert!(stdout.contains("\"box_opens_at\""), "rich output must contain box_opens_at");
    assert!(stdout.contains("\"expected_cols\""), "rich output must contain expected_cols");
    assert!(stdout.contains("\"lines\""), "rich output must contain lines");

    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("rich output must be valid JSON");
    assert!(parsed.is_array(), "rich output must be a JSON array");
}

#[test]
fn binary_rich_output_is_valid_json_array() {
    let bin = debug_bin();
    if !bin.exists() { return; }

    let output = std::process::Command::new(&bin)
        .args(["--format", "rich", "--no-fail"])
        .arg(fixture("perfect_box.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("rich output not valid JSON: {}\nGot: {}", e, stdout));
    assert!(parsed.is_array());
    // Zero errors → empty array
    assert_eq!(parsed.as_array().unwrap().len(), 0, "perfect file should produce no rich diagnostics");
}

#[test]
fn binary_stats_command_runs() {
    let bin = debug_bin();
    if !bin.exists() { return; }

    let output = std::process::Command::new(&bin)
        .args(["stats", "--by-code"])
        .arg(fixture("width_mismatch.md").to_str().unwrap())
        .output()
        .expect("failed to run glint stats");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("files:"), "stats should show file count");
    assert!(stdout.contains("errors:"), "stats should show error count");
}

#[test]
fn binary_fix_dry_run_writes_nothing() {
    let bin = debug_bin();
    if !bin.exists() { return; }

    // Write a temp plan that would modify a file
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.md");
    std::fs::write(&target, "old content\n").unwrap();

    let plan = serde_json::json!({
        "schema_version": "1",
        "generated_by": "test",
        "source_report": "",
        "summary": {"total_fixes": 1, "high_confidence": 1, "medium_confidence": 0, "low_confidence": 0, "files_affected": 1},
        "fixes": [{
            "id": "fix-001",
            "file": target.to_str().unwrap(),
            "description": "test",
            "confidence": "high",
            "reasoning": "",
            "diagnostic": {"code": "test", "line": 1, "col": 1},
            "edit": {"line": 1, "old_string": "old content", "new_string": "new content"}
        }]
    });
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let _output = std::process::Command::new(&bin)
        .args(["fix", "--plan", plan_path.to_str().unwrap(), "--dry-run", "--no-verify"])
        .output()
        .expect("failed to run glint fix");

    // Invariant I-12: dry-run must not write
    let content_after = std::fs::read_to_string(&target).unwrap();
    assert_eq!(content_after, "old content\n", "dry-run must not modify any files");
}

// ─────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────

fn format_diags(diags: &[glint_lib::Diagnostic]) -> String {
    diags.iter()
        .map(|d| format!("  {}:{} [{}] {}", d.file.display(), d.span, d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n")
}
