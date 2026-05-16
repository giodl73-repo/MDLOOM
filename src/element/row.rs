use std::collections::HashMap;

use crate::element::{render_element, ElementAlign, ElementConfig, ElementData, ElementKind};

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

/// One element slot in a proof:row compositor.
#[derive(Debug, Clone)]
pub struct RowElement {
    pub kind: ElementKind,
    pub field: String,
    pub width: usize,
    pub align: ElementAlign,
    pub format: String,
    pub max: Option<f64>,
    pub fill_char: char,
    pub empty_char: char,
}

impl RowElement {
    pub fn to_element_config(&self) -> ElementConfig {
        ElementConfig {
            kind: self.kind,
            width: self.width,
            align: self.align,
            format: self.format.clone(),
            no_chrome: true,
            max: self.max,
            fill_char: self.fill_char,
            empty_char: self.empty_char,
        }
    }
}

/// Configuration for a proof:row compositor.
#[derive(Debug, Clone)]
pub struct RowConfig {
    pub source_uri: String,
    pub var_name: String,
    pub separator: String,
    pub declared_width: Option<usize>,
    pub elements: Vec<RowElement>,
    pub no_chrome: bool,
}

// ─────────────────────────────────────────────────────────
// R-1 invariant validation
// ─────────────────────────────────────────────────────────

