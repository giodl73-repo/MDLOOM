#![allow(dead_code, unused_imports, unused_variables)]
//! proof-math — LaTeX math rendering to Unicode and ASCII art for terminal output.
//!
//! No LaTeX installation required. Pure Rust implementation covering:
//!
//! - **Tier 1**: Direct Unicode substitution (Greek, operators, arrows, sets, logic)
//! - **Tier 2**: Single-line ASCII art (superscripts, subscripts, √, primes, limits)
//! - **Tier 3**: Multi-line display math (stacked fractions, integrals, sums, matrices)
//!
//! # Inline expansion
//!
//! ```rust
//! use proof_math::expand_inline_math;
//!
//! let (result, diags) = expand_inline_math("$\\alpha + \\beta = \\gamma$");
//! assert_eq!(result, "α + β = γ");
//! assert!(diags.is_empty());
//! ```
//!
//! # Display blocks
//!
//! ```rust
//! use proof_math::{render_display_math, MathAlign};
//!
//! let (lines, diags) = render_display_math(r"\frac{n(n+1)}{2}", 0, MathAlign::Left);
//! // lines: ["n(n+1)", "──────", "  2"]
//! ```

pub mod symbols;
pub mod tokenizer;
pub mod superscript;
pub mod tier2;
pub mod fraction;
pub mod integral;
pub mod matrix;
pub mod render;

pub use tokenizer::{MathDiag, DiagSeverity};
pub use render::{render_display_math, MathAlign};

// ─────────────────────────────────────────────────────────
// Visual width — public API for terminal layout
// ─────────────────────────────────────────────────────────

/// Visual column width of a string under East Asian Width rules.
///
/// Box-drawing characters (U+2500..U+257F), Braille (U+2800..U+28FF), and
/// geometric shapes (U+25A0..U+25FF) are always 1 column wide regardless of
/// what `unicode-width` reports for them. Everything else uses the standard
/// East Asian Width tables: ASCII and Latin-1 are 1 column, CJK and other
/// Wide/Fullwidth characters are 2 columns, combining marks are 0 columns.
///
/// Use this when laying out mixed ASCII/Unicode content in a terminal —
/// counting `chars()` overcounts wide CJK glyphs and undercounts combining
/// marks, while raw `unicode-width` overcounts box-drawing characters that
/// most terminal fonts render at 1 column.
///
/// ```
/// use proof_math::visual_width;
/// assert_eq!(visual_width("hello"), 5);
/// assert_eq!(visual_width("─┼─"), 3);   // box-drawing forced to 1 col each
/// assert_eq!(visual_width("日本"), 4);  // CJK wide, 2 cols each
/// ```
pub fn visual_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthChar;
    s.chars().map(|ch| {
        let cp = ch as u32;
        if (0x2500..=0x28FF).contains(&cp) || (0x25A0..=0x25FF).contains(&cp) {
            1
        } else {
            UnicodeWidthChar::width(ch).unwrap_or(0)
        }
    }).sum()
}

// ─────────────────────────────────────────────────────────
// Inline math expansion — public entry point
// ─────────────────────────────────────────────────────────

