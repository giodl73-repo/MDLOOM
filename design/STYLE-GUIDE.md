# ASCII Art Style Guide for Markdown Documents

**Version:** 1.0 — 2026-04-25  
**Applies to:** All fenced code block diagrams in markdown documents  
**Enforced by:** glint (see constraint IDs below — e.g. S-01)  
**Specs:** `specs/commonmark-code-blocks.md` · `specs/gfm-code-blocks.md` ·
           `specs/unicode-east-asian-width.md` · `specs/mkdocs-rendering.md`

---

## Rationale

ASCII art diagrams in markdown are rendered in a monospace font where every
character occupies exactly one display column. Any misalignment — a row one
character wider than its border, a `|` at column 8 when column 9 is expected
— breaks the visual structure. At scale (2,000+ guides), these errors are
invisible to authors and accumulate silently.

This style guide establishes the rules that make diagrams render correctly
across CommonMark, GitHub GFM, and MkDocs Material. glint enforces them.

---

## Rule S-01 — Use only alignment-safe character ranges

**Constraint:** ASCII art diagrams must use only characters from these Unicode
ranges. All characters in these ranges render at exactly 1 display column in
every supported renderer.

| Range | Description | Width |
|-------|-------------|-------|
| U+0020–U+007E | Basic Latin (printable ASCII) | 1 col |
| U+2500–U+257F | Box Drawing | 1 col |
| U+2580–U+259F | Block Elements | 1 col |
| U+25A0–U+25FF | Geometric Shapes | 1 col |
| U+2190–U+21FF | Arrows | 1 col |

**Prohibited in diagrams:**
- CJK ideographs (U+4E00–U+9FFF) — 2 columns wide; breaks horizontal alignment
- Fullwidth Latin (U+FF01–U+FF60) — 2 columns wide; looks like ASCII but isn't
- Combining characters — 0 columns wide; shifts all subsequent characters
- Emoji (U+1F000+) — variable width; renderer-dependent

