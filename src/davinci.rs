/// DaVinci — pinned figure validation.
///
/// When `mdloom check --daVinci` runs, each `[[davinci]]` entry is:
///   1. Resolved via md:// URI using mdpath
///   2. Checked against all its invariants
///   3. Violations emitted as diagnostics with code `fig_invariant_violated`
use crate::config::{DaVinciEntry, Invariant, MdloomConfig, ProtectionTier};
use crate::diagnostic::{Diagnostic, Severity};
use mdpath::{parse as parse_uri, resolve as resolve_uri};
use std::path::{Path, PathBuf};

/// Validate all DaVinci entries in a config against their invariants.
/// Returns diagnostics — one per violated invariant.
#[allow(non_snake_case)]
pub fn check_daVinci(config: &MdloomConfig, root: &Path) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    for entry in &config.davinci {
        match validate_entry(entry, root) {
            Ok(violations) => diags.extend(violations),
            Err(e) => {
                diags.push(Diagnostic::warning(
                    PathBuf::from(&entry.uri),
                    1,
                    1,
                    "fig_resolve_error",
                    format!(
                        "DaVinci '{}' — cannot resolve {}: {}",
                        entry.id, entry.uri, e
                    ),
                ));
            }
        }
    }

    diags
}

fn validate_entry(entry: &DaVinciEntry, root: &Path) -> Result<Vec<Diagnostic>, String> {
    let parsed = parse_uri(&entry.uri).map_err(|e| format!("invalid URI: {}", e))?;

    let element = resolve_uri(&parsed, root).map_err(|e| format!("{}", e))?;

    let severity = match entry.protection {
        ProtectionTier::Warn => Severity::Warning,
        ProtectionTier::Error | ProtectionTier::Lock => Severity::Error,
    };

    let mut diags = Vec::new();
    let content = &element.content;
    let file = PathBuf::from(&element.file);
    let line = element.line_start;

    for inv in &entry.invariants {
        if let Some(violation) = evaluate_invariant_inner(inv, content) {
            diags.push(Diagnostic {
                file: file.clone(),
                span: crate::diagnostic::Span { line, col: 1 },
                end_span: None,
                severity: severity,
                code: "fig_invariant_violated",
                message: format!(
                    "DaVinci '{}' [{} rule={}]: {}",
                    entry.id, entry.protection, inv.rule, violation
                ),
                note: Some(format!("URI: {}", entry.uri)),
                rich: None,
                group_id: Some(format!("davinci-{}", entry.id)),
            });
        }
    }

    Ok(diags)
}

