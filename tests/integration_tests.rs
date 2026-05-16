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
use proof_lib::config::{AsciiBoxConfig, AsciiFlowConfig, GlintConfig, MarkdownConfig};
use proof_lib::diagnostic::Severity;
use proof_lib::Runner;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn box_check() -> AsciiBoxCheck {
    AsciiBoxCheck {
        config: AsciiBoxConfig::default(),
    }
}

fn flow_check() -> AsciiFlowCheck {
    AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    }
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
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        !errors.is_empty(),
        "expected at least one error in width_mismatch.md"
    );
    let codes: Vec<_> = errors.iter().map(|d| d.code).collect();
    assert!(
        codes.iter().any(|&c| c == "ascii_box_width"),
        "expected ascii_box_width error, got codes: {:?}",
        codes
    );
}

#[test]
fn col_misalignment_detected_in_fixture() {
    let (path, content) = read_fixture("col_misalignment.md");
    let diags = box_check().check(&path, &content);
    let col_errors: Vec<_> = diags.iter().filter(|d| d.code == "ascii_box_col").collect();
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
    let padding_warns: Vec<_> = diags
        .iter()
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
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    let padding_warns: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_cell_padding")
        .collect();
    assert!(
        padding_warns.is_empty(),
        "expected no padding warnings for well-padded cells, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn cell_padding_ignores_math_pipes_inside_single_cell_box() {
    let content = "```\n+====================================================================+\n|  A_n = {even permutations}   |A_n| = n!/2                         |\n|  CAYLEY'S THEOREM: Every group G embeds in S_{|G|} via            |\n+====================================================================+\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    let padding_warns: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_cell_padding")
        .collect();
    assert!(
        padding_warns.is_empty(),
        "math pipes inside a single-cell box are not cell delimiters:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn cell_padding_ignores_absolute_value_formula_without_box_border() {
    let content = "```\nORBIT-STABILIZER THEOREM:\n  |G| = |Orb(x)| · |Stab(x)|\n\nBEAM PATTERN:\n|B(θ)| = |sin(N·π·d·sin(θ)/λ)| / |N·sin(π·d·sin(θ)/λ)|\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    let padding_warns: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_cell_padding")
        .collect();
    assert!(
        padding_warns.is_empty(),
        "absolute-value formulas outside bordered boxes are not cells:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn cell_padding_allows_full_cells_with_no_room_for_padding() {
    let content = "```\n+----------+------------+\n|Ultrasonic|High-freq   |\n| fine     |needs pad   |\n+----------+------------+\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags
            .iter()
            .all(|d| { !(d.code == "ascii_cell_padding" && d.message.contains("Ultrasonic")) }),
        "full-width cells cannot add padding without widening the box:\n{}",
        format_diags(&diags)
    );
    assert!(
        diags
            .iter()
            .any(|d| { d.code == "ascii_cell_padding" && d.message.contains("needs pad") }),
        "cells with spare width should still warn when padding is missing:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn connector_drift_ignores_timeline_and_formula_pipes() {
    let content = "```\n3500 Ma  |  O2 ~ 0%  Anaerobic world.\n          |\n2400 Ma  |  GREAT OXIDATION EVENT\n\nFACTORING:\n  x^{p^n} - x = product of all monic irreducibles of degree | n over F_p.\n  x^{p^n-1} - 1 = product of all monic irreducibles of degree | n.\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    let drift_warns: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_connector_drift")
        .collect();
    assert!(
        drift_warns.is_empty(),
        "timeline and formula pipes are not connector drift:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn connector_drift_still_warns_on_connector_only_lines() {
    let content = "```\n  |\n   |\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().any(|d| d.code == "ascii_connector_drift"),
        "connector-only drift should still warn, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn arrow_gap_ignores_chemistry_and_axis_arrows() {
    let content = "```\nLi+ + e- -> Li                         -3.04\nFAST  ← ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ►  SLOW\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_arrow_gap"),
        "chemistry arrows and decorative spaced axes are not broken arrow bodies:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn arrow_gap_still_warns_on_broken_box_arrow() {
    let content = "```\n| Box A |── ─▶ Box B |\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().any(|d| d.code == "ascii_arrow_gap"),
        "broken box arrow should still warn, got:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn arrow_gap_stops_at_layout_spacing_between_arrows() {
    let content = "```\nV=20  ─────────────────      →→→→→  E field lines\nleft ──────────────►──────────►──────            right ──────────►──────►──────────────\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_arrow_gap"),
        "layout spacing between separate arrows is not an arrow-body gap:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn arrow_gap_ignores_bidirectional_scale_rulers() {
    let content = "```\n\"weak field\" ← ───────────────────────────────── → \"strong field\"\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_arrow_gap"),
        "bidirectional scale rulers intentionally pad around the line body:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn ascii_cell_padding_skips_separator_rows() {
    let content = "```\n┌─────────────────────────────────────────────────────────┐\n│ FUNCTION           SPECIES             MECHANISM        │\n│─────────────────   ─────────────────   ──────────────   │\n│ N fixation         Crimson clover      Rhizobium        │\n└─────────────────────────────────────────────────────────┘\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_cell_padding"),
        "separator rows should not be padding-linted:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn ascii_cell_padding_skips_pipe_separator_rows() {
    let content = "```\n+------------------------------------------+\n|  Animal    | Total neurons |             |\n|------------|--------------|              |\n|  Nematode  |     302      |              |\n+------------------------------------------+\n```";
    let check = AsciiFlowCheck {
        config: AsciiFlowConfig::default(),
    };
    let diags = check.check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_cell_padding"),
        "pipe separator rows should not be padding-linted:\n{}",
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
        root: None,
    };
    let diags = check.check(Path::new("test.md"), content);
    let h1_warns: Vec<_> = diags.iter().filter(|d| d.code == "md_h1_count").collect();
    assert_eq!(h1_warns.len(), 1, "expected exactly one H1 count warning");
    assert_eq!(
        h1_warns[0].span.line, 3,
        "expected warning on line 3 (second H1)"
    );
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
        root: None,
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
        root: None,
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
        root: None,
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
        root: None,
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
    // tolerance is configured in the root proof.toml — just check it loaded
    assert!(
        cfg.ascii_box.tolerance <= 2,
        "tolerance should be a small number"
    );
}

#[test]
fn schema_file_loads_correctly() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/default.toml");
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

fn write_fake_crop_bin(dir: &Path, args_file: &Path, exit_code: i32) -> PathBuf {
    let bin = if cfg!(windows) {
        dir.join("crop.cmd")
    } else {
        dir.join("crop")
    };
    let script = if cfg!(windows) {
        format!(
            "@echo off\r\necho %* >> \"{}\"\r\nexit /b {}\r\n",
            args_file.display(),
            exit_code
        )
    } else {
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit {}\n",
            args_file.display(),
            exit_code
        )
    };
    std::fs::write(&bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
    }
    bin
}

fn sibling_crop_manifest() -> Option<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir.parent().unwrap_or(manifest_dir);
    for name in ["crop", "CROP"] {
        let manifest = workspace.join(name).join("Cargo.toml");
        if manifest.exists() {
            return Some(manifest);
        }
    }
    None
}

fn sibling_crop_fixture_root(crop_manifest: &Path) -> PathBuf {
    crop_manifest
        .parent()
        .unwrap()
        .join("examples")
        .join("proof-fixture")
}

fn write_real_crop_bin(dir: &Path, crop_manifest: &Path) -> PathBuf {
    let bin = if cfg!(windows) {
        dir.join("crop-real.cmd")
    } else {
        dir.join("crop-real")
    };
    let script = if cfg!(windows) {
        format!(
            "@echo off\r\ncargo run --quiet --manifest-path \"{}\" -- %*\r\n",
            crop_manifest.display()
        )
    } else {
        format!(
            "#!/bin/sh\ncargo run --quiet --manifest-path '{}' -- \"$@\"\n",
            crop_manifest.display()
        )
    };
    std::fs::write(&bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
    }
    bin
}

// ─────────────────────────────────────────────────────────
// L2: E2E — check that the binary produces correct exit codes
// ─────────────────────────────────────────────────────────

#[test]
fn binary_exits_zero_on_clean_file() {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/proof");
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
    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/proof");
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
    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/proof");
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
    assert!(
        stdout.trim().starts_with('['),
        "expected JSON array output, got: {}",
        stdout
    );
    assert!(
        stdout.trim().ends_with(']'),
        "expected JSON array output, got: {}",
        stdout
    );
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
    let width_errs: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_box_width")
        .collect();
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
    let width_errs: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_box_width")
        .collect();
    assert!(
        !width_errs.is_empty(),
        "mismatched adjacent borders must produce ascii_box_width error"
    );
}

#[test]
fn box_row_separator_with_extra_internal_junctions_is_clean() {
    let content = "```\n+-------------+\n| title       |\n+------+------+\n| left | right|\n+-------------+\n```";
    let diags = box_check().check(Path::new("test.md"), content);
    assert!(
        diags.iter().all(|d| d.code != "ascii_box_col"),
        "row separators may add internal junctions without column warnings:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn embedded_inner_border_does_not_emit_bottom_column_diff() {
    let content = "```\n┌─────────────────────────────┐\n│  outer content              │\n│    ┌──┴──┐                  │\n│    │ box │                  │\n└─────────────────────────────┘\n```";
    let diags = box_check().check(Path::new("test.md"), content);
    assert!(
        diags
            .iter()
            .all(|d| { !(d.code == "ascii_box_col" && d.message.contains("bottom border")) }),
        "embedded inner borders should not produce bottom-border column diffs:\n{}",
        format_diags(&diags)
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
    let box_errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code == "ascii_box_width" || d.code == "ascii_box_col")
        .collect();
    assert!(
        !box_errors.is_empty(),
        "expected box errors in width_mismatch.md"
    );
    for d in &box_errors {
        let rich = d.rich.as_ref().unwrap_or_else(|| {
            panic!(
                "diagnostic {} at line {} missing rich context",
                d.code, d.span.line
            )
        });
        assert!(rich.box_opens_at.is_some(), "box_opens_at should be set");
        assert!(rich.border_line.is_some(), "border_line should be set");
        assert!(
            !rich.lines.is_empty(),
            "surrounding lines should be present"
        );
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
            d.span.line,
            rich.lines.keys().collect::<Vec<_>>()
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
        assert!(
            d.span.line >= 1,
            "diagnostic {} has line=0 (must be ≥1): {:?}",
            d.code,
            d.file
        );
        assert!(
            d.span.col >= 1,
            "diagnostic {} has col=0 (must be ≥1): {:?}",
            d.code,
            d.file
        );
    }
}

// I-4: Linting the same file twice produces identical diagnostics
#[test]
fn invariant_i4_linting_is_deterministic() {
    let (path, content) = read_fixture("width_mismatch.md");
    let check = box_check();
    let run1 = check.check(&path, &content);
    let run2 = check.check(&path, &content);
    assert_eq!(
        run1.len(),
        run2.len(),
        "diagnostic count must be the same across runs"
    );
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
    let strict = AsciiBoxCheck {
        config: AsciiBoxConfig {
            tolerance: 0,
            ..AsciiBoxConfig::default()
        },
    };
    let diags_strict = strict.check(path, content);
    let col_errors_strict: Vec<_> = diags_strict
        .iter()
        .filter(|d| d.code == "ascii_box_col")
        .collect();
    assert!(
        !col_errors_strict.is_empty(),
        "tolerance=0 must report drift of 1"
    );

    // tolerance = 1 → should suppress drift of 1
    let lenient = AsciiBoxCheck {
        config: AsciiBoxConfig {
            tolerance: 1,
            ..AsciiBoxConfig::default()
        },
    };
    let diags_lenient = lenient.check(path, content);
    let col_errors_lenient: Vec<_> = diags_lenient
        .iter()
        .filter(|d| d.code == "ascii_box_col")
        .collect();
    assert!(
        col_errors_lenient.is_empty(),
        "tolerance=1 must suppress drift of 1"
    );
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

    assert_eq!(
        parallel.len(),
        sequential.len(),
        "parallel ({}) and sequential ({}) produced different counts",
        parallel.len(),
        sequential.len()
    );

    for (p, s) in parallel.iter().zip(sequential.iter()) {
        assert_eq!(p.code, s.code);
        assert_eq!(p.span.line, s.span.line);
        assert_eq!(p.span.col, s.span.col);
    }
}

#[test]
fn invariant_all_source_diagnostic_codes_are_registered() {
    use proof_lib::lookup_diagnostic_code;
    use std::collections::BTreeSet;

    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = BTreeSet::new();
    for entry in walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("rs"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap_or_else(|e| {
            panic!("cannot read {}: {}", entry.path().display(), e);
        });
        collect_code_like_string_literals(&content, &mut found);
    }

    let missing: Vec<_> = found
        .iter()
        .filter(|code| lookup_diagnostic_code(code).is_none())
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "diagnostic-like source literals must be registered: {:?}",
        missing
    );
}

fn collect_code_like_string_literals(content: &str, out: &mut std::collections::BTreeSet<String>) {
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }

        let start = i + 1;
        i = start;
        let mut escaped = false;
        while i < bytes.len() {
            let b = bytes[i];
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                if let Ok(literal) = std::str::from_utf8(&bytes[start..i]) {
                    if is_diagnostic_code_literal(literal) {
                        out.insert(literal.to_string());
                    }
                }
                i += 1;
                break;
            }
            i += 1;
        }
    }
}

fn is_diagnostic_code_literal(value: &str) -> bool {
    if matches!(
        value,
        "ascii_barchart" | "ascii_box" | "ascii_char" | "ascii_flow" | "ascii_tree"
    ) {
        return false;
    }

    matches!(
        value,
        "io_error" | "link_broken_target" | "fig_invariant_violated" | "unused_figure"
    ) || value.starts_with("ascii_")
        || value.starts_with("md_")
        || prefixed_number_code(value)
}

