# md:// — Markdown Element Addressing Specification v0.1

The `md://` URI scheme provides stable, named addresses for individual elements
(diagrams, charts, tables, code blocks, and prose) within markdown documents.
It is the foundation of proof's DaVinci protection tier, cross-file consistency
checks, and stable error reporting.

`md://` is an open addressing scheme — any tool (editors, CI systems, AI agents)
can implement a resolver. `proof` is the reference implementation.

The scheme describes the resource type (markdown content), not the resolver —
the same principle as `http://` being independent of which browser opens it.

**Status:** Design — implementation in progress in proof.

---

## Naming Principle: Strings Over Numbers

**Names are always preferred over numeric indexes. Numbers are the last resort.**

Wherever an element has a discoverable name, use it. This makes URIs stable
across edits, readable without opening the file, and natural for humans and AI.

| Element | Name source | Named URI |
|---------|------------|-----------|
| Figure | First text-only line inside fence | `:figure:goroutine-scheduler` |
| Figure box | First word(s) inside the box | `[box=PREPROCESSOR]` |
| Table row | First column cell value | `[row=Binding]` |
| Table column | Header cell text | `[col=Value]` |
| Table cell | Row + column name | `[row=Binding,col=Value]` |
| Chart bar | Label text before the bar | `[bar=Option A]` |
| Section | Heading text (normalized) | `#concurrency-model` |
| Subsection | Parent heading / child heading | `#section/sub-section` |

When proof generates a URI (error output, fix plans, `proof pin`), it always
resolves to the named form first. Numeric fallback only when no name exists:

```
# Named (preferred):
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler

# Numeric fallback (no label found):
md://languages/10-GO.md#concurrency-model:figure:2
```

---

## URI Grammar

```
md://path[#heading-path][:[type[.kind]:]selector][sub-selector][?query]

path          = file path relative to proof root (.md file)
heading-path  = segment[/segment]*   (parent/child heading hierarchy)
segment       = GitHub-normalized heading text (lowercase, spaces→dashes)
type          = figure | table | chart | text | heading
kind          = subtype: figure.flowchart, table.key-value, etc.
selector      = string-label (preferred) | integer-index (fallback)
sub-selector  = [key=value,...] — select within an element
query         = ?select=cols&filter=cond&count&top=N&skip=M
```

### Addressing levels (most to least specific)

```
md://path.md                               → whole file
md://path.md#heading                       → a section
md://path.md#parent/child                  → a subsection
md://path.md#section:heading              → just the heading line
md://path.md#section:0                    → shorthand: first figure
md://path.md#section:figure.flowchart:name → named figure of specific kind
md://path.md#section:table.key-value:0    → first key-value table
md://path.md#section:table:0[row=Binding] → a specific row
md://path.md#section:table:0[row=Binding,col=Value] → a specific cell
md://path.md#section:chart.bar:0[bar=Option A]      → a specific bar
md://path.md#section:figure.flowchart:name[box=PREPROCESSOR] → box in figure
md://path.md#section:text:0               → first prose paragraph
```

### Full examples

```
md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:package-hierarchy
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler[box=GRQ]
md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding]
md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding,col=Value]
md://sections/computing-software.md#directories:table:0[col=Entry Point]
md://computing/02-C.md#compilation-pipeline:figure.flowchart:0[box=PREPROCESSOR]
md://file.md#section:table:0?select=Axis,Value&filter=Axis eq Binding
md://file.md#section:figure:0/$count
```

---

## Types and Kinds

### `figure` — fenced code block

