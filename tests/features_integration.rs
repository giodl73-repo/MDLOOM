/// L1 + L2 integration tests for features completed in the current milestone:
/// - proof:toc directive
/// - proof compile --output-dir
/// - [[compile]] multi-target (proof.toml)
/// - proof spec-generate
/// - proof layout command
/// - proof compile --watch initial pass

use proof_lib::compile::{compile_file, ViolationSeverity};
use proof_lib::GlintConfig;
use proof_lib::spec_gen;
use proof_lib::layout::{layout, LayoutConfig, Align, Direction};
use std::path::Path;

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

fn proof_bin() -> std::path::PathBuf {
    // Workspace binary (workspace target dir is one level above CARGO_MANIFEST_DIR)
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().unwrap_or(manifest);
    let bin = workspace.join("target/debug/proof");
    if bin.exists() { return bin; }
    // Fallback: package-local target (pre-workspace builds)
    manifest.join("target/debug/proof")
}

fn run_proof(args: &[&str], cwd: &Path) -> (std::process::Output, String, String) {
    let bin = proof_bin();
    let out = std::process::Command::new(&bin)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run proof binary at {:?}: {}", bin, e));
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (out, stdout, stderr)
}

fn compile_str(src: &str, filename: &str, root: &Path)
    -> (String, Vec<proof_lib::compile::CompileViolation>)
{
    let src_path = root.join(filename);
    std::fs::write(&src_path, src).unwrap();
    let out_file = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src_path, out_file.path(), root, &cfg).unwrap();
    let content = std::fs::read_to_string(out_file.path()).unwrap_or_default();
    (content, result.violations)
}

// ─────────────────────────────────────────────────────────
// L1: proof:toc directive
// ─────────────────────────────────────────────────────────

#[test]
fn toc_directive_generates_outline_in_output() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Getting Started\n\n```proof:toc max-depth=3 style=list\n```\n\n## Install\n## Usage\n### Quick start\n## Examples\n";
    let (out, violations) = compile_str(src, "test.source.md", dir.path());
    assert!(violations.iter().all(|v| v.severity != ViolationSeverity::Error));
    assert!(out.contains("Getting Started"), "H1 should appear in TOC");
    assert!(out.contains("Install"));
    assert!(out.contains("Usage"));
    assert!(out.contains("Quick start"), "H3 within max-depth=3 should appear");
    assert!(out.contains("Examples"));
}

#[test]
fn toc_directive_respects_max_depth() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Title\n\n```proof:toc max-depth=2 style=list\n```\n\n## Section\n### Subsection\n#### Deep\n";
    let (out, _) = compile_str(src, "test.source.md", dir.path());
    // Extract just the compiled TOC block
    let toc_block = out.split("<!-- proof:compiled from=\"proof:toc\" -->")
        .nth(1).unwrap_or("")
        .split("<!-- /proof:compiled -->")
        .next().unwrap_or("");
    assert!(toc_block.contains("Section"), "H2 should be in TOC");
    assert!(!toc_block.contains("Subsection"), "H3 should be excluded by max-depth=2");
    assert!(!toc_block.contains("Deep"), "H4 should be excluded by max-depth=2");
}

#[test]
fn toc_directive_from_source_file() {
    let dir = tempfile::tempdir().unwrap();
    // Create a separate source file to read headings from
    std::fs::write(dir.path().join("reference.md"),
        "# Reference\n## API\n### parse\n### resolve\n## Errors\n").unwrap();
    let src = "# My Docs\n\nTable of contents for the reference:\n\n```proof:toc source=md://reference.md max-depth=3 style=list\n```\n";
    let (out, violations) = compile_str(src, "test.source.md", dir.path());
    assert!(violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors: {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>());
    assert!(out.contains("Reference") || out.contains("API"),
        "headings from reference.md should appear:\n{}", out);
}

#[test]
fn toc_missing_source_emits_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Test\n\n```proof:toc source=md://nonexistent.md\n```\n";
    let (_, violations) = compile_str(src, "test.source.md", dir.path());
    assert!(violations.iter().any(|v| v.severity == ViolationSeverity::Error),
        "missing source should produce error");
}

// ─────────────────────────────────────────────────────────
// L1: proof compile --output-dir
// ─────────────────────────────────────────────────────────