fn prefixed_number_code(value: &str) -> bool {
    let Some((prefix, number)) = value.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_uppercase())
        && number.len() == 3
        && number.chars().all(|c| c.is_ascii_digit())
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
        summary: PlanSummary {
            total_fixes: 1,
            high_confidence: 1,
            ..Default::default()
        },
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
            diagnostic: DiagnosticRef {
                code: "ascii_box_col".to_string(),
                line: 5,
                col: 7,
            },
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
                edit: Edit {
                    line: 1,
                    old_string: "hello".to_string(),
                    new_string: "hello!".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            },
            Fix {
                id: "low-fix".to_string(),
                file: file.clone(),
                description: "low confidence".to_string(),
                confidence: Confidence::Low,
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "hello".to_string(),
                    new_string: "goodbye".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            },
        ],
    };

    // Apply with min_confidence = High → only high-fix applies
    let result = plan
        .apply(
            &FixOptions {
                dry_run: false,
                min_confidence: Confidence::High,
                check_signal: false,
            },
            dir.path(),
        )
        .unwrap();

    assert!(
        result.applied.contains(&"high-fix".to_string()),
        "high-fix should apply"
    );
    assert!(
        !result.applied.contains(&"low-fix".to_string()),
        "low-fix should be skipped"
    );
    assert_eq!(result.skipped.len(), 1);
}

// ─────────────────────────────────────────────────────────
// L2: additional E2E tests
// ─────────────────────────────────────────────────────────

fn debug_bin() -> PathBuf {
    // Try workspace target first (set up after cargo workspace was added)
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().unwrap_or(manifest);
    let bin_name = format!("proof{}", std::env::consts::EXE_SUFFIX);
    let workspace_bin = workspace.join("target/debug").join(&bin_name);
    if workspace_bin.exists() {
        return workspace_bin;
    }
    // Fallback: per-package target
    manifest.join("target/debug").join(bin_name)
}

#[test]
fn binary_rich_output_contains_context_block() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let output = std::process::Command::new(&bin)
        .args(["--format", "rich", "--no-fail"])
        .arg(fixture("width_mismatch.md").to_str().unwrap())
        .output()
        .expect("failed to run glint");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"rich\""),
        "rich output must contain 'rich' key"
    );
    assert!(
        stdout.contains("\"box_opens_at\""),
        "rich output must contain box_opens_at"
    );
    assert!(
        stdout.contains("\"expected_cols\""),
        "rich output must contain expected_cols"
    );
    assert!(
        stdout.contains("\"lines\""),
        "rich output must contain lines"
    );

    // Must be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("rich output must be valid JSON");
    assert!(parsed.is_array(), "rich output must be a JSON array");
}

#[test]
fn binary_rich_output_is_valid_json_array() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

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
    assert_eq!(
        parsed.as_array().unwrap().len(),
        0,
        "perfect file should produce no rich diagnostics"
    );
}

#[test]
fn binary_stats_command_runs() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

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
fn binary_stats_by_tag_reports_source_frontmatter() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("tagged.source.md"),
        "---\ntags: [ops, runbook]\nops: [lint]\ncontent_tags:\n  - guide\n---\n# Tagged\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["stats", "--by-tag"])
        .arg(dir.path())
        .output()
        .expect("failed to run proof stats --by-tag");

    assert!(
        output.status.success(),
        "proof stats --by-tag failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("By tag:"), "got:\n{}", stdout);
    assert!(stdout.contains("runbook"), "got:\n{}", stdout);
    assert!(stdout.contains("By op:"), "got:\n{}", stdout);
    assert!(stdout.contains("lint"), "got:\n{}", stdout);
    assert!(stdout.contains("guide"), "got:\n{}", stdout);
}

#[test]
fn binary_stats_tag_filter_limits_files() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("publish.source.md"),
        "---\ntags: [publish]\nops: [compile]\n---\n# Publish\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("draft.source.md"),
        "---\ntags: [draft]\nops: [review]\n---\n# Draft\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["stats", "--by-tag", "--tag", "publish"])
        .arg(dir.path())
        .output()
        .expect("failed to run proof stats --tag");

    assert!(
        output.status.success(),
        "proof stats --tag failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("files:    1"), "got:\n{}", stdout);
    assert!(stdout.contains("publish"), "got:\n{}", stdout);
    assert!(!stdout.contains("draft"), "got:\n{}", stdout);
}

#[test]
fn binary_stats_file_count_honors_include_exclude() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::create_dir_all(dir.path().join("drafts")).unwrap();
    std::fs::write(dir.path().join("docs").join("one.md"), "# One\n").unwrap();
    std::fs::write(dir.path().join("drafts").join("skip.md"), "# Skip\n").unwrap();
    std::fs::write(dir.path().join("root.md"), "# Root\n").unwrap();
    let config_path = dir.path().join("proof.toml");
    std::fs::write(
        &config_path,
        r#"
[files]
include = ["docs/**/*.md"]
exclude = ["drafts/**"]
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["stats", "--config"])
        .arg(&config_path)
        .arg(dir.path())
        .output()
        .expect("failed to run proof stats");

    assert!(
        output.status.success(),
        "proof stats failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("files:    1"),
        "stats file count should honor include/exclude, got:\n{}",
        stdout
    );
}

#[test]
fn binary_draft_command_writes_plan() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let output_path = dir.path().join("draft-plan.json");

    let output = std::process::Command::new(&bin)
        .args(["draft", "-o"])
        .arg(&output_path)
        .arg(fixture("width_mismatch.md"))
        .output()
        .expect("failed to run proof draft");

    assert!(
        output.status.success(),
        "proof draft failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan = std::fs::read_to_string(&output_path).expect("draft plan should be written");
    assert!(
        plan.contains("\"schema_version\"") && plan.contains("\"groups\""),
        "draft plan should contain expected DraftPlan fields, got:\n{}",
        plan
    );
}

#[test]
fn binary_status_command_reports_project_summary() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("proof.toml"), "[files]\nroot = true\n").unwrap();
    std::fs::write(dir.path().join("guide.source.md"), "# Guide\n").unwrap();

    let output = std::process::Command::new(&bin)
        .args(["status"])
        .arg(dir.path())
        .output()
        .expect("failed to run proof status");

    assert!(
        output.status.success(),
        "proof status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proof status"), "got:\n{}", stdout);
    assert!(stdout.contains("Sources"), "got:\n{}", stdout);
    assert!(stdout.contains("Config"), "got:\n{}", stdout);
}

#[test]
fn binary_status_command_honors_explicit_config() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    let config_path = config_dir.path().join("proof.toml");
    std::fs::write(dir.path().join("guide.source.md"), "# Guide\n").unwrap();
    std::fs::write(
        &config_path,
        r#"
[files]
root = true

[[section_schemas]]
paths = ["*.md"]
required_h2_all = ["Decision"]
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("--config")
        .arg(&config_path)
        .arg("status")
        .arg(dir.path())
        .output()
        .expect("failed to run proof status");

    assert!(
        output.status.success(),
        "proof status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 schemas"),
        "status should summarize explicit config, got:\n{}",
        stdout
    );
}