Any fenced code block (``` or ~~~). Default type.

| Kind | Description | Detection |
|------|-------------|-----------|
| *(no kind)* | Any code block | Has opening fence |
| `figure.flowchart` | Boxes connected by arrows | Box chars + connector chars (│▼→) |
| `figure.layer-stack` | Stacked horizontal layers | Multiple equal-width boxes, stacked |
| `figure.side-by-side` | Boxes on same border line | Multiple boxes on one border |
| `figure.box` | Single container box | One outer box |
| `figure.tree` | Hierarchical branches | │ ├ └ without forward arrows |
| `figure.sequence` | Sequence diagram style | Vertical timeline with actors |
| `figure.matrix` | Grid layout | N×M cell structure |

### `table` — GFM pipe table

Pipe-delimited markdown table in prose (outside fences).

| Kind | Description |
|------|-------------|
| *(no kind)* | Any GFM table |
| `table.key-value` | 2 columns, key in first |
| `table.comparison` | Options compared across columns |
| `table.reference` | Reference data rows |
| `table.decision` | When-to-use guide |

### `chart` — bar chart

Detected by consecutive block characters (█▓▒░#) in a code block.

| Kind | Description |
|------|-------------|
| `chart.bar` | Horizontal or vertical bars |
| `chart.timeline` | Time-based progression |

### `text` — prose paragraph

Contiguous non-blank prose lines, not inside a fence, not a list, not a heading.

### `heading` — the heading line itself

Just the `## Section Name` line — distinct from the section as a whole.

---

## Sub-Selectors

Sub-selectors address elements within a resolved element using `[key=value]`:

### Table sub-selectors

```
[row=Name]              → row whose first column value starts with "Name"
[col=Name]              → column whose header matches "Name"
[row=Name,col=Name]     → the specific cell at that intersection
[row=2]                 → row by 0-based index (fallback when no name)
[col=1]                 → column by 0-based index (fallback)
```

### Figure sub-selectors

```
[box=Label]             → box whose first interior word(s) match "Label"
[box=2]                 → 3rd detected box (fallback)
[row=3]                 → 4th line of the code block (raw line access)
```

### Chart sub-selectors

```
[bar=Label]             → bar whose label matches "Label"
[bar=2]                 → 3rd bar (fallback)
```

---

## Query Parameters (OData-inspired)

Optional query modifiers appended with `?`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `select=cols` | Return only specified columns (tables) | `?select=Axis,Value` |
| `filter=condition` | Filter rows by condition | `?filter=Axis eq Binding` |
| `count` | Return count instead of content | `?count` |
| `top=N` | Return first N results | `?top=5` |
| `skip=N` | Skip first N results | `?skip=2` |

---

## Label Resolution

For figures and charts, the name/label is determined in priority order:

1. **Inline label** — first non-empty line INSIDE a fence with no language info string,
   if it contains only text characters (no `┌│─+` box chars):
   ```
   GOROUTINE SCHEDULER — M:N multiplexing   ← this is the label
   ┌──────────────────────────────────────┐
   ```

2. **Preceding label** — last non-empty markdown line BEFORE the fence,
   if bold (`**text**`) or a short standalone text line (≤ 60 chars).

3. **No label** — falls back to numeric index.

For boxes within a figure: the first word(s) inside the box (before any `│` or
non-text character) are the box name: `│ PREPROCESSOR │` → name = `PREPROCESSOR`.

For table rows: the first column cell value is the row name.
For table columns: the header cell value is the column name.
For chart bars: the text before the bar characters is the bar label.

**Matching:** normalize both name and selector (lowercase, collapse whitespace,
strip punctuation), check if selector is a substring. Prefer exact match;
fall back to substring. Error `md_label_ambiguous` if multiple matches.

---

## Use in Error Reporting

Every proof diagnostic carries a `uri` field with the md:// address of the
containing element. This makes errors:

- **Stable** — survives line number changes
- **Navigable** — any md:// resolver can jump directly to the element
- **Groupable** — errors sharing the same URI base belong to the same element
- **AI-addressable** — the fix-guide can operate on a named element, not a line number

### Error output format

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
  line 26  error [ascii_box_width]: row width 70 ≠ box width 71

md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding,col=Value]
  line 14  warning [md_table_cell_padding]: cell missing left padding: "Late"
```

### In fix plans (draft-plan.json)

```json
{
  "id": "fix-042",
  "uri": "md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler",
  "description": "Outer box wall at col 70, expected 71",
  "file": "languages/10-GO.md",
  "line": 26,
  "old_string": "│  Goroutines (G) — lightweight, 2KB stack      │",
  "new_string": "",
  "auto": false,
  "context": { "expected_cols": [1, 71], "actual_cols": [1, 70] }
}
```

The AI reviewer sees the md:// URI, knows exactly which named figure is affected,
and fills `new_string` with the precise fix — no file-hunting required.

### Fix plan grouping

`proof draft` groups all diagnostics sharing the same md:// base URI into one
fix group. All errors inside `goroutine-scheduler` figure = one group, one
AI decision, one `decision` field in the plan.

---

## Use in DaVinci Registration

```toml
[[davinci]]
id = "goroutine-scheduler"
uri = "md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler"
description = "The M:N goroutine scheduling diagram — referenced across documentation"
template = "stacked-flowchart"
protection = "error"

  [[davinci.invariants]]
  rule = "contains-text"
  text = "M:N multiplexing"