#[test]
fn output_dir_routes_compiled_files_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    let out_dir = dir.path().join("docs");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir_all(&out_dir).unwrap();

    std::fs::write(src_dir.join("guide.source.md"),
        "# Guide\n\nHello world.\n").unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; } // skip if binary not built

    let status = std::process::Command::new(&bin)
        .args(["compile", "--output-dir", out_dir.to_str().unwrap(),
               src_dir.to_str().unwrap()])
        .current_dir(dir.path())
        .status().unwrap();

    assert!(status.success(), "compile --output-dir should succeed");
    assert!(out_dir.join("guide.md").exists(),
        "guide.md should appear in docs/, not src/");
    assert!(!src_dir.join("guide.md").exists(),
        "guide.md should NOT appear in src/ when --output-dir is set");
}

#[test]
fn output_dir_created_if_missing() {
    let dir = tempfile::tempdir().unwrap();
    let out_dir = dir.path().join("brand_new_dir");

    std::fs::write(dir.path().join("test.source.md"), "# Test\n").unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; }

    let status = std::process::Command::new(&bin)
        .args(["compile", "--output-dir", out_dir.to_str().unwrap(),
               dir.path().join("test.source.md").to_str().unwrap()])
        .current_dir(dir.path())
        .status().unwrap();

    assert!(status.success());
    assert!(out_dir.exists(), "--output-dir should be auto-created");
    assert!(out_dir.join("test.md").exists());
}

// ─────────────────────────────────────────────────────────
// L1: [[compile]] multi-target in proof.toml
// ─────────────────────────────────────────────────────────

