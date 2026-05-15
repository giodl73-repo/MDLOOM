use super::{align_in_width, ElementConfig};
use crate::layout::visual_width;

/// Format a scalar value using cfg.format and align to cfg.width.
pub fn render_value(v: f64, cfg: &ElementConfig) -> String {
    let formatted = apply_format(v, &cfg.format);
    let fw = visual_width(&formatted);
    if fw >= cfg.width {
        // Truncate from left for numeric values (keep significant digits on right)
        formatted
            .chars()
            .rev()
            .take(cfg.width)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    } else {
        align_in_width(&formatted, fw, cfg.width, cfg.align)
    }
}

/// Format a delta value — always has explicit sign prefix.
/// Positive → '+', negative → '-'. Uses cfg.format for the number part.
pub fn render_delta(v: f64, cfg: &ElementConfig) -> String {
    let formatted = apply_format_signed(v, &cfg.format);
    let fw = visual_width(&formatted);
    if fw >= cfg.width {
        formatted
            .chars()
            .rev()
            .take(cfg.width)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    } else {
        align_in_width(&formatted, fw, cfg.width, cfg.align)
    }
}

/// Apply a Rust-style format specifier to a float value.
/// Supported specifiers: {}, {:.N}, {:+.N}, {:>W.N}, {:0W.N}, {:.N}%
///
/// Strategy: extract the content between the outermost `{` and `}`, check for a `%`
/// suffix immediately before `}`, then parse the format spec (`:` followed by flags/width/precision).
pub fn apply_format(v: f64, fmt: &str) -> String {
    let spec = parse_format_spec(fmt);
    format_with_spec(v, &spec, false)
}

/// Like apply_format but always renders with an explicit + sign for positive values.
pub fn apply_format_signed(v: f64, fmt: &str) -> String {
    let spec = parse_format_spec(fmt);
    format_with_spec(v, &spec, true)
}

/// Parsed representation of a format spec extracted from `{...}`.
struct FormatSpec {
    precision: Option<usize>,
    percent: bool,
}

/// Parse `{:.2}`, `{:.0}%`, `{:+.2}`, `{}`, `{:.1%}`, etc. into a FormatSpec.
/// `%` is accepted both inside the braces (e.g. `{:.1%}`) and outside (e.g. `{:.0}%`).
fn parse_format_spec(fmt: &str) -> FormatSpec {
    // Handle `%` outside the braces: `{:.0}%` → treat as `{:.0}` with percent=true
    let (fmt_clean, percent_outside) = if fmt.ends_with("}%") {
        (&fmt[..fmt.len() - 1], true)
    } else {
        (fmt, false)
    };

    // Strip surrounding `{` and `}`
    let inner = fmt_clean
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(fmt_clean);

    // Handle `%` inside the braces (e.g. `{:.1%}`)
    let (inner, percent_inside) = if inner.ends_with('%') {
        (&inner[..inner.len() - 1], true)
    } else {
        (inner, false)
    };
    let percent = percent_inside || percent_outside;

    // Strip optional `:` prefix (the `:` separates argument index from format spec)
    let spec = inner.strip_prefix(':').unwrap_or(inner);

    // Strip sign flag `+`
    let spec = spec.trim_start_matches('+');

    // Strip alignment (`>`, `<`, `^`) and optional width digits
    let spec = strip_align_and_width(spec);

    // Strip zero-padding flag `0` followed by width digits
    let spec = if spec.starts_with('0') {
        let rest = &spec[1..];
        let skip = rest.chars().take_while(|c| c.is_ascii_digit()).count();
        &rest[skip..]
    } else {
        spec
    };

    // Now spec is either "" (no precision) or ".N" (precision)
    let precision = if let Some(prec_str) = spec.strip_prefix('.') {
        prec_str.parse::<usize>().ok()
    } else {
        None
    };

    FormatSpec { precision, percent }
}

fn strip_align_and_width(s: &str) -> &str {
    let mut chars = s.chars();
    match chars.next() {
        Some('>') | Some('<') | Some('^') => {
            let rest = chars.as_str();
            let skip = rest.chars().take_while(|c| c.is_ascii_digit()).count();
            &rest[skip..]
        }
        _ => s,
    }
}

fn format_with_spec(v: f64, spec: &FormatSpec, force_sign: bool) -> String {
    let num = match spec.precision {
        None => {
            // "{}" → integer-like if whole number
            if v.fract() == 0.0 && v.abs() < 1e15 {
                format!("{}", v as i64)
            } else {
                format!("{}", v)
            }
        }
        Some(prec) => format!("{:.prec$}", v, prec = prec),
    };

    let signed = if force_sign && v >= 0.0 && !num.starts_with('+') {
        format!("+{}", num)
    } else {
        num
    };

    if spec.percent {
        format!("{}%", signed)
    } else {
        signed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementAlign, ElementConfig, ElementKind};

    fn val_cfg(width: usize, fmt: &str, align: ElementAlign) -> ElementConfig {
        ElementConfig {
            kind: ElementKind::Value,
            width,
            format: fmt.to_string(),
            align,
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_format_default() {
        assert_eq!(apply_format(138.0, "{}"), "138");
    }

    #[test]
    fn test_apply_format_one_decimal() {
        assert_eq!(apply_format(138.0, "{:.1}"), "138.0");
    }

    #[test]
    fn test_apply_format_two_decimals() {
        assert_eq!(apply_format(0.19, "{:.2}"), "0.19");
    }

    #[test]
    fn test_apply_format_percent() {
        // {:.0}% — % is a suffix on the format string (outside braces)
        assert_eq!(apply_format(72.0, "{:.0}%"), "72%");
        // Percent with one decimal: use inner % notation
        assert_eq!(apply_format(50.0, "{:.1%}"), "50.0%");
    }

    #[test]
    fn test_apply_format_signed_positive() {
        let s = apply_format_signed(0.19, "{:.2}");
        assert!(s.starts_with('+'), "expected +: {:?}", s);
    }

    #[test]
    fn test_apply_format_signed_negative() {
        let s = apply_format_signed(-4.1, "{:.1}");
        assert!(s.starts_with('-'), "expected -: {:?}", s);
    }

    #[test]
    fn test_render_value_left_default() {
        let cfg = val_cfg(6, "{:.1}", ElementAlign::Left);
        let out = render_value(1.5, &cfg);
        assert_eq!(crate::layout::visual_width(&out), 6);
        assert!(out.starts_with("1.5"), "output: {:?}", out);
    }

    #[test]
    fn test_render_value_right_align() {
        let cfg = val_cfg(8, "{:.1}", ElementAlign::Right);
        let out = render_value(1.5, &cfg);
        assert_eq!(crate::layout::visual_width(&out), 8);
        assert!(out.starts_with(' '), "should right-align: {:?}", out);
    }

    #[test]
    fn test_render_delta_positive() {
        let cfg = ElementConfig {
            kind: ElementKind::Delta,
            width: 6,
            format: "{:+.2}".to_string(),
            ..Default::default()
        };
        let out = render_delta(0.19, &cfg);
        assert!(out.contains('+'), "output: {:?}", out);
        assert_eq!(crate::layout::visual_width(&out), 6);
    }

    #[test]
    fn test_render_delta_negative() {
        let cfg = ElementConfig {
            kind: ElementKind::Delta,
            width: 6,
            format: "{:+.2}".to_string(),
            ..Default::default()
        };
        let out = render_delta(-4.1, &cfg);
        assert!(out.contains('-'), "output: {:?}", out);
        assert_eq!(crate::layout::visual_width(&out), 6);
    }
}