**Why:** Unicode East Asian Width standard (UAX #11) assigns 2-column width to
CJK and fullwidth characters. A `─` at the wrong position offsets every character
to its right.

**glint check:** Future — `ascii_char_range` warning when prohibited characters
appear in diagram lines.

---

## Rule S-02 — Box borders must be consistent width

**Constraint:** Every row of a box — top border, content rows, bottom border —
must have identical visual width.

```
CORRECT:
+----------+----------+   ← 22 cols
| cell one | cell two |   ← 22 cols ✓
| cell thr | cell fou |   ← 22 cols ✓
+----------+----------+   ← 22 cols ✓

WRONG:
+----------+----------+   ← 22 cols
| cell one | cell two |   ← 22 cols ✓
| cell thr | cell fou  |  ← 23 cols ✗ (extra trailing space)
+----------+----------+   ← 22 cols ✓
```

**Common cause:** One trailing space too many/few before the closing `|` or `│`.
Usually introduced when editing cell content without adjusting padding.

**Fix:** Count characters. Top border defines the expected width. Every other
row must match exactly.

**glint check:** `ascii_box_width` (error) — already enforced.

---

## Rule S-03 — Column separators must align with border junctions

**Constraint:** Every `|` or `│` in a content row must appear at the same visual
column as the corresponding `+`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┬`, `┴`, or `┼`
in the top border.

```
CORRECT:
+------+------+        ← junctions at cols 1, 8, 15
| good | good |        ← | at cols 1, 8, 15 ✓

WRONG:
+------+------+        ← junctions at cols 1, 8, 15
| bad |  bad  |        ← | at cols 1, 7, 15 ✗ (inner | shifted left)
```

**Fix:** Add or remove exactly one character in the cell that ends before the
misaligned `|`. The arithmetic: expected_col − actual_col = chars to add.

**glint check:** `ascii_box_col` (error) — already enforced.

---

## Rule S-04 — Cell content must have whitespace padding

**Constraint:** Every cell must have at least 1 space of padding on each side:

```
CORRECT:
+----------+
| content  |   ← 1 space on left, 1+ on right

WRONG:
+----------+
|content   |   ← 0 spaces on left ✗
|  content |   ← 0 spaces on right ✗
```

**Rationale:** Zero-padding makes diagrams hard to read and is a common editing
artifact (cell content copied in without surrounding spaces).

**glint check:** `ascii_cell_padding` (warning) — already enforced.

---

## Rule S-05 — No text after the closing delimiter

**Constraint:** Content rows in a box must end with `|` or `│`. Text after the
closing delimiter is an annotation error:

```
WRONG:
+-------------------+
| JVM Runtime       |  ← same JVM as Java    ← annotation makes row wider ✗
+-------------------+

CORRECT:
+-------------------+
| JVM Runtime       |
+-------------------+
Note: same JVM as Java   ← annotation goes OUTSIDE the box
```

**Why:** The annotation makes the row wider than the border, triggering
`ascii_box_width`. It's also semantically wrong — the box content is the
cell, not the prose after it.

**Fix:** Move the annotation to a line outside the code block, or create a
second cell column with a proper border.

**glint check:** `ascii_box_width` (error) — caught by width mismatch.

---

## Rule S-06 — Stacked boxes use connectors, not adjacent borders

**Constraint:** When stacking multiple boxes vertically (flowchart style),
separate them with connector lines (`│`, `▼`, `↓`, arrow text) — never place
a bottom border directly adjacent to a top border:

```
CORRECT:
┌──────┐
│ Box1 │
└──────┘
    │         ← connector line
    ▼
┌──────┐
│ Box2 │
└──────┘

WRONG:
┌──────┐
│ Box1 │
└──────┘
┌──────┐      ← bottom border of Box1 immediately followed by top border of Box2
│ Box2 │
└──────┘
```

**Why:** Even with the `can_open_box()` guard (glint won't detect a phantom box),
the visual appearance without a connector is ambiguous — readers can't tell where
one box ends and another begins.

**Note:** glint no longer generates false errors for the correct (connector)
pattern after the Pattern C fix. The wrong (adjacent) pattern is also handled
cleanly — it's just hard to read.

**glint check:** No error currently — this is a visual quality rule.

---

## Rule S-07 — Side-by-side boxes require precise width agreement

**Constraint:** When placing multiple boxes side by side on a single border line,
every content row must have exactly the same total visual width as the border:

```
CORRECT:
┌──────────┐  ┌──────────┐  ┌──────────┐    ← 36 cols total
│ column 1 │  │ column 2 │  │ column 3 │    ← 36 cols ✓
└──────────┘  └──────────┘  └──────────┘    ← 36 cols ✓

WRONG:
┌──────────┐  ┌──────────┐  ┌──────────┐    ← 36 cols
│ column 1 │  │ column 2 │  │ column 3  │   ← 37 cols ✗
```

**Why:** Side-by-side boxes form a single composite structure. Every row of the
composite must have the same width. This is Pattern F.

**Fix:** Verify total width of each content row. Add/remove trailing spaces in
the last cell to match the border width.

**glint check:** `ascii_box_width` (error) — already enforced.

---

## Rule S-08 — One box structure per code block (recommended)

**Recommendation:** Unless the layout specifically requires it, prefer one diagram
structure per code block. Multiple separate box structures in a single code block
increase detection complexity and false positive risk.

**Exception:** Flowcharts with multiple stacked or side-by-side boxes are the
natural use case for this library. They are expected and supported.

**glint check:** Not enforced — recommendation only.

---

---

## Markdown Table Rules (GFM §4.10)

Tables are the primary format for comparison data and decision guides.
These rules apply to GFM pipe tables in prose (outside code blocks).
See `specs/gfm-tables.md` for the full GFM spec.

---

### Rule S-09 — Separator row required, well-formed

**Constraint:** Every pipe table must have a delimiter row in position 2.
Each delimiter cell must contain ≥ 3 dashes (plus optional `:` for alignment).

```
CORRECT:
| Axis | Value |
|------|-------|    ← valid separator (≥ 3 dashes)
| x    | y     |

WRONG (2 dashes):
| Axis | Value |
|--|--|            ← invalid separator
| x | y |

WRONG (no separator):
| Axis | Value |
| x    | y     |   ← second row is body, not separator → not a GFM table
```

**glint check:** `md_table_separator_invalid` (warning)

---

### Rule S-10 — Consistent column count

**Constraint:** All rows (header, separator, body) must have the same number of
columns. Mismatches are rendering-ambiguous — different markdown parsers
handle them differently (truncate vs. pad).

```
CORRECT:
| A | B |
|---|---|
| x | y |

WRONG:
| A | B |
|---|---|
| x | y | extra |   ← 3 cols in body, 2 in header
```

**glint check:** `md_table_col_mismatch` (error)

---

### Rule S-11 — Cell padding in tables

Same as S-04 for ASCII art boxes: every table cell must have ≥ 1 space of
padding on each side.

```
CORRECT:  | content |
WRONG:    |content|     ← no padding
WRONG:    | content|    ← missing right padding
```

**glint check:** `md_table_cell_padding` (warning)

---

### Rule S-12 — Every guide must contain at least one comparison table

**Constraint:** Every content guide must have at least one GFM pipe table.
The style contract requires tables for comparisons and cheat sheets.

```toml
[markdown_table]
required_tables = 1
```

**glint check:** `md_missing_table` (warning)

---

### Rule S-13 — Named tables must exist under their headings

**Constraint:** When a table schema specifies `heading = "## Section Name"`,
that section must contain a pipe table. If the heading exists but has no table,
glint warns.

```toml
[[markdown_table.table_schemas]]
heading = "## Decision Cheat Sheet"
min_body_rows = 2
```

**glint check:** `md_missing_table` (warning)

---

### Rule S-14 — Table schema: required columns, row keys, allowed values

**Constraint:** For sections with a formal schema (like Type System Snapshot),
the table must have the required column headers and required first-column values.
Allowed value sets can also be enforced per column.

```toml
[[markdown_table.table_schemas]]
heading = "## Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = ["Binding", "Typing", "Strength", "Type system", "Type inference", "Memory model"]
min_body_rows = 4

[[markdown_table.table_schemas.column_allowed_values]]
# Future: constrain what values are valid for specific columns
```

**glint check:** `md_table_schema` (warning)

---

## Enforcement Summary

| Rule | glint Code | Severity | Status |
|------|-----------|----------|--------|
| S-01: alignment-safe chars only | `ascii_char_range` | error/warn | ✅ Enforced |
| S-02: consistent border width | `ascii_box_width` | error | ✅ Enforced |
| S-03: column separator alignment | `ascii_box_col` | error | ✅ Enforced |
| S-04: cell padding ≥ 1 space | `ascii_cell_padding` | warning | ✅ Enforced |
| S-05: no text after closing `\|` | `ascii_box_width` | error | ✅ Enforced |
| S-06: connectors between stacked boxes | — | — | Not enforced (visual quality) |
| S-07: side-by-side width agreement | `ascii_box_width` | error | ✅ Enforced |
| S-08: one structure per block | — | — | Not enforced (recommendation) |
| S-09: table separator required + ≥ 3 dashes | `md_table_separator_invalid` | warning | ✅ Enforced |
| S-10: consistent column count | `md_table_col_mismatch` | error | ✅ Enforced |
| S-11: table cell padding | `md_table_cell_padding` | warning | ✅ Enforced |
| S-12: ≥ 1 table per guide | `md_missing_table` | warning | ✅ Enforced (config) |
| S-13: named table must exist under heading | `md_missing_table` | warning | ✅ Enforced |
| S-14: table schema (columns, row keys, values) | `md_table_schema` | warning | ✅ Enforced |

---

## Quick Reference — Checking a Diagram

```
1. Count chars in top border → N
2. Count chars in every content row → must equal N
3. Find + positions in top border → call these "junction columns"
4. Find | positions in each content row → must match junction columns exactly
5. Each cell: starts with space, ends with space before |
6. Closing | is the last char on the line (nothing after it)
7. Use only chars from U+0020-U+007E, U+2500-U+257F, U+2190-U+21FF
```

Run `glint check --format rich myfile.md` to get precise line:col locations
with surrounding context for every violation.
