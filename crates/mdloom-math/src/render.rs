//! Display math entry point: parse + render + width/align padding.

use super::fraction::{center_in, left_align_in, render_frac, RenderedExpr};
use super::integral::{render_int, render_sum_prod};
use super::matrix::{render_matrix, MatrixKind};
use super::superscript::{to_subscript, to_superscript};
use super::symbols::{is_font_command, is_unsupported, lookup_symbol};
use super::tokenizer::{self, DiagSeverity, MathDiag, Token};
use super::{consume_group, expand_tokens, tokens_to_plain};

/// Horizontal alignment for display math output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathAlign {
    Left,
    Center,
    Right,
}

const AUTO_WIDTH_CAP: usize = 200;

/// Render a display math expression.
/// `width=0` → auto (max line width of rendered output, capped at 200).
/// Returns (lines, diagnostics).
pub fn render_display_math(
    expr: &str,
    width: usize,
    align: MathAlign,
) -> (Vec<String>, Vec<MathDiag>) {
    let (tokens, mut diags) = tokenizer::tokenize(expr);
    let mut pos = 0;
    let rendered = render_expr(&tokens, &mut pos, &mut diags, false, 0);

    let auto_w = rendered
        .lines
        .iter()
        .map(|l| crate::visual_width(l))
        .max()
        .unwrap_or(0)
        .min(AUTO_WIDTH_CAP);

    let out_width = if width == 0 { auto_w } else { width };

    let lines: Vec<String> = rendered
        .lines
        .into_iter()
        .map(|line| {
            let lw = crate::visual_width(&line);
            if lw > out_width {
                // Clip with MATH-004
                diags.push(MathDiag {
                    code: "MATH-004",
                    severity: DiagSeverity::Warning,
                    col: 0,
                    message: format!(
                        "Math expression ({} cols) exceeds declared width ({}) — clipped",
                        lw, out_width
                    ),
                });
                // Clip to out_width visual columns
                clip_to_width(&line, out_width)
            } else if out_width == 0 {
                line
            } else {
                pad_to_width(&line, lw, out_width, align)
            }
        })
        .collect();

    (lines, diags)
}

fn pad_to_width(line: &str, lw: usize, width: usize, align: MathAlign) -> String {
    if lw >= width {
        return line.to_string();
    }
    let pad = width - lw;
    match align {
        MathAlign::Left => format!("{}{}", line, " ".repeat(pad)),
        MathAlign::Right => format!("{}{}", " ".repeat(pad), line),
        MathAlign::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), line, " ".repeat(right))
        }
    }
}

