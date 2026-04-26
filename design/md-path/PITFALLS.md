# md-path Pitfalls

Structural failure modes in the md:// URI resolver. Each describes a class
of error that is easy to introduce and hard to notice.

---

## MP-01: Heading normalization divergence between implementations

**Pattern:** The spec says "GitHub-compatible heading normalization" but different
implementations interpret this differently. `C++ Basics` might normalize to
`c-basics`, `c--basics`, or `cpp-basics` depending on how punctuation is handled.
Two tools using the same `md://` URI will resolve different headings.

**Structural solution:** The normalization algorithm must be fully specified and
tested against a canonical test suite — not referenced as "GitHub-compatible."

**Status:** SOLVED in spec v0.1 (explicit algorithm with character table)
**Test:** `tests/normalization.rs` — canonical input/output pairs for every edge case

---

## MP-02: Label false positive on language-tagged code blocks

**Pattern:** A code block with a language info string (` ```python `) starts
with `def foo():` — text-only, no box chars. The label detector incorrectly
identifies `def foo():` as the figure label, and `proof pin` registers it.

Later, `md://file.md#section:figure:my-function` resolves to the Python code
block, not the diagram it was supposed to address.

**Structural solution:** Rule 1 (inline label) requires NO language info string
on the opening fence. ` ```python ` or any fence with text after backticks
disqualifies the block from inline label detection.

**Status:** SOLVED in spec
**Test:** `tests/label-detection/` — language-tagged block should never produce label

---

## MP-03: Slash in heading text breaks subsection path parsing

**Pattern:** A document has `## Input/Output Handling` as a heading. The normalized
form is `input-output-handling`. Another document has `## Input` with child `## Output Handling`.
Both produce path `#input/output-handling`.

Worse: a URI `md://file.md#input/output` is ambiguous — is it heading `input/output`
(single heading with slash) or parent `input` / child `output`?

**Structural solution:** Special characters in heading text that collide with the
URI path separator must be percent-encoded when constructing the URI.
`Input/Output` → heading anchor becomes `input-output-handling` (slash stripped,
not a separator). The `/` in the heading path is ONLY a path separator, never
literal content.

**Alternative:** Require headings with `/` to use a different separator (`::` or `>`).
Chosen approach: strip `/` during normalization (it becomes a dash), not a separator.

**Status:** SOLVED in normalization algorithm (strip `/` as punctuation)
**Test:** `tests/normalization.rs` — heading with slash normalizes without creating path ambiguity

---

## MP-04: Integer vs. string selector visual ambiguity

**Pattern:** `md://file.md#section:figure:0` — is `:0` the integer index 0, or a
figure with the label "0" (e.g., a step-zero label, a license plate diagram)?

If the resolver treats all digit-only selectors as integers, a figure named
literally "0" can never be addressed by name. If it tries name first, the
integer address breaks whenever a figure is labeled.

**Structural solution:** Parse rule: if selector matches `^\d+$`, treat as integer.
Integer-addressed elements cannot have a coincident string label. Label detection
must reject pure-digit labels to eliminate the ambiguity.

**Status:** SOLVED in spec
**Test:** `tests/selector-parsing.rs` — `:0` is always integer, labels may not be pure digits

---

## MP-05: Substring label matching resolves wrong element

**Pattern:** A section contains two figures: one labeled "Request Handler" and one
labeled "Request Handler — async". The selector `:figure:request-handler` (normalized)
is a substring of both. `md://` resolves to whichever appears first in document order,
which may not be what the author intended.

Adding the invariant `contains-text: "async"` to the DaVinci catches the drift,
but the wrong element was pinned in the first place.

**Structural solution:** Matching hierarchy: exact → starts-with → substring.
At each level, if >1 match, emit `md_label_ambiguous` instead of silently
picking first. Authors must use a more specific selector.

**Status:** SOLVED in spec (exact → starts-with → substring → error)
**Test:** `tests/label-matching.rs` — ambiguous labels must always error, never silently pick

---

## MP-06: Numeric URI breaks when label added later

**Pattern:** A figure has no label when first pinned. proof generates
`md://file.md#section:figure:0`. Later, an author adds an inline label
`GOROUTINE SCHEDULER` to the figure. The figure now has a name, but the
DaVinci entry still uses `:0`.

Two problems:
1. The numeric URI still works (resolves by index) but is now stale — it
   doesn't express the element's identity
2. If another figure is added ABOVE this one in the same section, `:0`
   now addresses the WRONG figure silently

**Structural solution:** When proof resolves a numeric URI and discovers the
element now HAS a label, emit a warning: `[md_numeric_uri_stale]` with the
named form. `proof pin` should refuse to register a numeric URI if a named
form is available.

**Status:** OPEN — warning not yet implemented
**Action:** Add `md_numeric_uri_stale` warning to resolver output when named form available

---

## MP-07: Code block detection inside code blocks

**Pattern:** A code block contains documentation showing example code, including
markdown fences. The md:// resolver parses the OUTER file line-by-line and
misidentifies the inner ` ``` ` markers as real fence boundaries.

```markdown
```
Here is an example of a markdown code block:

` `` `
some code
` `` `
```
```

The resolver enters the outer block, then exits early when it sees the inner
backticks (interpreted as a closing fence).

**Structural solution:** The code block parser must use the exact fence length
rule from CommonMark: a closing fence must use the same character type and be
at least as long as the opening fence. An inner ` ``` ` (3 backticks) cannot
close an outer ```` ``` ```` (3 backticks) if the inner appears at a different
indentation level — UNLESS it exactly matches the opening fence's length and
indent.

**Status:** Known — standard CommonMark fence parsing required
**Test:** `tests/nested-fences.rs` — fence inside fence must not prematurely close outer

---

## MP-08: Empty section — type selector on section with no matching elements

**Pattern:** `md://file.md#section:figure:0` — the section exists but contains
no code blocks. The resolver returns `md_element_not_found`. The error message
says "element not found" but doesn't distinguish between "section not found"
and "section exists but has no figures."

Authors may assume their heading anchor is wrong and waste time debugging the
section path when the real issue is that no figures exist yet.

**Structural solution:** Distinguish error codes:
- `md_section_not_found` — heading anchor doesn't match any section
- `md_element_not_found` — section found but no elements of that type exist

Both errors should include the resolved heading path to confirm which case occurred.

**Status:** SOLVED in error codes (two distinct codes)
**Test:** `tests/error-messages.rs` — empty section produces `md_element_not_found`, not `md_section_not_found`
