/// proof spec generate — analyze a resolved figure and suggest DaVinci invariants.
///
/// Works entirely offline: uses proof's own detection infrastructure to inspect
/// the figure's structure (boxes, widths, line count, labels) and emit a ready-to-paste
/// [[davinci]] TOML block with commented rationale.
use crate::layout::visual_width;

// ─────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────

pub struct SpecSuggestion {
    pub id: String,
    pub uri: String,
    pub protection: String,
    pub invariants: Vec<InvariantSuggestion>,
}

pub struct InvariantSuggestion {
    pub rule: String,
    pub params: InvariantParams,
    pub rationale: String,
    pub confidence: SuggestionConfidence,
}

#[derive(Debug)]
pub enum InvariantParams {
    Text {
        value: String,
    },
    MinMax {
        min: Option<usize>,
        max: Option<usize>,
    },
    Exact {
        value: usize,
    },
    Values {
        values: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SuggestionConfidence {
    High,
    Medium,
    Low,
}

impl SuggestionConfidence {
    pub fn label(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

// ─────────────────────────────────────────────────────────
// Core analysis
// ─────────────────────────────────────────────────────────

/// Analyze figure content and return suggested invariants.
///
/// `content` is the raw figure text (may or may not be fenced).
/// `label` is the figure label from mdpath (e.g. "GOROUTINE SCHEDULER — M:N multiplexing").
/// `uri` is the full md:// URI.
/// `id` is the requested invariant ID (derived from URI if not provided).
pub fn generate(content: &str, label: Option<&str>, uri: &str, id: &str) -> SpecSuggestion {
    let lines = extract_content_lines(content);
    let line_count = lines.len();

    let mut invariants: Vec<InvariantSuggestion> = Vec::new();

    // ── 1. line-count (always high confidence) ──────────────────────────
    let slack = (line_count / 5).max(2); // ±20% or ±2, whichever larger
    invariants.push(InvariantSuggestion {
        rule: "line-count".to_string(),
        params: InvariantParams::MinMax {
            min: Some(line_count.saturating_sub(slack)),
            max: Some(line_count + slack),
        },
        rationale: format!(
            "current figure is {} lines; range allows ±{} lines of growth/shrink",
            line_count, slack
        ),
        confidence: SuggestionConfidence::High,
    });

    // ── 2. contains-text for the label / first distinctive phrase ────────
    if let Some(lbl) = label {
        let phrase = pick_distinctive_phrase(lbl);
        if !phrase.is_empty() {
            invariants.push(InvariantSuggestion {
                rule: "contains-text".to_string(),
                params: InvariantParams::Text {
                    value: phrase.to_string(),
                },
                rationale: format!(
                    "figure label {:?} — this phrase identifies the figure's purpose",
                    lbl
                ),
                confidence: SuggestionConfidence::High,
            });
        }
    }

    // ── 3. contains-text for all-caps identifiers in content ─────────────
    let caps_phrases = find_allcaps_phrases(&lines);
    for phrase in caps_phrases.iter().take(2) {
        // Don't duplicate the label phrase
        let already_added = invariants
            .iter()
            .any(|inv| matches!(&inv.params, InvariantParams::Text { value } if value == phrase));
        if !already_added {
            invariants.push(InvariantSuggestion {
                rule: "contains-text".to_string(),
                params: InvariantParams::Text {
                    value: phrase.clone(),
                },
                rationale: format!(
                    "all-caps identifier {:?} anchors the figure's key concept",
                    phrase
                ),
                confidence: SuggestionConfidence::Medium,
            });
        }
    }

    // ── 4. box-count ─────────────────────────────────────────────────────
    let box_count = count_boxes(&lines);
    if box_count > 0 {
        invariants.push(InvariantSuggestion {
            rule: "box-count".to_string(),
            params: InvariantParams::MinMax {
                min: Some(box_count),
                max: None,
            },
            rationale: format!(
                "figure contains {} box{} — structural minimum",
                box_count,
                if box_count == 1 { "" } else { "es" }
            ),
            confidence: SuggestionConfidence::High,
        });
    }

    // ── 5. box-width for single-column boxes ─────────────────────────────
    let widths = detect_box_widths(&lines);
    if !widths.is_empty() && widths.len() <= 2 {
        // Only suggest width for simple diagrams (1-2 distinct widths)
        let min_w = *widths.iter().min().unwrap();
        let max_w = *widths.iter().max().unwrap();
        invariants.push(InvariantSuggestion {
            rule: "box-width".to_string(),
            params: InvariantParams::MinMax {
                min: Some(min_w.saturating_sub(2)),
                max: Some(max_w + 2),
            },
            rationale: format!(
                "box border width is {} col{}; ±2 tolerance for content changes",
                if min_w == max_w {
                    format!("{}", min_w)
                } else {
                    format!("{}-{}", min_w, max_w)
                },
                if min_w == 1 { "" } else { "s" }
            ),
            confidence: SuggestionConfidence::Medium,
        });
    }

    // ── 6. column-count for multi-column boxes ───────────────────────────
    let col_count = detect_column_count(&lines);
    if col_count >= 2 {
        invariants.push(InvariantSuggestion {
            rule: "column-count".to_string(),
            params: InvariantParams::Exact { value: col_count },
            rationale: format!(
                "figure has {} column separators per row — structural layout invariant",
                col_count
            ),
            confidence: SuggestionConfidence::High,
        });
    }

    // ── 7. required-row-keys for table-like boxes ────────────────────────
    let row_keys = detect_row_keys(&lines);
    if row_keys.len() >= 3 {
        invariants.push(InvariantSuggestion {
            rule: "required-row-keys".to_string(),
            params: InvariantParams::Values {
                values: row_keys.clone(),
            },
            rationale: format!(
                "table has {} rows with stable key column values",
                row_keys.len()
            ),
            confidence: SuggestionConfidence::Medium,
        });
    }

    SpecSuggestion {
        id: id.to_string(),
        uri: uri.to_string(),
        protection: "error".to_string(),
        invariants,
    }
}

/// Format a SpecSuggestion as a TOML block ready to paste into proof.toml.
pub fn format_toml(spec: &SpecSuggestion) -> String {
    let mut out = String::new();

    out.push_str(
        &"# Generated by `proof spec generate` — review and adjust before committing\n".to_string(),
    );
    out.push_str(
        &"# Confidence: high = structural anchor, medium = heuristic, low = brittle\n\n"
            .to_string(),
    );
    out.push_str("[[davinci]]\n");
    out.push_str(&format!("id = {:?}\n", spec.id));
    out.push_str(&format!("uri = {:?}\n", spec.uri));
    out.push_str(&format!("protection = {:?}\n", spec.protection));

    for inv in &spec.invariants {
        out.push('\n');
        out.push_str(&format!(
            "  # [{}] {}\n",
            inv.confidence.label(),
            inv.rationale
        ));
        out.push_str("  [[davinci.invariants]]\n");
        out.push_str(&format!("  rule = {:?}\n", inv.rule));
        match &inv.params {
            InvariantParams::Text { value } => {
                // The Invariant struct uses `text` for string params (not `value`)
                out.push_str(&format!("  text = {:?}\n", value));
            }
            InvariantParams::MinMax { min, max } => {
                if let Some(v) = min {
                    out.push_str(&format!("  min = {}\n", v));
                }
                if let Some(v) = max {
                    out.push_str(&format!("  max = {}\n", v));
                }
            }
            InvariantParams::Exact { value } => {
                out.push_str(&format!("  value = {}\n", value));
            }
            InvariantParams::Values { values } => {
                out.push_str("  values = [\n");
                for v in values {
                    out.push_str(&format!("    {:?},\n", v));
                }
                out.push_str("  ]\n");
            }
        }
    }

    out
}

// ─────────────────────────────────────────────────────────
// Analysis helpers
// ─────────────────────────────────────────────────────────

fn extract_content_lines(content: &str) -> Vec<String> {
    let raw: Vec<&str> = content.lines().collect();
    if raw.is_empty() {
        return vec![];
    }
    let first = raw[0].trim();
    if (first.starts_with("```") || first.starts_with("~~~")) && raw.len() >= 2 {
        let last = raw[raw.len() - 1].trim();
        if last.starts_with("```") || last.starts_with("~~~") {
            return raw[1..raw.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
    }
    raw.iter().map(|s| s.to_string()).collect()
}

/// Pick the most distinctive short phrase from a label string.
/// Prefers the first em-dash segment, then the first word sequence.
fn pick_distinctive_phrase(label: &str) -> &str {
    // Try to extract a portion before "—" or "–"
    if let Some(pos) = label.find('—').or_else(|| label.find('–')) {
        let before = label[..pos].trim();
        if before.len() >= 4 {
            return before;
        }
    }
    // Otherwise use the whole label up to 40 chars
    let trimmed = label.trim();
    if trimmed.len() > 40 {
        // Find last space before col 40
        if let Some(space) = trimmed[..40].rfind(' ') {
            return &trimmed[..space];
        }
        return &trimmed[..40];
    }
    trimmed
}

/// Find all-caps phrases (≥4 chars) that appear in the figure content.
/// These are strong structural anchors (section headers, diagram labels).
fn find_allcaps_phrases(lines: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();

    for line in lines {
        // Find sequences of uppercase letters/spaces (at least 4 chars)
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut run: Vec<&str> = Vec::new();

        for word in &words {
            // Strip trailing punctuation for the alpha check
            let stripped: String = word
                .chars()
                .filter(|c| c.is_alphabetic() || *c == '/')
                .collect();
            let alpha: String = stripped.chars().filter(|c| c.is_alphabetic()).collect();
            if !alpha.is_empty() && alpha == alpha.to_uppercase() && alpha.len() >= 2 {
                // Use the stripped version (no trailing parens/colons) in the phrase
                run.push(word);
            } else {
                if !run.is_empty() {
                    // Build phrase, stripping trailing non-alpha from the last word
                    let last = run.last().unwrap();
                    let last_clean = last.trim_end_matches(|c: char| !c.is_alphabetic());
                    let mut phrase_parts = run[..run.len() - 1].to_vec();
                    phrase_parts.push(last_clean);
                    let phrase = phrase_parts.join(" ");
                    if phrase.len() >= 4 && !seen.contains(&phrase) {
                        seen.insert(phrase.clone());
                        out.push(phrase);
                    }
                }
                run.clear();
            }
        }
        if !run.is_empty() {
            let last = run.last().unwrap();
            let last_clean = last.trim_end_matches(|c: char| !c.is_alphabetic());
            let mut phrase_parts = run[..run.len() - 1].to_vec();
            phrase_parts.push(last_clean);
            let phrase = phrase_parts.join(" ");
            if phrase.len() >= 4 && !seen.contains(&phrase) {
                seen.insert(phrase.clone());
                out.push(phrase);
            }
        }
    }

    out
}

/// Count the number of distinct boxes in the figure.
/// Counts each `┌` or `+—` occurrence on top-border lines.
fn count_boxes(lines: &[String]) -> usize {
    let mut total = 0;
    for line in lines {
        let t = line.trim();
        // Unicode box style: count each ┌ on a line that has matching ┐
        let unicode_tops = t
            .chars()
            .filter(|&c| c == '┌' || c == '╔' || c == '╒' || c == '╓')
            .count();
        let unicode_bots = t
            .chars()
            .filter(|&c| c == '┐' || c == '╗' || c == '╕' || c == '╖')
            .count();
        if unicode_tops > 0 && unicode_bots > 0 {
            total += unicode_tops;
            continue;
        }
        // ASCII style: +---+ — count each `+` pair
        if t.contains("+-") || t.contains("+─") {
            let plus_count = t.chars().filter(|&c| c == '+').count();
            if plus_count >= 2 {
                total += plus_count / 2;
            }
        }
    }
    total
}

/// Detect the visual width of box borders in the figure.
fn detect_box_widths(lines: &[String]) -> Vec<usize> {
    let mut widths = std::collections::BTreeSet::new();
    for line in lines {
        let t = line.trim();
        if (t.starts_with('┌') || t.starts_with('└') || t.starts_with('+'))
            && (t.ends_with('┐') || t.ends_with('┘') || t.ends_with('+'))
            && t.len() >= 4
        {
            widths.insert(visual_width(t));
        }
    }
    widths.into_iter().collect()
}

/// Detect the consistent number of column separators in content rows.
/// Returns the most common count (or 0 if inconsistent).
fn detect_column_count(lines: &[String]) -> usize {
    let mut counts: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for line in lines {
        let t = line.trim();
        // Content row: starts and ends with a bar
        if (t.starts_with('│') || t.starts_with('|'))
            && (t.ends_with('│') || t.ends_with('|'))
            && t.len() >= 4
        {
            let inner_bars = t.chars().filter(|&c| c == '│' || c == '|').count();
            // At least 2 bars (open + close) required; interior separators = inner_bars - 2 + 1 = inner_bars - 1
            if inner_bars >= 2 {
                *counts.entry(inner_bars - 1).or_default() += 1;
            }
        }
    }
    // Most common count
    counts
        .into_iter()
        .max_by_key(|&(_, v)| v)
        .map(|(k, _)| k)
        .unwrap_or(0)
}

/// Detect row key values from the first cell of each content row in a table-like box.
fn detect_row_keys(lines: &[String]) -> Vec<String> {
    let mut keys = Vec::new();
    for line in lines {
        let t = line.trim();
        // Must be a multi-column content row
        if !(t.starts_with('│') || t.starts_with('|')) {
            continue;
        }
        let bar_count = t.chars().filter(|&c| c == '│' || c == '|').count();
        if bar_count < 3 {
            continue;
        } // need at least 2 columns

        // Extract first cell content
        let without_first = if t.starts_with('│') {
            &t[3..] // '│' is 3 bytes
        } else {
            &t[1..]
        };

        let cell_end = without_first
            .find(['│', '|'])
            .unwrap_or(without_first.len());
        let cell = without_first[..cell_end].trim();

        // Only include non-empty, non-separator, non-code cells
        let looks_like_code = cell.contains(":=")
            || cell.contains("<-")
            || cell.contains("->")
            || cell.contains("//")
            || cell.starts_with("fn ")
            || cell.starts_with("def ")
            || cell.contains("()");
        if !cell.is_empty()
            && !cell
                .chars()
                .all(|c| c == '-' || c == '─' || c == '=' || c == '≡')
            && !looks_like_code
            && cell.len() <= 40
        // row keys shouldn't be full sentences
        {
            keys.push(cell.to_string());
        }
    }
    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    keys.retain(|k| seen.insert(k.clone()));
    keys
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const GOROUTINE_FIG: &str = r#"
GOROUTINE SCHEDULER — M:N multiplexing
┌──────────────────────────────────────┐
│  G  G  G  G  ← goroutines           │
│         M:N                          │
│  P  P  P  P  ← OS threads           │
└──────────────────────────────────────┘
"#;

    const TABLE_FIG: &str = r#"
┌─────────────────┬──────────────┐
│ Axis            │ Value        │
├─────────────────┼──────────────┤
│ Binding         │ Late         │
│ Typing          │ Static       │
│ Strength        │ Strong       │
│ Type system     │ Structural   │
└─────────────────┴──────────────┘
"#;

    #[test]
    fn test_generates_line_count() {
        let spec = generate(
            GOROUTINE_FIG,
            Some("GOROUTINE SCHEDULER — M:N multiplexing"),
            "md://fig.md#:0",
            "scheduler",
        );
        let line_count_inv = spec.invariants.iter().find(|i| i.rule == "line-count");
        assert!(line_count_inv.is_some(), "should suggest line-count");
        assert!(matches!(
            line_count_inv.unwrap().confidence,
            SuggestionConfidence::High
        ));
    }

    #[test]
    fn test_generates_contains_text_from_label() {
        let spec = generate(
            GOROUTINE_FIG,
            Some("GOROUTINE SCHEDULER — M:N multiplexing"),
            "md://fig.md#:0",
            "scheduler",
        );
        let ct = spec.invariants.iter().find(|i| i.rule == "contains-text");
        assert!(ct.is_some(), "should suggest contains-text from label");
        if let InvariantParams::Text { value } = &ct.unwrap().params {
            assert!(value.contains("GOROUTINE"), "should use label prefix");
        }
    }

    #[test]
    fn test_generates_box_count() {
        let spec = generate(GOROUTINE_FIG, None, "md://fig.md#:0", "scheduler");
        let bc = spec.invariants.iter().find(|i| i.rule == "box-count");
        assert!(bc.is_some(), "should detect box count");
        if let InvariantParams::MinMax { min, .. } = &bc.unwrap().params {
            assert_eq!(*min, Some(1));
        }
    }

    #[test]
    fn test_detects_column_count_for_table() {
        let spec = generate(TABLE_FIG, None, "md://table.md#:0", "type-table");
        let cc = spec.invariants.iter().find(|i| i.rule == "column-count");
        assert!(cc.is_some(), "should detect column count for table");
    }

    #[test]
    fn test_detects_row_keys_for_table() {
        let spec = generate(TABLE_FIG, None, "md://table.md#:0", "type-table");
        let rk = spec
            .invariants
            .iter()
            .find(|i| i.rule == "required-row-keys");
        assert!(rk.is_some(), "should suggest required-row-keys for table");
        if let InvariantParams::Values { values } = &rk.unwrap().params {
            assert!(values.contains(&"Binding".to_string()));
            assert!(values.contains(&"Typing".to_string()));
        }
    }

    #[test]
    fn test_format_toml_output() {
        let spec = generate(
            GOROUTINE_FIG,
            Some("GOROUTINE SCHEDULER — M:N multiplexing"),
            "md://fig.md#goroutine-scheduler:0",
            "scheduler",
        );
        let toml = format_toml(&spec);
        assert!(toml.contains("[[davinci]]"));
        assert!(toml.contains("id = \"scheduler\""));
        assert!(toml.contains("[[davinci.invariants]]"));
        assert!(toml.contains("# [high]"));
        assert!(toml.contains("rule = \"line-count\""));
    }

    #[test]
    fn test_pick_distinctive_phrase_with_emdash() {
        assert_eq!(
            pick_distinctive_phrase("GOROUTINE SCHEDULER — M:N multiplexing"),
            "GOROUTINE SCHEDULER"
        );
    }

    #[test]
    fn test_pick_distinctive_phrase_no_dash() {
        let result = pick_distinctive_phrase("THE BIG PICTURE");
        assert_eq!(result, "THE BIG PICTURE");
    }

    #[test]
    fn test_count_boxes_single() {
        let lines = vec![
            "┌────────┐".to_string(),
            "│ content │".to_string(),
            "└────────┘".to_string(),
        ];
        assert_eq!(count_boxes(&lines), 1);
    }

    #[test]
    fn test_count_boxes_multiple() {
        let lines = vec![
            "┌────┐ ┌────┐".to_string(),
            "│ A  │ │ B  │".to_string(),
            "└────┘ └────┘".to_string(),
            "┌────┐".to_string(),
            "│ C  │".to_string(),
            "└────┘".to_string(),
        ];
        assert_eq!(count_boxes(&lines), 3);
    }

    #[test]
    fn test_find_allcaps_phrases() {
        let lines = vec![
            "  GOROUTINE SCHEDULER — M:N multiplexing".to_string(),
            "  some lowercase text".to_string(),
            "  M:N architecture".to_string(),
        ];
        let phrases = find_allcaps_phrases(&lines);
        assert!(phrases.iter().any(|p| p.contains("GOROUTINE")));
    }
}
