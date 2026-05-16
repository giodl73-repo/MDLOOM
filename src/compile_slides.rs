use anyhow::Result;
use std::path::Path;

use crate::compile_types::{CompileResult, CompileViolation, ViolationSeverity};
use crate::slide::bullets::has_reveal_markers;
use crate::slide::layout::{render_slide_pages, render_slide_with_warnings_in_deck};
use crate::slide::parser::parse_slide_doc;

/// Compile a .slides.source.md file into a .slides.md output.
/// Each slide is rendered as a fixed-width ASCII canvas block, separated
/// by a slide divider header showing the slide number.
pub(crate) fn compile_slides_file(source_path: &Path, output_path: &Path) -> Result<CompileResult> {
    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let mut violations: Vec<CompileViolation> = Vec::new();

    let doc = match parse_slide_doc(&source_text) {
        Ok(d) => d,
        Err(errs) => {
            let mut vv = Vec::new();
            for e in errs {
                vv.push(CompileViolation {
                    code: "SLIDE-002",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: e.to_string(),
                    source_line: 0,
                });
            }
            return Ok(CompileResult {
                output_path: output_path.to_path_buf(),
                directives_resolved: 0,
                violations: vv,
                from_cache: false,
                resolved_files: vec![],
                written: false,
            });
        }
    };

    let total = doc.slides.len();
    let meta = &doc.meta;

    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "<!-- proof:compiled from=\"proof:slides\" count={} -->",
        total
    ));
    parts.push("```slides".to_string());

    for slide in &doc.slides {
        let n = slide.index;

        let use_reveal = has_reveal_markers(&slide.body_content);
        let (pages, warnings) = if use_reveal {
            let pgs = render_slide_pages(slide, meta);
            (pgs, Vec::new())
        } else {
            let (rendered, warns) = render_slide_with_warnings_in_deck(slide, meta, &doc.slides);
            (vec![rendered], warns)
        };

        let warnings = if use_reveal {
            render_slide_with_warnings_in_deck(slide, meta, &doc.slides).1
        } else {
            warnings
        };

        if !warnings.is_empty() {
            let mut seen: std::collections::HashSet<(&'static str, String)> = Default::default();
            for w in &warnings {
                if seen.insert((w.code, w.message.clone())) {
                    parts.push(format!(
                        "<!-- SLIDE-WARN {} slide={}: {} -->",
                        w.code, n, w.message
                    ));
                    violations.push(CompileViolation {
                        code: w.code,
                        severity: ViolationSeverity::Warning,
                        uri: String::new(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("slide {}: {}", n, w.message),
                        source_line: slide.source_line,
                    });
                }
            }
        }

        let num_pages = pages.len();
        for (page_idx, rendered) in pages.into_iter().enumerate() {
            let separator = format!(
                "SLIDE {} {}",
                n,
                "─".repeat(meta.width.saturating_sub(format!("SLIDE {}  ", n).len()))
            );
            if use_reveal && num_pages > 1 {
                parts.push(format!(
                    "{} {}/{} (reveal {}/{})",
                    separator,
                    n,
                    total,
                    page_idx + 1,
                    num_pages
                ));
            } else {
                parts.push(format!("{} {}/{}", separator, n, total));
            }
            if meta.progress_bar && total > 0 {
                parts.push(render_progress_bar(n, total, meta.width));
            }
            parts.extend(rendered);
        }
    }
    parts.push("```".to_string());
    parts.push("<!-- /proof:compiled -->".to_string());

    let output_text = parts.join("\n") + "\n";

    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path).map_err(|e| anyhow::anyhow!("renaming output: {}", e))?;

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: doc.slides.len(),
        violations,
        from_cache: false,
        resolved_files: vec![],
        written: true,
    })
}

/// Render a `████░░░  N/M` progress bar for slide N of M.
/// Width is `canvas_width`. Bar occupies the full width minus a ` N/M` label.
fn render_progress_bar(n: usize, total: usize, width: usize) -> String {
    let label = format!(" {}/{}", n, total);
    let bar_width = width.saturating_sub(label.len());
    let filled = (bar_width * n / total).min(bar_width);
    let empty = bar_width - filled;
    format!("{}{}{}", "█".repeat(filled), "░".repeat(empty), label)
}
