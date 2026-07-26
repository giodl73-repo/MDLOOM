//! Tier 1: direct LaTeX command -> Unicode character mappings.

pub static LATEX_SYMBOLS: &[(&str, &str)] = &[
    // Greek lowercase
    ("alpha", "α"),
    ("beta", "β"),
    ("gamma", "γ"),
    ("delta", "δ"),
    ("epsilon", "ε"),
    ("varepsilon", "ε"),
    ("zeta", "ζ"),
    ("eta", "η"),
    ("theta", "θ"),
    ("vartheta", "ϑ"),
    ("iota", "ι"),
    ("kappa", "κ"),
    ("lambda", "λ"),
    ("mu", "μ"),
    ("nu", "ν"),
    ("xi", "ξ"),
    ("pi", "π"),
    ("varpi", "ϖ"),
    ("rho", "ρ"),
    ("varrho", "ϱ"),
    ("sigma", "σ"),
    ("varsigma", "ς"),
    ("tau", "τ"),
    ("upsilon", "υ"),
    ("phi", "φ"),
    ("varphi", "φ"),
    ("chi", "χ"),
    ("psi", "ψ"),
    ("omega", "ω"),
    // Greek uppercase
    ("Gamma", "Γ"),
    ("Delta", "Δ"),
    ("Theta", "Θ"),
    ("Lambda", "Λ"),
    ("Xi", "Ξ"),
    ("Pi", "Π"),
    ("Sigma", "Σ"),
    ("Upsilon", "Υ"),
    ("Phi", "Φ"),
    ("Psi", "Ψ"),
    ("Omega", "Ω"),
    // Math constants and symbols
    ("infty", "∞"),
    ("partial", "∂"),
    ("nabla", "∇"),
    ("sum", "∑"),
    ("prod", "∏"),
    ("int", "∫"),
    ("oint", "∮"),
    ("iint", "∬"),
    ("iiint", "∭"),
    // Operators
    ("times", "×"),
    ("div", "÷"),
    ("pm", "±"),
    ("mp", "∓"),
    ("cdot", "·"),
    ("circ", "∘"),
    ("bullet", "•"),
    ("star", "⋆"),
    ("ast", "∗"),
    ("oplus", "⊕"),
    ("otimes", "⊗"),
    ("odot", "⊙"),
    // Relations
    ("leq", "≤"),
    ("geq", "≥"),
    ("neq", "≠"),
    ("approx", "≈"),
    ("equiv", "≡"),
    ("sim", "∼"),
    ("simeq", "≃"),
    ("cong", "≅"),
    ("propto", "∝"),
    ("ll", "≪"),
    ("gg", "≫"),
    // Sets
    ("in", "∈"),
    ("notin", "∉"),
    ("ni", "∋"),
    ("subset", "⊂"),
    ("supset", "⊃"),
    ("subseteq", "⊆"),
    ("supseteq", "⊇"),
    ("cup", "∪"),
    ("cap", "∩"),
    ("emptyset", "∅"),
    ("varnothing", "∅"),
    ("setminus", "∖"),
    // Logic
    ("forall", "∀"),
    ("exists", "∃"),
    ("nexists", "∄"),
    ("neg", "¬"),
    ("lnot", "¬"),
    ("land", "∧"),
    ("lor", "∨"),
    ("top", "⊤"),
    ("bot", "⊥"),
    ("vdash", "⊢"),
    ("models", "⊨"),
    // Arrows
    ("to", "→"),
    ("rightarrow", "→"),
    ("leftarrow", "←"),
    ("Rightarrow", "⇒"),
    ("Leftarrow", "⇐"),
    ("Leftrightarrow", "⟺"),
    ("leftrightarrow", "↔"),
    ("iff", "⟺"),
    ("implies", "⟹"),
    ("gets", "←"),
    ("uparrow", "↑"),
    ("downarrow", "↓"),
    ("Uparrow", "⇑"),
    ("Downarrow", "⇓"),
    ("mapsto", "↦"),
    ("hookrightarrow", "↪"),
    ("hookleftarrow", "↩"),
    // Other symbols
    ("therefore", "∴"),
    ("because", "∵"),
    ("prime", "′"),
    ("degree", "°"),
    ("angle", "∠"),
    ("triangle", "△"),
    ("square", "□"),
    ("diamond", "◇"),
    ("langle", "⟨"),
    ("rangle", "⟩"),
    ("vert", "|"),
    ("Vert", "‖"),
    ("|", "‖"),
    ("hbar", "ℏ"),
    ("ell", "ℓ"),
    ("wp", "℘"),
    ("Re", "ℜ"),
    ("Im", "ℑ"),
    ("aleph", "ℵ"),
    ("beth", "ℶ"),
    ("complement", "∁"),
    ("natural", "♮"),
    ("sharp", "♯"),
    ("flat", "♭"),
    // Spacing commands → single space or empty
    (",", " "),
    (";", " "),
    ("!", ""),
    (":", " "),
    (" ", " "),
    // Misc
    ("ldots", "…"),
    ("cdots", "⋯"),
    ("vdots", "⋮"),
    ("ddots", "⋱"),
    ("dots", "…"),
];

