//! Unicode super/subscript mapping for Tier 2 rendering.

const SUPERSCRIPT_MAP: &[(char, char)] = &[
    ('0', '⁰'),
    ('1', '¹'),
    ('2', '²'),
    ('3', '³'),
    ('4', '⁴'),
    ('5', '⁵'),
    ('6', '⁶'),
    ('7', '⁷'),
    ('8', '⁸'),
    ('9', '⁹'),
    ('+', '⁺'),
    ('-', '⁻'),
    ('=', '⁼'),
    ('(', '⁽'),
    (')', '⁾'),
    ('a', 'ᵃ'),
    ('b', 'ᵇ'),
    ('c', 'ᶜ'),
    ('d', 'ᵈ'),
    ('e', 'ᵉ'),
    ('f', 'ᶠ'),
    ('g', 'ᵍ'),
    ('h', 'ʰ'),
    ('i', 'ⁱ'),
    ('j', 'ʲ'),
    ('k', 'ᵏ'),
    ('l', 'ˡ'),
    ('m', 'ᵐ'),
    ('n', 'ⁿ'),
    ('o', 'ᵒ'),
    ('p', 'ᵖ'),
    ('r', 'ʳ'),
    ('s', 'ˢ'),
    ('t', 'ᵗ'),
    ('u', 'ᵘ'),
    ('v', 'ᵛ'),
    ('w', 'ʷ'),
    ('x', 'ˣ'),
    ('y', 'ʸ'),
    ('z', 'ᶻ'),
];

const SUBSCRIPT_MAP: &[(char, char)] = &[
    ('0', '₀'),
    ('1', '₁'),
    ('2', '₂'),
    ('3', '₃'),
    ('4', '₄'),
    ('5', '₅'),
    ('6', '₆'),
    ('7', '₇'),
    ('8', '₈'),
    ('9', '₉'),
    ('+', '₊'),
    ('-', '₋'),
    ('=', '₌'),
    ('(', '₍'),
    (')', '₎'),
    ('a', 'ₐ'),
    ('e', 'ₑ'),
    ('h', 'ₕ'),
    ('i', 'ᵢ'),
    ('j', 'ⱼ'),
    ('k', 'ₖ'),
    ('l', 'ₗ'),
    ('m', 'ₘ'),
    ('n', 'ₙ'),
    ('o', 'ₒ'),
    ('p', 'ₚ'),
    ('r', 'ᵣ'),
    ('s', 'ₛ'),
    ('t', 'ₜ'),
    ('u', 'ᵤ'),
    ('v', 'ᵥ'),
    ('x', 'ₓ'),
];

fn map_char(c: char, table: &[(char, char)]) -> Option<char> {
    table.iter().find(|(from, _)| *from == c).map(|(_, to)| *to)
}

/// Render `s` as a Unicode superscript.
/// Single character: try Unicode mapping, fall back to "^(s)".
/// Multiple characters: always use "^(s)" bracket notation.
pub fn to_superscript(s: &str) -> String {
    if s.is_empty() {
        return "^()".to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 1 {
        if let Some(c) = map_char(chars[0], SUPERSCRIPT_MAP) {
            return c.to_string();
        }
    }
    format!("^({})", s)
}

/// Render `s` as a Unicode subscript.
/// Single character: try Unicode mapping, fall back to "_(s)".
/// Multiple characters: always use "_(s)" bracket notation.
pub fn to_subscript(s: &str) -> String {
    if s.is_empty() {
        return "_()".to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 1 {
        if let Some(c) = map_char(chars[0], SUBSCRIPT_MAP) {
            return c.to_string();
        }
    }
    format!("_({})", s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_digit() {
        assert_eq!(to_superscript("2"), "²");
        assert_eq!(to_superscript("3"), "³");
        assert_eq!(to_superscript("0"), "⁰");
    }

    #[test]
    fn super_letter() {
        assert_eq!(to_superscript("n"), "ⁿ");
        assert_eq!(to_superscript("x"), "ˣ");
    }

    #[test]
    fn super_multi_char_bracket() {
        assert_eq!(to_superscript("n+1"), "^(n+1)");
        assert_eq!(to_superscript("ab"), "^(ab)");
        assert_eq!(to_superscript("2"), "²"); // single char still maps
    }

    #[test]
    fn sub_digit() {
        assert_eq!(to_subscript("0"), "₀");
        assert_eq!(to_subscript("1"), "₁");
    }

    #[test]
    fn sub_letter() {
        assert_eq!(to_subscript("i"), "ᵢ");
        assert_eq!(to_subscript("x"), "ₓ");
    }

    #[test]
    fn sub_multi_char_bracket() {
        assert_eq!(to_subscript("n+1"), "_(n+1)");
        assert_eq!(to_subscript("0"), "₀"); // single char still maps
    }

    #[test]
    fn super_unmapped_single_char() {
        // '@' has no Unicode superscript — falls back to bracket
        assert_eq!(to_superscript("@"), "^(@)");
        assert_eq!(to_superscript("?"), "^(?)");
        assert_eq!(to_superscript("q"), "^(q)"); // 'q' not in SUPERSCRIPT_MAP
    }

    #[test]
    fn sub_unmapped_single_char() {
        assert_eq!(to_subscript("@"), "_(@)");
        assert_eq!(to_subscript("q"), "_(q)"); // 'q' not in SUBSCRIPT_MAP
    }

    #[test]
    fn super_empty() {
        assert_eq!(to_superscript(""), "^()");
    }

    #[test]
    fn sub_empty() {
        assert_eq!(to_subscript(""), "_()");
    }
}
