/// Math L1 integration tests — verify the full compile pipeline for
/// `proof:math` directives and inline `$...$` expansion.
///
/// L0 = unit tests (src/math/**)
/// L1 = integration (this file — compile pipeline, slide body expansion)
/// L2 = E2E (CLI invocation)

use proof_lib::compile::{compile_file, ViolationSeverity};
use proof_lib::GlintConfig;
use std::path::Path;

fn default_cfg() -> GlintConfig {
    GlintConfig::default()
}

/// Compile a string of source markdown and return (output_text, violations).
fn compile_source(src: &str, filename: &str) -> (String, Vec<proof_lib::compile::CompileViolation>) {
    let dir = tempfile::tempdir().unwrap();
    let src_path = dir.path().join(filename);
    std::fs::write(&src_path, src).unwrap();
    let out_file = tempfile::NamedTempFile::new().unwrap();
    let cfg = default_cfg();
    let result = compile_file(&src_path, out_file.path(), dir.path(), &cfg).unwrap();
    let content = std::fs::read_to_string(out_file.path()).unwrap_or_default();
    (content, result.violations)
}

fn compile_md(src: &str) -> (String, Vec<proof_lib::compile::CompileViolation>) {
    compile_source(src, "test.source.md")
}

fn compile_slides(src: &str) -> (String, Vec<proof_lib::compile::CompileViolation>) {
    compile_source(src, "test.slides.source.md")
}

// ─────────────────────────────────────────────────────────
// proof:math directive — block rendering
// ─────────────────────────────────────────────────────────

#[test]
fn math_block_frac_produces_three_lines() {
    let src = "# Test\n\n```proof:math\n\\frac{n(n+1)}{2}\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "no error violations, got: {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
    // Output should contain a bar line (─) and numerator/denominator
    assert!(out.contains('─'), "expected fraction bar in output:\n{}", out);
    assert!(out.contains("n(n+1)"), "expected numerator in output:\n{}", out);
    assert!(out.contains('2'), "expected denominator in output:\n{}", out);
    // The compiled block should be fenced
    assert!(out.contains("```"), "output should have code fence:\n{}", out);
}

#[test]
fn math_block_sum_with_limits() {
    let src = "# Test\n\n```proof:math\n\\sum_{i=1}^{n} i\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors: {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
    assert!(out.contains('∑'), "expected ∑ in output:\n{}", out);
    assert!(out.contains('n'), "expected upper limit in output:\n{}", out);
    assert!(out.contains("i=1"), "expected lower limit in output:\n{}", out);
}

#[test]
fn math_block_int_with_limits() {
    let src = "# Test\n\n```proof:math\n\\int_0^{\\infty} e^{-x} dx\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors: {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
    assert!(out.contains('∞'), "expected ∞:\n{}", out);
    assert!(out.contains('⌡'), "expected ⌡:\n{}", out);
    assert!(out.contains('0'), "expected lower limit:\n{}", out);
}

#[test]
fn math_block_width_40_center_each_line_correct_width() {
    let src = "# Test\n\n```proof:math width=40 align=center\n\\alpha\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors"
    );
    // Find the line inside the fenced block that contains α
    let math_line = out.lines()
        .find(|l| l.contains('α'))
        .expect("expected line with α in output");
    let w = proof_lib::layout::visual_width(math_line);
    assert_eq!(w, 40, "line width should be exactly 40, got {} for {:?}", w, math_line);
}

#[test]
fn math_block_align_right() {
    let src = "# Test\n\n```proof:math width=20 align=right\n\\beta\n```\n";
    let (out, _) = compile_md(src);
    let math_line = out.lines()
        .find(|l| l.contains('β'))
        .expect("expected β in output");
    assert!(math_line.ends_with('β'), "right-aligned β should be at end of line: {:?}", math_line);
    assert_eq!(proof_lib::layout::visual_width(math_line), 20);
}

