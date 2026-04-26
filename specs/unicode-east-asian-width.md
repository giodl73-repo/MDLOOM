---
title: Unicode Standard Annex #11 — East Asian Width
source: https://www.unicode.org/reports/tr11/
retrieved: 2026-04-25
version: "Unicode 15.1"
relevance: >
  Defines how many display columns each Unicode character occupies in
  monospace (fixed-pitch) contexts. proof uses the `unicode-width` Rust crate
  which implements this standard to compute visual column positions for
  alignment validation.
---

# Unicode East Asian Width (UAX #11)

## Purpose

Specifies the display width of Unicode characters for East Asian legacy systems
and modern terminals. The standard arose from the need to mix narrow (Latin)
and wide (CJK) characters in fixed-pitch displays.

## The Six Width Categories

| Category | Symbol | Display Width | Description |
|----------|--------|--------------|-------------|
| Wide | W | **2 columns** | CJK ideographs, Hiragana, Katakana, Hangul |
| Fullwidth | F | **2 columns** | Fullwidth ASCII (Ａ, Ｂ, …) — legacy East Asian forms |
| Halfwidth | H | **1 column** | Halfwidth Katakana — legacy narrow forms |
| Narrow | Na | **1 column** | ASCII letters, digits, punctuation |
| Neutral | N | **1 column** | Everything else not classified above |
| Ambiguous | A | **1 or 2** | Context-dependent (e.g., accented Latin, Greek, Cyrillic) |

### Ambiguous (A) — Default Treatment

> "Ambiguous characters should be treated as narrow (1 column) by default
> when context cannot be established reliably."

For Western/terminal contexts: treat A as 1 column. This is what
`unicode-width` does and what most terminals do.

## Width Algorithm

```
visual_width(s: &str) -> usize:
  sum over each char c in s:
    if EAW(c) == Wide or Fullwidth:  +2
    if EAW(c) == Halfwidth or Narrow or Neutral:  +1
    if EAW(c) == Ambiguous:  +1  (Western default)
    if EAW(c) == zero-width (combining, etc.): +0
```

The `unicode-width` Rust crate implements this exactly.

## Characters Relevant to ASCII Art Diagrams

| Unicode Range | EAW | Width | Examples |
|--------------|-----|-------|---------|
| U+0020–U+007E (Basic Latin) | Na | 1 | Space, `+`, `-`, `\|`, `A`–`Z` |
| U+2500–U+2509 (Box Drawing) | N | 1 | `─`, `│`, `┌`, `┐`, `└`, `┘` |
| U+250A–U+257F (Box Drawing) | N | 1 | `├`, `┤`, `┬`, `┴`, `┼`, `═`, `║` |
| U+2190–U+21FF (Arrows) | N | 1 | `←`, `→`, `↑`, `↓`, `↔`, `▶`, `◀` |
| U+25A0–U+25CF (Geometric) | A→1 | 1 | `▼`, `▲`, `■`, `□` |
| U+4E00–U+9FFF (CJK) | W | **2** | 中, 国, 語 |
| U+FF01–U+FF60 (Fullwidth) | F | **2** | Ａ, Ｂ, ０ |

## Implications for proof

### Why unicode-width is the right choice

1. **Standards-based** — Implements UAX #11, the authoritative standard for
   terminal/monospace character width.

2. **Matches GitHub rendering** — GitHub's code block font stack renders
   box-drawing characters at 1 column, matching unicode-width's output.

3. **Matches MkDocs rendering** — MkDocs Material uses standard browser fonts
   where box-drawing characters are 1 column wide.

4. **Known divergence** — CJK characters in ASCII art diagrams will measure as
   2 columns in proof but may render differently in some web fonts (font
   fallbacks can cause display width ≠ measured width). Style guide constraint:
   avoid CJK in ASCII art diagrams.

### Style Constraint Derived from This Spec

**Constraint S-02:** ASCII art in code blocks must not use characters with
`EAW = Wide (W)` or `EAW = Fullwidth (F)` — these are 2-column wide and will
break horizontal alignment in diagrams designed with 1-column-per-character
assumptions.

Specifically: no CJK ideographs, fullwidth Latin, or fullwidth symbols inside
box-drawing diagrams.

**Constraint S-03:** Ambiguous (A) characters (accented Latin, Greek, Cyrillic,
some symbols) default to 1 column in proof. Authors should avoid using them
in horizontally-aligned diagrams because their width may vary across renderers.
