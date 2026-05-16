/// ASCII bar chart validator.
///
/// Detects and validates ASCII bar charts inside code blocks:
///
///   Item A  █████████████████████ 78%
///   Item B  ████████████          45%
///   Item C  ███                   12%
///
/// A bar chart is a group of 2+ consecutive lines in a code block where
/// each line contains a run of ≥ min_bar_width block characters.
///
/// Three implicit columns are parsed from each row:
///   [label padding] [label text] [bar padding] [bar] [bar padding] [value]
///
/// Checks:
///   ascii_barchart_char   — inconsistent bar characters across rows
///   ascii_barchart_pad    — missing padding between label/bar or bar/value
///   ascii_barchart_value  — inconsistent value format (% vs integer vs none)
///   ascii_barchart_align  — value column not aligned to the same visual column
use crate::checks::Check;
use crate::config::AsciiBarchartConfig;
use crate::diagnostic::Diagnostic;
use std::path::Path;

pub struct AsciiBarchartCheck {
    pub config: AsciiBarchartConfig,
}

impl Check for AsciiBarchartCheck {
    fn name(&self) -> &'static str {
        "ascii_barchart"
    }

    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        if !self.config.enabled {
            return vec![];
        }
        let lines: Vec<&str> = content.lines().collect();
        let in_code = code_block_mask(&lines);
        let charts = detect_charts(&lines, &in_code, &self.config);
        let mut diags = Vec::new();
        for chart in &charts {
            diags.extend(validate_chart(path, chart, &self.config));
        }
        diags
    }
}

// ─────────────────────────────────────────────────────────
// Data model
// ─────────────────────────────────────────────────────────

/// A parsed bar chart row.
#[derive(Debug, Clone)]
struct BarRow {
    /// 1-based line number in the file
    line: usize,
    /// Label text (trimmed), before the bar
    label: String,
    /// Spaces between label text and bar start
    label_to_bar_gap: usize,
    /// The bar characters (may be mixed — that's a validation error)
    bar: String,
    /// Number of bar-fill characters (visual width of bar)
    bar_width: usize,
    /// Spaces between bar end and value start
    bar_to_value_gap: usize,
    /// Optional trailing value (e.g. "78%", "42", "1.5s")
    value: Option<String>,
    /// Visual column where the value starts (1-based)
    value_col: usize,
}

#[derive(Debug)]
struct BarChart {
    rows: Vec<BarRow>,
}

// ─────────────────────────────────────────────────────────
// Detection
// ─────────────────────────────────────────────────────────

/// True if char is a bar fill character.
fn is_bar_char(c: char, allowed: &[String]) -> bool {
    allowed.iter().any(|s| s.starts_with(c))
}

#[allow(dead_code)]
fn default_bar_chars() -> &'static [&'static str] {
    &["█", "▓", "▒", "░", "#", "="]
}

fn is_default_bar_char(c: char) -> bool {
    matches!(c, '█' | '▓' | '▒' | '░' | '#') || c == '='
}

fn is_configured_bar_char(c: char, config: &AsciiBarchartConfig) -> bool {
    if config.bar_chars.is_empty() {
        is_default_bar_char(c)
    } else {
        is_bar_char(c, &config.bar_chars)
    }
}

