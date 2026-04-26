---
title: GitHub Flavored Markdown Specification — Fenced Code Blocks
source: https://github.github.com/gfm/#fenced-code-blocks
retrieved: 2026-04-25
version: "0.29-gfm"
relevance: >
  GFM is the rendering target for GitHub repository browsing. The MAXIM library
  is published to GitHub and browsed via the GitHub web interface. GFM fenced
  code blocks are identical to CommonMark — no GFM-specific extensions apply.
---

# GitHub Flavored Markdown — Fenced Code Blocks (§4.5)

## Relationship to CommonMark

**GFM fenced code blocks are identical to CommonMark 0.29 §4.5.**

GFM extends CommonMark in several areas:
- Tables (§4.10) — pipe-delimited tables
- Task list items (§5.3) — `- [ ]` checkboxes  
- Strikethrough (§6.5) — `~~deleted~~`
- Autolinks extended (§6.9) — bare URLs
- Disallowed raw HTML (§6.11) — certain tags filtered

Fenced code blocks are **explicitly not extended** — the spec states them
under "Leaf blocks" with no GFM-specific annotations.

## Practical Implications for proof

Since GFM = CommonMark for code blocks:
- A CommonMark-compliant parser is GFM-compliant for code blocks
- No separate GFM handling needed
- Validation results identical on GitHub as on any CommonMark renderer

## GitHub Rendering Specifics (Non-Spec Behavior)

These are observed behaviors not formally specified:

### Monospace Font

GitHub renders code blocks in a monospace font (Cascadia Mono, SFMono, Consolas,
Liberation Mono, Menlo, Monaco, Courier New — in priority order depending on OS).

**Character width consequences:**
- ASCII characters (U+0021–U+007E): always 1 column wide
- Box-drawing chars (U+2500–U+257F): rendered at 1 column wide in GitHub's font stack
- CJK ideographs: 2 columns wide (East Asian Width = W)
- Combining characters: 0 columns wide

### Syntax Highlighting

GitHub applies syntax highlighting based on the info string. This does NOT affect
character widths or whitespace — it's purely visual (color). proof may use info
strings to enable language-specific validation in future.

### Rendering Parity with proof

proof uses the `unicode-width` crate (Unicode East Asian Width standard) for
visual width calculation. This aligns with GitHub's rendering behavior for all
characters in the Basic Multilingual Plane.

**Known divergence:** Box-drawing characters (U+2500–U+257F) have EAW=Neutral,
so `unicode-width` assigns them 1 column — correct for GitHub's rendering.

## Style Constraint Derived from This Spec

**Constraint S-01:** ASCII art in code blocks must use only characters from these
Unicode ranges to guarantee consistent rendering across all CommonMark renderers:

| Range | Description | Width |
|-------|-------------|-------|
| U+0020–U+007E | Basic Latin (printable ASCII) | 1 col |
| U+2500–U+257F | Box Drawing | 1 col |
| U+2580–U+259F | Block Elements | 1 col |
| U+25A0–U+25FF | Geometric Shapes | 1 col |
| U+2190–U+21FF | Arrows | 1 col |

Characters outside these ranges may render at different widths in different
fonts/renderers and should not appear in ASCII art diagrams.