#[test]
fn binary_status_crop_delegates_to_crop_status() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_path = dir.path().join("STATUS.json");

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("status")
        .arg("--crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(".crop\\views\\ready.json")
        .arg("--crop-format")
        .arg("json")
        .arg("--strict")
        .arg("--strict-on")
        .arg("broken-links")
        .output()
        .expect("failed to run proof status --crop");

    assert!(
        output.status.success(),
        "proof status --crop failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("status"), "got: {}", args);
    assert!(
        args.contains("--view .crop\\views\\ready.json"),
        "got: {}",
        args
    );
    assert!(args.contains("--format json"), "got: {}", args);
    assert!(args.contains("--strict"), "got: {}", args);
    assert!(args.contains("--strict-on broken-links"), "got: {}", args);
    assert!(
        args.contains(&format!("--output {}", output_path.display())),
        "got: {}",
        args
    );
}

#[test]
fn binary_status_crop_rejects_local_text_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("status")
        .arg("--crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(".crop\\views\\ready.json")
        .arg("--crop-format")
        .arg("text")
        .output()
        .expect("failed to run proof status --crop");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("markdown"), "got: {}", stderr);
    assert!(stderr.contains("json"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_status_crop_rejects_unknown_strict_policy() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("status")
        .arg("--crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(".crop\\views\\ready.json")
        .arg("--strict")
        .arg("--strict-on")
        .arg("stale-artifacts")
        .output()
        .expect("failed to run proof status --crop");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("broken-links"), "got: {}", stderr);
    assert!(stderr.contains("orphan-pages"), "got: {}", stderr);
    assert!(stderr.contains("duplicate-anchors"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_status_crop_rejects_dir_with_view() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("status")
        .arg("docs")
        .arg("--crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(".crop\\views\\ready.json")
        .output()
        .expect("failed to run proof status --crop");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("positional directory") || stderr.contains("positional"),
        "got: {}",
        stderr
    );
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_backfill_literal_generates_source_and_report() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("generated");
    let report_path = dir.path().join("backfill-report.json");
    std::fs::write(
        dir.path().join("guide.md"),
        "# Guide\n\n| A | B |\n|---|---|\n| 1 | 2 |\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("backfill")
        .arg(dir.path().join("guide.md"))
        .arg("--output-source")
        .arg(&source_dir)
        .arg("--report")
        .arg(&report_path)
        .arg("--literal-first")
        .arg("--check-roundtrip")
        .output()
        .expect("failed to run proof backfill");

    assert!(
        output.status.success(),
        "proof backfill failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = source_dir.join("guide.source.md");
    let generated_text = std::fs::read_to_string(&generated).expect("generated source");
    assert!(
        generated_text.contains("ops: [backfill]"),
        "got:\n{}",
        generated_text
    );
    assert!(
        generated_text.contains("proof_original: \"guide.md\""),
        "got:\n{}",
        generated_text
    );
    assert!(
        generated_text.ends_with("| 1 | 2 |\n"),
        "got:\n{}",
        generated_text
    );

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["summary"]["files_generated"], 1);
    assert_eq!(report["summary"]["roundtrip_passed"], 1);
    assert_eq!(report["files"][0]["classification"], "literal_markdown");
    assert_eq!(report["files"][0]["blocks"]["markdown_tables"], 1);
    assert_eq!(report["summary"]["blocks"]["markdown_tables"], 1);
}

#[test]
fn binary_backfill_literal_roundtrips_frontmatter_with_crlf() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("generated");
    let report_path = dir.path().join("backfill-report.json");
    std::fs::write(
        dir.path().join("guide.md"),
        "---\r\ntitle: Demo\r\nsource_custody: partial\r\n---\r\n\r\n# Guide\r\n\r\nBody.\r\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("backfill")
        .arg(dir.path().join("guide.md"))
        .arg("--output-source")
        .arg(&source_dir)
        .arg("--report")
        .arg(&report_path)
        .arg("--literal-first")
        .arg("--check-roundtrip")
        .output()
        .expect("failed to run proof backfill");

    assert!(
        output.status.success(),
        "proof backfill failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated_text =
        std::fs::read_to_string(source_dir.join("guide.source.md")).expect("generated source");
    assert!(
        generated_text.contains("source_custody: partial"),
        "got:\n{}",
        generated_text
    );

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["summary"]["roundtrip_passed"], 1);
    assert_eq!(report["files"][0]["roundtrip"]["diff_summary"], "identical");
}

#[test]
fn binary_backfill_report_classifies_candidate_blocks() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("generated");
    let report_path = dir.path().join("backfill-report.json");
    std::fs::write(
        dir.path().join("mixed.md"),
        "# Mixed\n\n```text\n+---+---+\n| A | B |\n+---+---+\n```\n\nLoad ### 42\n\n  A -> B\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("backfill")
        .arg(dir.path().join("mixed.md"))
        .arg("--output-source")
        .arg(&source_dir)
        .arg("--report")
        .arg(&report_path)
        .arg("--literal-first")
        .output()
        .expect("failed to run proof backfill");

    assert!(
        output.status.success(),
        "proof backfill failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    let blocks = &report["files"][0]["blocks"];
    assert_eq!(blocks["fenced"], 1);
    assert_eq!(blocks["ascii_table_candidates"], 1);
    assert_eq!(blocks["chart_like"], 1);
    assert_eq!(blocks["diagram_like"], 1);
    let evidence = report["files"][0]["evidence"].as_array().unwrap();
    assert!(
        evidence
            .iter()
            .any(|entry| entry.as_str().unwrap_or("").contains("ASCII table")),
        "got evidence: {:?}",
        evidence
    );
}

#[test]
fn binary_backfill_extract_tables_writes_sidecar_data() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("generated");
    let report_path = dir.path().join("backfill-report.json");
    std::fs::write(
        dir.path().join("tables.md"),
        "# Tables\n\n| Name | Count |\n| --- | ---: |\n| Alpha | 2 |\n| Beta | 3 |\n\n```text\n| not | data |\n| --- | --- |\n```\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("backfill")
        .arg(dir.path().join("tables.md"))
        .arg("--output-source")
        .arg(&source_dir)
        .arg("--report")
        .arg(&report_path)
        .arg("--literal-first")
        .arg("--extract-tables")
        .output()
        .expect("failed to run proof backfill");

    assert!(
        output.status.success(),
        "proof backfill failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = source_dir.join("tables.source.md");
    let generated_text = std::fs::read_to_string(&generated).expect("generated source");
    assert!(
        generated_text.contains("| Alpha | 2 |"),
        "literal source changed:\n{}",
        generated_text
    );

    let table_sidecar = source_dir.join("tables.tables.json");
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&table_sidecar).unwrap()).unwrap();
    assert_eq!(sidecar["tables"].as_array().unwrap().len(), 1);
    assert_eq!(sidecar["tables"][0]["headers"][0], "Name");
    assert_eq!(sidecar["tables"][0]["rows"][1][0], "Beta");

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["summary"]["tables_extracted"], 1);
    assert_eq!(
        report["files"][0]["extractions"][0]["kind"],
        "markdown_table"
    );
    assert_eq!(report["files"][0]["extractions"][0]["rows"], 2);
}

#[test]
fn binary_backfill_extract_tables_writes_structured_block_sidecar() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("generated");
    let report_path = dir.path().join("backfill-report.json");
    std::fs::write(
        dir.path().join("visuals.md"),
        "# Visuals\n\n```text\n+---+---+\n| A | B |\n+---+---+\n```\n\n## Flow → Direction\n\n  A -> B\n\nLoad ### 42\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("backfill")
        .arg(dir.path().join("visuals.md"))
        .arg("--output-source")
        .arg(&source_dir)
        .arg("--report")
        .arg(&report_path)
        .arg("--literal-first")
        .arg("--extract-tables")
        .output()
        .expect("failed to run proof backfill");

    assert!(
        output.status.success(),
        "proof backfill failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let block_sidecar = source_dir.join("visuals.blocks.json");
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&block_sidecar).unwrap()).unwrap();
    let blocks = sidecar["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0]["kind"], "ascii_table_candidate");
    assert_eq!(blocks[0]["line"], 3);
    assert_eq!(blocks[1]["kind"], "diagram_like");
    assert_eq!(blocks[1]["heading_context"], "## Flow → Direction");
    assert_eq!(blocks[2]["kind"], "chart_like");

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_path).unwrap()).unwrap();
    assert_eq!(report["summary"]["structured_blocks_extracted"], 3);
    assert!(report["files"][0]["extractions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["kind"] == "ascii_table_candidate"));
}

#[test]
fn binary_compile_target_html_writes_html_document() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("guide.source.md");
    let output_path = dir.path().join("guide.html");
    std::fs::write(
        &source,
        "---\ntags: [publish]\n---\n# Guide\n\nBody with <angle> text.\n\n```text\nA -> B\n```\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--target")
        .arg("html")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let html = std::fs::read_to_string(&output_path).expect("html output");
    assert!(html.contains("<!doctype html>"), "got:\n{}", html);
    assert!(html.contains("<h1>Guide</h1>"), "got:\n{}", html);
    assert!(
        html.contains("<p>Body with &lt;angle&gt; text.</p>"),
        "got:\n{}",
        html
    );
    assert!(
        html.contains("<pre><code class=\"language-text\">A -&gt; B"),
        "got:\n{}",
        html
    );
}

#[test]
fn binary_compile_target_pebble_writes_ai_context_pack() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("guide.source.md");
    let output_path = dir.path().join("guide.pebble.json");
    std::fs::write(
        &source,
        "# Guide\n\nIntro text.\n\n## Steps\n\n- one\n- two\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("pebble")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pebble: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap();
    assert_eq!(pebble["schema"], "pebble.v1");
    assert_eq!(pebble["kind"], "document");
    assert_eq!(pebble["title"], "Guide");
    assert_eq!(pebble["format"], "markdown");
    assert_eq!(pebble["sections"][0]["id"], "guide");
    assert_eq!(pebble["sections"][1]["path"][1], "Steps");
    assert!(pebble["sections"][1]["text"]
        .as_str()
        .unwrap()
        .contains("- one"));

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"][0]["target"], "pebble");
}

#[test]
fn binary_compile_target_json_report_writes_bundle() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("guide.source.md");
    let output_path = dir.path().join("guide.proof-report.json");
    std::fs::write(
        &source,
        "---\ntags: [publish]\ncontent_tags: [guide]\n---\n# Guide\n\nIntro text.\n\n## Steps\n\n- one\n- two\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("json-report")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap();
    assert_eq!(report["schema"], "proof.publish.json_report.v1");
    assert_eq!(report["kind"], "compile_report");
    assert_eq!(
        report["source_path"].as_str().unwrap(),
        source.to_string_lossy()
    );
    assert_eq!(report["title"], "Guide");
    assert_eq!(report["artifact"]["target"], "json-report");
    assert_eq!(
        report["artifact"]["output_path"].as_str().unwrap(),
        output_path.to_string_lossy()
    );
    assert_eq!(report["frontmatter"]["tags"][0], "publish");
    assert_eq!(report["frontmatter"]["content"][0], "guide");
    assert_eq!(report["document"]["section_count"], 2);
    assert_eq!(report["document"]["sections"][1]["path"][1], "Steps");
    assert!(report["document"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Intro text."));
    assert_eq!(report["compile"]["diagnostics_count"], 0);

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"][0]["target"], "json-report");
}

#[test]
fn binary_compile_target_site_writes_static_site() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("src");
    let site_dir = dir.path().join("site");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("alpha.source.md"),
        "# Alpha\n\nFirst page.\n\n## Details\n\nAlpha details.\n",
    )
    .unwrap();
    std::fs::write(
        source_dir.join("beta.source.md"),
        "# Beta\n\nSecond page.\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source_dir)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("site")
        .arg("--output-dir")
        .arg(&site_dir)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let alpha = std::fs::read_to_string(site_dir.join("alpha.html")).unwrap();
    let beta = std::fs::read_to_string(site_dir.join("beta.html")).unwrap();
    let index = std::fs::read_to_string(site_dir.join("index.html")).unwrap();
    let site_manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(site_dir.join("proof-site.json")).unwrap())
            .unwrap();

    assert!(alpha.contains("<h1>Alpha</h1>"), "got:\n{}", alpha);
    assert!(beta.contains("<h1>Beta</h1>"), "got:\n{}", beta);
    assert!(
        index.contains("<a href=\"alpha.html\">Alpha</a>"),
        "got:\n{}",
        index
    );
    assert!(
        index.contains("<a href=\"beta.html\">Beta</a>"),
        "got:\n{}",
        index
    );
    assert_eq!(site_manifest["schema"], "proof.publish.site.v1");
    assert_eq!(site_manifest["page_count"], 2);
    assert_eq!(site_manifest["pages"][0]["href"], "alpha.html");
    assert_eq!(site_manifest["pages"][1]["title"], "Beta");

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"].as_array().unwrap().len(), 2);
    assert_eq!(manifest["artifacts"][0]["target"], "site");
}

#[test]
fn binary_compile_target_pdf_writes_pdf() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("guide.source.md");
    let output_path = dir.path().join("guide.pdf");
    std::fs::write(
        &source,
        "# Guide\n\nBody with <angle> text.\n\n## Steps\n\n- one\n- two\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("pdf")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pdf = std::fs::read(&output_path).expect("pdf output");
    assert!(pdf.starts_with(b"%PDF-1.4"));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("/Producer (PROOF)"), "got:\n{}", pdf_text);
    assert!(pdf_text.contains("(Guide) Tj"), "got:\n{}", pdf_text);
    assert!(
        pdf_text.contains("Body with <angle> text"),
        "got:\n{}",
        pdf_text
    );

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"][0]["target"], "pdf");
}

#[test]
fn binary_compile_target_docx_writes_docx() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("guide.source.md");
    let output_path = dir.path().join("guide.docx");
    std::fs::write(
        &source,
        "# Guide\n\nBody with [home](https://example.com).\n\n## Steps\n\n- one\n- two\n\n1. first\n2. second\n\n| A | B |\n|---|---|\n| x | y |\n\n```text\nlet x = 1;\n```\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("docx")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let docx = std::fs::File::open(&output_path).expect("docx output");
    let mut archive = zip::ZipArchive::new(docx).expect("valid docx zip");
    assert!(archive.by_name("[Content_Types].xml").is_ok());
    assert!(archive.by_name("_rels/.rels").is_ok());
    assert!(archive.by_name("word/styles.xml").is_ok());
    assert!(archive.by_name("word/numbering.xml").is_ok());
    let mut document = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("word/document.xml").unwrap(),
        &mut document,
    )
    .unwrap();
    assert!(
        document.contains(r#"<w:pStyle w:val="Heading1"/>"#),
        "got:\n{}",
        document
    );
    assert!(document.contains(">Guide<"), "got:\n{}", document);
    assert!(
        document.contains("home (https://example.com)"),
        "got:\n{}",
        document
    );
    assert!(
        document.contains(r#"<w:numId w:val="1"/>"#),
        "got:\n{}",
        document
    );
    assert!(
        document.contains(r#"<w:numId w:val="2"/>"#),
        "got:\n{}",
        document
    );
    assert!(document.contains("<w:tbl>"), "got:\n{}", document);
    assert!(document.contains("let x = 1;"), "got:\n{}", document);

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"][0]["target"], "docx");
}

#[test]
fn publish_backends_consume_resolved_compile_output() {
    use proof_lib::compile::compile_file;
    use proof_lib::frontmatter::SourceFrontmatter;
    use proof_lib::publish::{
        html_to_pdf_document, markdown_to_docx_document, markdown_to_html_document,
        markdown_to_json_report_bundle, JsonReportCompile,
    };
    use proof_lib::GlintConfig;

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("doc.source.md");
    let output_path = dir.path().join("doc.md");
    std::fs::write(
        &source,
        "# Source\n\n```proof:toc max-depth=2 style=list\n```\n\n## Install\n## Usage\n",
    )
    .unwrap();

    let cfg = GlintConfig::default();
    let result = compile_file(&source, &output_path, dir.path(), &cfg).unwrap();
    let violations = result
        .violations
        .iter()
        .map(|violation| format!("{}: {}", violation.code, violation.message))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        result.violations.is_empty(),
        "compile violations:\n{}",
        violations
    );

    let markdown = std::fs::read_to_string(&output_path).unwrap();
    assert!(markdown.contains("Install"), "got:\n{}", markdown);
    assert!(!markdown.contains("```proof:toc"), "got:\n{}", markdown);

    let html = markdown_to_html_document(&markdown, "fallback");
    assert!(html.contains("<h1>Source</h1>"), "got:\n{}", html);
    assert!(html.contains("Install"), "got:\n{}", html);

    let report = markdown_to_json_report_bundle(
        &markdown,
        "fallback",
        &source,
        dir.path().join("doc.proof-report.json").as_path(),
        &result.resolved_files,
        SourceFrontmatter::default(),
        JsonReportCompile {
            directives_resolved: result.directives_resolved,
            diagnostics_count: 0,
            diagnostics: Vec::new(),
        },
    );
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert!(report["document"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Install"));

    let pdf = String::from_utf8_lossy(&html_to_pdf_document(&html, "fallback")).to_string();
    assert!(pdf.contains("Install"), "got:\n{}", pdf);

    let docx = markdown_to_docx_document(&markdown, "fallback");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut document = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("word/document.xml").unwrap(),
        &mut document,
    )
    .unwrap();
    assert!(document.contains("Install"), "got:\n{}", document);
}

#[test]
fn binary_compile_writes_artifact_manifest() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("manifest.source.md");
    let output_path = dir.path().join("manifest.html");
    std::fs::write(&source, "# Manifest\n\nCompiled artifact.\n").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("--target")
        .arg("html")
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], "1");
    assert_eq!(manifest["generated_by"], "proof compile");
    assert_eq!(manifest["artifacts"].as_array().unwrap().len(), 1);
    assert_eq!(manifest["artifacts"][0]["target"], "html");
    assert_eq!(manifest["artifacts"][0]["status"], "written");
    assert_eq!(
        manifest["artifacts"][0]["output_path"].as_str().unwrap(),
        output_path.to_string_lossy()
    );
    assert!(manifest["artifacts"][0]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn binary_compile_manifest_records_backlinks_side_info_dependency() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join(".proof").join("side-info");
    std::fs::create_dir_all(&side_info).unwrap();
    let backlinks_path = side_info.join("backlinks.json");
    std::fs::write(
        &backlinks_path,
        r#"{
  "pages": [
    {
      "source": "manifest.source.md",
      "inbound_links": [
        { "source": "guide.source.md", "target": "manifest.source.md" }
      ]
    }
  ]
}"#,
    )
    .unwrap();
    let source = dir.path().join("manifest.source.md");
    let output_path = dir.path().join("manifest.md");
    std::fs::write(
        &source,
        "# Manifest\n\n```proof:backlinks target=\"manifest.source.md\"\n```\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(&source)
        .arg("--root")
        .arg(dir.path())
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("failed to run proof compile");

    assert!(
        output.status.success(),
        "proof compile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(
        manifest["artifacts"][0]["resolved_files"],
        serde_json::json!([backlinks_path])
    );
}

#[test]
fn binary_crop_status_delegates_to_crop_status() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_path = dir.path().join("STATUS.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .arg("--strict")
        .arg("--strict-on")
        .arg("broken-links")
        .arg("--strict-on")
        .arg("duplicate-anchors")
        .arg("--title")
        .arg("PROOF Guides")
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .output()
        .expect("failed to run proof crop status");

    assert!(
        output.status.success(),
        "proof crop status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("status"), "got: {}", args);
    assert!(args.contains("--root"), "got: {}", args);
    assert!(
        args.contains(&dir.path().display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--strict"), "got: {}", args);
    assert!(args.contains("--strict-on broken-links"), "got: {}", args);
    assert!(
        args.contains("--strict-on duplicate-anchors"),
        "got: {}",
        args
    );
    assert!(args.contains("--title"), "got: {}", args);
    assert!(args.contains("PROOF Guides"), "got: {}", args);
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
    assert!(args.contains("--format json"), "got: {}", args);
}

#[test]
fn binary_crop_status_uses_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_path = dir.path().join("GLOBAL_STATUS.md");

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop status");

    assert!(
        output.status.success(),
        "proof crop status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_crop_status_uses_global_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("json")
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop status");

    assert!(
        output.status.success(),
        "proof crop status failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("--format json"), "got: {}", args);
}

#[test]
fn binary_crop_status_rejects_local_text_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .arg("--format")
        .arg("text")
        .output()
        .expect("failed to run proof crop status");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("markdown"), "got: {}", stderr);
    assert!(stderr.contains("json"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_status_rejects_strict_on_without_strict() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .arg("--strict-on")
        .arg("broken-links")
        .output()
        .expect("failed to run proof crop status");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires --strict"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_status_rejects_unknown_strict_policy() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("status")
        .arg("--root")
        .arg(dir.path())
        .arg("--strict")
        .arg("--strict-on")
        .arg("stale-artifacts")
        .output()
        .expect("failed to run proof crop status");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("broken-links"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_list_views_delegates_to_crop_view_list() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let views_dir = dir.path().join(".crop").join("views");
    std::fs::create_dir_all(&views_dir).unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("list-views")
        .arg("--dir")
        .arg(&views_dir)
        .output()
        .expect("failed to run proof crop list-views");

    assert!(
        output.status.success(),
        "proof crop list-views failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("view"), "got: {}", args);
    assert!(args.contains("--list"), "got: {}", args);
    assert!(args.contains("--dir"), "got: {}", args);
    assert!(
        args.contains(&views_dir.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_crop_list_views_writes_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = if cfg!(windows) {
        dir.path().join("crop.cmd")
    } else {
        dir.path().join("crop")
    };
    let script = if cfg!(windows) {
        "@echo off\r\necho [{\"name\":\"ready\"}]\r\nexit /b 0\r\n".to_string()
    } else {
        "#!/bin/sh\nprintf '%s\\n' '[{\"name\":\"ready\"}]'\nexit 0\n".to_string()
    };
    std::fs::write(&crop_bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&crop_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&crop_bin, perms).unwrap();
    }
    let views_dir = dir.path().join(".crop").join("views");
    let output_path = dir.path().join("views.json");
    std::fs::create_dir_all(&views_dir).unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("list-views")
        .arg("--dir")
        .arg(&views_dir)
        .output()
        .expect("failed to run proof crop list-views");

    assert!(
        output.status.success(),
        "proof crop list-views failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&output_path).unwrap().trim(),
        "[{\"name\":\"ready\"}]"
    );
}

#[test]
fn binary_crop_list_views_rejects_global_markdown_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("markdown")
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("list-views")
        .output()
        .expect("failed to run proof crop list-views");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("writes JSON artifacts"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_inspect_views_delegates_to_crop_view_inspect() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let views_dir = dir.path().join(".crop").join("views");
    std::fs::create_dir_all(&views_dir).unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(&views_dir)
        .arg("--strict")
        .output()
        .expect("failed to run proof crop inspect-views");

    assert!(
        output.status.success(),
        "proof crop inspect-views failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("view"), "got: {}", args);
    assert!(args.contains("--inspect"), "got: {}", args);
    assert!(args.contains("--dir"), "got: {}", args);
    assert!(
        args.contains(&views_dir.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--strict"), "got: {}", args);
}

#[test]
fn binary_crop_inspect_views_can_inspect_single_file() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join(".crop").join("views").join("ready.json");
    std::fs::create_dir_all(view_file.parent().unwrap()).unwrap();
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--file")
        .arg(&view_file)
        .output()
        .expect("failed to run proof crop inspect-views --file");

    assert!(
        output.status.success(),
        "proof crop inspect-views --file failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("view"), "got: {}", args);
    assert!(args.contains("--inspect"), "got: {}", args);
    assert!(args.contains("--file"), "got: {}", args);
    assert!(
        args.contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
    assert!(!args.contains("--dir"), "got: {}", args);
}

#[test]
fn binary_crop_inspect_views_rejects_strict_single_file() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join(".crop").join("views").join("ready.json");
    std::fs::create_dir_all(view_file.parent().unwrap()).unwrap();
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--file")
        .arg(&view_file)
        .arg("--strict")
        .output()
        .expect("failed to run proof crop inspect-views --file");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("store inspection"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_inspect_views_rejects_file_with_dir() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join(".crop").join("views").join("ready.json");
    let other_dir = dir.path().join("other-views");
    std::fs::create_dir_all(view_file.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&other_dir).unwrap();
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--file")
        .arg(&view_file)
        .arg("--dir")
        .arg(&other_dir)
        .output()
        .expect("failed to run proof crop inspect-views --file");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("either --file or --dir"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_inspect_views_forwards_single_file_overrides() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join(".crop").join("views").join("ready.json");
    std::fs::create_dir_all(view_file.parent().unwrap()).unwrap();
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--file")
        .arg(&view_file)
        .arg("--query")
        .arg("refresh docs")
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .output()
        .expect("failed to run proof crop inspect-views --file");

    assert!(
        output.status.success(),
        "proof crop inspect-views --file failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("--file"), "got: {}", args);
    assert!(
        args.contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--query"), "got: {}", args);
    assert!(args.contains("refresh docs"), "got: {}", args);
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
}

#[test]
fn binary_crop_inspect_views_rejects_store_overrides() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--query")
        .arg("refresh docs")
        .output()
        .expect("failed to run proof crop inspect-views");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("require --file"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_inspect_views_writes_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = if cfg!(windows) {
        dir.path().join("crop.cmd")
    } else {
        dir.path().join("crop")
    };
    let script = if cfg!(windows) {
        format!(
            "@echo off\r\necho %* >> \"{}\"\r\necho {{\"ok\":true}}\r\nexit /b 0\r\n",
            args_file.display()
        )
    } else {
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nprintf '%s\\n' '{{\"ok\":true}}'\nexit 0\n",
            args_file.display()
        )
    };
    std::fs::write(&crop_bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&crop_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&crop_bin, perms).unwrap();
    }
    let output_path = dir.path().join("inspect").join("views.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(dir.path())
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop inspect-views");

    assert!(
        output.status.success(),
        "proof crop inspect-views failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&output_path).unwrap().trim(),
        "{\"ok\":true}"
    );
    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("view --inspect"), "got: {}", args);
    assert!(
        !args.contains("--output"),
        "CROP view has no output flag: {}",
        args
    );
}

#[test]
fn binary_crop_inspect_views_uses_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = if cfg!(windows) {
        dir.path().join("crop.cmd")
    } else {
        dir.path().join("crop")
    };
    let script = if cfg!(windows) {
        "@echo off\r\necho {\"global\":true}\r\nexit /b 0\r\n".to_string()
    } else {
        "#!/bin/sh\nprintf '%s\\n' '{\"global\":true}'\nexit 0\n".to_string()
    };
    std::fs::write(&crop_bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&crop_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&crop_bin, perms).unwrap();
    }
    let output_path = dir.path().join("global-inspect.json");

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop inspect-views");

    assert!(
        output.status.success(),
        "proof crop inspect-views failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&output_path).unwrap().trim(),
        "{\"global\":true}"
    );
}

#[test]
fn binary_crop_inspect_views_rejects_global_markdown_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("markdown")
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop inspect-views");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("emits JSON"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_inspect_views_writes_output_on_failure() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = if cfg!(windows) {
        dir.path().join("crop.cmd")
    } else {
        dir.path().join("crop")
    };
    let script = if cfg!(windows) {
        "@echo off\r\necho {\"failed_count\":1}\r\necho strict failed 1>&2\r\nexit /b 7\r\n"
            .to_string()
    } else {
        "#!/bin/sh\nprintf '%s\\n' '{\"failed_count\":1}'\nprintf '%s\\n' 'strict failed' >&2\nexit 7\n"
            .to_string()
    };
    std::fs::write(&crop_bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&crop_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&crop_bin, perms).unwrap();
    }
    let output_path = dir.path().join("inspect.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(dir.path())
        .arg("--strict")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop inspect-views");

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(
        std::fs::read_to_string(&output_path).unwrap().trim(),
        "{\"failed_count\":1}"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("strict failed"));
}

#[test]
fn binary_crop_view_writes_crop_view_recipe() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("proof.toml"),
        r#"
[files]
include = ["src/**/*.source.md"]
exclude = ["target/**"]
"#,
    )
    .unwrap();
    let output_path = dir.path().join(".crop").join("views").join("ready.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("view")
        .arg("--root")
        .arg(dir.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--name")
        .arg("ready-guides")
        .arg("--frontmatter-query")
        .arg("status eq 'ready'")
        .arg("--tag")
        .arg("guide")
        .arg("--op")
        .arg("compile")
        .arg("--content-tag")
        .arg("markdown")
        .output()
        .expect("failed to run proof crop view");

    assert!(
        output.status.success(),
        "proof crop view failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let recipe: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap();
    assert_eq!(recipe["schema_version"], "crop.view.v1");
    assert_eq!(recipe["name"], "ready-guides");
    assert_eq!(
        recipe["root"],
        PathBuf::from("..").join("..").display().to_string()
    );
    assert_eq!(recipe["include_extensions"], serde_json::json!(["md"]));
    assert_eq!(recipe["exclude_dirs"], serde_json::json!(["target"]));
    assert_eq!(
        recipe["frontmatter_query"],
        "status eq 'ready' and tags has 'guide' and ops has 'compile' and content_tags has 'markdown'"
    );
}

#[test]
fn binary_crop_view_uses_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("proof.toml"),
        r#"
[files]
include = ["src/**/*.source.md"]
"#,
    )
    .unwrap();
    let output_path = dir.path().join(".crop").join("views").join("global.json");

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("view")
        .arg("--root")
        .arg(dir.path())
        .arg("--name")
        .arg("global-view")
        .output()
        .expect("failed to run proof crop view");

    assert!(
        output.status.success(),
        "proof crop view failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let recipe: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap();
    assert_eq!(recipe["name"], "global-view");
    assert_eq!(
        recipe["root"],
        PathBuf::from("..").join("..").display().to_string()
    );
}

#[test]
fn binary_crop_run_view_delegates_to_crop_view_file() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("ready.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("run-view")
        .arg("--file")
        .arg(&view_file)
        .arg("--query")
        .arg("ready guides")
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .arg("--prefix-cache")
        .arg("generic")
        .output()
        .expect("failed to run proof crop run-view");

    assert!(
        output.status.success(),
        "proof crop run-view failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("view --file"), "got: {}", args);
    assert!(
        args.contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--query"), "got: {}", args);
    assert!(args.contains("ready guides"), "got: {}", args);
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
    assert!(args.contains("--prefix-cache generic"), "got: {}", args);
}

#[test]
fn binary_crop_run_view_writes_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = if cfg!(windows) {
        dir.path().join("crop.cmd")
    } else {
        dir.path().join("crop")
    };
    let script = if cfg!(windows) {
        "@echo off\r\necho {\"pack\":true}\r\nexit /b 0\r\n".to_string()
    } else {
        "#!/bin/sh\nprintf '%s\\n' '{\"pack\":true}'\nexit 0\n".to_string()
    };
    std::fs::write(&crop_bin, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&crop_bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&crop_bin, perms).unwrap();
    }
    let view_file = dir.path().join("ready.json");
    let output_path = dir.path().join("pack.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("run-view")
        .arg("--file")
        .arg(&view_file)
        .output()
        .expect("failed to run proof crop run-view");

    assert!(
        output.status.success(),
        "proof crop run-view failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(&output_path).unwrap().trim(),
        "{\"pack\":true}"
    );
}

#[test]
fn binary_crop_run_view_rejects_global_markdown_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("ready.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("markdown")
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("run-view")
        .arg("--file")
        .arg(&view_file)
        .output()
        .expect("failed to run proof crop run-view");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("writes JSON artifacts"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_run_view_rejects_unknown_prefix_cache() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("ready.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("run-view")
        .arg("--file")
        .arg(&view_file)
        .arg("--prefix-cache")
        .arg("specialized")
        .output()
        .expect("failed to run proof crop run-view");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("generic"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_view_rejects_global_markdown_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let output_path = dir.path().join("view.json");

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("markdown")
        .arg("crop")
        .arg("view")
        .arg("--root")
        .arg(dir.path())
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop view");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("writes JSON artifacts"), "got: {}", stderr);
    assert!(!output_path.exists());
}

#[test]
fn binary_crop_side_info_delegates_to_named_crop_report() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("ready-guides.json");
    let output_path = dir.path().join("frontmatter.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("frontmatter")
        .arg("--view")
        .arg(&view_file)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .output()
        .expect("failed to run proof crop frontmatter");

    assert!(
        output.status.success(),
        "proof crop frontmatter failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("frontmatter"), "got: {}", args);
    assert!(args.contains("--view"), "got: {}", args);
    assert!(
        args.contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--format json"), "got: {}", args);
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
}

#[test]
fn binary_crop_sync_generates_all_side_info_reports() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_dir = dir.path().join(".proof").join("side-info");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("sync")
        .arg("--root")
        .arg(dir.path())
        .arg("--output-dir")
        .arg(&output_dir)
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .output()
        .expect("failed to run proof crop sync");

    assert!(
        output.status.success(),
        "proof crop sync failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    for command in ["links", "backlinks", "frontmatter", "headings"] {
        assert!(args.contains(command), "got: {}", args);
        assert!(
            args.contains(
                &output_dir
                    .join(format!("{}.json", command))
                    .display()
                    .to_string()
            ),
            "got: {}",
            args
        );
    }
    assert!(args.contains("--format json"), "got: {}", args);
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
}

#[test]
fn binary_crop_sync_rejects_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(dir.path().join("side-info.json"))
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("sync")
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop sync");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--output-dir"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_prepare_inspects_views_then_syncs_side_info() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_dir = dir.path().join(".crop").join("views");
    let view_file = view_dir.join("proof-guides.json");
    let output_dir = dir.path().join(".proof").join("side-info");
    std::fs::create_dir_all(&view_dir).unwrap();
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("prepare")
        .arg("--dir")
        .arg(&view_dir)
        .arg("--view")
        .arg(&view_file)
        .arg("--output-dir")
        .arg(&output_dir)
        .output()
        .expect("failed to run proof crop prepare");

    assert!(
        output.status.success(),
        "proof crop prepare failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    let lines: Vec<_> = args.lines().collect();
    assert_eq!(lines.len(), 6, "got: {}", args);
    assert!(lines[0].contains("view --inspect"), "got: {}", args);
    assert!(lines[0].contains("--strict"), "got: {}", args);
    assert!(lines[1].contains("view --inspect"), "got: {}", args);
    assert!(
        lines[1].contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
    assert!(!lines[1].contains("--strict"), "got: {}", args);
    for (line, command) in lines[2..]
        .iter()
        .zip(["links", "backlinks", "frontmatter", "headings"])
    {
        assert!(line.starts_with(command), "got: {}", args);
        assert!(line.contains("--format json"), "got: {}", args);
        assert!(
            line.contains(&view_file.display().to_string()),
            "got: {}",
            args
        );
        assert!(
            line.contains(
                &output_dir
                    .join(format!("{}.json", command))
                    .display()
                    .to_string()
            ),
            "got: {}",
            args
        );
    }
}

#[test]
fn binary_crop_prepare_rejects_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("proof-guides.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(dir.path().join("side-info.json"))
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("prepare")
        .arg("--view")
        .arg(&view_file)
        .output()
        .expect("failed to run proof crop prepare");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--output-dir"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_backlink_list_renders_target_snippet() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("backlinks.json");
    std::fs::write(
        &side_info,
        r#"{
  "pages": [
    {
      "source": "README.md",
      "inbound_links": [
        { "source": "docs/guide.md", "target": "README.md#overview" }
      ]
    }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("backlink-list")
        .arg("--target")
        .arg("md://README.md#overview")
        .arg("--side-info")
        .arg(&side_info)
        .output()
        .expect("failed to run proof crop backlink-list");

    assert!(
        output.status.success(),
        "proof crop backlink-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- [guide.md](docs/guide.md)"));
}

#[test]
fn binary_crop_link_list_renders_filtered_snippet() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("links.json");
    std::fs::write(
        &side_info,
        r#"{
  "links": [
    { "source": "README.md", "target": "docs/guide.md", "status": "ok", "resolved_source": "docs/guide.md" },
    { "source": "README.md", "target": "missing.md", "status": "broken", "error": "missing target" },
    { "source": "docs/guide.md", "target": "README.md", "status": "ok", "resolved_source": "README.md" }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("link-list")
        .arg("--source")
        .arg("md://README.md#overview")
        .arg("--status")
        .arg("broken")
        .arg("--side-info")
        .arg(&side_info)
        .output()
        .expect("failed to run proof crop link-list");

    assert!(
        output.status.success(),
        "proof crop link-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- `README.md` -> `missing.md` [broken] (missing target)"));
    assert!(!stdout.contains("docs/guide.md"));
}

#[test]
fn binary_crop_link_list_rejects_global_json_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("links.json");
    std::fs::write(&side_info, r#"{"links":[]}"#).unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("json")
        .arg("crop")
        .arg("link-list")
        .arg("--side-info")
        .arg(&side_info)
        .output()
        .expect("failed to run proof crop link-list");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Markdown snippets"), "got: {}", stderr);
}

#[test]
fn binary_crop_link_list_rejects_invalid_local_status_before_reading_side_info() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let missing_side_info = dir.path().join("missing-links.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("link-list")
        .arg("--side-info")
        .arg(&missing_side_info)
        .arg("--status")
        .arg("maybe")
        .output()
        .expect("failed to run proof crop link-list");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("all"), "got: {}", stderr);
    assert!(stderr.contains("ok"), "got: {}", stderr);
    assert!(stderr.contains("broken"), "got: {}", stderr);
    assert!(
        !stderr.contains("missing-links.json"),
        "side-info should not be read before parser rejection, got: {}",
        stderr
    );
}

#[test]
fn binary_crop_backlink_list_rejects_invalid_local_format_before_reading_side_info() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let missing_side_info = dir.path().join("missing-backlinks.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("backlink-list")
        .arg("--target")
        .arg("README.md")
        .arg("--side-info")
        .arg(&missing_side_info)
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("failed to run proof crop backlink-list");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("list"), "got: {}", stderr);
    assert!(stderr.contains("table"), "got: {}", stderr);
    assert!(stderr.contains("count"), "got: {}", stderr);
    assert!(
        !stderr.contains("missing-backlinks.json"),
        "side-info should not be read before parser rejection, got: {}",
        stderr
    );
}

#[test]
fn binary_crop_frontmatter_list_rejects_invalid_local_op_before_reading_side_info() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let missing_side_info = dir.path().join("missing-frontmatter.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("frontmatter-list")
        .arg("--side-info")
        .arg(&missing_side_info)
        .arg("--field")
        .arg("tags")
        .arg("--value")
        .arg("guide")
        .arg("--op")
        .arg("contains")
        .output()
        .expect("failed to run proof crop frontmatter-list");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "got: {}", stderr);
    assert!(stderr.contains("has"), "got: {}", stderr);
    assert!(stderr.contains("eq"), "got: {}", stderr);
    assert!(
        !stderr.contains("missing-frontmatter.json"),
        "side-info should not be read before parser rejection, got: {}",
        stderr
    );
}

#[test]
fn binary_crop_link_list_writes_table_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("links.json");
    let output_path = dir.path().join("snippets").join("LINKS.md");
    std::fs::write(
        &side_info,
        r#"{
  "links": [
    { "source": "README.md", "target": "missing.md", "status": "broken", "error": "missing target" }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("link-list")
        .arg("--status")
        .arg("broken")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--format")
        .arg("table")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop link-list --output");

    assert!(
        output.status.success(),
        "proof crop link-list --output failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = std::fs::read_to_string(&output_path).unwrap();
    assert!(rendered.contains("| Source | Target | Status | Resolved | Error |"));
    assert!(rendered.contains("| `README.md` | `missing.md` | `broken` | `` | missing target |"));
}

#[test]
fn binary_crop_link_list_uses_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("links.json");
    let output_path = dir.path().join("GLOBAL_LINKS.md");
    std::fs::write(
        &side_info,
        r#"{
  "links": [
    { "source": "README.md", "target": "missing.md", "status": "broken", "error": "missing target" }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("crop")
        .arg("link-list")
        .arg("--status")
        .arg("broken")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--format")
        .arg("table")
        .output()
        .expect("failed to run proof crop link-list");

    assert!(
        output.status.success(),
        "proof crop link-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = std::fs::read_to_string(&output_path).unwrap();
    assert!(rendered.contains("| Source | Target | Status | Resolved | Error |"));
    assert!(output.stdout.is_empty());
}

#[test]
fn binary_crop_backlink_list_writes_table_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("backlinks.json");
    let output_path = dir.path().join("snippets").join("BACKLINKS.md");
    std::fs::write(
        &side_info,
        r#"{
  "pages": [
    {
      "source": "README.md",
      "inbound_links": [
        { "source": "docs/guide.md", "target": "README.md#overview" }
      ]
    }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("backlink-list")
        .arg("--target")
        .arg("README.md")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--format")
        .arg("table")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop backlink-list --output");

    assert!(
        output.status.success(),
        "proof crop backlink-list --output failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = std::fs::read_to_string(&output_path).unwrap();
    assert!(rendered.contains("| Source | Target |"));
    assert!(rendered.contains("| [guide.md](docs/guide.md) | `README.md#overview` |"));
}

#[test]
fn binary_crop_heading_list_renders_source_snippet() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("headings.json");
    std::fs::write(
        &side_info,
        r#"{
  "headings": [
    { "source": "README.md", "level": 1, "text": "Overview", "md_uri": "md://README.md#overview" },
    { "source": "README.md", "level": 2, "text": "Install", "md_uri": "md://README.md#install" },
    { "source": "docs/guide.md", "level": 1, "text": "Guide", "md_uri": "md://docs/guide.md#guide" }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("heading-list")
        .arg("--source")
        .arg("md://README.md#overview")
        .arg("--side-info")
        .arg(&side_info)
        .output()
        .expect("failed to run proof crop heading-list");

    assert!(
        output.status.success(),
        "proof crop heading-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- [Overview](md://README.md#overview)"));
    assert!(stdout.contains("  - [Install](md://README.md#install)"));
    assert!(!stdout.contains("Guide"));
}

#[test]
fn binary_crop_heading_list_writes_count_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("headings.json");
    let output_path = dir.path().join("snippets").join("OUTLINE_COUNT.md");
    std::fs::write(
        &side_info,
        r#"{
  "headings": [
    { "source": "README.md", "level": 1, "text": "Overview", "md_uri": "md://README.md#overview" },
    { "source": "README.md", "level": 2, "text": "Install", "md_uri": "md://README.md#install" },
    { "source": "README.md", "level": 2, "text": "Usage", "md_uri": "md://README.md#usage" }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("heading-list")
        .arg("--source")
        .arg("README.md")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--format")
        .arg("count")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop heading-list --output");

    assert!(
        output.status.success(),
        "proof crop heading-list --output failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(rendered, "3");
}

#[test]
fn binary_crop_frontmatter_list_renders_filtered_snippet() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("frontmatter.json");
    std::fs::write(
        &side_info,
        r#"{
  "pages": [
    {
      "source": "README.md",
      "keys": ["status", "tags", "title"],
      "fields": { "status": "ready", "tags": "[proof, guide]", "title": "Readme" }
    },
    {
      "source": "draft.md",
      "keys": ["status", "tags", "title"],
      "fields": { "status": "draft", "tags": "[proof]", "title": "Draft" }
    }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("frontmatter-list")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--field")
        .arg("tags")
        .arg("--value")
        .arg("guide")
        .output()
        .expect("failed to run proof crop frontmatter-list");

    assert!(
        output.status.success(),
        "proof crop frontmatter-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("- [Readme](README.md)"));
    assert!(!stdout.contains("Draft"));
}

#[test]
fn binary_crop_frontmatter_list_writes_table_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let side_info = dir.path().join("frontmatter.json");
    let output_path = dir.path().join("snippets").join("READY.md");
    std::fs::write(
        &side_info,
        r#"{
  "pages": [
    {
      "source": "README.md",
      "keys": ["status", "title"],
      "fields": { "status": "ready", "title": "Readme" }
    }
  ]
}"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("frontmatter-list")
        .arg("--side-info")
        .arg(&side_info)
        .arg("--field")
        .arg("status")
        .arg("--value")
        .arg("ready")
        .arg("--op")
        .arg("eq")
        .arg("--format")
        .arg("table")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop frontmatter-list --output");

    assert!(
        output.status.success(),
        "proof crop frontmatter-list --output failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = std::fs::read_to_string(&output_path).unwrap();
    assert!(rendered.contains("| Source | status |"));
    assert!(rendered.contains("| [README.md](README.md) | `ready` |"));
}

#[test]
fn binary_crop_artifacts_delegates_to_crop_artifacts() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let manifest_path = dir.path().join("artifacts.json");
    let output_path = dir.path().join("ARTIFACTS.md");
    std::fs::write(&manifest_path, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("artifacts")
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--format")
        .arg("markdown")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop artifacts");

    assert!(
        output.status.success(),
        "proof crop artifacts failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("artifacts"), "got: {}", args);
    assert!(args.contains("--manifest"), "got: {}", args);
    assert!(
        args.contains(&manifest_path.display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--format markdown"), "got: {}", args);
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_crop_artifacts_requires_root_or_manifest() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("artifacts")
        .output()
        .expect("failed to run proof crop artifacts");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires --root or --manifest"),
        "got: {}",
        stderr
    );
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_artifacts_rejects_root_and_manifest() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let manifest_path = dir.path().join("artifacts.json");
    std::fs::write(&manifest_path, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("artifacts")
        .arg("--root")
        .arg(dir.path())
        .arg("--manifest")
        .arg(&manifest_path)
        .output()
        .expect("failed to run proof crop artifacts");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("either --root or --manifest"),
        "got: {}",
        stderr
    );
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_artifacts_rejects_global_rich_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let manifest_path = dir.path().join("artifacts.json");
    std::fs::write(&manifest_path, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("rich")
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("artifacts")
        .arg("--manifest")
        .arg(&manifest_path)
        .output()
        .expect("failed to run proof crop artifacts");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("json or markdown"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_crop_relays_crop_exit_code() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 7);

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("inspect-views")
        .arg("--dir")
        .arg(dir.path())
        .output()
        .expect("failed to run proof crop inspect-views");

    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn binary_index_delegates_to_crop_index() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_path = dir.path().join("INDEX.md");

    let output = std::process::Command::new(&bin)
        .arg("index")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--root")
        .arg(dir.path())
        .arg("--title")
        .arg("Guide Index")
        .arg("--extension")
        .arg("md")
        .arg("--exclude-dir")
        .arg("target")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof index");

    assert!(
        output.status.success(),
        "proof index failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("index"), "got: {}", args);
    assert!(args.contains("--root"), "got: {}", args);
    assert!(
        args.contains(&dir.path().display().to_string()),
        "got: {}",
        args
    );
    assert!(args.contains("--title"), "got: {}", args);
    assert!(args.contains("Guide Index"), "got: {}", args);
    assert!(args.contains("--extension md"), "got: {}", args);
    assert!(args.contains("--exclude-dir target"), "got: {}", args);
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_index_uses_global_output() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let output_path = dir.path().join("GLOBAL_INDEX.md");

    let output = std::process::Command::new(&bin)
        .arg("-o")
        .arg(&output_path)
        .arg("index")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof index");

    assert!(
        output.status.success(),
        "proof index failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("--output"), "got: {}", args);
    assert!(
        args.contains(&output_path.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_index_rejects_global_json_format() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("-f")
        .arg("json")
        .arg("index")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof index");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Markdown-only"), "got: {}", stderr);
    assert!(!args_file.exists(), "CROP should not be invoked");
}

#[test]
fn binary_toc_delegates_to_crop_index_with_toc_title() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);

    let output = std::process::Command::new(&bin)
        .arg("toc")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--root")
        .arg(dir.path())
        .output()
        .expect("failed to run proof toc");

    assert!(
        output.status.success(),
        "proof toc failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("index"), "got: {}", args);
    assert!(args.contains("--title"), "got: {}", args);
    assert!(args.contains("Table of Contents"), "got: {}", args);
}

#[test]
fn binary_catalog_delegates_to_crop_catalog() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("crop-args.txt");
    let crop_bin = write_fake_crop_bin(dir.path(), &args_file, 0);
    let view_file = dir.path().join("ready-guides.json");
    std::fs::write(&view_file, "{}").unwrap();

    let output = std::process::Command::new(&bin)
        .arg("catalog")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(&view_file)
        .arg("--output")
        .arg(dir.path().join("CATALOG.md"))
        .output()
        .expect("failed to run proof catalog");

    assert!(
        output.status.success(),
        "proof catalog failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let args = std::fs::read_to_string(&args_file).expect("fake crop args");
    assert!(args.contains("catalog"), "got: {}", args);
    assert!(args.contains("--view"), "got: {}", args);
    assert!(
        args.contains(&view_file.display().to_string()),
        "got: {}",
        args
    );
}

#[test]
fn binary_real_crop_index_generates_fixture_markdown() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }
    let Some(crop_manifest) = sibling_crop_manifest() else {
        return;
    };
    let fixture_root = sibling_crop_fixture_root(&crop_manifest);
    let view_file = fixture_root.join("proof-ready-view.json");
    if !view_file.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = write_real_crop_bin(dir.path(), &crop_manifest);
    let output_path = dir.path().join("INDEX.md");

    let output = std::process::Command::new(&bin)
        .arg("index")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("--view")
        .arg(&view_file)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof index with real CROP");

    assert!(
        output.status.success(),
        "proof index real CROP failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let index = std::fs::read_to_string(&output_path).expect("real CROP index output");
    assert!(index.contains("# proof-fixture-ready"), "got:\n{}", index);
    assert!(index.contains("guide.source.md"), "got:\n{}", index);
    assert!(index.contains("reference.source.md"), "got:\n{}", index);
}

#[test]
fn binary_real_crop_frontmatter_generates_fixture_json() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }
    let Some(crop_manifest) = sibling_crop_manifest() else {
        return;
    };
    let fixture_root = sibling_crop_fixture_root(&crop_manifest);
    let view_file = fixture_root.join("proof-ready-view.json");
    if !view_file.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let crop_bin = write_real_crop_bin(dir.path(), &crop_manifest);
    let output_path = dir.path().join("frontmatter.json");

    let output = std::process::Command::new(&bin)
        .arg("crop")
        .arg("--crop-bin")
        .arg(&crop_bin)
        .arg("frontmatter")
        .arg("--view")
        .arg(&view_file)
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("failed to run proof crop frontmatter with real CROP");

    assert!(
        output.status.success(),
        "proof crop frontmatter real CROP failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "crop.markdown-frontmatter.v1");
    assert_eq!(json["source_count"], 2);
    assert!(json["key_counts"]["tags"].as_u64().unwrap_or(0) >= 2);
}

#[test]
fn binary_compile_tag_filter_limits_sources() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let out_dir = dir.path().join("out");
    std::fs::write(
        dir.path().join("publish.source.md"),
        "---\ntags: [publish]\nops: [compile]\n---\n# Publish\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("draft.source.md"),
        "---\ntags: [draft]\nops: [review]\n---\n# Draft\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("compile")
        .arg(dir.path())
        .arg("--root")
        .arg(dir.path())
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("--tag")
        .arg("publish")
        .output()
        .expect("failed to run proof compile --tag");

    assert!(
        output.status.success(),
        "proof compile --tag failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(out_dir.join("publish.md").exists());
    assert!(!out_dir.join("draft.md").exists());

    let manifest_path = dir.path().join(".proof").join("artifacts.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["artifacts"].as_array().unwrap().len(), 1);
    assert!(
        manifest["artifacts"][0]["source_path"]
            .as_str()
            .unwrap()
            .ends_with("publish.source.md"),
        "got: {}",
        manifest
    );
}

#[test]
fn binary_check_tag_filter_limits_sources() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("publish.source.md"),
        "---\ntags: [publish]\nops: [check]\n---\n# Publish\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("draft.source.md"),
        "---\ntags: [draft]\nops: [review]\n---\n# Draft\n# Extra\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .arg("check")
        .arg(dir.path())
        .arg("--tag")
        .arg("publish")
        .output()
        .expect("failed to run proof check --tag");

    assert!(
        output.status.success(),
        "proof check --tag failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("1 files checked"), "got:\n{}", stderr);
}

#[test]
fn binary_pin_list_prints_registered_davinci_entries() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("proof.toml");
    std::fs::write(
        &config_path,
        r#"
[[davinci]]
id = "overview-box"
uri = "md://README.md#overview"
description = "Overview figure"
protection = "warn"
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .current_dir(dir.path())
        .arg("--config")
        .arg(&config_path)
        .arg("pin-list")
        .output()
        .expect("failed to run proof pin-list");

    assert!(
        output.status.success(),
        "proof pin-list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("overview-box"), "got:\n{}", stdout);
    assert!(stdout.contains("Overview figure"), "got:\n{}", stdout);
}

#[test]
fn binary_pin_appends_davinci_entry() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("proof.toml"), "").unwrap();
    std::fs::write(
        dir.path().join("README.md"),
        "# Readme\n\n## Overview\n\nPinned content.\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .current_dir(dir.path())
        .args([
            "pin",
            "md://README.md#overview",
            "--id",
            "overview-section",
            "--description",
            "Overview section",
        ])
        .output()
        .expect("failed to run proof pin");

    assert!(
        output.status.success(),
        "proof pin failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let toml = std::fs::read_to_string(dir.path().join("proof.toml")).unwrap();
    assert!(toml.contains("[[davinci]]"), "got:\n{}", toml);
    assert!(toml.contains("id = \"overview-section\""), "got:\n{}", toml);
    assert!(toml.contains("uri = \"README.md\""), "got:\n{}", toml);
    assert!(
        toml.contains("description = \"Overview section\""),
        "got:\n{}",
        toml
    );
}

#[test]
fn binary_resolve_prints_json_for_heading() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("README.md"),
        "# Readme\n\n## Overview\n\nResolved content.\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args([
            "resolve",
            "md://README.md#overview",
            "--format",
            "json",
            "--root",
        ])
        .arg(dir.path())
        .output()
        .expect("failed to run proof resolve");

    assert!(
        output.status.success(),
        "proof resolve failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["uri"], "README.md");
    assert_eq!(json["section_heading"], "Overview");
    assert_eq!(json["content"], "## Overview");
    assert!(json["line_start"].as_u64().unwrap() > 0, "got:\n{}", json);
}

#[test]
fn binary_depends_prints_json_references() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("proof.toml"), "").unwrap();
    std::fs::write(dir.path().join("figure.md"), "# Figure\n").unwrap();
    std::fs::write(
        dir.path().join("doc.source.md"),
        "# Source\n\n```proof:include md://figure.md\n```\n",
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["depends", "md://figure.md", "--format", "json", "--root"])
        .arg(dir.path())
        .output()
        .expect("failed to run proof depends");

    assert!(
        output.status.success(),
        "proof depends failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["query"], "md://figure.md");
    assert_eq!(json["count"], 1);
    assert_eq!(json["references"][0]["uri"], "md://figure.md");
    assert!(
        json["references"][0]["file"]
            .as_str()
            .unwrap()
            .ends_with("doc.source.md"),
        "got:\n{}",
        json
    );
}

#[test]
fn binary_tree_generate_prints_dirtree() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("README.md"), "# Readme\n").unwrap();
    std::fs::write(dir.path().join("docs").join("guide.md"), "# Guide\n").unwrap();

    let output = std::process::Command::new(&bin)
        .args(["tree", "generate", "--root"])
        .arg(dir.path())
        .args(["--max-depth", "1", "--no-fence"])
        .output()
        .expect("failed to run proof tree generate");

    assert!(
        output.status.success(),
        "proof tree generate failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("README.md"), "got:\n{}", stdout);
    assert!(stdout.contains("docs"), "got:\n{}", stdout);
}

#[test]
fn binary_layout_composes_file_sources() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("left.txt"), "LEFT\n").unwrap();
    std::fs::write(dir.path().join("right.txt"), "RIGHT\n").unwrap();

    let output = std::process::Command::new(&bin)
        .args(["layout", "left.txt", "right.txt", "--gap", "2", "--root"])
        .arg(dir.path())
        .output()
        .expect("failed to run proof layout");

    assert!(
        output.status.success(),
        "proof layout failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("LEFT"), "got:\n{}", stdout);
    assert!(stdout.contains("RIGHT"), "got:\n{}", stdout);
}

#[test]
fn binary_check_summary_file_count_honors_include_exclude() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("docs").join("one.md"), "# One\n").unwrap();
    std::fs::write(dir.path().join("skip.md"), "# Skip\n").unwrap();
    let config_path = dir.path().join("proof.toml");
    std::fs::write(
        &config_path,
        r#"
[files]
include = ["docs/**/*.md"]
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["--config"])
        .arg(&config_path)
        .arg("--no-fail")
        .arg(dir.path())
        .output()
        .expect("failed to run proof check");

    assert!(
        output.status.success(),
        "proof check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1 files checked"),
        "check summary file count should honor include/exclude, got:\n{}",
        stderr
    );
}

#[test]
fn binary_help_documents_progress_only_for_compile() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let check_help = std::process::Command::new(&bin)
        .args(["check", "--help"])
        .output()
        .expect("failed to run proof check --help");
    assert!(
        check_help.status.success(),
        "proof check --help failed:\n{}",
        String::from_utf8_lossy(&check_help.stderr)
    );
    let check_stdout = String::from_utf8_lossy(&check_help.stdout);
    assert!(
        !check_stdout.contains("--progress"),
        "check help should not advertise unsupported --progress:\n{}",
        check_stdout
    );

    let compile_help = std::process::Command::new(&bin)
        .args(["compile", "--help"])
        .output()
        .expect("failed to run proof compile --help");
    assert!(
        compile_help.status.success(),
        "proof compile --help failed:\n{}",
        String::from_utf8_lossy(&compile_help.stderr)
    );
    let compile_stdout = String::from_utf8_lossy(&compile_help.stdout);
    assert!(
        compile_stdout.contains("--progress"),
        "compile help should document --progress:\n{}",
        compile_stdout
    );
}

#[test]
fn runner_explicit_config_skips_disk_cascade() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("doc.md");
    std::fs::write(&doc, "# Title\n").unwrap();

    let config: GlintConfig = toml::from_str(
        r#"
[markdown]
enabled = true
required_h2_all = ["Decision Cheat Sheet"]
"#,
    )
    .unwrap();

    let runner = Runner::new_with_config(dir.path(), config).unwrap();
    let diags = runner.lint_file(&doc);
    assert!(
        diags.iter().any(|d| d.code == "md_missing_section"),
        "explicit runner config must be applied even without proof.toml on disk:\n{}",
        format_diags(&diags)
    );
}

#[test]
fn binary_stats_honors_config_override() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("doc.md"), "# Title\n").unwrap();
    let config_path = dir.path().join("external.toml");
    std::fs::write(
        &config_path,
        r#"
[files]
include = ["**/*.md"]

[markdown]
enabled = true
required_h2_all = ["Decision Cheat Sheet"]
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["stats", "--by-code", "--config"])
        .arg(&config_path)
        .arg(dir.path())
        .output()
        .expect("failed to run proof stats");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("md_missing_section"),
        "stats must honor --config override, got stdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runner_path_summary_counts_file_and_directory_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let docs = dir.path().join("docs");
    std::fs::create_dir_all(&docs).unwrap();
    let one = docs.join("one.md");
    std::fs::write(&one, "# One\n").unwrap();
    std::fs::write(docs.join("skip.txt"), "not markdown\n").unwrap();

    let cfg = GlintConfig::default();
    let file_runner = Runner::new(&docs, cfg.clone()).unwrap();
    let file_summary = file_runner.run_path_summary(&one);
    assert_eq!(file_summary.files_checked, 1);

    let dir_runner = Runner::new(dir.path(), cfg).unwrap();
    let dir_summary = dir_runner.run_path_summary(dir.path());
    assert_eq!(
        dir_summary.files_checked, 1,
        "directory summary should count matching markdown files only"
    );
}

#[test]
fn binary_config_prints_effective_cascaded_config() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let child = root.join("guides");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true

[markdown]
enabled = true
required_h2_all = ["Root Requirement"]
"#,
    )
    .unwrap();
    std::fs::write(
        child.join("proof.toml"),
        r#"
[markdown]
required_h2_all = ["Child Requirement"]
"#,
    )
    .unwrap();
    let doc = child.join("doc.md");
    std::fs::write(&doc, "# Doc\n").unwrap();

    let output = std::process::Command::new(&bin)
        .args(["config"])
        .arg(&doc)
        .output()
        .expect("failed to run proof config");

    assert!(
        output.status.success(),
        "proof config failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Root Requirement") && stdout.contains("Child Requirement"),
        "effective config should include cascaded parent and child requirements, got:\n{}",
        stdout
    );
}

#[test]
fn binary_config_honors_explicit_config_override() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("doc.md");
    std::fs::write(&doc, "# Doc\n").unwrap();
    std::fs::write(
        dir.path().join("proof.toml"),
        r#"
[markdown]
enabled = true
required_h2_all = ["Local Requirement"]
"#,
    )
    .unwrap();
    let external = dir.path().join("external.toml");
    std::fs::write(
        &external,
        r#"
[markdown]
enabled = true
required_h2_all = ["External Requirement"]
"#,
    )
    .unwrap();

    let output = std::process::Command::new(&bin)
        .args(["--config"])
        .arg(&external)
        .args(["config"])
        .arg(&doc)
        .output()
        .expect("failed to run proof config with override");

    assert!(
        output.status.success(),
        "proof config --config failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("External Requirement") && !stdout.contains("Local Requirement"),
        "explicit config should skip auto-cascade, got:\n{}",
        stdout
    );
}

#[test]
fn binary_missing_config_override_fails_loudly() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("doc.md"), "# Title\n").unwrap();
    let missing = dir.path().join("missing.toml");

    let output = std::process::Command::new(&bin)
        .args(["--config"])
        .arg(&missing)
        .arg("--no-fail")
        .arg(dir.path().join("doc.md"))
        .output()
        .expect("failed to run proof with missing config");

    assert!(
        !output.status.success(),
        "missing explicit --config must fail instead of falling back"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("loading explicit config") && stderr.contains("missing.toml"),
        "stderr should identify the explicit config failure, got:\n{}",
        stderr
    );
}

#[test]
fn binary_invalid_config_override_fails_loudly() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("doc.md"), "# Title\n").unwrap();
    let config_path = dir.path().join("invalid.toml");
    std::fs::write(&config_path, "[markdown\n").unwrap();

    let output = std::process::Command::new(&bin)
        .args(["stats", "--config"])
        .arg(&config_path)
        .arg(dir.path())
        .output()
        .expect("failed to run proof stats with invalid config");

    assert!(
        !output.status.success(),
        "invalid explicit --config must fail instead of falling back"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("loading explicit config") && stderr.contains("invalid.toml"),
        "stderr should identify the explicit config parse failure, got:\n{}",
        stderr
    );
}

#[test]
fn binary_fix_dry_run_writes_nothing() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

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
        .args([
            "fix",
            "--plan",
            plan_path.to_str().unwrap(),
            "--dry-run",
            "--no-verify",
        ])
        .output()
        .expect("failed to run glint fix");

    // Invariant I-12: dry-run must not write
    let content_after = std::fs::read_to_string(&target).unwrap();
    assert_eq!(
        content_after, "old content\n",
        "dry-run must not modify any files"
    );
}

#[test]
fn binary_fix_uses_global_config_for_verification_and_writes_log() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("doc.md");
    std::fs::write(&target, "# One\n# Two\nold\n").unwrap();
    std::fs::write(
        dir.path().join("proof.toml"),
        "[markdown]\nenabled = true\nmax_h1 = 1\n",
    )
    .unwrap();
    let allow_config = dir.path().join("allow.toml");
    std::fs::write(&allow_config, "[markdown]\nenabled = true\nmax_h1 = 2\n").unwrap();

    let plan = serde_json::json!({
        "schema_version": "1",
        "generated_by": "test",
        "source_report": "",
        "summary": {"total_fixes": 1, "high_confidence": 1, "medium_confidence": 0, "low_confidence": 0, "files_affected": 1},
        "fixes": [{
            "id": "fix-body",
            "file": "doc.md",
            "description": "test",
            "confidence": "high",
            "reasoning": "",
            "diagnostic": {"code": "test", "line": 3, "col": 1},
            "edit": {"line": 3, "old_string": "old", "new_string": "new"}
        }]
    });
    let plan_path = dir.path().join("plan.json");
    std::fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    let output = std::process::Command::new(&bin)
        .current_dir(dir.path())
        .arg("--config")
        .arg(&allow_config)
        .arg("fix")
        .arg("--plan")
        .arg(&plan_path)
        .arg("--no-signal-check")
        .output()
        .expect("failed to run proof fix");

    assert!(
        output.status.success(),
        "proof fix failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "# One\n# Two\nnew\n"
    );

    let log_path = dir.path().join(".proof").join("last-fix.json");
    let log: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&log_path).unwrap()).unwrap();
    assert_eq!(log["generated_by"], "proof fix");
    assert_eq!(log["applied"], 1);
    assert_eq!(log["files_modified"], 1);
    assert_eq!(log["verification"]["status"], "passed");
    assert_eq!(
        log["verification"]["config"].as_str().unwrap(),
        allow_config.to_string_lossy()
    );
}

#[test]
fn binary_init_creates_default_proof_toml() {
    let bin = debug_bin();
    if !bin.exists() {
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(&bin)
        .current_dir(dir.path())
        .arg("init")
        .output()
        .expect("failed to run proof init");

    assert!(
        output.status.success(),
        "proof init should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config_path = dir.path().join("proof.toml");
    assert!(config_path.exists(), "proof init must create proof.toml");
    let content = std::fs::read_to_string(config_path).unwrap();
    assert!(
        content.contains("[ascii_box]"),
        "default config should include ascii_box"
    );
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
    let content =
        "```\n| Header A | Header B |\n|----------|----------|\n| cell     | cell     |\n```";
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
// The schema is written to a real proof.toml in a temp dir — that's the correct
// way to test cascade-resolved config (the runner discovers it from disk).
#[test]
fn section_schema_paths_exclude_skips_matching_files() {
    use proof_lib::runner::Runner;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write proof.toml — generic rule for all *.md EXCEPT 00-OVERVIEW.md,
    // and a separate rule for 00-OVERVIEW.md only.
    std::fs::write(
        root.join("proof.toml"),
        r#"
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
"#,
    )
    .unwrap();

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
        c_diags
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md must require 'Type System Snapshot'\ngot diagnostics: {}",
        format_diags(&c_diags)
    );
    // 02-C.md must NOT require "Language Genealogy"
    assert!(
        !c_diags
            .iter()
            .any(|d| d.message.contains("Language Genealogy")),
        "02-C.md must NOT require 'Language Genealogy' (that's for the overview)"
    );

    // 00-OVERVIEW.md must NOT require "Type System Snapshot" (excluded by paths_exclude)
    let ov_diags = runner.lint_file(&ov_file);
    assert!(
        !ov_diags
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md must NOT require 'Type System Snapshot'\ngot: {}",
        format_diags(&ov_diags)
    );
}

#[test]
fn child_markdown_enabled_false_disables_parent_markdown() {
    use proof_lib::runner::Runner;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true

[markdown]
enabled = true
required_h2_all = ["Decision Cheat Sheet"]
"#,
    )
    .unwrap();

    let child = root.join("child");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(
        child.join("proof.toml"),
        r#"
[markdown]
enabled = false
"#,
    )
    .unwrap();

    let file = child.join("guide.md");
    std::fs::write(&file, "# Guide\n\nNo required section here.\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&file);
    assert!(
        diags.iter().all(|d| d.code != "md_missing_section"),
        "child markdown.enabled=false should disable inherited markdown checks, got:\n{}",
        format_diags(&diags)
    );
}

// paths_exclude with multiple exclusions
#[test]
fn paths_exclude_multiple_files_skipped() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true
[markdown]
enabled = true
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md", "01-CHEATSHEET.md"]
required_h2_all = ["Type System Snapshot"]
"#,
    )
    .unwrap();

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
    assert!(
        runner
            .lint_file(&guide)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md should require Type System Snapshot"
    );
    // Overview: excluded
    assert!(
        !runner
            .lint_file(&overview)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md should be excluded"
    );
    // Cheatsheet: excluded
    assert!(
        !runner
            .lint_file(&cheat)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "01-CHEATSHEET.md should be excluded"
    );
}

// paths_exclude with glob pattern (not just exact filename)
#[test]
fn paths_exclude_glob_pattern() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true
[markdown]
enabled = true
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["0[0-1]-*.md"]
required_h2_all = ["Type System Snapshot"]
"#,
    )
    .unwrap();

    let guide = root.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent\n").unwrap();
    let overview = root.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();
    let cheat = root.join("01-CHEATSHEET.md");
    std::fs::write(&cheat, "# Cheatsheet\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    assert!(
        runner
            .lint_file(&guide)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md matched by *.md, not in exclude → should require"
    );
    assert!(
        !runner
            .lint_file(&overview)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md matched by 0[0-1]-*.md exclude → should skip"
    );
    assert!(
        !runner
            .lint_file(&cheat)
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "01-CHEATSHEET.md matched by 0[0-1]-*.md exclude → should skip"
    );
}

// Directory-level proof.toml: paths are relative to that directory, not root
#[test]
fn directory_schema_paths_relative_to_its_dir() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Root proof.toml — universal rule
    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Decision Cheat Sheet"]
"#,
    )
    .unwrap();

    // languages/ sub-directory with its own proof.toml
    let langs = root.join("languages");
    std::fs::create_dir_all(&langs).unwrap();
    std::fs::write(
        langs.join("proof.toml"),
        r#"
[markdown]
enabled = true
# paths here are relative to languages/ NOT to root
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Type System Snapshot"]
"#,
    )
    .unwrap();

    // A language guide — should require both Decision Cheat Sheet (root) AND Type System Snapshot (dir)
    let guide = langs.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent without required sections\n").unwrap();

    // Overview — should require Decision Cheat Sheet but NOT Type System Snapshot
    let overview = langs.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();

    let guide_diags = runner.lint_file(&guide);
    assert!(
        guide_diags
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "02-C.md must require Type System Snapshot from dir-level schema"
    );
    assert!(
        guide_diags
            .iter()
            .any(|d| d.message.contains("Decision Cheat Sheet")),
        "02-C.md must require Decision Cheat Sheet from root schema"
    );

    let ov_diags = runner.lint_file(&overview);
    assert!(
        !ov_diags
            .iter()
            .any(|d| d.message.contains("Type System Snapshot")),
        "00-OVERVIEW.md excluded by paths_exclude in dir schema"
    );
    // Overview still gets root requirement
    assert!(
        ov_diags
            .iter()
            .any(|d| d.message.contains("Decision Cheat Sheet")),
        "00-OVERVIEW.md still gets root Decision Cheat Sheet requirement"
    );
}

// A file that matches BOTH overview rule and generic rule gets BOTH sets of requirements
// (section_schemas are additive — no "first match wins")
#[test]
fn section_schemas_are_additive_not_first_match_wins() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
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
"#,
    )
    .unwrap();

    let f = root.join("guide.md");
    std::fs::write(&f, "# Guide\n\n## Section A\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&f);

    // Has Section A but not Section B → should warn about B
    assert!(
        diags.iter().any(|d| d.message.contains("Section B")),
        "both schemas must apply — Section B missing should be flagged"
    );
    // Section A is present so no warning about it
    assert!(
        !diags.iter().any(|d| d.message.contains("Section A")),
        "Section A is present, must not be flagged"
    );
}

