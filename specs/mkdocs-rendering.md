---
title: MkDocs Material — Code Block Rendering
source: https://squidfunk.github.io/mkdocs-material/reference/code-blocks/
retrieved: 2026-04-25
version: "MkDocs Material 9.x"
relevance: >
  The MAXIM reference library uses MkDocs for its rendered site. Code blocks
  in MkDocs Material are rendered with a specific font stack. proof must produce
  validation results that match the visual rendering in the MkDocs output.
---

# MkDocs Material Code Block Rendering

## Rendering Stack

MkDocs Material renders fenced code blocks using:

```
Markdown source
    ↓
Python-Markdown (parser — CommonMark subset)
    ↓
Pygments (syntax highlighting)
    ↓
HTML <pre><code> block
    ↓
CSS: font-family: monospace stack
    ↓
Browser: renders with available fonts
```

## Monospace Font Stack (default theme)

MkDocs Material uses the following CSS font stack for code:

```css
code, pre, kbd {
  font-family:
    var(--md-code-font),           /* user override */
    SFMono-Regular,                /* macOS */
    Consolas,                      /* Windows */
    Menlo,                         /* macOS fallback */
    Roboto Mono,                   /* Android/Chrome */
    monospace;                     /* system fallback */
}
```

All fonts in this stack are monospace fonts that render ASCII characters at
exactly 1 column width. Box-drawing characters are supported in all listed
fonts.

## Character Width in MkDocs Rendering

For the characters used in ASCII art diagrams:

| Character Type | Renders at | Notes |
|---------------|-----------|-------|
| Basic ASCII (U+0020–U+007E) | 1 column | Guaranteed by monospace font |
| Box Drawing (U+2500–U+257F) | 1 column | Supported in SFMono, Consolas, Menlo |
| Unicode Arrows (U+2190–U+21FF) | 1 column | Supported in most modern fonts |
| Geometric Shapes (U+25A0–U+25FF) | 1 column | Supported in most modern fonts |
| CJK Ideographs (U+4E00–U+9FFF) | **2 columns** | Font fallback to CJK font |
| Fullwidth Latin (U+FF01–U+FF60) | **2 columns** | Legacy fullwidth forms |

This matches the `unicode-width` crate's output exactly for these ranges.

## Syntax Highlighting Note

MkDocs Material applies Pygments syntax highlighting based on info strings.
For ASCII art diagrams, no info string should be used (or use `text`):

```markdown
` ` `
+------+------+
| good | data |
+------+------+
` ` `
```

Avoid language-specific highlighting for diagram blocks — some highlighters
transform or color characters in ways that may confuse authors about alignment.

## Validation Alignment

proof's visual width measurement using `unicode-width` aligns with MkDocs
Material rendering for all characters recommended in the style guide.

**Verified compatible:**
- Box-drawing characters: `unicode-width` = 1, MkDocs renders at 1 ✓
- ASCII: `unicode-width` = 1, MkDocs renders at 1 ✓
- Unicode arrows: `unicode-width` = 1, MkDocs renders at 1 ✓

**Known incompatibility (style guide prohibits these):**
- CJK characters: `unicode-width` = 2, MkDocs renders at 2 ✓ (matches)
  but visual alignment depends on font having proper 2-col CJK glyphs

## Python-Markdown vs CommonMark

Python-Markdown (used by MkDocs) is not fully CommonMark-compliant. Key
differences that affect code blocks:

1. **Indentation requirement**: Python-Markdown sometimes requires 4 spaces
   for indented code blocks (not relevant for fenced blocks)
2. **Fenced blocks**: Supported via the `fenced_code` extension (included in
   `mkdocs.yml` by default)
3. **Info strings**: Passed to Pygments for syntax highlighting

For proof purposes: fenced code block detection is compatible between
Python-Markdown and CommonMark. The differences do not affect ASCII art
validation.
