pub mod library;
pub mod shape;

use library::{SymbolEntry, BUILT_IN_SYMBOLS};

// ─────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CustomSymbol {
    pub name: String,
    pub character: String,
    pub width: usize,
    pub art: Option<String>,
    pub height: Option<usize>,
}

/// Resolved symbol — either a built-in static entry or an owned custom entry.
#[derive(Clone)]
pub enum ResolvedSymbol<'a> {
    BuiltIn(&'a SymbolEntry),
    Custom(CustomSymbol),
}

impl<'a> ResolvedSymbol<'a> {
    pub fn chars(&self) -> &str {
        match self {
            Self::BuiltIn(e) => e.chars,
            Self::Custom(c) => &c.character,
        }
    }

    pub fn width(&self) -> usize {
        match self {
            Self::BuiltIn(e) => e.width,
            Self::Custom(c) => c.width,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::BuiltIn(e) => e.name,
            Self::Custom(c) => &c.name,
        }
    }
}

// ─────────────────────────────────────────────────────────
// SymbolLibrary — wraps built-ins + custom symbols
// ─────────────────────────────────────────────────────────

pub struct SymbolLibrary {
    pub custom: Vec<CustomSymbol>,
}

impl SymbolLibrary {
    pub fn new() -> Self {
        Self { custom: Vec::new() }
    }

    pub fn with_custom(custom: Vec<CustomSymbol>) -> Self {
        Self { custom }
    }
}

impl Default for SymbolLibrary {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────
// Resolution — custom first, then built-in exact, then alias
// ─────────────────────────────────────────────────────────

pub fn resolve<'a>(name: &str, lib: &'a SymbolLibrary) -> Option<ResolvedSymbol<'a>> {
    let lower = name.to_lowercase();
    let lower = lower.as_str();

    // 1. Custom symbols (case-insensitive name match, consistent with lookup())
    if let Some(c) = lib.custom.iter().find(|c| c.name.to_lowercase() == lower) {
        return Some(ResolvedSymbol::Custom(c.clone()));
    }

    // 2. Built-in exact name match (case-insensitive)
    if let Some(entry) = BUILT_IN_SYMBOLS.iter().find(|s| s.name == lower) {
        return Some(ResolvedSymbol::BuiltIn(entry));
    }

    // 3. Built-in alias match (case-insensitive)
    if let Some(entry) = BUILT_IN_SYMBOLS.iter().find(|s| s.aliases.contains(&lower)) {
        return Some(ResolvedSymbol::BuiltIn(entry));
    }

    None
}

// ─────────────────────────────────────────────────────────
// Did-you-mean suggestion
// ─────────────────────────────────────────────────────────

/// Return the closest symbol name (or alias) to `query`, if within edit distance 3.
/// Used to produce "did you mean 'X'?" suggestions in diagnostics.
pub fn suggest_symbol<'a>(query: &str, lib: &'a SymbolLibrary) -> Option<&'a str> {
    let q = query.to_lowercase();
    let mut best: Option<(&str, usize)> = None;

    let check = |name: &'a str, best: &mut Option<(&'a str, usize)>| {
        let d = edit_distance(&q, name);
        if d <= 3 && best.is_none_or(|(_, bd)| d < bd) {
            *best = Some((name, d));
        }
    };

    // Custom symbols
    for sym in &lib.custom {
        let lower = sym.name.to_lowercase();
        // Need to store as &str with long enough lifetime — use BUILT_IN_SYMBOLS for statics,
        // but custom names are heap-allocated. We return a reference into the library only
        // for built-ins. For custom, we leak a short copy (rare — only on error path).
        let d = edit_distance(&q, &lower);
        if d <= 3 && best.is_none_or(|(_, bd)| d < bd) {
            // SAFETY: we're on the error path; this small leak is acceptable.
            let leaked: &'static str = Box::leak(sym.name.clone().into_boxed_str());
            best = Some((leaked, d));
        }
    }

    // Built-in names
    for entry in BUILT_IN_SYMBOLS {
        check(entry.name, &mut best);
        for alias in entry.aliases {
            check(alias, &mut best);
        }
    }

    best.map(|(name, _)| name)
}

/// Simple edit distance (Levenshtein) for short strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut row: Vec<usize> = (0..=n).collect();
    for i in 1..=m {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=n {
            let old = row[j];
            row[j] = if a[i - 1] == b[j - 1] {
                prev
            } else {
                1 + prev.min(row[j]).min(row[j - 1])
            };
            prev = old;
        }
    }
    row[n]
}