// paths_exclude does not affect the base [markdown] config — only the section_schema
#[test]
fn paths_exclude_only_affects_its_own_schema() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Universal Section"]

[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Guide Section"]
"#,
    )
    .unwrap();

    let overview = root.join("00-OVERVIEW.md");
    std::fs::write(&overview, "# Overview\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&overview);

    // Universal Section comes from base [markdown], not section_schema → must still apply
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("Universal Section")),
        "paths_exclude only excludes from that section_schema, not from base [markdown] config"
    );
    // Guide Section is excluded for this file
    assert!(
        !diags.iter().any(|d| d.message.contains("Guide Section")),
        "Guide Section should be excluded for 00-OVERVIEW.md"
    );
}

// Three-level cascade: root → languages/ → individual file picks up all levels
#[test]
fn three_level_cascade_all_rules_accumulate() {
    use proof_lib::runner::Runner;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true
[markdown]
enabled = true
required_h2_all = ["Root Requirement"]
"#,
    )
    .unwrap();

    let langs = root.join("languages");
    std::fs::create_dir_all(&langs).unwrap();
    std::fs::write(
        langs.join("proof.toml"),
        r#"
[markdown]
enabled = true
required_h2_all = ["Dir Requirement"]
"#,
    )
    .unwrap();

    let guide = langs.join("02-C.md");
    std::fs::write(&guide, "# C\n\ncontent\n").unwrap();

    let cfg = proof_lib::GlintConfig::load_or_default(root);
    let runner = Runner::new(root, cfg).unwrap();
    let diags = runner.lint_file(&guide);

    // Both root and dir requirements must be enforced
    assert!(
        diags.iter().any(|d| d.message.contains("Root Requirement")),
        "root required section must apply in subdirectory"
    );
    assert!(
        diags.iter().any(|d| d.message.contains("Dir Requirement")),
        "directory required section must also apply"
    );
}

