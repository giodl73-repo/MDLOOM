//! Tier 3: matrix and cases environments.

use super::fraction::{center_in, left_align_in, right_align_in, RenderedExpr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatrixKind {
    Pmatrix,  // ⎛ ⎜ ⎝  ⎞ ⎟ ⎠
    Bmatrix,  // ⎡ ⎢ ⎣  ⎤ ⎥ ⎦
    Matrix,   // no delimiters
    Vmatrix,  // | on each side
    Vmatrix2, // ‖ on each side
    Cases,    // ⎧ ⎨ ⎩ on left only
}

impl MatrixKind {
    pub fn from_env(env: &str) -> Option<Self> {
        match env {
            "pmatrix" => Some(MatrixKind::Pmatrix),
            "bmatrix" => Some(MatrixKind::Bmatrix),
            "matrix" => Some(MatrixKind::Matrix),
            "vmatrix" => Some(MatrixKind::Vmatrix),
            "Vmatrix" => Some(MatrixKind::Vmatrix2),
            "cases" => Some(MatrixKind::Cases),
            _ => None,
        }
    }
}

/// Render a matrix or cases environment.
///
/// `rows` is a list of rows; each row is a list of cell contents.
/// Ragged rows are padded with empty strings on the right (M-7).
pub fn render_matrix(kind: MatrixKind, rows: Vec<Vec<String>>) -> RenderedExpr {
    if rows.is_empty() {
        return RenderedExpr::empty();
    }

    // Normalize: all rows to same column count (pad with empty)
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|mut row| {
            while row.len() < max_cols {
                row.push(String::new());
            }
            row
        })
        .collect();

    let n_rows = rows.len();
    let n_cols = max_cols;

    if n_cols == 0 {
        return RenderedExpr::empty();
    }

    // Compute column widths
    let col_widths: Vec<usize> = (0..n_cols)
        .map(|c| {
            rows.iter()
                .map(|row| crate::visual_width(&row[c]))
                .max()
                .unwrap_or(0)
        })
        .collect();

    // Determine justification per column: if ALL cells in the column are numeric → right-align
    let col_justify: Vec<bool> = (0..n_cols)
        .map(|c| {
            // true = right-align (numeric), false = left-align (text)
            rows.iter().all(|row| is_numeric(&row[c]))
        })
        .collect();

    // Render each row as a string of cells joined by "  "
    let cell_sep = "  ";
    let cell_sep_width = crate::visual_width(cell_sep);
    let content_width: usize =
        col_widths.iter().sum::<usize>() + cell_sep_width * n_cols.saturating_sub(1);

    let rendered_rows: Vec<String> = rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(c, cell)| {
                    let w = col_widths[c];
                    if col_justify[c] {
                        right_align_in(cell, w)
                    } else {
                        left_align_in(cell, w)
                    }
                })
                .collect();
            cells.join(cell_sep)
        })
        .collect();

    // Build final lines with delimiters
    let (left_top, left_mid, left_bot, right_top, right_mid, right_bot) = delimiters(kind);

    let mut lines: Vec<String> = Vec::with_capacity(n_rows);

    for (i, row_str) in rendered_rows.iter().enumerate() {
        let (left, right) = if n_rows == 1 {
            // Single row: use top glyphs only
            (left_top, right_top)
        } else if i == 0 {
            (left_top, right_top)
        } else if i == n_rows - 1 {
            (left_bot, right_bot)
        } else {
            (left_mid, right_mid)
        };

        match kind {
            MatrixKind::Matrix => {
                lines.push(row_str.clone());
            }
            MatrixKind::Cases => {
                lines.push(format!("{} {}", left, row_str));
            }
            _ => {
                lines.push(format!("{} {} {}", left, row_str, right));
            }
        }
    }

    let total_width = match kind {
        MatrixKind::Matrix => content_width,
        MatrixKind::Cases => 2 + content_width, // "⎧ " prefix
        _ => 2 + content_width + 3,             // "⎛ " + content + " ⎞"
    };

    let baseline = n_rows / 2;

    RenderedExpr {
        lines,
        width: total_width,
        baseline,
    }
}

fn is_numeric(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'))
}