fn clip_to_width(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if w + cw > width {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out
}

/// Render tokens into a `RenderedExpr` for display context.
/// This is the recursive display-mode renderer (Tier 1+2+3 all allowed).
pub fn render_expr(
    tokens: &[Token],
    pos: &mut usize,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) -> RenderedExpr {
    // Collect all sub-expressions on this "level" into a single-line accumulation,
    // then combine with any multi-line sub-expressions found.
    // Strategy: build a Vec<RenderedExpr> horizontally and concatenate.
    let mut parts: Vec<RenderedExpr> = Vec::new();
    let mut text_buf = String::new();

    macro_rules! flush_text {
        () => {
            if !text_buf.is_empty() {
                parts.push(RenderedExpr::leaf(&text_buf));
                text_buf.clear();
            }
        };
    }

    while *pos < tokens.len() {
        match &tokens[*pos].clone() {
            Token::Char(c) => {
                text_buf.push(*c);
                *pos += 1;
            }
            Token::Superscript => {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                text_buf.push_str(&to_superscript(&s));
            }
            Token::Subscript => {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                text_buf.push_str(&to_subscript(&s));
            }
            Token::Group(inner) => {
                let inner = inner.clone();
                *pos += 1;
                let mut inner_pos = 0;
                let sub = render_expr(&inner, &mut inner_pos, diags, inline, base_col);
                if sub.lines.len() == 1 {
                    text_buf.push_str(&sub.lines[0]);
                } else {
                    flush_text!();
                    parts.push(sub);
                }
            }
            Token::Command(cmd) => {
                let cmd = cmd.clone();
                *pos += 1;
                let sub = render_display_command(&cmd, tokens, pos, diags, inline, base_col);
                if sub.lines.len() == 1 {
                    text_buf.push_str(&sub.lines[0]);
                } else {
                    flush_text!();
                    parts.push(sub);
                }
            }
            Token::BeginEnv(env) => {
                let env = env.clone();
                *pos += 1;
                flush_text!();
                let sub = render_display_env(&env, tokens, pos, diags, inline, base_col);
                parts.push(sub);
            }
            Token::EndEnv(env) => {
                // Stray \end{}
                diags.push(MathDiag {
                    code: "MATH-003",
                    severity: DiagSeverity::Error,
                    col: base_col,
                    message: format!("Unexpected \\end{{{}}}", env),
                });
                *pos += 1;
            }
            Token::OptArg(_) => {
                *pos += 1;
            }
        }
    }

    flush_text!();

    if parts.is_empty() {
        return RenderedExpr::empty();
    }
    if parts.len() == 1 {
        return parts.remove(0);
    }

    // Concatenate all parts horizontally
    hconcat(parts)
}

/// Horizontal concatenation of RenderedExprs — align on baseline.
fn hconcat(parts: Vec<RenderedExpr>) -> RenderedExpr {
    if parts.is_empty() {
        return RenderedExpr::empty();
    }
    let max_above = parts.iter().map(|p| p.baseline).max().unwrap_or(0);
    let max_below = parts
        .iter()
        .map(|p| p.lines.len() - 1 - p.baseline)
        .max()
        .unwrap_or(0);
    let total_lines = max_above + 1 + max_below;
    let total_width: usize = parts.iter().map(|p| p.width).sum();

    let mut lines: Vec<String> = vec![String::new(); total_lines];
    for part in &parts {
        let offset = max_above - part.baseline;
        let part_height = part.lines.len();
        for (i, line) in part.lines.iter().enumerate() {
            let row = offset + i;
            lines[row].push_str(line);
        }
        // Fill blank space above and below this part
        for line in lines.iter_mut().take(offset) {
            line.push_str(&" ".repeat(part.width));
        }
        for line in lines
            .iter_mut()
            .take(total_lines)
            .skip(offset + part_height)
        {
            line.push_str(&" ".repeat(part.width));
        }
    }

    RenderedExpr {
        width: total_width,
        baseline: max_above,
        lines,
    }
}

fn render_display_command(
    cmd: &str,
    tokens: &[Token],
    pos: &mut usize,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) -> RenderedExpr {
    // Prime
    if let Some(rest) = cmd.strip_prefix("prime") {
        let n: usize = rest.parse().unwrap_or(1);
        let sym = match n {
            1 => "′",
            2 => "″",
            _ => "‴",
        };
        return RenderedExpr::leaf(sym);
    }

    // Limit operators
    if matches!(cmd, "lim" | "max" | "min" | "sup" | "inf") {
        let mut s = cmd.to_string();
        if *pos < tokens.len() {
            if let Token::Subscript = &tokens[*pos] {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let sub = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                s.push_str(&format!("_({})", sub));
            }
        }
        if *pos < tokens.len() {
            if let Token::Superscript = &tokens[*pos] {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let sup = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                s.push_str(&format!("^({})", sup));
            }
        }
        return RenderedExpr::leaf(&s);
    }

    // \frac → Tier 3 stacked fraction
    if cmd == "frac" {
        let num_toks = consume_group(tokens, pos);
        let den_toks = consume_group(tokens, pos);
        let mut np = 0;
        let num = render_expr(&num_toks, &mut np, diags, false, base_col);
        let mut dp = 0;
        let den = render_expr(&den_toks, &mut dp, diags, false, base_col);
        return render_frac(num, den);
    }

    // \sqrt
    if cmd == "sqrt" {
        let root = if *pos < tokens.len() {
            if let Token::OptArg(inner) = &tokens[*pos] {
                let s = tokens_to_plain(inner);
                *pos += 1;
                Some(to_superscript(&s))
            } else {
                None
            }
        } else {
            None
        };
        let arg_toks = consume_group(tokens, pos);
        let s = tokens_to_plain(&arg_toks);
        let rendered = match root {
            Some(r) => format!("{}√{}", r, if s.len() > 1 { format!("({})", s) } else { s }),
            None => format!("√{}", if s.len() > 1 { format!("({})", s) } else { s }),
        };
        return RenderedExpr::leaf(&rendered);
    }

    // \int
    if cmd == "int" {
        return render_int_command(tokens, pos);
    }

    // \sum, \prod
    if cmd == "sum" {
        return render_sum_command('∑', tokens, pos);
    }
    if cmd == "prod" {
        return render_sum_command('∏', tokens, pos);
    }

    // Font commands: strip, pass content
    if is_font_command(cmd) {
        let arg_toks = consume_group(tokens, pos);
        if cmd == "text" {
            let s: String = arg_toks
                .iter()
                .map(|t| match t {
                    Token::Char(c) => c.to_string(),
                    Token::Command(c) => lookup_symbol(c).unwrap_or("").to_string(),
                    _ => String::new(),
                })
                .collect();
            return RenderedExpr::leaf(&s);
        }
        let mut ip = 0;
        return render_expr(&arg_toks, &mut ip, diags, false, base_col);
    }

    // Unsupported
    if is_unsupported(cmd) {
        diags.push(MathDiag {
            code: "MATH-001",
            severity: DiagSeverity::Warning,
            col: base_col,
            message: format!("Unsupported command `\\{}` — passed through", cmd),
        });
        return RenderedExpr::leaf(&format!("\\{}", cmd));
    }

    // Tier 1 symbol
    if let Some(sym) = lookup_symbol(cmd) {
        return RenderedExpr::leaf(sym);
    }

    // Unknown
    diags.push(MathDiag {
        code: "MATH-001",
        severity: DiagSeverity::Warning,
        col: base_col,
        message: format!("Unknown command `\\{}` — passed through", cmd),
    });
    RenderedExpr::leaf(&format!("\\{}", cmd))
}

fn render_int_command(tokens: &[Token], pos: &mut usize) -> RenderedExpr {
    let (lower_toks, upper_toks) = consume_limit_args(tokens, pos);
    // Everything remaining until end of tokens is the integrand
    let mut integrand_text = String::new();
    let save = *pos;
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Char(c) => {
                integrand_text.push(*c);
                *pos += 1;
            }
            Token::Command(c) => {
                if let Some(s) = lookup_symbol(c) {
                    integrand_text.push_str(s);
                } else {
                    integrand_text.push_str(&format!("\\{}", c));
                }
                *pos += 1;
            }
            _ => {
                *pos += 1;
            }
        }
    }
    let integrand = RenderedExpr::leaf(integrand_text.trim());
    let lower = lower_toks.map(|t| RenderedExpr::leaf(&tokens_to_plain(&t)));
    let upper = upper_toks.map(|t| RenderedExpr::leaf(&tokens_to_plain(&t)));
    render_int(lower, upper, integrand)
}