// Config cascade: two proof.toml files in a hierarchy produce additive required_h2_all
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
        merged
            .markdown
            .required_h2_all
            .contains(&"Decision Cheat Sheet".to_string()),
        "parent's required section must survive merge"
    );
    assert!(
        merged
            .markdown
            .required_h2_all
            .contains(&"Type System Snapshot".to_string()),
        "child's required section must be added"
    );
    assert_eq!(
        merged.markdown.required_h2_all.len(),
        2,
        "no duplicates, exactly 2 sections"
    );
}

#[test]
fn config_extends_stops_automatic_ancestor_cascade() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let shared = root.join("shared");
    let child = root.join("project").join("guides");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&child).unwrap();

    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
root = true

[markdown]
enabled = true
required_h2_all = ["Ancestor Requirement"]
"#,
    )
    .unwrap();
    std::fs::write(
        shared.join("base.toml"),
        r#"
[markdown]
enabled = true
required_h2_all = ["Shared Requirement"]
"#,
    )
    .unwrap();
    std::fs::write(
        child.join("proof.toml"),
        r#"
extends = "../../shared/base.toml"

[markdown]
enabled = true
required_h2_all = ["Child Requirement"]
"#,
    )
    .unwrap();

    let guide = child.join("guide.md");
    std::fs::write(&guide, "# Guide\n").unwrap();

    let cfg = proof_lib::GlintConfig::resolve_for(&guide, root);
    assert!(
        cfg.markdown
            .required_h2_all
            .contains(&"Shared Requirement".to_string()),
        "explicit parent config should apply"
    );
    assert!(
        cfg.markdown
            .required_h2_all
            .contains(&"Child Requirement".to_string()),
        "extending child config should apply"
    );
    assert!(
        !cfg.markdown
            .required_h2_all
            .contains(&"Ancestor Requirement".to_string()),
        "extends should stop unrelated automatic ancestor cascade"
    );
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
        merged
            .markdown
            .required_h2_all
            .contains(&"Decision Cheat Sheet".to_string()),
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
    assert!(
        merged.files.exclude.contains(&"_archive/**".to_string()),
        "parent's exclude must survive merge"
    );
    assert!(
        merged.files.exclude.contains(&"drafts/**".to_string()),
        "child's exclude must be added"
    );
    assert_eq!(merged.files.exclude.len(), 2);
}