fn contains_bar_run(s: &str, config: &AsciiBarchartConfig) -> bool {
    let mut run = 0usize;
    for c in s.chars() {
        if is_configured_bar_char(c, config) {
            run += 1;
            if run >= config.min_bar_width {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn is_stacked_bar(bar: &str) -> bool {
    let mut chars = bar.chars().filter(|c| is_default_bar_char(*c));
    let Some(first) = chars.next() else {
        return false;
    };
    chars.any(|c| c != first)
}

/// Try to parse a line as a bar chart row. Returns None if no bar found.
fn parse_bar_row(line: &str, abs_line: usize, config: &AsciiBarchartConfig) -> Option<BarRow> {
    if line.contains(['│', '║', '┃']) {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();
    let _n = chars.len();

    // Find the start of the first bar run
    let bar_start = chars
        .iter()
        .position(|&c| is_configured_bar_char(c, config))?;

    // Measure bar length
    let bar_end = bar_start
        + chars[bar_start..]
            .iter()
            .take_while(|&&c| is_configured_bar_char(c, config))
            .count();

    let bar_width = bar_end - bar_start;
    if bar_width < config.min_bar_width {
        return None;
    }

    let bar: String = chars[bar_start..bar_end].iter().collect();

    // Label is everything before the bar
    let label_raw: String = chars[..bar_start].iter().collect();
    let label = label_raw.trim_end().to_string();
    let trimmed_label = label.trim();
    if trimmed_label.ends_with('|')
        || trimmed_label.contains('\\')
        || trimmed_label.contains('/')
        || trimmed_label.is_empty()
    {
        return None;
    }
    let label_to_bar_gap = label_raw.len() - label.len(); // trailing spaces = gap

    // After bar: optional padding then optional value
    let after_bar: String = chars[bar_end..].iter().collect();
    if after_bar.starts_with(|c: char| !c.is_whitespace() && !c.is_ascii_digit()) {
        return None;
    }
    if contains_bar_run(&after_bar, config) {
        return None;
    }
    if bar.chars().all(|c| c == '=') && bar_width <= config.min_bar_width {
        return None;
    }
    let trimmed_after = after_bar.trim_start();
    let bar_to_value_gap = after_bar.len() - trimmed_after.len();
    let value = if trimmed_after.is_empty() {
        None
    } else {
        Some(trimmed_after.trim_end().to_string())
    };

    // Value column (1-based, visual)
    let value_col = if value.is_some() {
        bar_end + bar_to_value_gap + 1
    } else {
        0
    };

    Some(BarRow {
        line: abs_line,
        label,
        label_to_bar_gap,
        bar,
        bar_width,
        bar_to_value_gap,
        value,
        value_col,
    })
}

/// Find all bar chart groups in the file.
fn detect_charts(lines: &[&str], in_code: &[bool], config: &AsciiBarchartConfig) -> Vec<BarChart> {
    let mut charts = Vec::new();
    let mut current: Vec<BarRow> = Vec::new();

    for (i, &line) in lines.iter().enumerate() {
        if !in_code[i] {
            flush_chart(&mut current, &mut charts, config.min_chart_rows);
            continue;
        }
        match parse_bar_row(line, i + 1, config) {
            Some(row) => current.push(row),
            None => flush_chart(&mut current, &mut charts, config.min_chart_rows),
        }
    }
    flush_chart(&mut current, &mut charts, config.min_chart_rows);
    charts
}

fn flush_chart(current: &mut Vec<BarRow>, charts: &mut Vec<BarChart>, min_rows: usize) {
    if current.len() >= min_rows {
        charts.push(BarChart {
            rows: std::mem::take(current),
        });
    } else {
        current.clear();
    }
}

// ─────────────────────────────────────────────────────────
// Validation
// ─────────────────────────────────────────────────────────

fn validate_chart(path: &Path, chart: &BarChart, config: &AsciiBarchartConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // Check 1: consistent bar characters
    if let Some(first_row) = chart.rows.first() {
        let first_char = first_row.bar.chars().next();
        if !chart.rows.iter().any(|row| is_stacked_bar(&row.bar)) {
            for row in chart.rows.iter().skip(1) {
                let this_char = row.bar.chars().next();
                if this_char != first_char {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        row.line,
                        1,
                        "ascii_barchart_char",
                        format!(
                            "inconsistent bar character: row uses {:?}, first row uses {:?}",
                            this_char.unwrap_or('?'),
                            first_char.unwrap_or('?')
                        ),
                    ));
                }
            }
        }

        // Check 2: minimum padding — label to bar gap
        if config.min_label_padding > 0 {
            for row in &chart.rows {
                if row.label_to_bar_gap < config.min_label_padding {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        row.line,
                        1,
                        "ascii_barchart_pad",
                        format!(
                            "label {:?} has {} space{} before bar — need at least {}",
                            row.label,
                            row.label_to_bar_gap,
                            if row.label_to_bar_gap == 1 { "" } else { "s" },
                            config.min_label_padding
                        ),
                    ));
                }
            }
        }

        // Check 3: minimum padding — bar to value gap
        if config.min_value_padding > 0 {
            for row in chart.rows.iter().filter(|r| r.value.is_some()) {
                if row.bar_to_value_gap < config.min_value_padding {
                    diags.push(Diagnostic::warning(
                        path.to_path_buf(),
                        row.line,
                        1,
                        "ascii_barchart_pad",
                        format!(
                            "bar has {} space{} before value — need at least {}",
                            row.bar_to_value_gap,
                            if row.bar_to_value_gap == 1 { "" } else { "s" },
                            config.min_value_padding
                        ),
                    ));
                }
            }
        }

        // Check 4: value format consistency
        if config.check_value_format {
            let formats: Vec<ValueFormat> = chart
                .rows
                .iter()
                .filter_map(|r| r.value.as_deref())
                .map(detect_value_format)
                .collect();
            if !formats.is_empty() {
                let first_fmt = &formats[0];
                for (idx, fmt) in formats.iter().enumerate().skip(1) {
                    if std::mem::discriminant(fmt) != std::mem::discriminant(first_fmt) {
                        let row = &chart.rows[idx];
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            row.line,
                            1,
                            "ascii_barchart_value",
                            format!(
                                "inconsistent value format: {:?} is {:?} but first row is {:?}",
                                row.value.as_deref().unwrap_or(""),
                                fmt,
                                first_fmt
                            ),
                        ));
                    }
                }
            }
        }

        // Check 5: proportionality — bar widths must match numeric values
        // For percentage charts: bar at 78% must not fill 100% of the max bar width.
        // We fit a linear scale from the widest bar to its corresponding value.
        if config.check_proportionality {
            let pairs: Vec<(usize, f64)> = chart
                .rows
                .iter()
                .filter_map(|r| {
                    let val = r.value.as_deref().and_then(parse_numeric_value)?;
                    Some((r.bar_width, val))
                })
                .collect();

            if pairs.len() >= 2 {
                // Find the row with the max value — it defines the scale
                let max_val = pairs
                    .iter()
                    .map(|(_, v)| *v)
                    .fold(f64::NEG_INFINITY, f64::max);
                let max_bar = pairs
                    .iter()
                    .filter(|(_, v)| (*v - max_val).abs() < 0.01)
                    .map(|(w, _)| *w)
                    .max()
                    .unwrap_or(1);

                for (row, (bar_w, val)) in chart
                    .rows
                    .iter()
                    .filter(|r| r.value.is_some())
                    .zip(pairs.iter())
                {
                    let expected = (max_bar as f64 * val / max_val).round() as usize;
                    let drift = bar_w.abs_diff(expected);
                    if drift > config.proportionality_tolerance {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(),
                            row.line,
                            1,
                            "ascii_barchart_scale",
                            format!(
                                "bar width {} for value {} is disproportionate — \
                                 expected ~{} chars (scale: {} → {} chars), off by {}",
                                bar_w, val, expected, max_val, max_bar, drift
                            ),
                        ));
                    }
                }
            }
        }

        // Check 6: value column alignment
        if config.require_value_alignment {
            let cols: Vec<usize> = chart
                .rows
                .iter()
                .filter(|r| r.value.is_some())
                .map(|r| r.value_col)
                .collect();
            if cols.len() >= 2 {
                let max_col = *cols.iter().max().unwrap();
                for row in chart.rows.iter().filter(|r| r.value.is_some()) {
                    let drift = max_col.abs_diff(row.value_col);
                    if drift > config.alignment_tolerance {
                        diags.push(Diagnostic::warning(
                            path.to_path_buf(), row.line, row.value_col,
                            "ascii_barchart_align",
                            format!(
                                "value {:?} starts at col {} — other values start at col {} (drift {})",
                                row.value.as_deref().unwrap_or(""),
                                row.value_col, max_col, drift
                            ),
                        ));
                    }
                }
            }
        }
    }

    diags
}