fn render_sum_command(op: char, tokens: &[Token], pos: &mut usize) -> RenderedExpr {
    let (lower_toks, upper_toks) = consume_limit_args(tokens, pos);
    let mut body_text = String::new();
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Char(c) => {
                body_text.push(*c);
                *pos += 1;
            }
            Token::Command(c) => {
                if let Some(s) = lookup_symbol(c) {
                    body_text.push_str(s);
                } else {
                    body_text.push_str(&format!("\\{}", c));
                }
                *pos += 1;
            }
            _ => {
                *pos += 1;
            }
        }
    }
    let body = RenderedExpr::leaf(body_text.trim());
    let lower = lower_toks.map(|t| RenderedExpr::leaf(&tokens_to_plain(&t)));
    let upper = upper_toks.map(|t| RenderedExpr::leaf(&tokens_to_plain(&t)));
    render_sum_prod(op, lower, upper, body)
}

/// Consume optional `_{}` and `^{}` in any order after an operator.
fn consume_limit_args(
    tokens: &[Token],
    pos: &mut usize,
) -> (Option<Vec<Token>>, Option<Vec<Token>>) {
    let mut lower = None;
    let mut upper = None;
    for _ in 0..2 {
        if *pos >= tokens.len() {
            break;
        }
        match &tokens[*pos] {
            Token::Subscript => {
                *pos += 1;
                lower = tokenizer::consume_one(tokens, pos);
            }
            Token::Superscript => {
                *pos += 1;
                upper = tokenizer::consume_one(tokens, pos);
            }
            _ => break,
        }
    }
    (lower, upper)
}