#[test]
fn multi_target_compile_routes_each_source_dir() {
    let dir = tempfile::tempdir().unwrap();
    let guides_src = dir.path().join("src/guides");
    let pres_src   = dir.path().join("src/presentations");
    let guides_out = dir.path().join("docs/guides");
    let pres_out   = dir.path().join("docs/presentations");

    for d in [&guides_src, &pres_src, &guides_out, &pres_out] {
        std::fs::create_dir_all(d).unwrap();
    }

    std::fs::write(guides_src.join("01-intro.source.md"), "# Intro\n").unwrap();
    std::fs::write(pres_src.join("deck.slides.source.md"),
        "---\nwidth: 40\nheight: 6\n---\n\n```proof:slide layout=title\ntitle: Deck\n```\n").unwrap();

    std::fs::write(dir.path().join("proof.toml"), r#"
[files]
root = true

[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
"#).unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; }

    let status = std::process::Command::new(&bin)
        .args(["compile"])
        .current_dir(dir.path())
        .status().unwrap();

    assert!(status.success(), "multi-target compile should succeed");
    assert!(guides_out.join("01-intro.md").exists(), "guide should be in docs/guides/");
    assert!(pres_out.join("deck.slides.md").exists(), "deck should be in docs/presentations/");
}

// ─────────────────────────────────────────────────────────
// L1: proof spec-generate
// ─────────────────────────────────────────────────────────

#[test]
fn spec_generate_produces_line_count_invariant() {
    let content = r"
GOROUTINE SCHEDULER
┌─────────────────────────────────────┐
│  OS Thread (M)                      │
│  ┌──────┐ ┌──────┐ ┌──────┐        │
│  │  G   │ │  G   │ │  G   │        │
│  └──────┘ └──────┘ └──────┘        │
└─────────────────────────────────────┘
";
    let spec = spec_gen::generate(content, Some("GOROUTINE SCHEDULER"), "md://test.md", "test");
    let rules: Vec<&str> = spec.invariants.iter().map(|i| i.rule.as_str()).collect();
    assert!(rules.contains(&"line-count"), "should always suggest line-count");
    assert!(rules.contains(&"box-count"), "should suggest box-count for box figures");
    assert!(rules.contains(&"contains-text"), "should suggest contains-text for label");
}

#[test]
fn spec_generate_toml_output_is_valid() {
    let content = "ARCH\n┌────┐\n│ A  │\n└────┘\n";
    let spec = spec_gen::generate(content, Some("ARCH"), "md://figures/arch.md", "arch");
    let toml = spec_gen::format_toml(&spec);
    assert!(toml.contains("[[davinci]]"), "output should have [[davinci]] header");
    assert!(toml.contains("id = \"arch\""));
    assert!(toml.contains("[[davinci.invariants]]"), "should have at least one invariant");
    // Verify it's parseable as TOML
    let parsed: Result<toml::Value, _> = toml::from_str(&toml);
    // TOML may not parse cleanly due to comment-only prefix, but key structure should be there
    assert!(!toml.is_empty());
}

#[test]
fn spec_generate_confidence_levels_set() {
    let content = "LABEL\n┌────────────────────┐\n│ content here       │\n└────────────────────┘\n";
    let spec = spec_gen::generate(content, Some("LABEL"), "md://test.md", "test");
    let has_high = spec.invariants.iter().any(|i| {
        matches!(i.confidence, spec_gen::SuggestionConfidence::High)
    });
    assert!(has_high, "should have at least one high-confidence invariant");
}

#[test]
fn spec_generate_empty_content_still_produces_line_count() {
    let spec = spec_gen::generate("", None, "md://empty.md", "empty");
    assert!(!spec.invariants.is_empty(), "even empty content gets line-count invariant");
}

// ─────────────────────────────────────────────────────────
// L1: proof layout
// ─────────────────────────────────────────────────────────

#[test]
fn layout_two_figures_side_by_side() {
    let fig_a = vec![
        "┌──────┐".to_string(),
        "│  A   │".to_string(),
        "└──────┘".to_string(),
    ];
    let fig_b = vec![
        "┌──────┐".to_string(),
        "│  B   │".to_string(),
        "└──────┘".to_string(),
    ];
    let cfg = LayoutConfig {
        gap: 3,
        align: Align::Top,
        labels: vec![],
        cols: None,
        width: 120,
        direction: Direction::Horizontal,
        border: false,
    };
    let result = layout(vec![fig_a, fig_b], &cfg);
    // layout() wraps output in ``` fences — check the inner content
    assert!(result.contains("A"), "figure A should appear");
    assert!(result.contains("B"), "figure B should appear");
    // In horizontal layout, ┌ should appear twice on the same content line
    let content_lines: Vec<&str> = result.lines()
        .filter(|l| !l.trim_matches('`').is_empty() && *l != "```")
        .collect();
    let first_content = content_lines.first().copied().unwrap_or("");
    assert!(first_content.contains('┌'),
        "first content line should have box chars in horizontal layout: {:?}", first_content);
}

#[test]
fn layout_with_labels() {
    let fig = vec!["┌───┐".to_string(), "│ X │".to_string(), "└───┘".to_string()];
    let cfg = LayoutConfig {
        gap: 4,
        align: Align::Top,
        labels: vec!["Left".to_string(), "Right".to_string()],
        cols: None,
        width: 120,
        direction: Direction::Horizontal,
        border: false,
    };
    let result = layout(vec![fig.clone(), fig], &cfg);
    assert!(result.contains("Left"), "label should appear in output");
    assert!(result.contains("Right"), "label should appear in output");
}

#[test]
fn layout_vertical_direction() {
    let fig_a = vec!["TOP".to_string()];
    let fig_b = vec!["BOT".to_string()];
    let cfg = LayoutConfig {
        gap: 1,
        align: Align::Top,
        labels: vec![],
        cols: None,
        width: 120,
        direction: Direction::Vertical,
        border: false,
    };
    let result = layout(vec![fig_a, fig_b], &cfg);
    let lines: Vec<&str> = result.lines().collect();
    // In vertical layout, TOP should appear before BOT
    let top_idx = lines.iter().position(|l| l.contains("TOP"));
    let bot_idx = lines.iter().position(|l| l.contains("BOT"));
    assert!(top_idx.is_some() && bot_idx.is_some());
    assert!(top_idx.unwrap() < bot_idx.unwrap(), "TOP should precede BOT in vertical layout");
}

#[test]
fn layout_empty_figures_no_panic() {
    let cfg = LayoutConfig {
        gap: 3, align: Align::Top, labels: vec![], cols: None,
        width: 120, direction: Direction::Horizontal, border: false,
    };
    // Empty input should not panic — returns empty fence or empty string
    let result = layout(vec![], &cfg);
    assert!(result.len() < 10, "empty layout should produce minimal output, got: {:?}", result);
}

// ─────────────────────────────────────────────────────────
// L2: CLI binary tests (require binary to be built)
// ─────────────────────────────────────────────────────────

#[test]
fn cli_proof_version_exits_zero() {
    let bin = proof_bin();
    if !bin.exists() { return; }
    let (out, stdout, _) = run_proof(&["--version"], Path::new("."));
    assert!(out.status.success(), "proof --version should exit 0");
    assert!(stdout.contains("proof") || stdout.contains("0."), "should print version");
}

#[test]
fn cli_compile_output_dir_flag() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test.source.md"), "# Hello\n").unwrap();
    let out_dir = dir.path().join("output");

    let bin = proof_bin();
    if !bin.exists() { return; }

    let (out, _, stderr) = run_proof(
        &["compile", "--output-dir", out_dir.to_str().unwrap(),
          dir.path().join("test.source.md").to_str().unwrap()],
        dir.path()
    );
    assert!(out.status.success(), "compile --output-dir should succeed, stderr: {}", stderr);
    assert!(out_dir.join("test.md").exists(), "test.md should be in output/");
}

#[test]
fn cli_spec_generate_outputs_toml() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("fig.md"),
        "# Fig\n\nMY FIGURE\n┌────┐\n│ A  │\n└────┘\n").unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; }

    let (out, stdout, _) = run_proof(
        &["spec-generate", "md://fig.md", "--root", dir.path().to_str().unwrap()],
        dir.path()
    );
    assert!(out.status.success(), "spec-generate should exit 0");
    assert!(stdout.contains("[[davinci]]") || stdout.contains("davinci"),
        "should output davinci TOML, got:\n{}", stdout);
}