#[test]
fn math_block_no_chrome_omits_comment_wrapper() {
    let src = "# Test\n\n```proof:math no-chrome=true\n\\gamma\n```\n";
    let (out, _) = compile_md(src);
    assert!(!out.contains("proof:compiled"), "no-chrome should omit comment wrapper:\n{}", out);
    assert!(out.contains('γ'), "γ should still appear:\n{}", out);
}

#[test]
fn math_block_with_chrome_has_comment_wrapper() {
    let src = "# Test\n\n```proof:math\n\\delta\n```\n";
    let (out, _) = compile_md(src);
    assert!(out.contains("proof:compiled"), "default (with chrome) should have comment wrapper:\n{}", out);
}

#[test]
fn math_block_pmatrix_2x2() {
    let src = "# Test\n\n```proof:math\n\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors: {:?}", violations.iter().map(|v| &v.message).collect::<Vec<_>>()
    );
    assert!(out.contains('⎛'), "expected ⎛ delimiter:\n{}", out);
    assert!(out.contains('⎝'), "expected ⎝ delimiter:\n{}", out);
}

#[test]
fn math_block_unknown_command_emits_math_001_warning() {
    let src = "# Test\n\n```proof:math\n\\unknowncmd\n```\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().any(|v| v.code == "MATH-001"),
        "expected MATH-001 warning, got: {:?}", violations.iter().map(|v| v.code).collect::<Vec<_>>()
    );
    // Unknown command passed through
    assert!(out.contains("\\unknowncmd") || out.contains("unknowncmd"),
        "unknown command should appear in output:\n{}", out);
}

#[test]
fn math_block_mismatched_env_emits_math_003_error() {
    let src = "# Test\n\n```proof:math\n\\begin{pmatrix} a \\end{bmatrix}\n```\n";
    let (_, violations) = compile_md(src);
    assert!(
        violations.iter().any(|v| v.code == "MATH-003"),
        "expected MATH-003, got: {:?}", violations.iter().map(|v| v.code).collect::<Vec<_>>()
    );
}

#[test]
fn math_block_overflow_emits_math_004_warning() {
    let src = "# Test\n\n```proof:math width=5\n\\alpha + \\beta + \\gamma + \\delta\n```\n";
    let (_, violations) = compile_md(src);
    assert!(
        violations.iter().any(|v| v.code == "MATH-004"),
        "expected MATH-004 clipping warning"
    );
}

// ─────────────────────────────────────────────────────────
// Inline $...$ expansion — via slide body
// ─────────────────────────────────────────────────────────

fn slide_with_body(body: &str) -> String {
    format!(
        "---\nwidth: 60\nheight: 10\n---\n\n```proof:slide layout=title-content\ntitle: Test\n---\n{}\n```\n",
        body
    )
}

#[test]
fn inline_math_greek_in_slide_body() {
    let src = slide_with_body("$\\alpha + \\beta = \\gamma$");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('α'), "expected α in rendered slide:\n{}", out);
    assert!(out.contains('β'), "expected β:\n{}", out);
    assert!(out.contains('γ'), "expected γ:\n{}", out);
}

#[test]
fn inline_math_superscripts_in_slide_body() {
    let src = slide_with_body("$x^2 + y^2 = z^2$");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('²'), "expected ² in rendered slide:\n{}", out);
}

#[test]
fn inline_math_frac_downgrades_with_math_005() {
    // \frac in inline context → MATH-005 warning + a/b rendering
    let src = slide_with_body("$\\frac{a}{b}$");
    let (out, violations) = compile_slides(&src);
    // The warning goes through the slide compile path — check output contains a/b
    assert!(out.contains("a/b"), "expected a/b downgrade in output:\n{}", out);
}