fn render_display_env(
    env: &str,
    tokens: &[Token],
    pos: &mut usize,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) -> RenderedExpr {
    // Collect all tokens until matching \end{env}
    let mut body: Vec<Token> = Vec::new();
    let mut depth = 1usize;
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::BeginEnv(_) => {
                depth += 1;
                body.push(tokens[*pos].clone());
                *pos += 1;
            }
            Token::EndEnv(e) => {
                depth -= 1;
                if depth == 0 {
                    if e != env {
                        diags.push(MathDiag {
                            code: "MATH-003",
                            severity: DiagSeverity::Error,
                            col: base_col,
                            message: format!("Mismatched \\begin{{{}}} ... \\end{{{}}}", env, e),
                        });
                    }
                    *pos += 1;
                    break;
                }
                body.push(tokens[*pos].clone());
                *pos += 1;
            }
            _ => {
                body.push(tokens[*pos].clone());
                *pos += 1;
            }
        }
    }
    if depth > 0 {
        diags.push(MathDiag {
            code: "MATH-003",
            severity: DiagSeverity::Error,
            col: base_col,
            message: format!("Unclosed \\begin{{{}}}", env),
        });
    }

    let kind = match MatrixKind::from_env(env) {
        Some(k) => k,
        None => {
            diags.push(MathDiag {
                code: "MATH-001",
                severity: DiagSeverity::Warning,
                col: base_col,
                message: format!("Unknown environment `{}` — passed through", env),
            });
            return RenderedExpr::leaf(&format!("[{}]", env));
        }
    };

    // Parse body into rows: split on \\ and then on &
    let rows = parse_matrix_body(&body);
    render_matrix(kind, rows)
}

