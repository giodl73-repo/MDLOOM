/// US-27: proof-math standalone binary.
///
/// A CLI tool that reads lines of prose from stdin and writes the same
/// lines with all $...$ math spans expanded to Unicode/ASCII art on stdout.
///
/// Run:
///   echo 'The energy $E = mc^2$ is famous.' | cargo run --example proof-math-binary
///
/// Cargo.toml dependencies:
///   proof-math = { path = "../../crates/proof-math" }

use proof_math::{expand_inline_math, render_display_math, MathAlign};
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut display_buf: Vec<String> = Vec::new();
    let mut in_display = false;

    for line_result in stdin.lock().lines() {
        let line = line_result?;

        // Detect display math fences: lines containing only $$ (or $$...$$)
        if line.trim() == "$$" {
            if in_display {
                // End of display block — render collected expression
                let expr = display_buf.join(" ");
                let (rendered, diags) = render_display_math(&expr, 0, MathAlign::Center);
                for d in &diags {
                    eprintln!("math: {} col {}: {}", d.code, d.col, d.message);
                }
                for row in &rendered {
                    writeln!(out, "{}", row)?;
                }
                display_buf.clear();
                in_display = false;
            } else {
                in_display = true;
            }
            continue;
        }

        if in_display {
            display_buf.push(line);
            continue;
        }

        // Inline expansion
        let (expanded, diags) = expand_inline_math(&line);
        for d in &diags {
            eprintln!("math: {} col {}: {}", d.code, d.col, d.message);
        }
        writeln!(out, "{}", expanded)?;
    }

    // Unterminated display block — flush as-is with a warning
    if in_display && !display_buf.is_empty() {
        eprintln!("math: unterminated $$ block");
        for line in &display_buf {
            writeln!(out, "{}", line)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use proof_math::expand_inline_math;

    #[test]
    fn greek_letters_expand() {
        let (out, diags) = expand_inline_math("$\\alpha + \\beta = \\gamma$");
        assert_eq!(out, "α + β = γ");
        assert!(diags.is_empty());
    }

    #[test]
    fn superscript_expands() {
        let (out, _) = expand_inline_math("$x^2 + y^2 = r^2$");
        assert!(out.contains('²'), "expected superscript 2: {}", out);
    }

    #[test]
    fn inline_outside_dollar_unchanged() {
        let (out, _) = expand_inline_math("plain text with no math");
        assert_eq!(out, "plain text with no math");
    }

    #[test]
    fn no_expand_inside_backtick() {
        let (out, _) = expand_inline_math("use `$x$` to write math");
        assert!(out.contains("$x$"), "should not expand inside backtick: {}", out);
    }
}
