use std::path::Path;

use crate::compile_types::{CompileViolation, ViolationSeverity};
use crate::config::{MdloomConfig, ProtectionTier};
use crate::davinci::evaluate_invariant;
use crate::diagnostic::Severity;
use crate::runner::Runner;

/// Validate figure content with the mdloom linter before embedding.
/// Emits COMPILE-007 warnings for each lint error found in the figure.
pub(crate) fn lint_figure(
    uri: &str,
    content: &str,
    figure_file: &Path,
    source_line: usize,
    runner: &Runner,
    violations: &mut Vec<CompileViolation>,
) {
    let synthetic = format!("```\n{}\n```\n", content);
    let diags = runner.lint_content(&synthetic, figure_file);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    if !errors.is_empty() {
        violations.push(CompileViolation {
            code: "COMPILE-007",
            severity: ViolationSeverity::Warning,
            uri: uri.to_string(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "figure has {} lint error{} — embedded output may be misaligned ({})",
                errors.len(),
                if errors.len() == 1 { "" } else { "s" },
                errors.iter().map(|d| d.code).collect::<Vec<_>>().join(", ")
            ),
            source_line,
        });
    }
}

pub(crate) fn validate_davinci(
    uri: &str,
    content: &str,
    config: &MdloomConfig,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
) {
    for entry in &config.davinci {
        let uri_matches = entry.uri == uri || uri.ends_with(&entry.uri) || entry.uri.ends_with(uri);
        if !uri_matches {
            continue;
        }
        for inv in &entry.invariants {
            if let Err(msg) = evaluate_invariant(inv, content) {
                let (code, sev) = match entry.protection {
                    ProtectionTier::Error | ProtectionTier::Lock => {
                        ("COMPILE-001", ViolationSeverity::Error)
                    }
                    ProtectionTier::Warn => ("COMPILE-003", ViolationSeverity::Warning),
                };
                violations.push(CompileViolation {
                    code,
                    severity: sev,
                    uri: uri.to_string(),
                    figure_id: Some(entry.id.clone()),
                    invariant: format!("{:?}", inv),
                    message: msg,
                    source_line: source_line + 1,
                });
            }
        }
    }
}
