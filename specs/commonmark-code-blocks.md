---
title: CommonMark Specification 0.31.2 — Fenced Code Blocks
source: https://spec.commonmark.org/0.31.2/#fenced-code-blocks
retrieved: 2026-04-25
version: "0.31.2"
relevance: >
  Defines the authoritative syntax for fenced code blocks. proof restricts
  ASCII art validation to content inside fenced code blocks; this spec governs
  what counts as a code block, how indentation is stripped, and what content
  is preserved verbatim.
---

# CommonMark Fenced Code Blocks (§4.5)

## Formal Definition

A **code fence** is a sequence of at least three consecutive backtick characters
`` ` `` (U+0060) or tildes `~` (U+007E). Backticks and tildes cannot be mixed.

A **fenced code block** begins with a code fence and ends with a matching
closing fence of the same type and at least the same length.

## Opening Fence Rules

- Minimum 3 identical characters
- Optional leading indentation of 0–3 spaces
- Optional **info string** on the same line, separated from the fence by optional whitespace
- **Backtick fences only**: info string must not contain any backtick characters
- Tilde fences allow any characters in the info string

## Content Preservation

> "The content of the code block consists of all subsequent lines, until a
> closing code fence."

Content is **treated as literal text, not parsed as inlines**:
- All spaces and tabs preserved exactly
- Blank lines preserved
- No Markdown processing occurs inside the block

### Indentation Stripping

If the opening fence is indented N spaces (0–3):
- Up to N spaces are stripped from the start of each content line
- Lines with fewer than N leading spaces have all leading spaces removed
- Tabs are treated as single characters (not expanded to multiple spaces)

## Closing Fence Rules

1. Same character type (backticks or tildes) as the opening fence
2. At least as many delimiters as the opening fence
3. Indented 0–3 spaces (irrespective of opening indentation)
4. Only spaces or tabs after the closing delimiters — nothing else

## Unclosed Blocks

If no closing fence is found, the code block extends to end of document
(or end of the containing block element).

## Key Examples (from spec §4.5)

### Minimal
```example
```
foo
bar
```
```
→ code block with content `foo\nbar\n`

### With info string
```example
```ruby
def foo(x)
  return 3
end
```
```
→ code block, language `ruby`

### Indentation stripping (fence indented 1 space)
```example
 ```
 aaa
aaa
```
```
→ content: `aaa\naaa\n` (1 space stripped where present)

## Relevance to proof

1. **Code block detection** — proof's `code_blocks_only = true` mode (default) must
   correctly identify fenced block boundaries using CommonMark rules.

2. **Info string** — proof may use info strings to select language-specific checks
   (future feature).

3. **Indentation stripping** — when a code block is indented inside a list or
   blockquote, proof must strip leading indentation before measuring visual widths.
   *(Current implementation does not strip — known limitation.)*

4. **Blank lines** — blank lines inside a code block are valid content.
   proof must not report `ascii_box_width: row width 0` for blank lines that
   appear between two unrelated diagram elements.
   *(This was fixed by skipping width checks on rows with no vertical separators.)*
