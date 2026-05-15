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

## AD-07: Bottom-close border treated as top of new box (Pattern C)

**Pattern:** When a multi-box flowchart has `└──┘` (bottom border of one box) followed by
connector lines (`│`, `▼`, text) and then `┌──┐` (top of next box), the detector greedily
pairs `└──┘` as the TOP of a phantom box and `┌──┐` as its BOTTOM. The connector lines
between them have wildly different widths → hundreds of false `ascii_box_width` and
`ascii_box_col` errors per flowchart.

**Domain:** Any ASCII/Unicode flowchart with stacked boxes separated by arrows or labels.
This is the dominant structure in the MAXIM reference library's landscape diagrams —
every guide's main diagram uses this pattern.

**Scale of impact:** 95 false errors eliminated from a 20-file directory scan after fix.
In a 2,170-file library this could account for thousands of false positives.

**Structural solution:** Add `can_open_box(line)` check before accepting a border line
as the TOP of a new box. A line whose first junction character is a bottom-left corner
(`└`, `╚`, `╰`) is closing a previous box, not opening a new one. Only `+`, `┌`, `╔`,
`╭` can legitimately open a box.

**Status:** SOLVED  
**Proved by:** `stacked_boxes_no_phantom_box_errors`, `bottom_close_border_not_treated_as_box_top`,
`bottom_left_corner_cannot_open_box` in `tests/integration_tests.rs`  
**Test:** All three tests in the Pattern C section of integration_tests.rs

---

## AD-10: GFM table rows contain `\|`, code spans, and operators — split('|') is wrong

**Pattern:** A simple `split('|')` on a GFM table row treats ALL pipe characters as column
separators. But GFM tables allow three contexts where `|` is content, not structure:
1. `\|` — escaped pipe (e.g. `catch (A\|B)`, `T \| null`)
2. Backtick code spans — `` `A|B` `` contains a literal pipe
3. Multi-char operators — `||` (SQL concat), `|>` (F# pipeline)

When an auto-fix based on simple splitting is applied to these rows, it inserts spaces
INSIDE escaped pipes (`\|` → `\ |`), inside code spans (`` `A | B` ``), and into operators
(`|>` → `| >`), producing corrupted markdown.

**Domain:** Any parser or transformer that processes GFM table rows character-by-character
or splits on `|` without handling escaping rules.

**Structural solution:** Walk the row character-by-character, tracking:
- `\\` before `|` → escaped pipe, treat as content
- Opening/closing backtick → inside code span, `|` is content
- Unescaped `|` outside code spans → column separator

A simple `split('|')` is never correct for GFM table rows.

**Status:** SOLVED in `parse_row()` in `markdown_table.rs`  
**Auto-fix NOT enabled:** `md_table_cell_padding` auto-fix remains disabled in `draft.rs`
until the fix generator itself uses the correct escaped-pipe-aware parser.  
**Discovered by:** Running `proof fix --dry-run` on a draft plan — fixes broke `\|`, `|>`,
and `` ` `` spans in language guide tables.  
**Test:** `markdown_table::tests::parse_row_handles_escaped_pipe_and_code_span`;
`markdown_table::tests::parse_row_handles_sql_concat_as_content_when_escaped_or_in_code`

---

## AD-09: Language names containing symbols trigger symbol-based heuristics

**Pattern:** Language names like `C#`, `F#`, `C++`, and `Objective-C` contain `#` and `+`
characters that are also used as markdown structural markers. A heuristic that checks
"does this heading end with `#`?" will false-positive on `## Gotchas from C#` and
`# Language: F#`. Similarly, a check for `++` would fire on headings about C++.

**Domain:** Any check that uses symbol characters as structural markers in contexts
where those same characters appear in human-readable identifiers.

**Structural solution:** Require the structural character to be preceded by a space
or appear at a specific structural position. For trailing `#` in headings: only flag
when the char before the trailing `#` run is whitespace — `## Title ##` (space before
`##`) is decoration, `## Gotchas from C#` (no space before `#`) is a language name.

**Status:** SOLVED  
**Discovered by:** Running `proof draft` on the languages/ directory — all 17
`md_heading_format` warnings were false positives on C# and F# guide headings.  
**Proved by:** Zero `md_heading_format` errors after fix; real trailing-hash headings
(with space before `##`) still detected correctly.  
**Test:** `markdown::tests::trailing_hash_style_warns_but_csharp_heading_is_clean`

---

## AD-08: Fixture design must match reality — never create a "clean" fixture that has errors

**Pattern:** Writing a fixture file intended to test "zero errors" but getting the widths
or padding wrong during authoring. The fixture has real errors (e.g., content row 1 char
wider than border, or missing trailing space). Tests then either fail immediately or — worse —
pass because the test is insufficiently specific.

**Domain:** Any test that writes fixture `.md` files and asserts "zero diagnostics."

**Structural solution:**
1. After creating a fixture file, run `proof check` on it manually before writing the test.
2. Tests that assert zero diagnostics should always verify against the specific fixture's
   content first.
3. If a "clean" fixture has errors, fix the fixture (not the test).

**Status:** SOLVED — procedure established  
**How discovered:** `stacked_boxes.md` initially had `│ Box Three│` (no trailing space),
triggering `ascii_cell_padding`. `bottom_border_only.md` had a branching diagram whose
`┌───────┴───────┐` created a false box. Both required fixture redesign.

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
**Workaround:** Use `tolerance = 1` in `proof.toml` to absorb single trailing-space drift.
**Test:** Not yet written — write a fixture with trailing spaces to lock in the behavior.