/// Font-styling commands: strip the command, pass content through.
pub static FONT_COMMANDS: &[&str] = &[
    "mathbf",
    "mathrm",
    "mathit",
    "mathtt",
    "mathsf",
    "mathcal",
    "mathfrak",
    "mathbb",
    "text",
    "textrm",
    "textbf",
    "textit",
    "boldsymbol",
    "bm",
];

/// Commands that require special inline rendering (not just symbol lookup).
pub static UNSUPPORTED_COMMANDS: &[&str] = &[
    "left",
    "right",
    "color",
    "textcolor",
    "colorbox",
    "includegraphics",
    "tikz",
    "draw",
    "node",
    "newcommand",
    "def",
    "renewcommand",
    "newpage",
    "pagebreak",
    "phantom",
    "hspace",
    "vspace",
    "over",
    "atop",
];

pub fn lookup_symbol(cmd: &str) -> Option<&'static str> {
    LATEX_SYMBOLS
        .iter()
        .find(|(k, _)| *k == cmd)
        .map(|(_, v)| *v)
}

pub fn is_font_command(cmd: &str) -> bool {
    FONT_COMMANDS.contains(&cmd)
}

pub fn is_unsupported(cmd: &str) -> bool {
    UNSUPPORTED_COMMANDS.contains(&cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_greek() {
        assert_eq!(lookup_symbol("alpha"), Some("α"));
        assert_eq!(lookup_symbol("beta"), Some("β"));
        assert_eq!(lookup_symbol("Omega"), Some("Ω"));
    }

    #[test]
    fn lookup_operators() {
        assert_eq!(lookup_symbol("times"), Some("×"));
        assert_eq!(lookup_symbol("leq"), Some("≤"));
        assert_eq!(lookup_symbol("infty"), Some("∞"));
    }

    #[test]
    fn lookup_uppercase_greek() {
        assert_eq!(lookup_symbol("Gamma"), Some("Γ"));
        assert_eq!(lookup_symbol("Delta"), Some("Δ"));
        assert_eq!(lookup_symbol("Pi"), Some("Π"));
        assert_eq!(lookup_symbol("Sigma"), Some("Σ"));
        assert_eq!(lookup_symbol("Lambda"), Some("Λ"));
    }

    #[test]
    fn lookup_set_symbols() {
        assert_eq!(lookup_symbol("in"), Some("∈"));
        assert_eq!(lookup_symbol("notin"), Some("∉"));
        assert_eq!(lookup_symbol("subset"), Some("⊂"));
        assert_eq!(lookup_symbol("cup"), Some("∪"));
        assert_eq!(lookup_symbol("cap"), Some("∩"));
        assert_eq!(lookup_symbol("emptyset"), Some("∅"));
    }

    #[test]
    fn lookup_logic_symbols() {
        assert_eq!(lookup_symbol("forall"), Some("∀"));
        assert_eq!(lookup_symbol("exists"), Some("∃"));
        assert_eq!(lookup_symbol("neg"), Some("¬"));
        assert_eq!(lookup_symbol("land"), Some("∧"));
        assert_eq!(lookup_symbol("lor"), Some("∨"));
    }

    #[test]
    fn lookup_arrows() {
        assert_eq!(lookup_symbol("to"), Some("→"));
        assert_eq!(lookup_symbol("leftarrow"), Some("←"));
        assert_eq!(lookup_symbol("Rightarrow"), Some("⇒"));
        assert_eq!(lookup_symbol("Leftrightarrow"), Some("⟺"));
        assert_eq!(lookup_symbol("implies"), Some("⟹"));
        assert_eq!(lookup_symbol("mapsto"), Some("↦"));
    }

    #[test]
    fn lookup_misc_symbols() {
        assert_eq!(lookup_symbol("therefore"), Some("∴"));
        assert_eq!(lookup_symbol("because"), Some("∵"));
        assert_eq!(lookup_symbol("partial"), Some("∂"));
        assert_eq!(lookup_symbol("nabla"), Some("∇"));
        assert_eq!(lookup_symbol("hbar"), Some("ℏ"));
        assert_eq!(lookup_symbol("aleph"), Some("ℵ"));
    }

    #[test]
    fn lookup_spacing_commands() {
        assert_eq!(lookup_symbol(","), Some(" "));
        assert_eq!(lookup_symbol("!"), Some(""));
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert_eq!(lookup_symbol("notacommand"), None);
        assert_eq!(lookup_symbol("alph"), None);
    }

    #[test]
    fn font_commands_recognized() {
        assert!(is_font_command("mathbf"));
        assert!(is_font_command("text"));
        assert!(!is_font_command("alpha"));
    }

    #[test]
    fn unsupported_recognized() {
        assert!(is_unsupported("left"));
        assert!(is_unsupported("right"));
        assert!(!is_unsupported("alpha"));
    }
}
