/// LaTeX tokenizer for mdloom math rendering.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Command(String),    // \alpha, \frac, \begin, \text, ...
    Char(char),         // literal character
    Group(Vec<Token>),  // {content}
    Superscript,        // ^
    Subscript,          // _
    BeginEnv(String),   // \begin{pmatrix}
    EndEnv(String),     // \end{pmatrix}
    OptArg(Vec<Token>), // [content] (for \sqrt[3]{x})
}

#[derive(Debug, Clone)]
pub struct MathDiag {
    pub code: &'static str,
    pub severity: DiagSeverity,
    pub col: usize,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiagSeverity {
    Warning,
    Error,
}

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<MathDiag>) {
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();
    let mut diags = Vec::new();
    tokenize_seq(&chars, &mut pos, &mut tokens, &mut diags, false);
    (tokens, diags)
}

fn tokenize_seq(
    chars: &[char],
    pos: &mut usize,
    out: &mut Vec<Token>,
    diags: &mut Vec<MathDiag>,
    in_group: bool,
) {
    while *pos < chars.len() {
        let start = *pos;
        let ch = chars[*pos];

        match ch {
            '}' if in_group => {
                *pos += 1;
                return;
            }
            '}' => {
                diags.push(MathDiag {
                    code: "MATH-006",
                    severity: DiagSeverity::Warning,
                    col: *pos,
                    message: "Unmatched `}` — ignored".to_string(),
                });
                *pos += 1;
            }
            '{' => {
                *pos += 1;
                let mut inner = Vec::new();
                tokenize_seq(chars, pos, &mut inner, diags, true);
                out.push(Token::Group(inner));
            }
            '[' => {
                *pos += 1;
                let mut inner = Vec::new();
                tokenize_seq(chars, pos, &mut inner, diags, false);
                // consumed by tokenize_seq until ']' or end
                out.push(Token::OptArg(inner));
            }
            ']' if !in_group => {
                *pos += 1;
                return;
            }
            '^' => {
                out.push(Token::Superscript);
                *pos += 1;
            }
            '_' => {
                out.push(Token::Subscript);
                *pos += 1;
            }
            '\\' => {
                *pos += 1;
                if *pos >= chars.len() {
                    break;
                }
                let next = chars[*pos];
                if next.is_ascii_alphabetic() {
                    // consume full command name
                    let cmd_start = *pos;
                    while *pos < chars.len() && chars[*pos].is_ascii_alphabetic() {
                        *pos += 1;
                    }
                    let cmd: String = chars[cmd_start..*pos].iter().collect();
                    // \begin{env} and \end{env} are special
                    if cmd == "begin" || cmd == "end" {
                        if *pos < chars.len() && chars[*pos] == '{' {
                            *pos += 1;
                            let env_start = *pos;
                            while *pos < chars.len() && chars[*pos] != '}' {
                                *pos += 1;
                            }
                            let env: String = chars[env_start..*pos].iter().collect();
                            if *pos < chars.len() {
                                *pos += 1;
                            } // consume '}'
                            if cmd == "begin" {
                                out.push(Token::BeginEnv(env));
                            } else {
                                out.push(Token::EndEnv(env));
                            }
                        } else {
                            diags.push(MathDiag {
                                code: "MATH-006",
                                severity: DiagSeverity::Error,
                                col: start,
                                message: format!("\\{} not followed by {{env}}", cmd),
                            });
                        }
                    } else {
                        out.push(Token::Command(cmd));
                    }
                } else {
                    // single non-letter command: \, \! \; \[ \] etc.
                    out.push(Token::Command(next.to_string()));
                    *pos += 1;
                }
            }
            '\'' => {
                // count consecutive primes → prime command
                let mut count = 0usize;
                while *pos < chars.len() && chars[*pos] == '\'' {
                    count += 1;
                    *pos += 1;
                }
                out.push(Token::Command(format!("prime{}", count)));
            }
            _ => {
                out.push(Token::Char(ch));
                *pos += 1;
            }
        }
    }

    if in_group {
        diags.push(MathDiag {
            code: "MATH-006",
            severity: DiagSeverity::Warning,
            col: chars.len(),
            message: "Unmatched `{` — group not closed".to_string(),
        });
    }
}