[[davinci]]
id = "csharp-binding-cell"
uri = "md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0[row=Binding,col=Value]"
description = "C# Binding description must mention non-virtual dispatch"
invariants = [{ rule = "contains-text", text = "non-virtual" }]
protection = "error"
```

---

## Resolution Algorithm

1. Locate file at `path` relative to proof root.
2. Parse into heading hierarchy. Walk `heading-path` segments to find the target section.
   If no heading path: whole file is the section.
3. If type is absent and selector is absent: return the section.
4. Collect elements of the specified `type` within the section.
5. Apply `selector` (string label preferred, integer fallback).
6. Apply `sub-selector` `[key=value]` within the resolved element.
7. Apply `query` parameters (`?select`, `?filter`, etc.).

---

## Templates

```toml
[templates]
files = ["templates/my-templates.yaml"]

[[templates]]  # inline alternative
name = "company-flowchart"
kind = "figure.flowchart"
```

```yaml
# templates/my-templates.yaml
- name: "architecture-overview"
  kind: "figure.layer-stack"
  invariants:
    - rule: "box-count"
      min: 3
      max: 5
    - rule: "all-boxes-same-width"
      value: true

- name: "language-type-table"
  kind: "table.key-value"
  invariants:
    - rule: "required-row-keys"
      values: ["Binding", "Typing", "Strength", "Type system", "Type inference", "Memory model"]
```

---

## CLI Commands

```bash
proof resolve "md://..."                    # print element + metadata
proof check "md://..."                      # validate against invariants
proof pin "md://..." --id name --template t # register as DaVinci
proof spec generate "md://..."              # AI generates invariants
proof spec template "md://..."              # AI generates template YAML
proof pin list                              # list all DaVinci elements
proof scan . --suggest-daVinci              # suggest candidates for pinning
```

---

## Error Codes

| Code | Condition |
|------|-----------|
| `md_file_not_found` | Path does not exist |
| `md_section_not_found` | Heading anchor matches no section |
| `md_section_ambiguous` | Multiple sections match; use longer path |
| `md_element_not_found` | Index out of range |
| `md_label_not_found` | Label matches no element |
| `md_label_ambiguous` | Label matches multiple elements |
| `md_subkey_not_found` | `[row=X]` or `[box=Y]` not found in element |
| `md_invariant_violated` | DaVinci invariant check failed |
| `md_template_not_found` | Named template not registered |

---

## Built-in Invariant Rules

| Rule | Parameters | Applies to |
|------|-----------|------------|
| `box-width` | `min`, `max` | figure |
| `box-count` | `value`, `min`, `max` | figure |
| `column-count` | `value` | figure, table |
| `row-count` | `value`, `min`, `max` | table |
| `contains-text` | `text` | all |
| `not-contains-text` | `text` | all |
| `equals` | `text` | heading, cell |
| `required-row-keys` | `values: [...]` | table |
| `all-boxes-same-width` | `value: true` | figure |
| `starts-with` | `text` | figure, text |
| `ends-with` | `text` | figure, text |
| `heading-exists` | `text` | section |
| `pattern` | `regex` | all |
| `bar-proportional` | `tolerance: N` | chart |

---

## Cross-File Consistency Groups

```toml
[[consistency-group]]
name = "package-hierarchy-references"
uris = [
    "md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:package-hierarchy",
    "md://computing/00-OVERVIEW.md#landscape:figure:0",
]
rules = ["same-box-count", "same-box-width"]
```

---

## Design Decisions

**`md://` not `proof://`**: The scheme names the resource type, not the resolver.
Any tool can implement an `md://` resolver. proof is the reference implementation.

**Strings over numbers**: Names are stable; numbers are not. Named URIs survive
document editing; numeric URIs break whenever content is added above.

**Heading path not line numbers**: `#section/sub` survives edits above the element.
Line numbers break on every insert.

**Sub-selectors for cells**: `[row=Binding,col=Value]` enables cell-level DaVinci
protection — the most granular quality guarantee possible.

**OData query params**: Adopted `$select`, `$filter`, `$count`, `$top/$skip` as
`?select`, `?filter`, `?count`, `?top`, `?skip` for collection operations.

**URI in error output**: Every diagnostic carries its md:// address so errors are
stable, groupable, and directly addressable by AI fix agents.

---

## Future Work

- `md://` resolver as standalone library crate (editor plugins, GitHub Actions)
- `proof diff md://A md://B` — diff two elements
- Cross-repo references: `md://repo:path#heading`
- Watch mode: re-validate DaVinci on file change
- Editor hover: show `md://` address + invariants for element under cursor
- Template registry: share template packs across teams