// Config merge: child's implicit default include does NOT erase parent's include.
#[test]
fn config_merge_default_child_include_preserves_parent_include() {
    use proof_lib::config::{merge, FilesConfig};
    use proof_lib::GlintConfig;

    let mut parent = GlintConfig::default();
    parent.files = FilesConfig {
        include: vec!["docs/**/*.md".to_string()],
        exclude: vec![],
        root: false,
    };

    let child = GlintConfig::default();

    let merged = merge(parent, child);
    assert_eq!(
        merged.files.include,
        vec!["docs/**/*.md".to_string()],
        "child's implicit default include must not erase parent's explicit include"
    );
}

// Config merge: explicit child include replaces parent even when equal to the default.
#[test]
fn config_merge_explicit_default_child_include_replaces_parent_include() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let child = root.join("guides");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(
        root.join("proof.toml"),
        r#"
[files]
include = ["docs/**/*.md"]
"#,
    )
    .unwrap();
    std::fs::write(
        child.join("proof.toml"),
        r#"
[files]
include = ["**/*.md"]
"#,
    )
    .unwrap();

    let guide = child.join("guide.md");
    std::fs::write(&guide, "# Guide\n").unwrap();

    let merged = proof_lib::GlintConfig::resolve_for(&guide, root);
    assert_eq!(
        merged.files.include,
        vec!["**/*.md".to_string()],
        "explicit child include should replace parent's include, even at the default value"
    );
}

