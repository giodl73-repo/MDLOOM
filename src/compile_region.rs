use std::path::Path;

use crate::compile::{render_one_directive_no_chrome, CompileViolation};
use crate::compile_directive::collect_directives;
use crate::config::GlintConfig;
use crate::runner::Runner;

/// Render the body of a proof:region directive: literal lines kept verbatim,
/// directive lines dispatched through per-directive renderers with no-chrome
/// implied so the canvas paste sees raw glyphs only.
pub(crate) fn render_region_body(
    body: &[String],
    root: &Path,
    config: &GlintConfig,
    runner: &Runner,
    abs_line: usize,
    violations: &mut Vec<CompileViolation>,
    resolved_count: &mut usize,
) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let line = &body[i];
        if let Some(header) = top_level_region_directive_header(line) {
            let mut j = i + 1;
            while j < body.len() && top_level_region_directive_header(&body[j]).is_none() {
                j += 1;
            }
            let body_slice: Vec<String> = body[i + 1..j].to_vec();
            let synth = if body_slice.is_empty() {
                format!("```{}\n```", header)
            } else {
                format!("```{}\n{}\n```", header, body_slice.join("\n"))
            };
            let nested = collect_directives(&synth);
            if let Some(directive) = nested.into_iter().next() {
                let rendered = render_one_directive_no_chrome(
                    &directive,
                    root,
                    config,
                    runner,
                    abs_line + i,
                    violations,
                    resolved_count,
                );
                for rline in rendered.lines() {
                    output.push(rline.to_string());
                }
            } else {
                output.push(line.clone());
                for b in &body_slice {
                    output.push(b.clone());
                }
            }
            i = j;
        } else {
            output.push(line.clone());
            i += 1;
        }
    }
    output
}

fn top_level_region_directive_header(line: &str) -> Option<&str> {
    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }
    const HEADERS: &[&str] = &[
        "proof:element",
        "proof:tree",
        "proof:chart",
        "proof:row",
        "proof:symbol",
        "proof:shape",
        "proof:bullets",
        "proof:centered",
        "proof:stat",
    ];
    for h in HEADERS {
        if line.starts_with(h) {
            let next = line.as_bytes().get(h.len()).copied();
            if next.is_none() || next == Some(b' ') || next == Some(b'\t') {
                return Some(line);
            }
        }
    }
    None
}
