use anyhow::Result;
use std::path::Path;

use crate::compile::{CompileResult, CompileViolation, ViolationSeverity};
use crate::compile_directive::{collect_directives, Directive};
use crate::compile_output::split_frontmatter;
use crate::compile_region::render_region_body;
use crate::config::GlintConfig;
use crate::dashboard::region::{
    compile_dashboard, parse_dashboard_frontmatter, DashboardError, RegionGeometry,
};
use crate::runner::Runner;

pub(crate) fn compile_dashboard_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &GlintConfig,
) -> Result<CompileResult> {
    let source_text = std::fs::read_to_string(source_path)
        .map_err(|e| anyhow::anyhow!("reading {}: {}", source_path.display(), e))?;

    let mut violations: Vec<CompileViolation> = Vec::new();
    let mut resolved_count = 0usize;

    let (frontmatter, body, body_offset) = split_frontmatter(&source_text);
    let (meta, regions) = parse_dashboard_frontmatter(&frontmatter);

    const CANVAS_WARN_WIDTH: usize = 220;
    if meta.width > CANVAS_WARN_WIDTH {
        violations.push(CompileViolation {
            code: "DASHBOARD-006",
            severity: ViolationSeverity::Warning,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "Canvas width {} exceeds terminal threshold {} — reduce or set a --width flag",
                meta.width, CANVAS_WARN_WIDTH
            ),
            source_line: 1,
        });
    }

    let directives = collect_directives(body);
    let runner = Runner::new(root, config.clone())?;

    let mut region_by_name: std::collections::HashMap<String, &RegionGeometry> =
        std::collections::HashMap::new();
    for r in &regions {
        region_by_name.insert(r.name.clone(), r);
    }

    let mut region_contents: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for directive in &directives {
        if let Directive::Region {
            name,
            body,
            line_start,
            ..
        } = directive
        {
            let abs_line = body_offset + line_start;

            if !region_by_name.contains_key(name) {
                violations.push(CompileViolation {
                    code: "DASHBOARD-004",
                    severity: ViolationSeverity::Error,
                    uri: String::new(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!(
                        "proof:region {:?} has no matching front-matter declaration",
                        name
                    ),
                    source_line: abs_line + 1,
                });
                continue;
            }

            let rendered = render_region_body(
                body,
                root,
                config,
                &runner,
                abs_line,
                &mut violations,
                &mut resolved_count,
            );
            region_contents.insert(name.clone(), rendered);
        }
    }

    if violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    let (canvas_text, dashboard_errors) = compile_dashboard(&meta, &regions, &region_contents);

    for de in dashboard_errors {
        let DashboardError { code, message } = de;
        let severity = match code {
            "DASHBOARD-005" => ViolationSeverity::Warning,
            _ => ViolationSeverity::Error,
        };
        violations.push(CompileViolation {
            code,
            severity,
            uri: String::new(),
            figure_id: None,
            invariant: String::new(),
            message,
            source_line: 1,
        });
    }

    if violations
        .iter()
        .any(|v| v.severity == ViolationSeverity::Error)
    {
        return Ok(CompileResult {
            output_path: output_path.to_path_buf(),
            directives_resolved: resolved_count,
            violations,
            from_cache: false,
            resolved_files: vec![],
            written: false,
        });
    }

    let title_attr = if meta.title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", meta.title)
    };
    let output_text = format!(
        "<!-- proof:compiled from=\"proof:dashboard\"{} -->\n```dashboard\n{}```\n<!-- /proof:compiled -->\n",
        title_attr, canvas_text
    );

    let tmp = output_path.with_extension("proof_tmp");
    std::fs::write(&tmp, &output_text)
        .map_err(|e| anyhow::anyhow!("writing temp output {}: {}", tmp.display(), e))?;
    std::fs::rename(&tmp, output_path)
        .map_err(|e| anyhow::anyhow!("renaming output {}: {}", output_path.display(), e))?;

    Ok(CompileResult {
        output_path: output_path.to_path_buf(),
        directives_resolved: resolved_count,
        violations,
        from_cache: false,
        resolved_files: vec![],
        written: true,
    })
}