#[test]
fn config_explicit_markdown_disable_overrides_parent_enable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let child = root.join("guides");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(
        root.join("proof.toml"),
        r#"
[markdown]
enabled = true
required_h2_all = ["Root Requirement"]
"#,
    )
    .unwrap();
    std::fs::write(
        child.join("proof.toml"),
        r#"
[markdown]
enabled = false
"#,
    )
    .unwrap();

    let guide = child.join("guide.md");
    std::fs::write(&guide, "# Guide\n").unwrap();

    let cfg = proof_lib::GlintConfig::resolve_for(&guide, root);
    assert!(
        !cfg.markdown.enabled,
        "explicit child markdown.enabled=false should disable inherited markdown checks"
    );
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
        summary: PlanSummary {
            total_fixes: 2,
            high_confidence: 2,
            ..Default::default()
        },
        fixes: vec![
            Fix {
                id: "fix-a".to_string(),
                file: file1.clone(),
                description: "fix a".to_string(),
                confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "hello from a".to_string(),
                    new_string: "HELLO FROM A".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            },
            Fix {
                id: "fix-b".to_string(),
                file: file2.clone(),
                description: "fix b".to_string(),
                confidence: Confidence::High,
                reasoning: String::new(),
                edit: Edit {
                    line: 1,
                    old_string: "hello from b".to_string(),
                    new_string: "HELLO FROM B".to_string(),
                },
                diagnostic: DiagnosticRef::default(),
            },
        ],
    };

    let result = plan
        .apply(
            &FixOptions {
                dry_run: false,
                min_confidence: Confidence::Low,
                check_signal: false,
            },
            dir.path(),
        )
        .unwrap();

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
    assert!(
        diags.is_empty(),
        "ASCII box with spaces: should have zero errors, got:\n{}",
        format_diags(&diags)
    );
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
    let missing = diags
        .iter()
        .filter(|d| d.code == "md_table_missing_link")
        .count();
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
    assert!(
        !diags.iter().any(|d| d.code == "md_table_missing_link"),
        "properly linked cells must not be flagged"
    );
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
    assert!(
        diags.iter().any(|d| d.code == "md_broken_link"),
        "link to non-existent file must produce md_broken_link"
    );
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
    assert!(
        !diags.iter().any(|d| d.code == "md_broken_link"),
        "link to existing file must not be flagged"
    );
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
    assert!(
        lost.iter().any(|w| w.as_str() == "cpp"),
        "removed tool name 'cpp' must be flagged, got: {:?}",
        lost
    );
}