/// Expand all `$...$` spans in one line of prose.
///
/// Returns `(expanded_line, diagnostics)`.
/// Does not expand inside backtick code spans.
pub fn expand_inline_math(line: &str) -> (String, Vec<MathDiag>) {
    let mut out = String::with_capacity(line.len());
    let mut diags = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    let opaque = find_code_spans(&chars);

    while i < n {
        if is_opaque(i, &opaque) {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if chars[i] == '$' {
            let open = i;
            i += 1;
            let mut found_close = None;
            while i < n {
                if is_opaque(i, &opaque) { i += 1; continue; }
                if chars[i] == '$' { found_close = Some(i); break; }
                i += 1;
            }
            match found_close {
                None => {
                    for k in open..n { out.push(chars[k]); }
                    i = n;
                }
                Some(close) => {
                    let inner: String = chars[open+1..close].iter().collect();
                    let col = open;
                    let (expanded, mut span_diags) = expand_math_span(&inner, col);
                    for d in &mut span_diags { d.col += col; }
                    diags.extend(span_diags);
                    out.push_str(&expanded);
                    i = close + 1;
                }
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    (out, diags)
}

fn find_code_spans(chars: &[char]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if chars[i] == '`' {
            let open = i;
            i += 1;
            while i < n && chars[i] != '`' { i += 1; }
            if i < n { spans.push((open, i + 1)); i += 1; }
            else { spans.push((open, n)); }
        } else { i += 1; }
    }
    spans
}

fn is_opaque(pos: usize, spans: &[(usize, usize)]) -> bool {
    spans.iter().any(|(s, e)| pos >= *s && pos < *e)
}

fn expand_math_span(src: &str, base_col: usize) -> (String, Vec<MathDiag>) {
    let (tokens, mut diags) = tokenizer::tokenize(src);
    let mut out = String::new();
    let mut pos = 0;
    expand_tokens(&tokens, &mut pos, &mut out, &mut diags, true, base_col);
    (out, diags)
}

pub(crate) fn expand_tokens(
    tokens: &[tokenizer::Token],
    pos: &mut usize,
    out: &mut String,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) {
    use tokenizer::Token;
    use symbols::{lookup_symbol, is_font_command, is_unsupported};

    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::Char(c) => { out.push(*c); *pos += 1; }
            Token::Superscript => {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                out.push_str(&superscript::to_superscript(&s));
            }
            Token::Subscript => {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                out.push_str(&superscript::to_subscript(&s));
            }
            Token::Group(inner) => {
                let mut inner_pos = 0;
                expand_tokens(inner, &mut inner_pos, out, diags, inline, base_col);
                *pos += 1;
            }
            Token::Command(cmd) => {
                let cmd = cmd.clone();
                *pos += 1;
                expand_command(&cmd, tokens, pos, out, diags, inline, base_col);
            }
            Token::BeginEnv(env) => {
                let env = env.clone();
                *pos += 1;
                expand_environment(&env, tokens, pos, out, diags, inline, base_col);
            }
            Token::EndEnv(env) => {
                diags.push(MathDiag {
                    code: "MATH-003",
                    severity: DiagSeverity::Error,
                    col: base_col,
                    message: format!("Unexpected \\end{{{}}}", env),
                });
                *pos += 1;
            }
            Token::OptArg(_) => { *pos += 1; }
        }
    }
}

fn expand_command(
    cmd: &str,
    tokens: &[tokenizer::Token],
    pos: &mut usize,
    out: &mut String,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) {
    use tokenizer::Token;
    use symbols::{lookup_symbol, is_font_command, is_unsupported};

    if cmd.starts_with("prime") {
        let n: usize = cmd["prime".len()..].parse().unwrap_or(1);
        out.push_str(match n { 1 => "′", 2 => "″", _ => "‴" });
        return;
    }

    if matches!(cmd, "lim" | "max" | "min" | "sup" | "inf") {
        out.push_str(cmd);
        if *pos < tokens.len() {
            if let Token::Subscript = &tokens[*pos] {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                out.push_str(&format!("_({})", s));
            }
        }
        if *pos < tokens.len() {
            if let Token::Superscript = &tokens[*pos] {
                *pos += 1;
                let arg = tokenizer::consume_one(tokens, pos);
                let s = tokens_to_plain(arg.as_deref().unwrap_or(&[]));
                out.push_str(&format!("^({})", s));
            }
        }
        return;
    }

    if cmd == "sqrt" {
        let root = if *pos < tokens.len() {
            if let Token::OptArg(inner) = &tokens[*pos] {
                let s = tokens_to_plain(inner);
                *pos += 1;
                Some(superscript::to_superscript(&s))
            } else { None }
        } else { None };
        let arg = if *pos < tokens.len() {
            if let Token::Group(inner) = &tokens[*pos] {
                let v = inner.clone(); *pos += 1; v
            } else { tokenizer::consume_one(tokens, pos).unwrap_or_default() }
        } else { vec![] };
        let s = tokens_to_plain(&arg);
        let inner = if s.len() > 1 && !s.starts_with('(') { format!("({})", s) } else { s };
        match root {
            Some(r) => out.push_str(&format!("{}√{}", r, inner)),
            None    => out.push_str(&format!("√{}", inner)),
        }
        return;
    }

    if cmd == "frac" {
        let num_toks = consume_group(tokens, pos);
        let den_toks = consume_group(tokens, pos);
        let num = tokens_to_plain(&num_toks);
        let den = tokens_to_plain(&den_toks);
        if inline {
            // Wrap numerator in parens if it contains operators (prevents ambiguous precedence)
            let num_display = if num.contains(['+', '-', '×', '÷', '·', '±', '∓'])
                && !num.starts_with('(') {
                format!("({})", num)
            } else {
                num.clone()
            };
            diags.push(MathDiag {
                code: "MATH-005",
                severity: DiagSeverity::Warning,
                col: base_col,
                message: format!("Tier 3 \\frac in inline context — rendered as {}/{}", num_display, den),
            });
            out.push_str(&format!("{}/{}", num_display, den));
        } else {
            out.push_str(&format!("{}/{}", num, den));
        }
        return;
    }

    if is_font_command(cmd) {
        let arg = consume_group(tokens, pos);
        if cmd == "text" {
            let s: String = arg.iter().map(|t| match t {
                Token::Char(c) => c.to_string(),
                Token::Command(c) => lookup_symbol(c).unwrap_or("").to_string(),
                _ => String::new(),
            }).collect();
            out.push_str(&s);
        } else {
            let mut inner_pos = 0;
            expand_tokens(&arg, &mut inner_pos, out, diags, inline, base_col);
        }
        return;
    }

    if is_unsupported(cmd) {
        diags.push(MathDiag {
            code: "MATH-001",
            severity: DiagSeverity::Warning,
            col: base_col,
            message: format!("Unsupported command `\\{}` — passed through", cmd),
        });
        out.push('\\');
        out.push_str(cmd);
        return;
    }

    if let Some(sym) = lookup_symbol(cmd) {
        out.push_str(sym);
        return;
    }

    diags.push(MathDiag {
        code: "MATH-001",
        severity: DiagSeverity::Warning,
        col: base_col,
        message: format!("Unknown command `\\{}` — passed through", cmd),
    });
    out.push('\\');
    out.push_str(cmd);
}

fn expand_environment(
    env: &str,
    tokens: &[tokenizer::Token],
    pos: &mut usize,
    out: &mut String,
    diags: &mut Vec<MathDiag>,
    inline: bool,
    base_col: usize,
) {
    use tokenizer::Token;
    if inline {
        diags.push(MathDiag {
            code: "MATH-005",
            severity: DiagSeverity::Warning,
            col: base_col,
            message: format!("Tier 3 \\begin{{{}}} in inline context — simplified", env),
        });
    }
    let mut depth = 1usize;
    while *pos < tokens.len() {
        match &tokens[*pos] {
            Token::BeginEnv(_) => { depth += 1; *pos += 1; }
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
                *pos += 1;
            }
            _ => { *pos += 1; }
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
    out.push_str(&format!("[{}]", env));
}

pub(crate) fn tokens_to_plain(tokens: &[tokenizer::Token]) -> String {
    use tokenizer::Token;
    tokens.iter().map(|t| match t {
        Token::Char(c) => c.to_string(),
        Token::Command(cmd) => symbols::lookup_symbol(cmd)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("\\{}", cmd)),
        Token::Group(inner) => tokens_to_plain(inner),
        Token::Superscript => "^".to_string(),
        Token::Subscript => "_".to_string(),
        Token::BeginEnv(e) => format!("\\begin{{{}}}", e),
        Token::EndEnv(e) => format!("\\end{{{}}}", e),
        Token::OptArg(inner) => format!("[{}]", tokens_to_plain(inner)),
    }).collect()
}

pub(crate) fn consume_group(tokens: &[tokenizer::Token], pos: &mut usize) -> Vec<tokenizer::Token> {
    use tokenizer::Token;
    if *pos >= tokens.len() { return vec![]; }
    match &tokens[*pos] {
        Token::Group(inner) => { let v = inner.clone(); *pos += 1; v }
        _ => { let t = tokens[*pos].clone(); *pos += 1; vec![t] }
    }
}
