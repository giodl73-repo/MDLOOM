---
title: GitHub Flavored Markdown Specification — Tables (§4.10)
source: https://github.github.com/gfm/#tables
retrieved: 2026-04-25
version: "0.29-gfm"
relevance: >
  GFM Tables are a GFM extension (not in CommonMark baseline). They are the
  primary structure for comparison data in the MAXIM library. proof's
  markdown_table check validates their structure and schema conformance.
---

# GFM Tables (§4.10)

## Overview

Tables are a GitHub Flavored Markdown extension. They are NOT in CommonMark
0.29 — if you need CommonMark portability, avoid GFM tables or use a plugin.

MkDocs supports GFM tables via the `tables` extension (enabled by default).

## Syntax

```
| Header 1 | Header 2 | Header 3 |
|----------|:--------:|----------:|
| Cell     | Cell     | Cell      |
| Cell     | Cell     | Cell      |
```

### Row 0: Header row

- Must contain at least one `|` character
- Cells separated by `|`
- Leading and trailing `|` are optional but conventional
- Whitespace around cell content is trimmed for comparison

### Row 1: Delimiter row (required)

Each cell must match:
```
\s*:?-+:?\s*
```

Where:
- Optional leading spaces
- Optional `:` — left-aligns if present
- One or more `-` characters
- Optional trailing `:` — right-aligns (or `:|` for center)
- Optional trailing spaces

**Minimum dashes:** GFM spec says "one or more" dashes. However, conventional
style and most renderers expect at least 3 dashes per cell for readability.
proof enforces `min_separator_dashes = 3` by default (configurable).

**Alignment syntax:**
- `---` — default (left align)
- `:---` — explicit left align
- `---:` — right align
- `:---:` — center align

### Rows 2+: Body rows

- Same `|`-delimited structure as header
- Column count must match header row
- Empty cells are allowed: `| |`
- Cells are trimmed of leading/trailing whitespace

## Column Count Consistency

> "The delimiter row determines the number of columns; the header row
> must have the same number of cells."

All body rows must have the same number of cells as the header row.
GFM renderers handle mismatches differently — some truncate, some pad.
proof reports them as errors to prevent this ambiguity.

## Cells Inside Code Spans

Cell content is parsed as inline markdown. This means `|` inside a code span
`` `foo | bar` `` within a cell is NOT a column separator. proof's simple
split-on-`|` parser does not handle this edge case — it will miscount columns
if cells contain code spans with `|`. Style guide recommendation: avoid `|`
inside table cells, or use HTML entities `&vert;` instead.

## Table Detection Heuristic

A GFM table is recognized when:
1. Row N: contains `|` (looks like a pipe-delimited row)
2. Row N+1: all cells match the delimiter pattern `:?-+:?`

Without the delimiter row in position 2, the preceding row is just a paragraph
containing pipes — not a table.

## Relationship to CommonMark

GFM tables are explicitly an extension:
> "Tables are not part of the CommonMark spec, but they are part of the GFM spec."

They are NOT supported by default in CommonMark parsers. MkDocs uses the
`tables` extension for Python-Markdown to support them.

## Relevance to proof Check Codes

| Check Code | What it validates | Spec reference |
|-----------|-------------------|---------------|
| `md_table_separator_invalid` | Delimiter row cell format; min dash count | §4.10 delimiter row syntax |
| `md_table_col_mismatch` | Column count consistent across all rows | §4.10 column count requirement |
| `md_table_cell_padding` | ≥ 1 space inside each cell | Style guide S-04 (not in spec) |
| `md_missing_table` | Required table count or named table | Schema config (not in spec) |
| `md_table_schema` | Column names, row keys, allowed values | Schema config (not in spec) |