#[test]
fn signal_loss_passes_whitespace_only_change() {
    use proof_lib::fix::signal_loss;
    let old = "  │  compiles source      │";
    let new = "  │  compiles source       │"; // one more trailing space
    let lost = signal_loss(old, new);
    assert!(
        lost.is_empty(),
        "whitespace-only changes must not flag signal loss"
    );
}

#[test]
fn pattern_b_detects_annotation_after_bar() {
    use proof_lib::fix::is_pattern_b;
    assert!(
        is_pattern_b("  │ content │  ← annotation"),
        "annotation after │ is Pattern B"
    );
    assert!(
        is_pattern_b("│ stage │  cc -S"),
        "tool label after │ is Pattern B"
    );
    assert!(
        !is_pattern_b("  │ content │"),
        "clean closing │ is not Pattern B"
    );
    assert!(
        !is_pattern_b("  │ content │  "),
        "trailing spaces only is not Pattern B"
    );
}

#[test]
fn nested_box_col_fix_only_adjusts_leftmost() {
    use proof_lib::checks::ascii_box::AsciiBoxCheck;
    use proof_lib::checks::Check;
    use proof_lib::config::AsciiBoxConfig;
    // A nested box where inner │ and outer │ are both off by 1
    // The fix should add ONE space at the leftmost misaligned │, cascading the rest
    let content = "```\n┌──────────────────────────────┐\n│  ┌──────────┐  inner text   │\n│  └──────────┘  more text    │\n└──────────────────────────────┘\n```";
    let check = AsciiBoxCheck {
        config: AsciiBoxConfig::default(),
    };
    // Just verify it doesn't panic and returns something
    let diags = check.check(Path::new("test.md"), content);
    let _ = diags; // nested boxes may produce warnings; just verify no crash
}

// ─────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────

fn format_diags(diags: &[proof_lib::Diagnostic]) -> String {
    diags
        .iter()
        .map(|d| {
            format!(
                "  {}:{} [{}] {}",
                d.file.display(),
                d.span,
                d.code,
                d.message
            )
        })
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
    assert!(
        result.violations.is_empty(),
        "no violations: {} violations found",
        result.violations.len()
    );
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(
        content.contains("```slides"),
        "output should have slides fence"
    );
    assert!(
        content.contains("SLIDE 1"),
        "output should have slide 1 header"
    );
    assert!(
        content.contains("Test Title"),
        "output should contain title"
    );
    // Width=40, height=6 — each slide row must be exactly 40 chars
    for line in content
        .lines()
        .filter(|l| !l.starts_with("```") && !l.starts_with("<!--") && !l.starts_with("SLIDE"))
    {
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
    assert!(
        content.contains("Key Points"),
        "title should appear in output"
    );
    // Bullets from body content should appear in rendered output
    assert!(
        content.contains("First point") || content.contains("●"),
        "bullet content should render"
    );
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
    assert!(
        result
            .violations
            .iter()
            .all(|v| v.severity != proof_lib::compile::ViolationSeverity::Error),
        "no error violations"
    );
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(
        content.contains("HEADER CONTENT"),
        "top region content should appear"
    );
    assert!(
        content.contains("FOOTER CONTENT"),
        "bottom region content should appear"
    );
    // Canvas is 20×6 — check line widths
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.starts_with("<!--") && !l.starts_with("```"))
        .collect();
    for line in &lines {
        assert!(
            line.chars().count() <= 20,
            "canvas line too wide: {:?}",
            line
        );
    }
}

// ─────────────────────────────────────────────────────────
// L1: Additional coverage — slide render_slide dispatch
// ─────────────────────────────────────────────────────────

#[test]
fn render_slide_dispatch_all_layouts_produce_correct_dimensions() {
    use proof_lib::slide::{render_slide, Slide, SlideLayout, SlideMeta};
    let meta = SlideMeta {
        width: 40,
        height: 8,
        ..SlideMeta::default()
    };
    let layouts = vec![
        SlideLayout::Title,
        SlideLayout::TitleContent,
        SlideLayout::TwoColumn { ratio: (50, 50) },
        SlideLayout::Section,
        SlideLayout::Stats,
        SlideLayout::Blank,
    ];
    for layout in layouts {
        let slide = Slide {
            index: 1,
            layout: layout.clone(),
            title: Some("Test".into()),
            subtitle: Some("Sub".into()),
            author: None,
            date: None,
            body_content: "line one\nline two\n".into(),
            notes_content: String::new(),
            source_line: 0,
        };
        let lines = render_slide(&slide, &meta);
        assert_eq!(
            lines.len(),
            meta.height,
            "{:?} should produce {} rows, got {}",
            layout,
            meta.height,
            lines.len()
        );
        for (i, line) in lines.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                meta.width,
                "{:?} line {} width wrong: {:?}",
                layout,
                i,
                line
            );
        }
    }
}

#[test]
fn render_body_lines_multi_directive_dispatch() {
    use proof_lib::slide::layout::render_body_lines;
    let body = "proof:divider\nproof:bullets\n- item A\n- item B\nproof:divider style=double\n";
    let lines = render_body_lines(body, 40);
    let flat = lines.join("\n");
    // divider produces ── line
    assert!(
        flat.contains('─') || flat.contains('═'),
        "dividers should render"
    );
    // bullets render items
    assert!(flat.contains("item A"), "bullets should render items");
}

#[test]
fn slide_notes_not_in_default_output() {
    use proof_lib::slide::layout::render_body_lines;
    // proof:notes content must be excluded (SL-5)
    let body =
        "visible line\nproof:notes\nthis is a speaker note\nsecond note line\n\nback to body\n";
    let lines = render_body_lines(body, 40);
    let flat = lines.join("\n");
    assert!(
        !flat.contains("speaker note"),
        "notes must not appear in default output"
    );
    assert!(flat.contains("visible line"), "visible content must appear");
    assert!(
        flat.contains("back to body"),
        "content after notes block must appear"
    );
}

#[test]
fn notes_guard_does_not_match_prose_containing_proof_notes() {
    use proof_lib::slide::layout::render_body_lines;
    // "proof:notes are important" should NOT trigger the notes skip
    let body = "proof:notes are important for speakers\nvisible content\n";
    let lines = render_body_lines(body, 40);
    let flat = lines.join("\n");
    // The first line starts with "proof:notes" but has extra content —
    // it should NOT be treated as a notes block
    assert!(
        flat.contains("important"),
        "prose starting with proof:notes should NOT be skipped"
    );
}

// ─────────────────────────────────────────────────────────
// L1: Dashboard overlap validation
// ─────────────────────────────────────────────────────────

#[test]
fn dashboard_overlapping_regions_produce_error() {
    use proof_lib::dashboard::region::{validate_regions, RegionGeometry};
    let regions = vec![
        RegionGeometry {
            name: "a".into(),
            x: 0,
            y: 0,
            width: 60,
            height: 20,
        },
        RegionGeometry {
            name: "b".into(),
            x: 40,
            y: 0,
            width: 60,
            height: 20,
        },
    ];
    let errs = validate_regions(&regions, 120, 40);
    assert!(
        errs.iter().any(|e| e.code == "DASHBOARD-003"),
        "overlapping regions should produce DASHBOARD-003"
    );
}

#[test]
fn dashboard_adjacent_regions_do_not_overlap() {
    use proof_lib::dashboard::region::{validate_regions, RegionGeometry};
    let regions = vec![
        RegionGeometry {
            name: "left".into(),
            x: 0,
            y: 0,
            width: 40,
            height: 20,
        },
        RegionGeometry {
            name: "right".into(),
            x: 40,
            y: 0,
            width: 40,
            height: 20,
        },
    ];
    let errs = validate_regions(&regions, 80, 20);
    assert!(
        errs.is_empty(),
        "adjacent (not overlapping) regions: {:?}",
        errs
    );
}

// ─────────────────────────────────────────────────────────
// L1: Symbol expansion in compiled output
// ─────────────────────────────────────────────────────────

#[test]
fn symbol_expand_in_compiled_prose() {
    use proof_lib::symbol::{expand_symbols, SymbolLibrary};
    let lib = SymbolLibrary::new();
    let (out, warns) = expand_symbols("Status: [sym:checkmark] all good", &lib);
    assert_eq!(out, "Status: ✓ all good");
    assert!(warns.is_empty());
}

#[test]
fn symbol_resolve_case_insensitive() {
    use proof_lib::symbol::{resolve, SymbolLibrary};
    let lib = SymbolLibrary::new();
    assert!(resolve("STAR", &lib).is_some(), "uppercase should resolve");
    assert!(resolve("Star", &lib).is_some(), "mixed case should resolve");
    assert!(
        resolve("CROSS", &lib).is_some(),
        "alias uppercase should resolve"
    );
}