/// Public entry point: evaluate one invariant against raw content string.
/// Returns Err(message) if the invariant is violated, Ok(()) if satisfied.
pub fn evaluate_invariant(inv: &Invariant, content: &str) -> Result<(), String> {
    match evaluate_invariant_inner(inv, content) {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}

/// Evaluate one invariant rule against element content.
/// Returns Some(violation_message) if violated, None if passed.
fn evaluate_invariant_inner(inv: &Invariant, content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    match inv.rule.as_str() {
        "contains-text" => {
            let text = inv.text.as_deref()?;
            if !content.to_lowercase().contains(&text.to_lowercase()) {
                return Some(format!("must contain {:?} but doesn't", text));
            }
        }
        "not-contains-text" => {
            let text = inv.text.as_deref()?;
            if content.to_lowercase().contains(&text.to_lowercase()) {
                return Some(format!("must NOT contain {:?} but does", text));
            }
        }
        "equals" => {
            let text = inv.text.as_deref()?;
            if content.trim() != text.trim() {
                return Some(format!("must equal {:?}", text));
            }
        }
        "starts-with" => {
            let text = inv.text.as_deref()?;
            if !content.trim_start().starts_with(text) {
                return Some(format!("must start with {:?}", text));
            }
        }
        "ends-with" => {
            let text = inv.text.as_deref()?;
            if !content.trim_end().ends_with(text) {
                return Some(format!("must end with {:?}", text));
            }
        }
        "pattern" => {
            // Substring match — for regex semantics use rule="regex" instead.
            let text = inv.text.as_deref()?;
            if !content.contains(text) {
                return Some(format!("must match pattern {:?}", text));
            }
        }
        "regex" => {
            let text = inv.text.as_deref()?;
            match regex::Regex::new(text) {
                Ok(re) => {
                    if !re.is_match(content) {
                        return Some(format!("regex {:?} did not match", text));
                    }
                }
                Err(e) => {
                    return Some(format!("invalid regex {:?}: {}", text, e));
                }
            }
        }
        "line-count" => {
            let count = lines.len();
            if let Some(min) = inv.min {
                if count < min {
                    return Some(format!("has {} lines, needs ≥ {}", count, min));
                }
            }
            if let Some(max) = inv.max {
                if count > max {
                    return Some(format!("has {} lines, needs ≤ {}", count, max));
                }
            }
            if let Some(exact) = inv.value {
                if count != exact {
                    return Some(format!("has {} lines, needs exactly {}", count, exact));
                }
            }
        }
        "box-width" => {
            let border_widths = detect_border_widths(&lines);
            if border_widths.is_empty() {
                return Some("no boxes detected — cannot validate box-width".to_string());
            }
            let max_width = *border_widths.iter().max().unwrap();
            let min_width = *border_widths.iter().min().unwrap();
            if let Some(min) = inv.min {
                if min_width < min {
                    return Some(format!(
                        "narrowest box is {} chars, needs ≥ {}",
                        min_width, min
                    ));
                }
            }
            if let Some(max) = inv.max {
                if max_width > max {
                    return Some(format!(
                        "widest box is {} chars, needs ≤ {}",
                        max_width, max
                    ));
                }
            }
        }
        "box-count" => {
            let count = count_boxes(&lines);
            if let Some(exact) = inv.value {
                if count != exact {
                    return Some(format!("has {} boxes, needs exactly {}", count, exact));
                }
            }
            if let Some(min) = inv.min {
                if count < min {
                    return Some(format!("has {} boxes, needs ≥ {}", count, min));
                }
            }
            if let Some(max) = inv.max {
                if count > max {
                    return Some(format!("has {} boxes, needs ≤ {}", count, max));
                }
            }
        }
        "all-boxes-same-width" => {
            let widths = detect_border_widths(&lines);
            if widths.len() >= 2 {
                let tol = inv.tolerance.unwrap_or(0);
                let min = *widths.iter().min().unwrap();
                let max = *widths.iter().max().unwrap();
                if max - min > tol {
                    return Some(format!(
                        "boxes have inconsistent widths (min={}, max={}, tolerance={})",
                        min, max, tol
                    ));
                }
            }
        }
        "column-count" => {
            // For figures: count | chars per content row
            // For tables: count separator cells
            if let Some(exact) = inv.value {
                let col_counts = detect_column_counts(&lines);
                let consistent = col_counts.iter().all(|&c| c == exact);
                if !consistent {
                    return Some(format!(
                        "column counts vary: {:?} — expected {} everywhere",
                        col_counts, exact
                    ));
                }
            }
        }
        "required-row-keys" => {
            let required = inv.values.as_deref().unwrap_or(&[]);
            let content_lower = content.to_lowercase();
            for key in required {
                if !content_lower.contains(&key.to_lowercase()) {
                    return Some(format!("missing required row key {:?}", key));
                }
            }
        }
        "heading-exists" => {
            // Checks for a specific heading text in content
            let text = inv.text.as_deref()?;
            if !content.lines().any(|l| {
                let t = l.trim_start_matches('#').trim();
                t.eq_ignore_ascii_case(text)
            }) {
                return Some(format!("expected heading {:?} not found in content", text));
            }
        }
        unknown => {
            return Some(format!("unknown invariant rule {:?}", unknown));
        }
    }

    None
}

fn detect_border_widths(lines: &[&str]) -> Vec<usize> {
    lines
        .iter()
        .filter(|l| {
            let t = l.trim();
            matches!(
                t.chars().next(),
                Some('+') | Some('┌') | Some('└') | Some('╔') | Some('╚')
            )
        })
        .map(|l| crate::layout::visual_width(l.trim()))
        .collect()
}

fn count_boxes(lines: &[&str]) -> usize {
    let borders = detect_border_widths(lines);
    // Each box has top + bottom border → approximately borders.len() / 2
    borders.len().div_ceil(2)
}

fn detect_column_counts(lines: &[&str]) -> Vec<usize> {
    lines
        .iter()
        .filter(|l| {
            (l.contains('│') || l.contains('|'))
                && !l.trim().starts_with('─')
                && !l.trim().starts_with('-')
        })
        .map(|l| l.chars().filter(|c| matches!(c, '│' | '|')).count())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_text_passes() {
        let inv = Invariant {
            rule: "contains-text".into(),
            text: Some("M:N multiplexing".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        assert!(
            evaluate_invariant_inner(&inv, "GOROUTINE SCHEDULER — M:N multiplexing\n┌──┐")
                .is_none()
        );
    }

    #[test]
    fn contains_text_fails() {
        let inv = Invariant {
            rule: "contains-text".into(),
            text: Some("missing text".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        assert!(evaluate_invariant_inner(&inv, "some content here").is_some());
    }

    #[test]
    fn box_count_passes() {
        let content = "┌──┐\n│  │\n└──┘\n┌──┐\n│  │\n└──┘";
        let inv = Invariant {
            rule: "box-count".into(),
            text: None,
            min: None,
            max: None,
            value: Some(2),
            values: None,
            tolerance: None,
        };
        assert!(evaluate_invariant_inner(&inv, content).is_none());
    }

    #[test]
    fn required_row_keys_all_present() {
        let inv = Invariant {
            rule: "required-row-keys".into(),
            text: None,
            min: None,
            max: None,
            value: None,
            values: Some(vec!["Binding".into(), "Typing".into()]),
            tolerance: None,
        };
        let content = "| Axis | Value |\n| Binding | Late |\n| Typing | Static |";
        assert!(evaluate_invariant_inner(&inv, content).is_none());
    }

    #[test]
    fn regex_matches_anchored_pattern() {
        // Substring match would over-match; regex anchors do not.
        let inv = Invariant {
            rule: "regex".into(),
            text: Some(r"^Status: (DONE|WIP)$".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        assert!(
            evaluate_invariant_inner(&inv, "Status: DONE").is_none(),
            "DONE on its own line satisfies the anchored regex"
        );
        // Default ^/$ are string-anchored. With multiline mode prefix `(?m)` they
        // become line-anchored; users opt in explicitly when needed.
        let inv_m = Invariant {
            rule: "regex".into(),
            text: Some(r"(?m)^Status: (DONE|WIP)$".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        assert!(
            evaluate_invariant_inner(&inv_m, "preamble\nStatus: DONE\n").is_none(),
            "(?m) enables per-line anchors"
        );
        // "Status: PROGRESS" — neither DONE nor WIP — fails.
        assert!(
            evaluate_invariant_inner(&inv, "Status: PROGRESS").is_some(),
            "PROGRESS doesn't match the alternation"
        );
    }

    #[test]
    fn regex_invalid_pattern_reports_error() {
        let inv = Invariant {
            rule: "regex".into(),
            text: Some("(unclosed".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        let result = evaluate_invariant_inner(&inv, "anything");
        assert!(result.is_some(), "invalid regex must violate");
        assert!(
            result.unwrap().contains("invalid regex"),
            "message names the failure"
        );
    }

    #[test]
    fn regex_distinct_from_pattern_substring() {
        // "pattern" is substring; "regex" interprets metacharacters.
        let content = "function fn_name() { ... }";
        let pattern_inv = Invariant {
            rule: "pattern".into(),
            text: Some("fn.*".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        // Substring "fn.*" doesn't appear literally → violates.
        assert!(evaluate_invariant_inner(&pattern_inv, content).is_some());

        let regex_inv = Invariant {
            rule: "regex".into(),
            text: Some("fn.*".into()),
            min: None,
            max: None,
            value: None,
            values: None,
            tolerance: None,
        };
        // Regex "fn.*" matches "fn_name() { ... }" → passes.
        assert!(evaluate_invariant_inner(&regex_inv, content).is_none());
    }

    #[test]
    fn required_row_keys_missing() {
        let inv = Invariant {
            rule: "required-row-keys".into(),
            text: None,
            min: None,
            max: None,
            value: None,
            values: Some(vec!["Binding".into(), "Missing Key".into()]),
            tolerance: None,
        };
        let content = "| Axis | Value |\n| Binding | Late |";
        assert!(evaluate_invariant_inner(&inv, content).is_some());
    }
}