fn delimiters(
    kind: MatrixKind,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    //                  left_top  left_mid  left_bot  right_top right_mid right_bot
    match kind {
        MatrixKind::Pmatrix => ("⎛", "⎜", "⎝", "⎞", "⎟", "⎠"),
        MatrixKind::Bmatrix => ("⎡", "⎢", "⎣", "⎤", "⎥", "⎦"),
        MatrixKind::Matrix => ("", "", "", "", "", ""),
        MatrixKind::Vmatrix => ("|", "|", "|", "|", "|", "|"),
        MatrixKind::Vmatrix2 => ("‖", "‖", "‖", "‖", "‖", "‖"),
        MatrixKind::Cases => ("⎧", "⎨", "⎩", "", "", ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn pmatrix_2x2() {
        let rows = vec![vec![s("a"), s("b")], vec![s("c"), s("d")]];
        let r = render_matrix(MatrixKind::Pmatrix, rows);
        assert_eq!(r.lines.len(), 2);
        assert!(r.lines[0].starts_with("⎛"));
        assert!(r.lines[1].starts_with("⎝"));
        assert!(r.lines[0].contains("a"));
        assert!(r.lines[0].contains("b"));
    }

    #[test]
    fn pmatrix_3x3() {
        let rows = vec![
            vec![s("1"), s("2"), s("3")],
            vec![s("4"), s("5"), s("6")],
            vec![s("7"), s("8"), s("9")],
        ];
        let r = render_matrix(MatrixKind::Pmatrix, rows);
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[0].starts_with("⎛"));
        assert!(r.lines[1].starts_with("⎜"));
        assert!(r.lines[2].starts_with("⎝"));
    }

    #[test]
    fn ragged_matrix_padded() {
        // Row 1: 2 cols, Row 2: 1 col → Row 2 gets padded to 2 cols
        let rows = vec![vec![s("a"), s("b")], vec![s("c")]];
        let r = render_matrix(MatrixKind::Matrix, rows);
        assert_eq!(r.lines.len(), 2);
        // second row should have same width as first row
        assert_eq!(
            crate::visual_width(&r.lines[0]),
            crate::visual_width(&r.lines[1])
        );
    }

    #[test]
    fn bmatrix_delimiters() {
        let rows = vec![vec![s("x"), s("y")]];
        let r = render_matrix(MatrixKind::Bmatrix, rows);
        assert!(r.lines[0].starts_with("⎡"));
        assert!(r.lines[0].ends_with("⎤"));
    }

    #[test]
    fn cases_2_rows() {
        let rows = vec![vec![s("n+1  if n odd")], vec![s("n/2  if n even")]];
        let r = render_matrix(MatrixKind::Cases, rows);
        assert_eq!(r.lines.len(), 2);
        assert!(r.lines[0].starts_with("⎧"));
        assert!(r.lines[1].starts_with("⎩"));
    }

    #[test]
    fn cases_3_rows() {
        let rows = vec![vec![s("a")], vec![s("b")], vec![s("c")]];
        let r = render_matrix(MatrixKind::Cases, rows);
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[0].starts_with("⎧"));
        assert!(r.lines[1].starts_with("⎨"));
        assert!(r.lines[2].starts_with("⎩"));
    }

    #[test]
    fn matrix_no_delimiters() {
        let rows = vec![vec![s("a"), s("b")]];
        let r = render_matrix(MatrixKind::Matrix, rows);
        assert!(!r.lines[0].contains("⎛"));
        assert!(!r.lines[0].contains("|"));
    }

    #[test]
    fn baseline_is_middle_row() {
        let rows = vec![vec![s("a")], vec![s("b")], vec![s("c")]];
        let r = render_matrix(MatrixKind::Pmatrix, rows);
        assert_eq!(r.baseline, 1);
    }

    #[test]
    fn vmatrix_delimiters() {
        let rows = vec![vec![s("a"), s("b")], vec![s("c"), s("d")]];
        let r = render_matrix(MatrixKind::Vmatrix, rows);
        assert!(r.lines[0].starts_with("|"));
        assert!(r.lines[0].ends_with("|"));
    }

    #[test]
    fn vmatrix2_delimiters() {
        let rows = vec![vec![s("a"), s("b")], vec![s("c"), s("d")]];
        let r = render_matrix(MatrixKind::Vmatrix2, rows);
        assert!(r.lines[0].starts_with("‖"));
        assert!(r.lines[0].ends_with("‖"));
    }

    #[test]
    fn single_row_uses_top_glyph_only() {
        // Single-row pmatrix: top glyph for both sides (no mid/bot)
        let rows = vec![vec![s("x"), s("y")]];
        let r = render_matrix(MatrixKind::Pmatrix, rows);
        assert_eq!(r.lines.len(), 1);
        assert!(r.lines[0].starts_with("⎛"));
        assert!(r.lines[0].ends_with("⎞"));
    }

    #[test]
    fn from_env_all_variants() {
        assert!(matches!(
            MatrixKind::from_env("pmatrix"),
            Some(MatrixKind::Pmatrix)
        ));
        assert!(matches!(
            MatrixKind::from_env("bmatrix"),
            Some(MatrixKind::Bmatrix)
        ));
        assert!(matches!(
            MatrixKind::from_env("matrix"),
            Some(MatrixKind::Matrix)
        ));
        assert!(matches!(
            MatrixKind::from_env("vmatrix"),
            Some(MatrixKind::Vmatrix)
        ));
        assert!(matches!(
            MatrixKind::from_env("Vmatrix"),
            Some(MatrixKind::Vmatrix2)
        ));
        assert!(matches!(
            MatrixKind::from_env("cases"),
            Some(MatrixKind::Cases)
        ));
        assert!(MatrixKind::from_env("unknown").is_none());
    }

    #[test]
    fn empty_matrix_no_panic() {
        let r = render_matrix(MatrixKind::Pmatrix, vec![]);
        // Empty matrix → RenderedExpr::empty() — no panic
        assert!(r.lines.is_empty() || r.lines.iter().all(|l| l.is_empty()));
    }
}