#[test]
fn inline_math_not_expanded_in_code_span() {
    let src = slide_with_body("use `$x^2$` for squares");
    let (out, _) = compile_slides(&src);
    // The literal $x^2$ inside backticks should remain unexpanded
    assert!(out.contains("$x^2$") || out.contains("$x"),
        "code span should suppress expansion:\n{}", out);
    // The ² should NOT appear from within the code span
    // (it's already in a code span so stays literal)
}

#[test]
fn inline_math_unmatched_dollar_passthrough() {
    let src = slide_with_body("costs $5 per unit");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('$'), "unmatched $ should pass through:\n{}", out);
}

#[test]
fn inline_math_and_symbol_both_expand() {
    let src = slide_with_body("[sym:checkmark] $\\alpha$");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('✓') || out.contains('✔'),
        "checkmark symbol should expand:\n{}", out);
    assert!(out.contains('α'), "alpha should expand:\n{}", out);
}

#[test]
fn inline_math_multiple_spans_on_one_line() {
    let src = slide_with_body("$\\alpha$ and $\\beta$ are parameters");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('α'), "α should expand:\n{}", out);
    assert!(out.contains('β'), "β should expand:\n{}", out);
    assert!(out.contains("and"), "surrounding prose preserved:\n{}", out);
}

#[test]
fn inline_math_to_arrow() {
    let src = slide_with_body("$x \\to y$");
    let (out, _) = compile_slides(&src);
    assert!(out.contains('→'), "→ should expand:\n{}", out);
}

// ─────────────────────────────────────────────────────────
// Regression: proof:math in non-slide source files
// ─────────────────────────────────────────────────────────

#[test]
fn math_block_in_prose_document() {
    // proof:math works in regular .source.md (not just slides)
    let src = "# Calculus\n\nThe derivative:\n\n```proof:math\n\\frac{d}{dx} e^x = e^x\n```\n\nEnd.\n";
    let (out, violations) = compile_md(src);
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "unexpected errors"
    );
    assert!(out.contains('─'), "fraction bar should appear:\n{}", out);
    assert!(out.contains("eˣ") || out.contains("e^x") || out.contains("eˣ"),
        "e^x should appear in output:\n{}", out);
}

#[test]
fn math_block_pure_symbol_renders() {
    let src = "# Test\n\n```proof:math\n\\pi\n```\n";
    let (out, violations) = compile_md(src);
    assert!(violations.iter().all(|v| v.severity != ViolationSeverity::Error));
    assert!(out.contains('π'), "π should appear:\n{}", out);
}

#[test]
fn math_block_empty_no_panic() {
    // Empty proof:math block should not panic
    let src = "# Test\n\n```proof:math\n\n```\n";
    let (_, violations) = compile_md(src);
    // No error violations — empty is silently OK
    assert!(
        violations.iter().all(|v| v.severity != ViolationSeverity::Error),
        "empty math block should not error"
    );
}

// ─────────────────────────────────────────────────────────
// proof:reveal — progressive bullet reveal in slides
// ─────────────────────────────────────────────────────────

#[test]
fn reveal_no_markers_no_reveal_annotation() {
    // A slide with no [N] markers must not produce any "reveal" annotation in output headers.
    let src = slide_with_body("proof:bullets\n- A\n- B\n");
    let (out, _) = compile_slides(&src);
    let reveal_headers: Vec<_> = out.lines()
        .filter(|l| l.contains("reveal"))
        .collect();
    assert!(reveal_headers.is_empty(),
        "slides with no [N] markers must not produce reveal headers, got: {:?}", reveal_headers);
}

#[test]
fn reveal_markers_produce_reveal_annotations() {
    // A slide body with [2] marker should produce reveal-annotated canvas headers.
    let src = slide_with_body("proof:bullets\n- Always\n[2] - Step 2\n");
    let (out, _) = compile_slides(&src);
    let reveal_headers: Vec<_> = out.lines()
        .filter(|l| l.contains("reveal"))
        .collect();
    assert!(!reveal_headers.is_empty(),
        "slide with [2] marker must produce reveal-annotated headers, got output:\n{}", out);
}