/// R-1: sum(widths) + sep_len * (n-1) = declared_width.
/// Returns Some((found, expected)) on violation, None if valid or declared_width absent.
pub fn validate_r1(
    elements: &[RowElement],
    separator_len: usize,
    declared_width: Option<usize>,
) -> Option<(usize, usize)> {
    let expected = declared_width?;
    let n = elements.len();
    let sum_widths: usize = elements.iter().map(|e| e.width).sum();
    let sep_total = if n > 0 { separator_len * (n - 1) } else { 0 };
    let found = sum_widths + sep_total;
    if found != expected {
        Some((found, expected))
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────
// Row rendering
// ─────────────────────────────────────────────────────────

/// Render one output line from one source data row.
/// Returns Err if any field is missing or any element fails to render.
pub fn render_row(row_data: &HashMap<String, String>, cfg: &RowConfig) -> Result<String, String> {
    let mut parts: Vec<String> = Vec::with_capacity(cfg.elements.len());

    for elem in &cfg.elements {
        let raw = match row_data.get(&elem.field) {
            Some(v) => v.clone(),
            None => return Err(format!("field {:?} not found in source row", elem.field)),
        };

        let data = match elem.kind {
            ElementKind::Sparkline => {
                let series: Result<Vec<f64>, _> =
                    raw.split(',').map(|s| s.trim().parse::<f64>()).collect();
                match series {
                    Ok(v) => ElementData::Series(v),
                    Err(_) => return Err(format!(
                        "sparkline field {:?} value {:?} cannot be parsed as comma-separated numbers",
                        elem.field, raw
                    )),
                }
            }
            ElementKind::Label | ElementKind::Badge => ElementData::Text(raw),
            _ => match raw.parse::<f64>() {
                Ok(v) => ElementData::Scalar(v),
                Err(_) => {
                    return Err(format!(
                        "element kind={:?} field {:?} requires numeric value; got {:?}",
                        elem.kind, elem.field, raw
                    ))
                }
            },
        };

        let elem_cfg = elem.to_element_config();
        match render_element(&data, &elem_cfg) {
            Ok(s) => parts.push(s),
            Err(e) => return Err(format!("render error for field {:?}: {}", elem.field, e)),
        }
    }

    Ok(parts.join(&cfg.separator))
}

/// Render one output line per source row. Returns Vec<String> (one per row).
pub fn render_row_foreach(
    source_rows: &[HashMap<String, String>],
    cfg: &RowConfig,
) -> Result<Vec<String>, String> {
    source_rows.iter().map(|row| render_row(row, cfg)).collect()
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::visual_width;

    fn label_elem(field: &str, width: usize) -> RowElement {
        RowElement {
            kind: ElementKind::Label,
            field: field.to_string(),
            width,
            align: ElementAlign::Left,
            format: "{}".to_string(),
            max: None,
            fill_char: '█',
            empty_char: '░',
        }
    }

    fn value_elem(field: &str, width: usize) -> RowElement {
        RowElement {
            kind: ElementKind::Value,
            field: field.to_string(),
            width,
            align: ElementAlign::Right,
            format: "{}".to_string(),
            max: None,
            fill_char: '█',
            empty_char: '░',
        }
    }

    fn mini_bar_elem(field: &str, width: usize, max: f64) -> RowElement {
        RowElement {
            kind: ElementKind::MiniBar,
            field: field.to_string(),
            width,
            align: ElementAlign::Left,
            format: "{}".to_string(),
            max: Some(max),
            fill_char: '█',
            empty_char: '░',
        }
    }

    fn make_row(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn cfg_with(elements: Vec<RowElement>, sep: &str, width: Option<usize>) -> RowConfig {
        RowConfig {
            source_uri: "md://test.md".to_string(),
            var_name: "row".to_string(),
            separator: sep.to_string(),
            declared_width: width,
            elements,
            no_chrome: false,
        }
    }

    // ── render_row ──────────────────────────────────────────

    #[test]
    fn render_row_three_elements_correct_total_width() {
        let elems = vec![
            label_elem("name", 10),
            value_elem("pts", 6),
            mini_bar_elem("bar", 8, 200.0),
        ];
        let sep = " ";
        let total = 10 + 1 + 6 + 1 + 8; // 26
        let cfg = cfg_with(elems, sep, Some(total));
        let row = make_row(&[("name", "McDavid   "), ("pts", "138.0"), ("bar", "138")]);
        let out = render_row(&row, &cfg).unwrap();
        assert_eq!(visual_width(&out), total, "output: {:?}", out);
    }

    #[test]
    fn render_row_label_plus_value_plus_mini_bar_column_offsets() {
        let elems = vec![
            label_elem("name", 8),
            value_elem("pts", 5),
            mini_bar_elem("bar", 6, 200.0),
        ];
        let cfg = cfg_with(elems, " ", Some(8 + 1 + 5 + 1 + 6));
        let row = make_row(&[("name", "McDavid"), ("pts", "138"), ("bar", "100")]);
        let out = render_row(&row, &cfg).unwrap();
        // First 8 chars = label, char 9 = sep, next 5 = value, etc.
        let chars: Vec<char> = out.chars().collect();
        assert_eq!(chars[8], ' ', "separator at position 8: {:?}", out);
        assert_eq!(chars[14], ' ', "separator at position 14: {:?}", out);
    }

    #[test]
    fn render_row_field_not_in_row_returns_err() {
        let elems = vec![label_elem("name", 8), value_elem("missing_field", 5)];
        let cfg = cfg_with(elems, " ", None);
        let row = make_row(&[("name", "McDavid")]);
        let err = render_row(&row, &cfg).unwrap_err();
        assert!(err.contains("missing_field"), "err: {:?}", err);
    }

    // ── render_row_foreach ─────────────────────────────────

    #[test]
    fn render_row_foreach_three_rows_three_lines() {
        let elems = vec![label_elem("name", 10), value_elem("pts", 5)];
        let cfg = cfg_with(elems, " ", None);
        let rows = vec![
            make_row(&[("name", "McDavid"), ("pts", "138")]),
            make_row(&[("name", "Draisaitl"), ("pts", "120")]),
            make_row(&[("name", "Nurse"), ("pts", "50")]),
        ];
        let lines = render_row_foreach(&rows, &cfg).unwrap();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn render_row_foreach_field_missing_propagates_err() {
        let elems = vec![label_elem("name", 8), value_elem("absent", 5)];
        let cfg = cfg_with(elems, " ", None);
        let rows = vec![make_row(&[("name", "McDavid")])];
        assert!(render_row_foreach(&rows, &cfg).is_err());
    }

    // ── validate_r1 ────────────────────────────────────────

    #[test]
    fn validate_r1_correct_sum_no_error() {
        let elems = vec![
            label_elem("n", 10),
            value_elem("p", 5),
            mini_bar_elem("b", 8, 200.0),
        ];
        // sep_len=1, n=3, total = 10+5+8 + 2 = 25
        let result = validate_r1(&elems, 1, Some(25));
        assert!(result.is_none(), "should be None for correct sum");
    }

    #[test]
    fn validate_r1_sum_exceeds_declared_width_returns_err() {
        let elems = vec![label_elem("n", 10), value_elem("p", 10)];
        // actual = 10+10+1 = 21, declared = 15
        let result = validate_r1(&elems, 1, Some(15));
        assert_eq!(result, Some((21, 15)));
    }

    #[test]
    fn validate_r1_sum_less_than_declared_width_returns_err() {
        let elems = vec![label_elem("n", 5), value_elem("p", 5)];
        // actual = 5+5+1 = 11, declared = 20
        let result = validate_r1(&elems, 1, Some(20));
        assert_eq!(result, Some((11, 20)));
    }

    #[test]
    fn validate_r1_no_declared_width_always_ok() {
        let elems = vec![label_elem("n", 100)];
        let result = validate_r1(&elems, 1, None);
        assert!(result.is_none());
    }

    // ── separator ──────────────────────────────────────────

    #[test]
    fn render_row_default_separator_space() {
        let elems = vec![label_elem("a", 4), label_elem("b", 4)];
        let cfg = cfg_with(elems, " ", None);
        let row = make_row(&[("a", "foo"), ("b", "bar")]);
        let out = render_row(&row, &cfg).unwrap();
        // "foo " + " " + "bar " → 9 chars
        assert_eq!(visual_width(&out), 9, "output: {:?}", out);
        // separator is space at position 4
        let chars: Vec<char> = out.chars().collect();
        assert_eq!(chars[4], ' ');
    }

    #[test]
    fn render_row_explicit_separator_pipe() {
        let elems = vec![label_elem("a", 4), label_elem("b", 4)];
        let cfg = cfg_with(elems, "|", None);
        let row = make_row(&[("a", "foo"), ("b", "bar")]);
        let out = render_row(&row, &cfg).unwrap();
        let chars: Vec<char> = out.chars().collect();
        assert_eq!(chars[4], '|', "separator should be pipe: {:?}", out);
    }

    // ── no-chrome (column pinning via exact widths) ────────

    #[test]
    fn render_row_no_chrome_output_has_no_fence() {
        // render_row always returns raw — no fence, no HTML comment
        let elems = vec![label_elem("name", 8)];
        let cfg = RowConfig {
            no_chrome: true,
            ..cfg_with(elems, " ", None)
        };
        let row = make_row(&[("name", "McDavid")]);
        let out = render_row(&row, &cfg).unwrap();
        assert!(!out.contains("```"), "should have no fence: {:?}", out);
        assert!(
            !out.contains("<!--"),
            "should have no HTML comment: {:?}",
            out
        );
    }

    // ── column pinning invariant ───────────────────────────

    #[test]
    fn column_pinning_element_n_starts_at_sum_of_prior_widths() {
        // 3 elements: widths 6, 8, 5; sep=" " (len=1)
        // offsets: 0, 7, 16
        let elems = vec![label_elem("a", 6), label_elem("b", 8), label_elem("c", 5)];
        let cfg = cfg_with(elems, " ", None);
        let row = make_row(&[("a", "AAAAAA"), ("b", "BBBBBBBB"), ("c", "CCCCC")]);
        let out = render_row(&row, &cfg).unwrap();
        let chars: Vec<char> = out.chars().collect();
        // element 0: positions 0..5
        assert_eq!(chars[0], 'A', "elem 0 at offset 0: {:?}", out);
        // separator at position 6
        assert_eq!(chars[6], ' ', "sep after elem 0: {:?}", out);
        // element 1: positions 7..14
        assert_eq!(chars[7], 'B', "elem 1 at offset 7: {:?}", out);
        // separator at position 15
        assert_eq!(chars[15], ' ', "sep after elem 1: {:?}", out);
        // element 2: positions 16..20
        assert_eq!(chars[16], 'C', "elem 2 at offset 16: {:?}", out);
    }
}
