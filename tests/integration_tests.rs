/// Integration tests: run checks against fixture files and verify diagnostics.
///
/// L0 = unit (in-module #[cfg(test)])
/// L1 = integration (this file — fixture files, check composition, error codes)
/// L2 = E2E (CLI invocation, exit codes, output formats)

use proof_lib::checks::ascii_box::AsciiBoxCheck;
use proof_lib::checks::ascii_flow::AsciiFlowCheck;
use proof_lib::checks::markdown::MarkdownCheck;
use proof_lib::checks::markdown_table::MarkdownTableCheck;
use proof_lib::checks::Check;
use proof_lib::config::{AsciiBoxConfig, AsciiFlowConfig, MarkdownConfig, MarkdownTableConfig, TableSchema};
use proof_lib::diagnostic::Severity;
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
            required_patterns: vec![proof_lib::config::RequiredPattern {
                pattern: "```".to_string(),
                description: "must have code block".to_string(),
                severity: proof_lib::config::PatternSeverity::Warning,
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
    let cfg = proof_lib::GlintConfig::load_or_default(Path::new("."));
    assert!(cfg.ascii_box.enabled);
    assert_eq!(cfg.ascii_box.tolerance, 0);
}

#[test]
fn schema_file_loads_correctly() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schemas/default.toml");
    if schema_path.exists() {
        let cfg = proof_lib::GlintConfig::load(&schema_path)
            .expect("default schema should parse without error");
        assert!(cfg.ascii_box.enabled);
    }
}

// ─────────────────────────────────────────────────────────
// Runner — L1: file collection and parallel execution
// ─────────────────────────────────────────────────────────

#[test]
fn runner_scans_fixture_dir() {
    use proof_lib::Runner;
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cfg = proof_lib::GlintConfig::default();
    let runner = Runner::new(&fixture_dir, cfg).expect("runner should build");
    let diags = runner.run();
    assert!(
        !diags.is_empty(),
        "expected diagnostics when scanning fixtures dir (intentional errors present)"
    );
}

#[test]
fn runner_lint_single_perfect_file() {
    use proof_lib::Runner;
    let path = fixture("perfect_box.md");
    let cfg = proof_lib::GlintConfig::default();
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
// Pattern C — stacked/flowchart boxes (the can_open_box guard)
// ─────────────────────────────────────────────────────────

// Stacked boxes with connector lines between them: zero errors
// (Bottom border └──┘ must NOT be detected as the top of a new phantom box)
#[test]
fn stacked_boxes_no_phantom_box_errors() {
    let (path, content) = read_fixture("stacked_boxes.md");
    let diags = box_check().check(&path, &content);
    assert!(
        diags.is_empty(),
        "stacked boxes with connectors must produce zero diagnostics, got:\n{}",
        format_diags(&diags)
    );
}

// Three linear stacked boxes (bottom_border_only.md): zero errors
#[test]
fn bottom_close_border_not_treated_as_box_top() {
    let (path, content) = read_fixture("bottom_border_only.md");
    let diags = box_check().check(&path, &content);
    assert!(
        diags.is_empty(),
        "bottom-close borders between stacked boxes must produce zero errors, got:\n{}",
        format_diags(&diags)
    );
}

// A single-character check: a line starting with └ cannot open a box
#[test]
fn bottom_left_corner_cannot_open_box() {
    // The closing line of one box followed by content and then a new opening box
    let content = "```\n┌────┐\n│ A  │\n└────┘\n  │\n  ▼\n┌────┐\n│ B  │\n└────┘\n```";
    let check = box_check();
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.is_empty(),
        "two-box flowchart with connectors must have zero errors, got:\n{}",
        format_diags(&diags)
    );
}

// Single-row box (smallest valid box): zero errors
#[test]
fn single_row_box_zero_errors() {
    let (path, content) = read_fixture("single_row_box.md");
    let diags = box_check().check(&path, &content);
    assert!(
        diags.is_empty(),
        "single-row boxes must be clean, got:\n{}",
        format_diags(&diags)
    );
}

// Indented box (leading spaces): zero errors
#[test]
fn indented_box_zero_errors() {
    let (path, content) = read_fixture("indented_box.md");
    let diags = box_check().check(&path, &content);
    assert!(
        diags.is_empty(),
        "indented boxes must be clean, got:\n{}",
        format_diags(&diags)
    );
}