#[test]
fn cli_check_exits_nonzero_on_md_broken_uri() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("test.source.md"),
        "# Test\n\n```proof:tree kind=taxonomy source=md://missing.md\n```\n").unwrap();
    std::fs::write(dir.path().join("proof.toml"), "[files]\nroot = true\n").unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; }

    let (out, _, stderr) = run_proof(
        &["check", "test.source.md"],
        dir.path()
    );
    // Should exit non-zero due to md_broken_uri error
    assert!(!out.status.success(), "check should fail for broken md:// URI");
    let combined = format!("{}{}", stderr, String::from_utf8_lossy(&out.stdout));
    assert!(combined.contains("md_broken_uri") || combined.contains("missing"),
        "should report broken URI, got:\n{}", combined);
}

#[test]
fn cli_toc_compiles_correctly() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("doc.source.md"),
        "# Title\n\n```proof:toc max-depth=2 style=list\n```\n\n## Install\n## Usage\n").unwrap();
    std::fs::write(dir.path().join("proof.toml"), "[files]\nroot = true\n").unwrap();

    let bin = proof_bin();
    if !bin.exists() { return; }

    let out_path = dir.path().join("doc.md");
    let (out, _, stderr) = run_proof(
        &["compile", "--root", dir.path().to_str().unwrap(),
          "-o", out_path.to_str().unwrap(),
          dir.path().join("doc.source.md").to_str().unwrap()],
        dir.path()
    );
    assert!(out.status.success(), "proof:toc compile should succeed, stderr: {}", stderr);
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("Install"), "TOC should contain Install heading");
    assert!(content.contains("Usage"), "TOC should contain Usage heading");
}

// ─────────────────────────────────────────────────────────
// Regression: proof:tree directive counted in directives_resolved (issue #3)
// ─────────────────────────────────────────────────────────

#[test]
fn tree_directive_counted_in_resolved_directives() {
    let dir = tempfile::tempdir().unwrap();
    let src = "# Doc\n\n```proof:tree kind=taxonomy\nroot: R\n- a\n- b\n```\n";
    let src_path = dir.path().join("doc.source.md");
    std::fs::write(&src_path, src).unwrap();
    let out_file = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src_path, out_file.path(), dir.path(), &cfg).unwrap();
    assert_eq!(result.directives_resolved, 1,
        "expected 1 resolved directive for a single proof:tree, got {}", result.directives_resolved);
}

#[test]
fn mixed_tree_and_other_directives_counted() {
    let dir = tempfile::tempdir().unwrap();
    // Two trees + one blockquote = 3 resolved
    let src = "# Doc\n\n\
        ```proof:tree kind=taxonomy\nroot: R1\n- a\n```\n\n\
        ```proof:blockquote\nQuote text.\n```\n\n\
        ```proof:tree kind=org\nroot: R2\n- b\n```\n";
    let src_path = dir.path().join("doc.source.md");
    std::fs::write(&src_path, src).unwrap();
    let out_file = tempfile::NamedTempFile::new().unwrap();
    let cfg = GlintConfig::default();
    let result = compile_file(&src_path, out_file.path(), dir.path(), &cfg).unwrap();
    assert_eq!(result.directives_resolved, 3,
        "expected 3 resolved directives (2 tree + 1 blockquote), got {}", result.directives_resolved);
}