/// Parse matrix/cases body tokens into rows×cols of plain strings.
fn parse_matrix_body(tokens: &[Token]) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    for tok in tokens {
        match tok {
            Token::Char('&') => {
                current_row.push(current_cell.trim().to_string());
                current_cell.clear();
            }
            Token::Command(cmd) if cmd == "\\" || cmd == "\\\\" => {
                current_row.push(current_cell.trim().to_string());
                current_cell.clear();
                rows.push(current_row.clone());
                current_row.clear();
            }
            Token::Char(c) => current_cell.push(*c),
            Token::Command(c) => {
                if let Some(s) = lookup_symbol(c) {
                    current_cell.push_str(s);
                } else {
                    current_cell.push_str(&format!("\\{}", c));
                }
            }
            Token::Group(inner) => {
                current_cell.push_str(&tokens_to_plain(inner));
            }
            _ => {}
        }
    }

    // Flush final cell and row
    current_row.push(current_cell.trim().to_string());
    if !current_row.iter().all(|s| s.is_empty()) || !rows.is_empty() {
        rows.push(current_row);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_frac() {
        let (lines, diags) = render_display_math(r"\frac{d}{dx}", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains('─'));
    }

    #[test]
    fn display_frac_width() {
        let (lines, diags) = render_display_math(r"\frac{n(n+1)}{2}", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        assert_eq!(lines[0].trim(), "n(n+1)");
        let bar_w: usize = lines[1].chars().filter(|&c| c == '─').count();
        assert_eq!(bar_w, 6);
    }

    #[test]
    fn display_align_center_pads() {
        let (lines, _) = render_display_math(r"\alpha", 20, MathAlign::Center);
        assert_eq!(lines.len(), 1);
        assert_eq!(crate::visual_width(&lines[0]), 20);
        assert!(lines[0].contains('α'));
    }

    #[test]
    fn display_align_left_pads() {
        let (lines, _) = render_display_math(r"\beta", 10, MathAlign::Left);
        assert!(lines[0].starts_with('β'));
        assert_eq!(crate::visual_width(&lines[0]), 10);
    }

    #[test]
    fn display_sum_both_limits() {
        let (lines, diags) = render_display_math(r"\sum_{i=1}^{n} i", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        let full = lines.join("\n");
        assert!(full.contains('∑'));
        assert!(full.contains('n'));
        assert!(full.contains("i=1"));
    }

    #[test]
    fn display_int_limits() {
        let (lines, diags) = render_display_math(r"\int_0^{\infty} e^{-x} dx", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        let full = lines.join("\n");
        assert!(full.contains('∞'));
        assert!(full.contains('⌡'));
        assert!(full.contains('0'));
    }

    #[test]
    fn display_pmatrix() {
        let (lines, diags) = render_display_math(
            r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
            0,
            MathAlign::Left,
        );
        assert_eq!(diags.len(), 0);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("⎛"));
        assert!(lines[1].starts_with("⎝"));
    }

    #[test]
    fn display_mismatched_env_emits_math_003() {
        let (_, diags) =
            render_display_math(r"\begin{pmatrix} a \end{bmatrix}", 0, MathAlign::Left);
        assert!(diags.iter().any(|d| d.code == "MATH-003"));
    }

    #[test]
    fn display_unknown_env_emits_math_001() {
        let (_, diags) = render_display_math(r"\begin{myenv} x \end{myenv}", 0, MathAlign::Left);
        assert!(diags.iter().any(|d| d.code == "MATH-001"));
    }

    #[test]
    fn display_math_004_on_overflow() {
        let (_, diags) = render_display_math(
            r"\alpha + \beta + \gamma + \delta + \epsilon",
            5,
            MathAlign::Left,
        );
        assert!(diags.iter().any(|d| d.code == "MATH-004"));
    }

    #[test]
    fn display_align_right_pads() {
        let (lines, _) = render_display_math(r"\gamma", 10, MathAlign::Right);
        assert_eq!(crate::visual_width(&lines[0]), 10);
        assert!(lines[0].ends_with('γ'));
    }

    #[test]
    fn display_auto_width_zero() {
        // width=0 → auto: output width = visual width of rendered expression
        let (lines, _) = render_display_math(r"\alpha + \beta", 0, MathAlign::Left);
        assert_eq!(lines.len(), 1);
        let content = "α + β";
        assert_eq!(lines[0], content);
    }

    #[test]
    fn display_unclosed_env_emits_math_003() {
        let (_, diags) = render_display_math(r"\begin{pmatrix} a & b", 0, MathAlign::Left);
        assert!(diags.iter().any(|d| d.code == "MATH-003"));
    }

    #[test]
    fn display_bmatrix() {
        let (lines, diags) = render_display_math(
            r"\begin{bmatrix} x & y \\ z & w \end{bmatrix}",
            0,
            MathAlign::Left,
        );
        assert_eq!(diags.len(), 0);
        assert!(lines[0].starts_with("⎡"));
        assert!(lines[1].starts_with("⎣"));
    }

    #[test]
    fn display_vmatrix() {
        let (lines, diags) = render_display_math(
            r"\begin{vmatrix} a & b \\ c & d \end{vmatrix}",
            0,
            MathAlign::Left,
        );
        assert_eq!(diags.len(), 0);
        assert!(lines[0].starts_with("|"));
    }

    #[test]
    fn display_Vmatrix() {
        let (lines, diags) = render_display_math(
            r"\begin{Vmatrix} a & b \\ c & d \end{Vmatrix}",
            0,
            MathAlign::Left,
        );
        assert_eq!(diags.len(), 0);
        assert!(lines[0].starts_with("‖"));
    }

    #[test]
    fn display_cases() {
        let (lines, diags) = render_display_math(
            r"\begin{cases} n+1 & \text{odd} \\ n/2 & \text{even} \end{cases}",
            0,
            MathAlign::Left,
        );
        assert_eq!(diags.len(), 0);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("⎧"));
        assert!(lines[1].starts_with("⎩"));
    }

    #[test]
    fn display_prod_with_limits() {
        let (lines, diags) = render_display_math(r"\prod_{i=1}^{n} a_i", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        let full = lines.join("\n");
        assert!(full.contains('∏'));
        assert!(full.contains("i=1"));
    }

    #[test]
    fn display_empty_expression() {
        let (lines, _) = render_display_math("", 0, MathAlign::Left);
        // empty expression produces empty or single empty line — no panic
        assert!(lines.is_empty() || lines.iter().all(|l| l.is_empty()));
    }

    #[test]
    fn display_pure_symbol() {
        let (lines, diags) = render_display_math(r"\pi", 0, MathAlign::Left);
        assert_eq!(diags.len(), 0);
        assert_eq!(lines, vec!["π"]);
    }

    #[test]
    fn display_text_command() {
        let (lines, _) = render_display_math(r"\text{hello}", 0, MathAlign::Left);
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn display_lim_op() {
        let (lines, _) = render_display_math(r"\lim_{x} f(x)", 0, MathAlign::Left);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("lim"));
    }
}
