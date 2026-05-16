use std::path::Path;

use crate::compile_directive::ElementAttrs;
use crate::compile_format;
use crate::compile_output::source_fallback;
use crate::compile_source;
use crate::compile_types::{CompileViolation, ViolationSeverity};
use crate::element::row::{render_row_foreach, validate_r1, RowConfig, RowElement};
use crate::element::{
    render_element, ElementAlign, ElementConfig, ElementData, ElementError, ElementKind,
};
use crate::tree::schema::{parse_json_source, parse_md_table};

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_element(
    kind: &str,
    source: Option<&str>,
    field: Option<&str>,
    inline_value: Option<&str>,
    attrs: &ElementAttrs,
    root: &Path,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
    source_lines: &[&str],
    line_end: usize,
    resolved_count: &mut usize,
) -> String {
    let uri_str = source.unwrap_or("inline");

    let raw_value: String = if let Some(lit) = inline_value {
        lit.to_string()
    } else {
        let src_uri = match source {
            Some(s) => s,
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: uri_str.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message:
                        "proof:element requires either value=\"...\" or a source URI in the body"
                            .to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        let content = match compile_source::resolve_source_for_compile(src_uri, root) {
            Ok(c) => {
                *resolved_count += 1;
                c
            }
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("{}", e),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        let format = if src_uri.ends_with(".json") {
            "json"
        } else {
            "table"
        };
        let rows = match if format == "json" {
            parse_json_source(&content)
        } else {
            parse_md_table(&content)
        } {
            Ok((_, r)) => r,
            Err(e) => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("source parse error: {}", e),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        let col = match field {
            Some(f) => f,
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: "proof:element with a source URI requires field=\"ColumnName\""
                        .to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        let first_row = match rows.first() {
            Some(r) => r,
            None => {
                violations.push(CompileViolation {
                    code: "COMPILE-002",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: "source resolved to empty table".to_string(),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        };

        match first_row.get(col) {
            Some(v) => v.clone(),
            None => {
                violations.push(CompileViolation {
                    code: "ELEMENT-005",
                    severity: ViolationSeverity::Error,
                    uri: src_uri.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!("field {:?} not found in source table headers", col),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        }
    };

    let elem_kind = match ElementKind::parse(kind) {
        Some(k) => k,
        None => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("unknown element kind {:?} — use value, delta, sparkline, mini-bar, label, or badge", kind),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    let width = match attrs.width {
        Some(w) => w,
        None => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: "proof:element requires width=N".to_string(),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    let cfg = ElementConfig {
        kind: elem_kind,
        width,
        align: ElementAlign::parse(&attrs.align),
        format: attrs.format.clone(),
        no_chrome: attrs.no_chrome,
        max: attrs.max,
        fill_char: attrs.fill,
        empty_char: attrs.empty,
    };

    let data = match elem_kind {
        ElementKind::Sparkline => {
            let series: Result<Vec<f64>, _> = raw_value
                .split(',')
                .map(|s| s.trim().parse::<f64>())
                .collect();
            match series {
                Ok(v) => {
                    if v.len() < width {
                        violations.push(CompileViolation {
                            code: "ELEMENT-003",
                            severity: ViolationSeverity::Warning,
                            uri: uri_str.to_string(),
                            figure_id: None,
                            invariant: String::new(),
                            message: format!("sparkline series ({} values) shorter than width ({}) — values will be repeated", v.len(), width),
                            source_line: source_line + 1,
                        });
                    }
                    ElementData::Series(v)
                }
                Err(_) => {
                    violations.push(CompileViolation {
                        code: "ELEMENT-002",
                        severity: ViolationSeverity::Error,
                        uri: uri_str.to_string(),
                        figure_id: None,
                        invariant: String::new(),
                        message: format!("sparkline field value {:?} cannot be parsed as comma-separated numbers", raw_value),
                        source_line: source_line + 1,
                    });
                    return source_fallback(source_lines, source_line, line_end);
                }
            }
        }
        ElementKind::Label | ElementKind::Badge => ElementData::Text(raw_value.clone()),
        ElementKind::Value => {
            let cleaned = raw_value.replace(',', "");
            let cleaned = cleaned.trim_end_matches('%');
            match cleaned.parse::<f64>() {
                Ok(v) => ElementData::Scalar(v),
                Err(_) => ElementData::Text(raw_value.clone()),
            }
        }
        _ => match raw_value.parse::<f64>() {
            Ok(v) => ElementData::Scalar(v),
            Err(_) => {
                violations.push(CompileViolation {
                    code: "ELEMENT-002",
                    severity: ViolationSeverity::Error,
                    uri: uri_str.to_string(),
                    figure_id: None,
                    invariant: String::new(),
                    message: format!(
                        "element kind={} requires a numeric value; got {:?}",
                        kind, raw_value
                    ),
                    source_line: source_line + 1,
                });
                return source_fallback(source_lines, source_line, line_end);
            }
        },
    };

    match render_element(&data, &cfg) {
        Ok(rendered) => {
            if attrs.no_chrome {
                rendered
            } else {
                compile_format::element_block(uri_str, &rendered)
            }
        }
        Err(ElementError::WidthExceeded { actual, budget }) => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!(
                    "rendered element width {} exceeds budget {}",
                    actual, budget
                ),
                source_line: source_line + 1,
            });
            source_fallback(source_lines, source_line, line_end)
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "ELEMENT-001",
                severity: ViolationSeverity::Error,
                uri: uri_str.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("element render error: {}", e),
                source_line: source_line + 1,
            });
            source_fallback(source_lines, source_line, line_end)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_row(
    source_uri: &str,
    separator: &str,
    declared_width: Option<usize>,
    elements: &[RowElement],
    no_chrome: bool,
    root: &Path,
    source_line: usize,
    violations: &mut Vec<CompileViolation>,
    source_lines: &[&str],
    line_end: usize,
    resolved_count: &mut usize,
) -> String {
    let content = match compile_source::resolve_source_for_compile(source_uri, root) {
        Ok(c) => {
            *resolved_count += 1;
            c
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("{}", e),
                source_line: source_line + 1,
            });
            return source_fallback(source_lines, source_line, line_end);
        }
    };

    let format = if source_uri.ends_with(".json") {
        "json"
    } else {
        "table"
    };
    let rows = match if format == "json" {
        parse_json_source(&content)
    } else {
        parse_md_table(&content)
    } {
        Ok((_, r)) => r,
        Err(e) => {
            violations.push(CompileViolation {
                code: "COMPILE-002",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("source parse error: {}", e),
                source_line: source_line + 1,
            });
            return source_lines[source_line..=line_end].join("\n");
        }
    };

    if rows.is_empty() {
        violations.push(CompileViolation {
            code: "COMPILE-004",
            severity: ViolationSeverity::Warning,
            uri: source_uri.to_string(),
            figure_id: None,
            invariant: String::new(),
            message: format!(
                "proof:row produced no output — source table {:?} has 0 data rows",
                source_uri
            ),
            source_line: source_line + 1,
        });
    }

    let sep_len = separator.chars().count();
    if let Some((found, expected)) = validate_r1(elements, sep_len, declared_width) {
        violations.push(CompileViolation {
            code: "ELEMENT-004",
            severity: ViolationSeverity::Error,
            uri: source_uri.to_string(),
            figure_id: None,
            invariant: "R-1: sum(widths) + (n-1)*sep_len = row_width".to_string(),
            message: format!(
                "ELEMENT-004: row width mismatch — found={} expected={} (sum of element widths + separators must equal declared width={})",
                found, expected, expected
            ),
            source_line: source_line + 1,
        });
        return source_lines[source_line..=line_end].join("\n");
    }

    let row_cfg = RowConfig {
        source_uri: source_uri.to_string(),
        var_name: String::new(),
        separator: separator.to_string(),
        declared_width,
        elements: elements.to_vec(),
        no_chrome,
    };

    match render_row_foreach(&rows, &row_cfg) {
        Ok(lines) => {
            let rendered = lines.join("\n");
            if no_chrome {
                rendered
            } else {
                compile_format::row_block(source_uri, &rendered)
            }
        }
        Err(e) => {
            violations.push(CompileViolation {
                code: "ELEMENT-005",
                severity: ViolationSeverity::Error,
                uri: source_uri.to_string(),
                figure_id: None,
                invariant: String::new(),
                message: format!("row render error: {}", e),
                source_line: source_line + 1,
            });
            source_lines[source_line..=line_end].join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn violation_messages(violations: &[CompileViolation]) -> Vec<&str> {
        violations.iter().map(|v| v.message.as_str()).collect()
    }

    #[test]
    fn element_value_inline_renders() {
        let attrs = ElementAttrs {
            width: Some(4),
            align: "right".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=value value=\"42\" width=4 align=right",
            "```",
        ];
        let out = compile_element(
            "value",
            None,
            None,
            Some("42"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "should have no violations: {:?}",
            violation_messages(&violations)
        );
        assert!(out.contains("42"), "output should contain value: {:?}", out);
        assert_eq!(crate::layout::visual_width(&" 42"), 3);
    }

    #[test]
    fn element_label_inline_renders() {
        let attrs = ElementAttrs {
            width: Some(8),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=label value=\"McDavid\" width=8 align=left",
            "```",
        ];
        let out = compile_element(
            "label",
            None,
            None,
            Some("McDavid"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "should have no violations: {:?}",
            violation_messages(&violations)
        );
        assert!(
            out.contains("McDavid"),
            "output should contain label: {:?}",
            out
        );
    }

    #[test]
    fn element_badge_inline_renders() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=badge value=\"UFA\" width=5", "```"];
        let out = compile_element(
            "badge",
            None,
            None,
            Some("UFA"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violation_messages(&violations)
        );
        assert!(out.contains("UFA"), "output: {:?}", out);
    }

    #[test]
    fn element_no_chrome_true_omits_wrapper() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: true,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=label value=\"Hi\" width=5 no-chrome",
            "```",
        ];
        let out = compile_element(
            "label",
            None,
            None,
            Some("Hi"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violation_messages(&violations)
        );
        assert!(
            !out.contains("```"),
            "no-chrome should have no fence: {:?}",
            out
        );
        assert!(
            !out.contains("<!--"),
            "no-chrome should have no HTML comment: {:?}",
            out
        );
    }

    #[test]
    fn element_no_chrome_false_has_wrapper() {
        let attrs = ElementAttrs {
            width: Some(5),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec!["```proof:element kind=label value=\"Hi\" width=5", "```"];
        let out = compile_element(
            "label",
            None,
            None,
            Some("Hi"),
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            1,
            &mut 0,
        );
        assert!(
            violations.is_empty(),
            "violations: {:?}",
            violation_messages(&violations)
        );
        assert!(
            out.contains("<!-- proof:compiled"),
            "should have traceability comment: {:?}",
            out
        );
        assert!(out.contains("```"), "should have fence: {:?}", out);
    }

    #[test]
    fn element_missing_field_emits_element_005() {
        let attrs = ElementAttrs {
            width: Some(6),
            align: "left".to_string(),
            format: "{}".to_string(),
            no_chrome: false,
            ..Default::default()
        };
        let mut violations = Vec::new();
        let lines = vec![
            "```proof:element kind=value field=absent width=6",
            "md://test",
            "```",
        ];
        compile_element(
            "value",
            None,
            Some("absent"),
            None,
            &attrs,
            Path::new("."),
            0,
            &mut violations,
            &lines,
            2,
            &mut 0,
        );
        let codes: Vec<&str> = violations.iter().map(|v| v.code).collect();
        assert!(
            codes.contains(&"ELEMENT-005"),
            "expected ELEMENT-005, got: {:?}",
            codes
        );
    }
}
