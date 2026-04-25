# ASCII Art Detection Pitfalls (AD-01..AD-06)

Structural failure modes in ASCII art detection. Each describes a class of false positive,
false negative, or edge case that makes the detection algorithm unreliable.

---

## AD-01: Visual-width vs. byte-width conflation

**Pattern:** Treating `len()` (byte count) as the display width of a string. Unicode box-drawing
characters like `│` and `─` are multi-byte (3 bytes each in UTF-8) but single-column wide.
Code that computes `line.len()` to determine box width will report every Unicode box as
misaligned.

**Domain:** Any code that measures line width for alignment comparison without using a
unicode-width library.

**Structural solution:** Always compute visual width through `unicode_width::UnicodeWidthChar`
or equivalent. Store and compare visual column positions, not byte offsets, for alignment checks.

**Status:** SOLVED
**Proved by:** `ascii_box.rs` uses `visual_width()` with `UnicodeWidthChar` throughout
**Test:** `tests/integration_tests.rs::perfect_box_zero_diagnostics` (Unicode box case in `perfect_box.md`)

---

## AD-02: Border detection fires on prose with pipes

**Pattern:** A heuristic that marks any line starting with `|` as a box content row will
false-positive on Markdown table rows, code snippets with `|` operators, and inline examples.
A box border heuristic that fires on `| Option A | Option B |` will claim it's the top row of
a box, then search for a bottom border and report an "unclosed box" everywhere.

**Domain:** Markdown files where prose tables (`| col | col |`) and ASCII art boxes coexist.

**Structural solution:** Restrict box detection to fenced code blocks (`code_blocks_only = true`).
Markdown tables outside code blocks are not ASCII art and should not be validated as boxes.
Inside code blocks, add the two-junction minimum rule: a border line must contain at least
two `+` or Unicode corner/junction characters, not just one `|` at each end.

**Status:** SOLVED
**Proved by:** `code_blocks_only = true` in default config; `is_border_line()` requires `junction_count >= 2`
**Test:** `tests/integration_tests.rs::perfect_box_zero_diagnostics` (prose tables in fixture)

---

## AD-03: Mixed ASCII and Unicode box characters cause false misalignments

**Pattern:** A box that mixes `+---+` borders (ASCII) with `│` vertical bars (Unicode) will
confuse a detector that only looks for `|` in content rows or only looks for `+` in border rows.
Some editors auto-complete Unicode while the author typed ASCII — the result looks correct
visually but uses different code points on different rows.

**Domain:** Any document created in multiple editors or pasted from different sources.

**Structural solution:** Normalize detection: `is_border_junction()` accepts both `+` and all
Unicode corner/junction variants (`┌┐└┘├┤┬┴┼`). `is_vertical()` accepts both `|` and Unicode
vertical variants (`│║╎┆┊`). The junction column extractor records visual column positions
regardless of whether the character is ASCII or Unicode.

**Status:** SOLVED
**Proved by:** `is_border_junction()` and `is_vertical()` in `ascii_box.rs` cover both character sets
**Test:** `tests/integration_tests.rs::perfect_box_zero_diagnostics` (both styles in fixture)

---

## AD-04: Nested boxes — inner border triggers false outer-row validation

**Pattern:** When a box contains another box (a common layout in architecture diagrams), the
inner top border line has `+` junction characters. If the detector is tracking the outer box and
encounters the inner border line as a "content row," it will check that the inner `+` positions
align with the outer `|` positions — and report misalignments that aren't real.

**Domain:** Architecture diagrams with multiple nested levels.

**Structural solution:** Classify a line as a border (not content) if it passes the `is_border_line()`
test. Only lines that are NOT border lines are validated as content rows. The outer box validator
skips inner border lines — it treats them as content rows only if they fail the border heuristic.
This means inner borders that are short enough may still be checked as content, but the width check
provides a natural catch: an inner box border will differ in width from the outer box border.

**Status:** PARTIAL — inner box borders inside outer content rows are not checked for their own
internal alignment. Full nested detection would require a recursive box parser.
**Test:** `tests/integration_tests.rs::complex_diagram_inner_box_misalignment`

---

## AD-05: Cell padding check fires on border lines

**Pattern:** A cell padding checker that iterates over all lines starting with `|` will also
process border lines like `+--+--+`. A border line `+------+------+` starts with `+`, not `|`,
so this specific case is safe — but a partially-drawn border that starts with `|` (e.g., a
continuation of a box) will be misidentified as a content row.

**Domain:** Cell padding validation.

**Structural solution:** Before checking cell padding, call `is_content_line()` to confirm the
line starts AND ends with `|` or `│`. Border lines typically start with `+` or `┌/└`, so they
pass through cleanly. Also guard against empty cells (after splitting) to avoid spurious warnings
on lines with `||` adjacency.

**Status:** SOLVED
**Proved by:** `check_cell_padding()` calls `is_content_line(trimmed)` before processing
**Test:** `tests/integration_tests.rs::cell_padding_correct_rows_no_warnings`

---

## AD-06: Tolerance=0 breaks on trailing spaces

**Pattern:** Authors sometimes add a trailing space to visually align lines in their editor.
A trailing space makes `visual_width()` return N+1 for that row, and with `tolerance=0` this
triggers a false width mismatch on every row that has a trailing space — even when the box
is visually correct.

**Domain:** Any file created with editors that do not strip trailing whitespace.

**Structural solution:** Either strip trailing whitespace before measuring visual width, or offer
`trim_trailing_whitespace = true` as a config option. The default config should tolerate trailing
spaces because they are invisible and don't constitute a real misalignment.

**Status:** OPEN — current implementation measures visual width including trailing spaces.
**Workaround:** Use `tolerance = 1` in `glint.toml` to absorb single trailing-space drift.
**Test:** Not yet written — write a fixture with trailing spaces to lock in the behavior.
