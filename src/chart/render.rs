//! Top-level chart renderer — parses attributes and dispatches to bar/line.

use super::{bar, line};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    Bar,
    Line,
}

impl ChartKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bar" => Some(Self::Bar),
            "line" => Some(Self::Line),
            _ => None,
        }
    }
}

/// One labeled data point. `label` is the category (bar) or x-label (line).
#[derive(Debug, Clone)]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
}

/// Parsed attributes for a proof:chart directive.
#[derive(Debug, Clone)]
pub struct ChartAttrs {
    pub kind: ChartKind,
    pub width: usize,
    pub height: usize,
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub max: Option<f64>,
    pub no_chrome: bool,
}

impl Default for ChartAttrs {
    fn default() -> Self {
        ChartAttrs {
            kind: ChartKind::Bar,
            width: 60,
            height: 8,
            title: None,
            x_label: None,
            y_label: None,
            max: None,
            no_chrome: false,
        }
    }
}

/// Resolved chart data: a sequence of labeled points.
#[derive(Debug, Clone)]
pub struct ChartData(pub Vec<ChartPoint>);

#[derive(Debug)]
pub struct ChartError {
    pub code: &'static str,
    pub message: String,
}

/// Render a chart to a Vec of lines. Caller wraps in fence + chrome.
pub fn render_chart(data: &ChartData, attrs: &ChartAttrs) -> Result<Vec<String>, ChartError> {
    if data.0.is_empty() {
        return Err(ChartError {
            code: "CHART-001",
            message: "chart has no data points".to_string(),
        });
    }
    let lines = match attrs.kind {
        ChartKind::Bar => bar::render_bar_chart(data, attrs),
        ChartKind::Line => line::render_line_chart(data, attrs),
    };
    Ok(lines)
}

/// Parse the body of a proof:chart directive: lines of `label: value` pairs.
/// Blank lines and lines starting with `#` are ignored.
/// Returns Err with line index (0-based, within body) on the first malformed
/// data line — callers can map this back to source lines for diagnostics.
pub fn parse_inline_body(body: &str) -> Result<ChartData, (usize, String)> {
    let mut points = Vec::new();
    for (i, raw) in body.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (label, val_str) = match line.rfind(':') {
            Some(idx) => (line[..idx].trim().to_string(), line[idx + 1..].trim()),
            None => return Err((i, format!("expected `label: value`, got {:?}", line))),
        };
        let value: f64 = val_str.parse().map_err(|_| {
            (
                i,
                format!("invalid number {:?} for label {:?}", val_str, label),
            )
        })?;
        points.push(ChartPoint { label, value });
    }
    Ok(ChartData(points))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_points() {
        let body = "Alpha: 10\nBeta: 20\n";
        let data = parse_inline_body(body).unwrap();
        assert_eq!(data.0.len(), 2);
        assert_eq!(data.0[0].label, "Alpha");
        assert_eq!(data.0[0].value, 10.0);
        assert_eq!(data.0[1].label, "Beta");
        assert_eq!(data.0[1].value, 20.0);
    }

    #[test]
    fn parse_skips_blank_and_comment() {
        let body = "# heading\n\nA: 1\n# mid\nB: 2\n";
        let data = parse_inline_body(body).unwrap();
        assert_eq!(data.0.len(), 2);
    }

    #[test]
    fn parse_label_with_colon() {
        // Use rfind so labels containing colons are tolerated.
        let body = "Time: hh:mm: 42\n";
        let data = parse_inline_body(body).unwrap();
        assert_eq!(data.0[0].label, "Time: hh:mm");
        assert_eq!(data.0[0].value, 42.0);
    }

    #[test]
    fn parse_missing_value_errors() {
        let body = "OnlyLabel\n";
        assert!(parse_inline_body(body).is_err());
    }

    #[test]
    fn parse_bad_value_errors() {
        let body = "X: notanumber\n";
        assert!(parse_inline_body(body).is_err());
    }

    #[test]
    fn render_empty_errors() {
        let data = ChartData(vec![]);
        let attrs = ChartAttrs::default();
        assert!(render_chart(&data, &attrs).is_err());
    }

    #[test]
    fn chart_kind_parse() {
        assert_eq!(ChartKind::parse("bar"), Some(ChartKind::Bar));
        assert_eq!(ChartKind::parse("line"), Some(ChartKind::Line));
        assert_eq!(ChartKind::parse("scatter"), None);
    }
}