// ─────────────────────────────────────────────────────────
// Value format detection
// ─────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum ValueFormat {
    Percentage,   // "78%"
    Integer,      // "42"
    Float,        // "3.14"
    TimeDuration, // "1.5s", "200ms"
    Other(String),
}

/// Parse the numeric part of a value for proportionality checking.
fn parse_numeric_value(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with('%') {
        return s[..s.len() - 1].trim().parse().ok();
    }
    if s.ends_with("ms") {
        return s[..s.len() - 2].trim().parse().ok();
    }
    if s.ends_with('s') {
        return s[..s.len() - 1].trim().parse().ok();
    }
    s.parse().ok()
}

fn detect_value_format(s: &str) -> ValueFormat {
    let s = s.trim();
    if s.ends_with('%') && s[..s.len() - 1].parse::<f64>().is_ok() {
        return ValueFormat::Percentage;
    }
    if (s.ends_with("ms") && s[..s.len() - 2].trim().parse::<f64>().is_ok())
        || ((s.ends_with('s') || s.ends_with('m'))
            && s[..s.len() - 1].trim().parse::<f64>().is_ok())
    {
        return ValueFormat::TimeDuration;
    }
    if s.parse::<i64>().is_ok() {
        return ValueFormat::Integer;
    }
    if s.parse::<f64>().is_ok() {
        return ValueFormat::Float;
    }
    ValueFormat::Other(s.to_string())
}