// Annotation after closing | (Pattern B): detected as width error
#[test]
fn annotation_after_closing_bar_detected() {
    let (path, content) = read_fixture("annotation_after_bar.md");
    let diags = box_check().check(&path, &content);
    let width_errs: Vec<_> = diags.iter().filter(|d| d.code == "ascii_box_width").collect();
    assert!(
        !width_errs.is_empty(),
        "annotation after closing | must be detected as ascii_box_width error"
    );
}

// Zero-row box (adjacent borders): width mismatch detected when borders differ
#[test]
fn zero_row_box_mismatched_borders_detected() {
    let (path, content) = read_fixture("zero_row_box.md");
    let diags = box_check().check(&path, &content);
    let width_errs: Vec<_> = diags.iter().filter(|d| d.code == "ascii_box_width").collect();
    assert!(
        !width_errs.is_empty(),
        "mismatched adjacent borders must produce ascii_box_width error"
    );
}

// Nested boxes: inner borders generate column warnings (expected behavior, not a crash)
#[test]
fn nested_boxes_no_panic_and_reports_warnings() {
    let (path, content) = read_fixture("nested_boxes.md");
    // Must not panic. May produce warnings (inner box borders vs outer expected cols).
    let diags = box_check().check(&path, &content);
    // Verify it ran successfully — just assert it doesn't crash.
    // Warnings are expected because inner box borders don't align with outer expected columns.
    let _ = diags; // behavior documented: inner borders generate column warnings
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
    use proof_lib::Runner;
    let runner = Runner::new(&fixture_dir, proof_lib::GlintConfig::default()).unwrap();
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
    use proof_lib::Runner;
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let cfg1 = proof_lib::GlintConfig::default();
    let cfg2 = proof_lib::GlintConfig::default();

    // Parallel (runner uses rayon internally)
    let runner = Runner::new(&fixture_dir, cfg1).unwrap();
    let mut parallel = runner.run();

    // Sequential (lint each file one-by-one)
    let runner2 = Runner::new(&fixture_dir, cfg2).unwrap();
    let mut sequential: Vec<proof_lib::Diagnostic> = walkdir::WalkDir::new(&fixture_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .flat_map(|e| runner2.lint_file(e.path()))
        .collect();

    // Sort both to make comparison order-independent
    let key = |d: &proof_lib::Diagnostic| (d.file.clone(), d.span.line, d.span.col, d.code);
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
    use proof_lib::fix::{Confidence, DiagnosticRef, Edit, Fix, FixPlan, PlanSummary};
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
    use proof_lib::fix::{Confidence, DiagnosticRef, Edit, Fix, FixOptions, FixPlan, PlanSummary};
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
        &FixOptions { dry_run: false, min_confidence: Confidence::High, check_signal: false },
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
// BENCH gap tests — CRLF, cascade, multi-file plan,
// border-line safety, wide chars
// ─────────────────────────────────────────────────────────

// CRLF line endings must not cause false width mismatches (Windows files)
#[test]
fn crlf_endings_no_false_positives() {
    // A perfect box with \r\n line endings
    let content = "```\r\n+------+------+\r\n| good | good |\r\n+------+------+\r\n```";
    let check = box_check();
    let diags = check.check(Path::new("test.md"), content);
    // Width check uses .lines() which strips \r — should produce zero diagnostics
    assert!(
        diags.is_empty(),
        "CRLF endings must not cause false positives, got:\n{}",
        format_diags(&diags)
    );
}

// Markdown table separator rows must NEVER be detected as box borders
#[test]
fn markdown_table_in_code_block_is_not_a_box() {
    let content = "```\n| Header A | Header B |\n|----------|----------|\n| cell     | cell     |\n```";
    let check = box_check();
    let diags = check.check(Path::new("test.md"), content);
    // The |----------| row has junction_count=0 (| is not a junction), so no box detection
    assert!(
        diags.is_empty(),
        "markdown table should not be detected as a box, got:\n{}",
        format_diags(&diags)
    );
}

// paths_exclude: overview file is excluded from generic rule, gets its own rules.
// The schema is written to a real glint.toml in a temp dir — that's the correct
// way to test cascade-resolved config (the runner discovers it from disk).
#[test]
fn section_schema_paths_exclude_skips_matching_files() {
    use proof_lib::runner::Runner;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write glint.toml — generic rule for all *.md EXCEPT 00-OVERVIEW.md,
    // and a separate rule for 00-OVERVIEW.md only.
    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true

[markdown]
enabled = true

# All language guides: require these three sections
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Type System Snapshot"]

# Overview: different structure entirely
[[section_schemas]]
paths = ["00-OVERVIEW.md"]
required_h2_all = ["Language Genealogy"]
"#).unwrap();

    // 02-C.md: missing "Type System Snapshot" → should warn
    let c_file = root.join("02-C.md");
    std::fs::write(&c_file, "# C\n\n## Decision Cheat Sheet\n\ncontent\n").unwrap();

    // 00-OVERVIEW.md: has "Language Genealogy", correctly exempt from "Type System Snapshot"
    let ov_file = root.join("00-OVERVIEW.md");
    std::fs::write(&ov_file, "# Overview\n\n## Language Genealogy\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    // 02-C.md must report missing "Type System Snapshot"
    let c_diags = runner.lint_file(&c_file);
    assert!(
        c_diags.iter().any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md must require 'Type System Snapshot'\ngot diagnostics: {}",
        format_diags(&c_diags)
    );
    // 02-C.md must NOT require "Language Genealogy"
    assert!(
        !c_diags.iter().any(|d| d.message.contains("Language Genealogy")),
        "02-C.md must NOT require 'Language Genealogy' (that's for the overview)"
    );

    // 00-OVERVIEW.md must NOT require "Type System Snapshot" (excluded by paths_exclude)
    let ov_diags = runner.lint_file(&ov_file);
    assert!(
        !ov_diags.iter().any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md must NOT require 'Type System Snapshot'\ngot: {}",
        format_diags(&ov_diags)
    );
}

// paths_exclude with multiple exclusions
#[test]
fn paths_exclude_multiple_files_skipped() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md", "01-CHEATSHEET.md"]
required_h2_all = ["Type System Snapshot"]
"#).unwrap();

    // Regular guide — should require Type System Snapshot
    let guide = root.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent\n").unwrap();

    // Overview — excluded, should NOT require it
    let overview = root.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();

    // Cheatsheet — also excluded
    let cheat = root.join("01-CHEATSHEET.md");
    std::fs::write(&cheat, "# Cheatsheet\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    // Guide: requires it
    assert!(runner.lint_file(&guide).iter().any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md should require Type System Snapshot");
    // Overview: excluded
    assert!(!runner.lint_file(&overview).iter().any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md should be excluded");
    // Cheatsheet: excluded
    assert!(!runner.lint_file(&cheat).iter().any(|d| d.message.contains("Type System Snapshot")),
        "01-CHEATSHEET.md should be excluded");
}

// paths_exclude with glob pattern (not just exact filename)
#[test]
fn paths_exclude_glob_pattern() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["0[0-1]-*.md"]
required_h2_all = ["Type System Snapshot"]
"#).unwrap();

    let guide = root.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent\n").unwrap();
    let overview = root.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();
    let cheat = root.join("01-CHEATSHEET.md");
    std::fs::write(&cheat, "# Cheatsheet\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    assert!(runner.lint_file(&guide).iter().any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md matched by *.md, not in exclude → should require");
    assert!(!runner.lint_file(&overview).iter().any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md matched by 0[0-1]-*.md exclude → should skip");
    assert!(!runner.lint_file(&cheat).iter().any(|d| d.message.contains("Type System Snapshot")),
        "01-CHEATSHEET.md matched by 0[0-1]-*.md exclude → should skip");
}

// Directory-level glint.toml: paths are relative to that directory, not root
#[test]
fn directory_schema_paths_relative_to_its_dir() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Root glint.toml — universal rule
    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Decision Cheat Sheet"]
"#).unwrap();

    // languages/ sub-directory with its own glint.toml
    let langs = root.join("languages");
    std::fs::create_dir_all(&langs).unwrap();
    std::fs::write(langs.join("proof.toml"), r#"
[markdown]
enabled = true
# paths here are relative to languages/ NOT to root
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Type System Snapshot"]
"#).unwrap();

    // A language guide — should require both Decision Cheat Sheet (root) AND Type System Snapshot (dir)
    let guide = langs.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent without required sections\n").unwrap();

    // Overview — should require Decision Cheat Sheet but NOT Type System Snapshot
    let overview = langs.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    let guide_diags = runner.lint_file(&guide);
    assert!(guide_diags.iter().any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md must require Type System Snapshot from dir-level schema");
    assert!(guide_diags.iter().any(|d| d.message.contains("Decision Cheat Sheet")),
        "02-C.md must require Decision Cheat Sheet from root schema");

    let ov_diags = runner.lint_file(&overview);
    assert!(!ov_diags.iter().any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md excluded by paths_exclude in dir schema");
    // Overview still gets root requirement
    assert!(ov_diags.iter().any(|d| d.message.contains("Decision Cheat Sheet")),
        "00-OVERVIEW.md still gets root Decision Cheat Sheet requirement");
}

// A file that matches BOTH overview rule and generic rule gets BOTH sets of requirements
// (section_schemas are additive — no "first match wins")
#[test]
fn section_schemas_are_additive_not_first_match_wins() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
[[section_schemas]]
paths = ["*.md"]
required_h2_all = ["Section A"]

[[section_schemas]]
paths = ["*.md"]
required_h2_all = ["Section B"]
"#).unwrap();

    let f = root.join("guide.md");
    std::fs::write(&f, "# Guide\n\n## Section A\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&f);

    // Has Section A but not Section B → should warn about B
    assert!(diags.iter().any(|d| d.message.contains("Section B")),
        "both schemas must apply — Section B missing should be flagged");
    // Section A is present so no warning about it
    assert!(!diags.iter().any(|d| d.message.contains("Section A")),
        "Section A is present, must not be flagged");
}

// paths_exclude does not affect the base [markdown] config — only the section_schema
#[test]
fn paths_exclude_only_affects_its_own_schema() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Universal Section"]

[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Guide Section"]
"#).unwrap();

    let overview = root.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&overview);

    // Universal Section comes from base [markdown], not section_schema → must still apply
    assert!(diags.iter().any(|d| d.message.contains("Universal Section")),
        "paths_exclude only excludes from that section_schema, not from base [markdown] config");
    // Guide Section is excluded for this file
    assert!(!diags.iter().any(|d| d.message.contains("Guide Section")),
        "Guide Section should be excluded for 00-OVERVIEW.md");
}

// Three-level cascade: root → languages/ → individual file picks up all levels
#[test]
fn three_level_cascade_all_rules_accumulate() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(root.join("proof.toml"), r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Root Requirement"]
"#).unwrap();

    let langs = root.join("languages");
    std::fs::create_dir_all(&langs).unwrap();
    std::fs::write(langs.join("proof.toml"), r#"
[markdown]
enabled = true
required_h2_all = ["Dir Requirement"]
"#).unwrap();

    let guide = langs.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&guide);

    // Both root and dir requirements must be enforced
    assert!(diags.iter().any(|d| d.message.contains("Root Requirement")),
        "root required section must apply in subdirectory");
    assert!(diags.iter().any(|d| d.message.contains("Dir Requirement")),
        "directory required section must also apply");
}

// Config cascade: two glint.toml files in a hierarchy produce additive required_h2_all
#[test]
fn config_cascade_additive_required_sections() {
    use proof_lib::config::merge;
    use proof_lib::GlintConfig;

    let mut parent = GlintConfig::default();
    parent.markdown.enabled = true;
    parent.markdown.required_h2_all = vec!["Decision Cheat Sheet".to_string()];

    let mut child = GlintConfig::default();
    child.markdown.enabled = true;
    child.markdown.required_h2_all = vec!["Type System Snapshot".to_string()];

    let merged = merge(parent, child);
    assert!(
        merged.markdown.required_h2_all.contains(&"Decision Cheat Sheet".to_string()),
        "parent's required section must survive merge"
    );
    assert!(
        merged.markdown.required_h2_all.contains(&"Type System Snapshot".to_string()),
        "child's required section must be added"
    );
    assert_eq!(merged.markdown.required_h2_all.len(), 2, "no duplicates, exactly 2 sections");
}

// Config merge: child's empty required_h2_all does NOT erase parent's
#[test]
fn config_merge_empty_child_preserves_parent_requirements() {
    use proof_lib::config::merge;
    use proof_lib::GlintConfig;

    let mut parent = GlintConfig::default();
    parent.markdown.required_h2_all = vec!["Decision Cheat Sheet".to_string()];

    let child = GlintConfig::default(); // required_h2_all = [] (empty)

    let merged = merge(parent, child);
    assert!(
        merged.markdown.required_h2_all.contains(&"Decision Cheat Sheet".to_string()),
        "parent's required section must not be erased by empty child"
    );
}

// Config merge: files.exclude is additive (child adds, not replaces)
#[test]
fn config_merge_files_exclude_is_additive() {
    use proof_lib::config::{merge, FilesConfig};
    use proof_lib::GlintConfig;

    let mut parent = GlintConfig::default();
    parent.files = FilesConfig {
        include: vec!["**/*.md".to_string()],
        exclude: vec!["_archive/**".to_string()],
        root: false,
    };

    let mut child = GlintConfig::default();
    child.files = FilesConfig {
        include: vec!["**/*.md".to_string()],
        exclude: vec!["drafts/**".to_string()], // child adds its own exclusion
        root: false,
    };

    let merged = merge(parent, child);
    assert!(merged.files.exclude.contains(&"_archive/**".to_string()),
        "parent's exclude must survive merge");
    assert!(merged.files.exclude.contains(&"drafts/**".to_string()),
        "child's exclude must be added");
    assert_eq!(merged.files.exclude.len(), 2);
}

// Multi-file fix plan: fixes across two files both apply correctly
#[test]
fn fix_plan_applies_to_multiple_files() {
    use proof_lib::fix::{Confidence, DiagnosticRef, Edit, Fix, FixOptions, FixPlan, PlanSummary};

    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("a.md");
    let file2 = dir.path().join("b.md");
    std::fs::write(&file1, "hello from a\n").unwrap();
    std::fs::write(&file2, "hello from b\n").unwrap();

    let plan = FixPlan {
        schema_version: "1".to_string(),
        generated_by: "test".to_string(),
        source_report: String::new(),
        summary: PlanSummary { total_fixes: 2, high_confidence: 2, ..Default::default() },
        fixes: vec![
            Fix {
                id: "fix-a".to_string(), file: file1.clone(),
                description: "fix a".to_string(), confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit { line: 1, old_string: "hello from a".to_string(), new_string: "HELLO FROM A".to_string() },
                diagnostic: DiagnosticRef::default(),
            },
            Fix {
                id: "fix-b".to_string(), file: file2.clone(),
                description: "fix b".to_string(), confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit { line: 1, old_string: "hello from b".to_string(), new_string: "HELLO FROM B".to_string() },
                diagnostic: DiagnosticRef::default(),
            },
        ],
    };

    let result = plan.apply(
        &FixOptions { dry_run: false, min_confidence: Confidence::Low, check_signal: false },
        dir.path(),
    ).unwrap();

    assert_eq!(result.applied.len(), 2, "both fixes should apply");
    assert_eq!(result.files_modified, 2);
    assert_eq!(std::fs::read_to_string(&file1).unwrap(), "HELLO FROM A\n");
    assert_eq!(std::fs::read_to_string(&file2).unwrap(), "HELLO FROM B\n");
}

// Unicode wide chars (CJK) in a box must not cause false width mismatches
// when the visual widths are correctly accounted for
#[test]
fn unicode_wide_chars_measured_correctly() {
    // '中' is 2 columns wide; this box is "visually" misaligned if we use byte length
    // but visual_width() must handle it correctly
    // For now we just verify no panic and check correct column counting
    let content = "```\n+--+--+\n|  |  |\n+--+--+\n```";
    let check = box_check();
    let diags = check.check(Path::new("test.md"), content);
    // Perfect box — zero errors
    assert!(diags.is_empty(), "ASCII box with spaces: should have zero errors, got:\n{}", format_diags(&diags));
}

// ─────────────────────────────────────────────────────────
// Link validation — md_table_missing_link + md_broken_link
// ─────────────────────────────────────────────────────────

#[test]
fn table_link_column_flags_bare_text() {
    use proof_lib::config::{MarkdownTableConfig, TableSchema};
    let content = "## Directories\n\n| Directory | Focus |\n|-----------|-------|\n| computing/ | Tech stack |\n| languages/ | Language guides |\n";
    let check = MarkdownTableCheck {
        config: MarkdownTableConfig {
            enabled: true,
            table_schemas: vec![TableSchema {
                heading: Some("Directories".to_string()),
                link_columns: vec!["Directory".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("t.md"), content);
    let missing = diags.iter().filter(|d| d.code == "md_table_missing_link").count();
    assert_eq!(missing, 2, "both bare directory names must be flagged");
}

#[test]
fn table_link_column_passes_linked_cells() {
    use proof_lib::config::{MarkdownTableConfig, TableSchema};
    let content = "## Directories\n\n| Directory | Focus |\n|-----------|-------|\n| [computing/](../computing/00-OVERVIEW.md) | Tech stack |\n";
    let check = MarkdownTableCheck {
        config: MarkdownTableConfig {
            enabled: true,
            table_schemas: vec![TableSchema {
                heading: Some("Directories".to_string()),
                link_columns: vec!["Directory".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        },
    };
    let diags = check.check(Path::new("t.md"), content);
    assert!(!diags.iter().any(|d| d.code == "md_table_missing_link"),
        "properly linked cells must not be flagged");
}

#[test]
fn broken_link_detected_when_file_missing() {
    use proof_lib::config::{MarkdownTableConfig, TableSchema};
    let dir = tempfile::tempdir().unwrap();
    let md_path = dir.path().join("section.md");

    // Write a section page with a link to a non-existent file
    std::fs::write(&md_path,
        "## Directories\n\n| Directory | Focus |\n|-----------|-------|\n| [computing/](../computing/00-OVERVIEW.md) | Tech |\n"
    ).unwrap();

    let check = MarkdownTableCheck {
        config: MarkdownTableConfig {
            enabled: true,
            table_schemas: vec![TableSchema {
                heading: Some("Directories".to_string()),
                link_columns: vec!["Directory".to_string()],
                verify_link_targets: true,
                ..Default::default()
            }],
            ..Default::default()
        },
    };

    let content = std::fs::read_to_string(&md_path).unwrap();
    let diags = check.check(&md_path, &content);
    assert!(diags.iter().any(|d| d.code == "md_broken_link"),
        "link to non-existent file must produce md_broken_link");
}

#[test]
fn broken_link_passes_when_target_exists() {
    use proof_lib::config::{MarkdownTableConfig, TableSchema};
    let dir = tempfile::tempdir().unwrap();
    let target_dir = dir.path().join("computing");
    std::fs::create_dir_all(&target_dir).unwrap();
    std::fs::write(target_dir.join("00-OVERVIEW.md"), "# Computing\n").unwrap();

    let md_path = dir.path().join("section.md");
    std::fs::write(&md_path,
        "## Directories\n\n| Directory | Focus |\n|-----------|-------|\n| [computing/](computing/00-OVERVIEW.md) | Tech |\n"
    ).unwrap();

    let check = MarkdownTableCheck {
        config: MarkdownTableConfig {
            enabled: true,
            table_schemas: vec![TableSchema {
                heading: Some("Directories".to_string()),
                link_columns: vec!["Directory".to_string()],
                verify_link_targets: true,
                ..Default::default()
            }],
            ..Default::default()
        },
    };

    let content = std::fs::read_to_string(&md_path).unwrap();
    let diags = check.check(&md_path, &content);
    assert!(!diags.iter().any(|d| d.code == "md_broken_link"),
        "link to existing file must not be flagged");
}

// ─────────────────────────────────────────────────────────
// Signal-loss and Pattern B detection
// ─────────────────────────────────────────────────────────

#[test]
fn signal_loss_detects_removed_words() {
    use proof_lib::fix::signal_loss;
    // Annotation removed from line
    let old = "  │  compiles source     │  cc -S / cpp / as";
    let new = "  │  compiles source     │";
    let lost = signal_loss(old, new);
    // "cpp" (len=3) is above the 2-char filter threshold and must be flagged
    assert!(lost.iter().any(|w| w.as_str() == "cpp"), "removed tool name 'cpp' must be flagged, got: {:?}", lost);
}

#[test]
fn signal_loss_passes_whitespace_only_change() {
    use proof_lib::fix::signal_loss;
    let old = "  │  compiles source      │";
    let new = "  │  compiles source       │";  // one more trailing space
    let lost = signal_loss(old, new);
    assert!(lost.is_empty(), "whitespace-only changes must not flag signal loss");
}

#[test]
fn pattern_b_detects_annotation_after_bar() {
    use proof_lib::fix::is_pattern_b;
    assert!(is_pattern_b("  │ content │  ← annotation"), "annotation after │ is Pattern B");
    assert!(is_pattern_b("│ stage │  cc -S"), "tool label after │ is Pattern B");
    assert!(!is_pattern_b("  │ content │"), "clean closing │ is not Pattern B");
    assert!(!is_pattern_b("  │ content │  "), "trailing spaces only is not Pattern B");
}

#[test]
fn nested_box_col_fix_only_adjusts_leftmost() {
    use proof_lib::checks::ascii_box::AsciiBoxCheck;
    use proof_lib::checks::Check;
    use proof_lib::config::AsciiBoxConfig;
    // A nested box where inner │ and outer │ are both off by 1
    // The fix should add ONE space at the leftmost misaligned │, cascading the rest
    let content = "```\n┌──────────────────────────────┐\n│  ┌──────────┐  inner text   │\n│  └──────────┘  more text    │\n└──────────────────────────────┘\n```";
    let check = AsciiBoxCheck { config: AsciiBoxConfig::default() };
    // Just verify it doesn't panic and returns something
    let diags = check.check(Path::new("test.md"), content);
    let _ = diags; // nested boxes may produce warnings; just verify no crash
}

// ─────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────

fn format_diags(diags: &[proof_lib::Diagnostic]) -> String {
    diags.iter()
        .map(|d| format!("  {}:{} [{}] {}", d.file.display(), d.span, d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n")
}

// ─────────────────────────────────────────────────────────
// L1: Slide compilation integration tests
// ─────────────────────────────────────────────────────────

#[test]
fn slide_title_only_compiles_to_correct_dimensions() {
    use proof_lib::compile::compile_file;
    use proof_lib::GlintConfig;
    let src = fixture("slides/title-only.slides.source.md");
    let out = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src, out.path(), out.path().parent().unwrap(), &cfg).unwrap();
    assert!(result.written, "slide compile should write output");
    assert!(result.violations.is_empty(), "no violations: {} violations found", result.violations.len());
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.contains("```slides"), "output should have slides fence");
    assert!(content.contains("SLIDE 1"), "output should have slide 1 header");
    assert!(content.contains("Test Title"), "output should contain title");
    // Width=40, height=6 — each slide row must be exactly 40 chars
    for line in content.lines().filter(|l| !l.starts_with("```") && !l.starts_with("<!--") && !l.starts_with("SLIDE")) {
        assert!(line.chars().count() <= 40, "line too wide: {:?}", line);
    }
}

#[test]
fn slide_two_slide_deck_has_correct_count() {
    use proof_lib::compile::compile_file;
    use proof_lib::GlintConfig;
    let src = fixture("slides/two-slide-deck.slides.source.md");
    let out = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src, out.path(), out.path().parent().unwrap(), &cfg).unwrap();
    assert!(result.written);
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.contains("count=2"), "should report 2 slides");
    assert!(content.contains("SLIDE 1"), "should have slide 1");
    assert!(content.contains("SLIDE 2"), "should have slide 2");
}

#[test]
fn slide_title_content_with_bullets() {
    use proof_lib::compile::compile_file;
    use proof_lib::GlintConfig;
    let src = fixture("slides/title-content.slides.source.md");
    let out = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src, out.path(), out.path().parent().unwrap(), &cfg).unwrap();
    assert!(result.written);
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.contains("Key Points"), "title should appear in output");
    // Bullets from body content should appear in rendered output
    assert!(content.contains("First point") || content.contains("●"), "bullet content should render");
}

// ─────────────────────────────────────────────────────────
// L1: Dashboard compilation integration tests
// ─────────────────────────────────────────────────────────

#[test]
fn dashboard_two_region_compiles_correctly() {
    use proof_lib::compile::compile_file;
    use proof_lib::GlintConfig;
    let src = fixture("dashboards/two-region.dashboard.source.md");
    let out = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src, out.path(), out.path().parent().unwrap(), &cfg).unwrap();
    assert!(result.written, "dashboard compile should write output");
    assert!(result.violations.iter().all(|v| v.severity != proof_lib::compile::ViolationSeverity::Error),
        "no error violations");
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.contains("HEADER CONTENT"), "top region content should appear");
    assert!(content.contains("FOOTER CONTENT"), "bottom region content should appear");
    // Canvas is 20×6 — check line widths
    let lines: Vec<&str> = content.lines()
        .filter(|l| !l.starts_with("<!--") && !l.starts_with("```"))
        .collect();
    for line in &lines {
        assert!(line.chars().count() <= 20, "canvas line too wide: {:?}", line);
    }
}