/// Consume exactly one token from `tokens[*pos..]`.
/// Returns the consumed token group (may be multiple if `^`/`_` needs a group).
/// This is used by super/subscript processing: `^` takes the next token.
pub fn consume_one(tokens: &[Token], pos: &mut usize) -> Option<Vec<Token>> {
    if *pos >= tokens.len() {
        return None;
    }
    let tok = tokens[*pos].clone();
    *pos += 1;
    match tok {
        Token::Group(inner) => Some(inner),
        other => Some(vec![other]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_command() {
        let (toks, diags) = tokenize(r"\alpha");
        assert_eq!(diags.len(), 0);
        assert_eq!(toks, vec![Token::Command("alpha".to_string())]);
    }

    #[test]
    fn tokenize_group() {
        let (toks, diags) = tokenize(r"x^{ab}");
        assert_eq!(diags.len(), 0);
        assert_eq!(
            toks,
            vec![
                Token::Char('x'),
                Token::Superscript,
                Token::Group(vec![Token::Char('a'), Token::Char('b')]),
            ]
        );
    }

    #[test]
    fn tokenize_begin_end() {
        let (toks, diags) = tokenize(r"\begin{pmatrix}\end{pmatrix}");
        assert_eq!(diags.len(), 0);
        assert_eq!(
            toks,
            vec![
                Token::BeginEnv("pmatrix".to_string()),
                Token::EndEnv("pmatrix".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_unmatched_brace_emits_diag() {
        let (_, diags) = tokenize(r"x}");
        assert!(diags.iter().any(|d| d.code == "MATH-006"));
    }

    #[test]
    fn tokenize_primes() {
        let (toks, _) = tokenize("f''");
        assert_eq!(
            toks,
            vec![Token::Char('f'), Token::Command("prime2".to_string()),]
        );
    }

    #[test]
    fn tokenize_opt_arg() {
        let (toks, _) = tokenize(r"\sqrt[3]{x}");
        assert!(matches!(&toks[0], Token::Command(s) if s == "sqrt"));
        assert!(matches!(&toks[1], Token::OptArg(_)));
        assert!(matches!(&toks[2], Token::Group(_)));
    }

    #[test]
    fn tokenize_empty_source() {
        let (toks, diags) = tokenize("");
        assert_eq!(toks.len(), 0);
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn tokenize_ampersand() {
        let (toks, _) = tokenize("a & b");
        assert!(toks.contains(&Token::Char('&')));
    }

    #[test]
    fn tokenize_row_separator() {
        // \\ is a single-char command with name "\\"
        let (toks, _) = tokenize(r"a \\ b");
        assert!(toks
            .iter()
            .any(|t| matches!(t, Token::Command(s) if s == "\\")));
    }

    #[test]
    fn tokenize_nested_groups() {
        let (toks, diags) = tokenize(r"{a{b}}");
        assert_eq!(diags.len(), 0);
        // Outer group containing 'a' and inner group containing 'b'
        assert!(matches!(&toks[0], Token::Group(inner) if inner.len() == 2));
    }

    #[test]
    fn tokenize_unclosed_group_emits_diag() {
        let (_, diags) = tokenize(r"x^{ab");
        assert!(diags.iter().any(|d| d.code == "MATH-006"));
    }

    #[test]
    fn tokenize_backslash_at_eof() {
        // Backslash at end of string should not panic
        let (_, _) = tokenize("x\\");
        // just shouldn't panic
    }

    #[test]
    fn tokenize_begin_without_braces_emits_diag() {
        let (_, diags) = tokenize(r"\begin pmatrix");
        assert!(diags.iter().any(|d| d.code == "MATH-006"));
    }

    #[test]
    fn tokenize_subscript_and_superscript() {
        let (toks, _) = tokenize(r"x_i^2");
        assert!(toks.contains(&Token::Subscript));
        assert!(toks.contains(&Token::Superscript));
    }
}