// ─────────────────────────────────────────────────────────
// Rendering
// ─────────────────────────────────────────────────────────

/// Render a resolved symbol at a given size (1=char, 2=3×3 block, 3=5×5, 5=9×9).
/// Returns raw ASCII/Unicode string without fence.
pub fn render_symbol_block(sym: &ResolvedSymbol<'_>, size: usize) -> String {
    let ch = sym.chars();
    match size {
        1 => ch.to_string(),
        2 => {
            // 3×3 block
            let row = ch.repeat(3);
            format!("{row}\n{row}\n{row}")
        }
        3 => {
            // 5×5 block
            let row = ch.repeat(5);
            format!("{row}\n{row}\n{row}\n{row}\n{row}")
        }
        5 => {
            // 9×9 block
            let row = ch.repeat(9);
            (0..9).map(|_| row.clone()).collect::<Vec<_>>().join("\n")
        }
        _ => ch.to_string(),
    }
}

// ─────────────────────────────────────────────────────────
// Inline [sym:name] expansion
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SymbolWarning {
    pub code: &'static str,
    pub name: String,
    pub message: String,
}

/// Expand [sym:name] in a prose string. Skips expansion when pos is
/// inside inline code (odd backtick count) or inside a URL.
/// Fenced code blocks must be stripped by the caller before passing text here.
pub fn expand_symbols(text: &str, lib: &SymbolLibrary) -> (String, Vec<SymbolWarning>) {
    let mut result = String::with_capacity(text.len());
    let mut warnings = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("[sym:") {
        // Check excluded context at the absolute position.
        // Only count backticks on the CURRENT LINE (inline code can't cross line boundaries;
        // fenced code blocks are pre-stripped by callers before this function is called).
        let abs_pos = text.len() - rest.len() + start;
        let before = &text[..abs_pos];
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let before_on_line = &before[line_start..];
        // Skip if inside inline code (odd backtick count on current line only)
        let in_code = before_on_line.chars().filter(|&c| c == '`').count() % 2 != 0;
        // Skip if inside a URL (preceded by scheme:// with no whitespace gap)
        let word_start = before_on_line
            .rfind(char::is_whitespace)
            .map(|i| line_start + i + 1)
            .unwrap_or(line_start);
        let word = &before[word_start..];
        let in_url = matches!(word.split_once("://"),
            Some((s, _)) if matches!(s, "http" | "https" | "md" | "ftp"));

        if in_code || in_url {
            // Pass through one char and keep scanning
            let next = rest
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(rest.len());
            result.push_str(&rest[..next]);
            rest = &rest[next..];
            continue;
        }

        result.push_str(&rest[..start]);
        rest = &rest[start..];

        if let Some(end) = rest.find(']') {
            let name = &rest[5..end]; // strip "[sym:"
            rest = &rest[end + 1..];

            match resolve(name, lib) {
                Some(sym) => result.push_str(sym.chars()),
                None => {
                    warnings.push(SymbolWarning {
                        code: "SYMBOL-001",
                        name: name.to_string(),
                        message: format!(
                            "symbol {:?} not found — check spelling or add [[symbol]] to proof.toml",
                            name
                        ),
                    });
                    result.push_str(&format!("[sym:{}]", name));
                }
            }
        } else {
            result.push_str(rest);
            rest = "";
        }
    }

    result.push_str(rest);
    (result, warnings)
}

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_built_in_checkmark_by_name() {
        let lib = SymbolLibrary::new();
        let sym = resolve("checkmark", &lib).expect("checkmark should resolve");
        assert_eq!(sym.chars(), "✓");
        assert_eq!(sym.width(), 1);
    }

    #[test]
    fn resolve_built_in_by_alias() {
        let lib = SymbolLibrary::new();
        let sym = resolve("cross", &lib).expect("cross alias should resolve to x");
        assert_eq!(sym.chars(), "✗");
    }

    #[test]
    fn resolve_missing_returns_none() {
        let lib = SymbolLibrary::new();
        assert!(resolve("does-not-exist", &lib).is_none());
    }

    #[test]
    fn resolve_custom_symbol_overrides_built_in() {
        let custom = vec![CustomSymbol {
            name: "checkmark".to_string(),
            character: "OK".to_string(),
            width: 2,
            art: None,
            height: None,
        }];
        let lib = SymbolLibrary::with_custom(custom);
        let sym = resolve("checkmark", &lib).unwrap();
        assert_eq!(sym.chars(), "OK", "custom should shadow built-in");
    }

    #[test]
    fn render_symbol_block_size_1() {
        let lib = SymbolLibrary::new();
        let sym = resolve("star", &lib).unwrap();
        let out = render_symbol_block(&sym, 1);
        assert_eq!(out, "★");
    }

    #[test]
    fn render_symbol_block_size_2_is_3x3() {
        let lib = SymbolLibrary::new();
        let sym = resolve("star", &lib).unwrap();
        let out = render_symbol_block(&sym, 2);
        assert_eq!(out.lines().count(), 3, "size=2 should produce 3 rows");
        for line in out.lines() {
            assert_eq!(
                line.chars().count(),
                3,
                "each row should be 3 chars: {:?}",
                line
            );
        }
    }

    #[test]
    fn render_symbol_block_size_3_is_5x5() {
        let lib = SymbolLibrary::new();
        let sym = resolve("star", &lib).unwrap();
        let out = render_symbol_block(&sym, 3);
        assert_eq!(out.lines().count(), 5, "size=3 should produce 5 rows");
    }

    // ── expand_symbols ────────────────────────────────────

    #[test]
    fn expand_known_symbol() {
        let lib = SymbolLibrary::new();
        let (out, warns) = expand_symbols("[sym:star]", &lib);
        assert_eq!(out, "★");
        assert!(warns.is_empty());
    }

    #[test]
    fn expand_unknown_gives_warning() {
        let lib = SymbolLibrary::new();
        let (out, warns) = expand_symbols("[sym:no-such-xyz]", &lib);
        assert!(out.contains("[sym:no-such-xyz]")); // unchanged
        assert_eq!(warns.len(), 1);
        assert_eq!(warns[0].code, "SYMBOL-001");
    }

    #[test]
    fn expand_prose() {
        let lib = SymbolLibrary::new();
        let (out, _) = expand_symbols("Result: [sym:checkmark] ok", &lib);
        assert_eq!(out, "Result: ✓ ok");
    }

    #[test]
    fn expand_multiple_symbols() {
        let lib = SymbolLibrary::new();
        let (out, _) = expand_symbols("[sym:checkmark] pass [sym:x] fail", &lib);
        assert_eq!(out, "✓ pass ✗ fail");
    }

    #[test]
    fn expand_no_symbols_unchanged() {
        let lib = SymbolLibrary::new();
        let text = "plain text no symbols";
        let (out, warns) = expand_symbols(text, &lib);
        assert_eq!(out, text);
        assert!(warns.is_empty());
    }

    #[test]
    fn expand_alias() {
        let lib = SymbolLibrary::new();
        let (out, _) = expand_symbols("[sym:cross]", &lib); // alias of x
        assert_eq!(out, "✗");
    }

    #[test]
    fn expand_skips_in_backtick() {
        let lib = SymbolLibrary::new();
        // `[sym:star]` — inside inline code, should not expand
        let (out, warns) = expand_symbols("`[sym:star]`", &lib);
        assert!(
            out.contains("[sym:star]"),
            "should not expand inside backtick: {}",
            out
        );
        assert!(warns.is_empty());
    }

    #[test]
    fn all_core_symbols_resolve() {
        let lib = SymbolLibrary::new();
        for name in &[
            "checkmark",
            "x",
            "warning",
            "info",
            "dot",
            "dot-open",
            "diamond",
            "star",
            "star-open",
            "arrow-right",
            "arrow-left",
            "arrow-up",
            "arrow-down",
            "triangle-up",
            "triangle-down",
            "rule-thin",
            "rule-double",
        ] {
            assert!(
                resolve(name, &lib).is_some(),
                "missing core symbol: {}",
                name
            );
        }
    }

    #[test]
    fn custom_symbol_overrides_builtin() {
        let custom = vec![CustomSymbol {
            name: "star".to_string(),
            character: "✨".to_string(),
            width: 2,
            art: None,
            height: None,
        }];
        let lib = SymbolLibrary::with_custom(custom);
        let sym = resolve("star", &lib).unwrap();
        assert_eq!(sym.chars(), "✨", "custom should override built-in star");
    }
}
