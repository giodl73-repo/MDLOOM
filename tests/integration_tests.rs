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
// Helper
// ─────────────────────────────────────────────────────────

fn format_diags(diags: &[glint_lib::Diagnostic]) -> String {
    diags.iter()
        .map(|d| format!("  {}:{} [{}] {}", d.file.display(), d.span, d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n")
}