// ─────────────────────────────────────────────────────────
// Code block mask
// ─────────────────────────────────────────────────────────

fn code_block_mask(lines: &[&str]) -> Vec<bool> {
    let mut mask = vec![false; lines.len()];
    let mut in_block = false;
    let mut eligible_block = false;
    let mut fence_char = '`';
    let mut fence_len = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !in_block {
            let ch = trimmed.chars().next();
            if matches!(ch, Some('`') | Some('~')) {
                let c = ch.unwrap();
                let run = trimmed.chars().take_while(|&x| x == c).count();
                if run >= 3 {
                    in_block = true;
                    eligible_block = is_barchart_fence(trimmed[run..].trim());
                    fence_char = c;
                    fence_len = run;
                }
            }
        } else {
            let ch = trimmed.chars().next();
            if ch == Some(fence_char) {
                let run = trimmed.chars().take_while(|&x| x == fence_char).count();
                if run >= fence_len {
                    in_block = false;
                    eligible_block = false;
                    continue;
                }
            }
            if eligible_block {
                mask[i] = true;
            }
        }
    }
    mask
}

fn is_barchart_fence(info: &str) -> bool {
    info.is_empty()
        || matches!(
            info.split_whitespace().next(),
            Some("text" | "txt" | "ascii" | "diagram" | "chart" | "barchart")
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AsciiBarchartConfig;

    fn check() -> AsciiBarchartCheck {
        AsciiBarchartCheck {
            config: AsciiBarchartConfig::default(),
        }
    }

    #[test]
    fn perfect_barchart_no_errors() {
        let content = "```\nItem A  █████████████████████ 78%\nItem B  ████████████          45%\nItem C  ███                   12%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "clean bar chart must produce no diagnostics: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn inconsistent_bar_char_detected() {
        // Row 2 uses ▓ instead of █
        let content = "```\nItem A  █████████████ 78%\nItem B  ▓▓▓▓▓▓▓▓▓▓▓▓▓ 45%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "ascii_barchart_char"),
            "mixed bar chars must be detected"
        );
    }

    #[test]
    fn misaligned_values_detected() {
        // Item A value at col 25, Item B at col 20 — drift > tolerance
        let content = "```\nItem A  ████████████████ 78%\nItem B  ████  45%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "ascii_barchart_align"),
            "misaligned values must be detected"
        );
    }

    #[test]
    fn inconsistent_value_format_detected() {
        // Row 1 uses %, row 2 uses integer
        let content = "```\nOption A  █████████████ 78%\nOption B  █████████     45\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "ascii_barchart_value"),
            "inconsistent value format must be detected"
        );
    }

    #[test]
    fn single_row_not_detected_as_chart() {
        // Only one bar row — below min_chart_rows (2)
        let content = "```\nOnly row  ████████████ 100%\n```";
        let lines: Vec<&str> = content.lines().collect();
        let mask = code_block_mask(&lines);
        let charts = detect_charts(&lines, &mask, &AsciiBarchartConfig::default());
        assert!(
            charts.is_empty(),
            "single bar row must not be detected as a chart"
        );
    }

    #[test]
    fn no_bar_chars_outside_code_block() {
        // Bar chars in prose — must not trigger
        let content = "Some text: █████ this is not a chart.\nMore text: ████████ still not.\n";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "bar chars outside code block must not trigger"
        );
    }

    #[test]
    fn hash_bars_supported() {
        // Using # as bar char (ASCII-only bar chart)
        let content = "```\nItem A  ############ 60%\nItem B  #######       35%\n```";
        let diags = check().check(Path::new("t.md"), content);
        // Should detect as a chart and validate
        let _ = diags; // just verify no panic
    }

    #[test]
    fn boxed_diagram_bars_are_not_barcharts() {
        let content = "```\n│  │  ████    │                      │  ██      │  │\n│  │  ██████  │                      │  ████    │  │\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "boxed multi-panel diagrams are not barcharts"
        );
    }

    #[test]
    fn adjacent_pattern_runs_are_not_barcharts() {
        let content = "```\nFIGURE-GROUND      COMMON FATE\n####.....          * * * * →  these dots\n####.....          * * * * →  moving together\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "adjacent pattern fills are not chart bars"
        );
    }

    #[test]
    fn multi_run_texture_rows_are_not_barcharts() {
        let content = "```\nCONVENTIONAL MAP:              NOLLI READING:\n████ ████ ████                 ████ ░░░░ ████\n████ ░░░░ ████                 ████ ████ ████\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "texture/map rows with repeated bar runs are not barcharts"
        );
    }

    #[test]
    fn equation_rows_are_not_barcharts() {
        let content = "```\nF phi  ===  top U phi\nG phi  ===  phi R bot\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(diags.is_empty(), "equation operators are not barchart bars");
    }

    #[test]
    fn axis_attached_bars_are_not_padding_linted_as_barcharts() {
        let content = "```\nl(x)\n100K|████████████████████\n    |                   ██████\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.is_empty(),
            "axis-attached bars are not label/value barcharts"
        );
    }

    #[test]
    fn stacked_bars_allow_different_fill_starts() {
        let content = "```\nHIGH  ████████░░░░ 80%\nLOW   ░░░░████████ 20%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            !diags.iter().any(|d| d.code == "ascii_barchart_char"),
            "stacked bars may begin with different fill chars"
        );
    }

    #[test]
    fn proportional_bars_pass() {
        // 78% → 23 chars, 45% → 14 chars, 12% → 4 chars (all proportional to 78%→23)
        let content = "```\nItem A  ███████████████████████        78%\nItem B  ██████████████                 45%\nItem C  ████                           12%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            !diags.iter().any(|d| d.code == "ascii_barchart_scale"),
            "proportional bars must not trigger scale warning"
        );
    }

    #[test]
    fn disproportionate_bar_detected() {
        // Item A at 78% has 30-char bar (fills 100%), but 78% should → ~23 chars
        // Scale: max value 78% → 30 chars. Item B 45% → expected 17, actual 13 → flagged
        let content = "```\nItem A  ██████████████████████████████ 78%\nItem B  █████████████                  45%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            diags.iter().any(|d| d.code == "ascii_barchart_scale"),
            "disproportionate bar (45% at wrong width) must be flagged"
        );
    }

    #[test]
    fn value_formats_consistent_passes() {
        // All percentages
        let content = "```\nA  ████████████████████ 80%\nB  █████████████       52%\nC  ████                19%\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            !diags.iter().any(|d| d.code == "ascii_barchart_value"),
            "consistent percentage values must not produce format errors"
        );
    }

    #[test]
    fn programming_fences_are_not_barcharts() {
        let content = "```javascript\n1 === 1         // true\n\"1\" === 1       // false\n```\n\nAfterward prose.";
        let diags = check().check(Path::new("t.md"), content);
        assert!(diags.is_empty(), "source-code fences are not bar charts");
    }

    #[test]
    fn prose_values_ending_in_s_are_not_durations() {
        let content = "```\nA  ████████░░  strong     ReLU activation (rate-like)\nB  ██████░░░░  moderate   Hebb-style update in SNNs\n```";
        let diags = check().check(Path::new("t.md"), content);
        assert!(
            !diags.iter().any(|d| d.code == "ascii_barchart_value"),
            "prose values ending in s are not time durations"
        );
    }
}
