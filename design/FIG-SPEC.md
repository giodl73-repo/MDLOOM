# md:// — Markdown Element Addressing Specification v0.1

The `md://` URI scheme provides stable, named addresses for individual elements
(diagrams, charts, tables, code blocks, and prose) within markdown documents.
It is the foundation of proof's DaVinci protection tier and cross-file
consistency checks.

`md://` is an open addressing scheme — any tool (editors, CI systems, AI agents)
can implement a resolver. `proof` is the reference implementation.

The scheme describes the resource type (markdown content), not the resolver —
the same principle as `http://` being independent of which browser opens it.

**Status:** Design — implementation in progress in proof.

---

## URI Grammar

```
md://path[#heading][:[type[.kind]:]selector]

Components:
  path      = file path relative to proof root (where proof.toml lives)
              Must end in .md
  heading   = GitHub-normalized heading anchor (optional)
              "## The Big Picture" → "the-big-picture"
  type      = element type: figure | table | chart | text
              Omit to default to figure
  kind      = subtype within a type (optional, see Types and Kinds)
              type.kind together, e.g. figure.flowchart
  selector  = integer (0-based) | label-text (normalized substring match)
              Omit selector to address the section itself (no element)
```

### Addressing levels

```
md://path.md                          → the whole file
md://path.md#heading                  → a section (heading + all content)
md://path.md#heading:0               → shorthand: first figure in section
md://path.md#heading:figure:0        → first figure (any kind)
md://path.md#heading:figure.flowchart:0  → first flowchart
md://path.md#heading:table:0         → first GFM table
md://path.md#heading:table.key-value:0   → first key-value table
md://path.md#heading:chart.bar:0     → first bar chart
md://path.md#heading:text:0          → first prose paragraph
md://path.md#heading:figure:label    → figure by label match
```

### Examples

```
md://computing/01-PACKAGE.md#the-big-picture:0
  → first figure in "## The Big Picture"

md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:0
  → first layer-stack diagram in "## The Big Picture"

md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
  → flowchart in "## Concurrency Model" whose label matches "goroutine scheduler"

md://languages/05-CSHARP.md#type-system-snapshot:table.key-value:0
  → the Type System Snapshot table

md://sections/computing-software.md#directories:table:0
  → the Directories navigation table

md://computing/01-PACKAGE.md#the-big-picture
  → the section itself (for section-level invariants)
```

---

## Types and Kinds

### `figure` — fenced code block

Any fenced code block (``` or ~~~). The default type — most diagrams are figures.

| Kind | Description | Detection |
|------|-------------|-----------|
| `figure` (no kind) | Any code block | Has opening fence |
| `figure.flowchart` | Boxes connected by arrows (│ ▼ →) | Has box chars + connector chars |
| `figure.layer-stack` | Horizontal layers stacked vertically | Multiple equal-width boxes, stacked |
| `figure.side-by-side` | Columns of boxes on same border line | Multiple boxes on one border line |
| `figure.box` | Single container or boundary box | One outer box |
| `figure.tree` | Hierarchical tree with branches | Has │ ├ └ connectors without forward arrows |
| `figure.sequence` | Sequence diagram style | Vertical timeline with actors |
| `figure.matrix` | Grid / matrix layout | N×M cell structure |

### `table` — GFM pipe table

Pipe-delimited table in markdown prose (outside code fences).

| Kind | Description | Detection |
|------|-------------|-----------|
| `table` (no kind) | Any GFM table | Has `\|---|` separator row |
| `table.key-value` | 2 columns, key in first | 2 cols, first col is axis/key |
| `table.comparison` | Options compared across columns | 3+ cols, first col is scenario |
| `table.reference` | Reference data, no "winner" | Many rows, factual cells |
| `table.decision` | When-to-use guide | Has "Use when" or "Choose" headers |

### `chart` — bar chart

Detected by consecutive block characters (█▓▒░#) in a code block.

| Kind | Description |
|------|-------------|
| `chart.bar` | Horizontal or vertical bar chart |
| `chart.timeline` | Time-based horizontal progression |

### `text` — prose paragraph

Contiguous non-blank prose lines not inside a code fence, not a list item,
not a heading. Useful for pinning explanatory text alongside DaVinci diagrams.

---

## Label Resolution

For figures and charts, a label is detected in priority order:

1. **Inline label** — first non-empty line INSIDE the code block that:
   - Contains only text characters (no box-drawing chars, no `|`, no `+`)
   - The fence has no language info string (` ``` ` not ` ```python `)
   ```
   GOROUTINE SCHEDULER — M:N multiplexing   ← label
   ┌──────────────────────────────────────┐
   ```

2. **Preceding label** — last non-empty markdown line BEFORE the fence,
   if it is bold (`**text**`) or a short standalone text line (≤ 60 chars).

3. **No label** — block cannot be addressed by label selector.

For tables: the label is the content of the first column header cell.

**Matching:** normalize (lowercase, collapse whitespace, strip punctuation),
check if selector is a substring. Error if ambiguous (multiple matches).

---

## Resolution Algorithm

1. Locate file at `path` relative to proof root.
2. Parse file into sections bounded by headings.
3. Find section whose normalized heading equals the `heading` component.
   If no heading component: treat whole file as one section.
4. Collect all elements of the specified `type` within the section,
   in document order.
5. Apply `selector`:
   - Integer N → element at 0-based index N
   - Label text → first element whose label matches (normalized substring)
   - Absent → return the section itself (no element selected)

---

## Templates

Templates define the expected structure of a figure/table/chart kind.
They provide base invariants that DaVinci entries inherit automatically.

### Built-in templates

| Template | Kind | Base invariants |
|----------|------|----------------|
| `stacked-flowchart` | figure.flowchart | ≥2 boxes, connector lines between boxes |
| `layer-stack` | figure.layer-stack | All boxes same width, stacked |
| `side-by-side` | figure.side-by-side | Boxes on one border line, equal height |
| `key-value-table` | table.key-value | Exactly 2 columns, ≥3 body rows |
| `comparison-table` | table.comparison | ≥3 columns, ≥2 body rows |
| `bar-chart` | chart.bar | Block chars, value column aligned |

### Custom templates (YAML)

Define your own templates in YAML and register them in proof.toml:

```yaml
# templates/my-templates.yaml

- name: "architecture-overview"
  kind: "figure.layer-stack"
  description: "3-5 horizontal layers for system architecture"
  invariants:
    - rule: "box-count"
      min: 3
      max: 5
    - rule: "box-width"
      min: 60
      max: 80

- name: "language-type-table"
  kind: "table.key-value"
  description: "Standard 6-row type system comparison"
  invariants:
    - rule: "column-count"
      value: 2
    - rule: "required-row-keys"
      values: ["Binding", "Typing", "Strength", "Type system", "Type inference", "Memory model"]
```

```toml
# proof.toml
[templates]
files = ["templates/my-templates.yaml"]
```

### Generating templates from existing elements

```bash
proof spec template "md://computing/01-PACKAGE.md#the-big-picture:0"
# → AI detects kind, generates template YAML from current figure state
# → Output: ready to paste into templates/my-templates.yaml
```

---

## DaVinci Registration

```toml
# proof.toml

[[davinci]]
id = "package-layer-stack"
uri = "md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:0"
description = "Canonical package manager hierarchy — 5-level stack"
template = "layer-stack"          # inherits template invariants
protection = "error"              # warn | error | lock

  # Additional invariants beyond the template:
  [[davinci.invariants]]
  rule = "contains-text"
  text = "SYSTEM / OS LAYER"

  [[davinci.invariants]]
  rule = "contains-text"
  text = "LANGUAGE / RUNTIME LAYER"

  [[davinci.invariants]]
  rule = "box-count"
  value = 5
```

### Protection tiers

| Tier | Behavior |
|------|---------|
| `warn` | `proof check` emits a warning if invariant violated |
| `error` | `proof check` emits an error (blocks CI) |
| `lock` | `proof check` fails AND `proof fix` refuses to touch this element |

---

## Built-in Invariant Rules

| Rule | Parameters | Description |
|------|-----------|-------------|
| `box-width` | `min`, `max` | Visual width of box border |
| `box-count` | `value`, `min`, `max` | Number of detected boxes |
| `column-count` | `value` | Number of column separators per row |
| `row-count` | `value`, `min`, `max` | Number of body rows (tables) or lines |
| `contains-text` | `text` | Must contain this string (case-insensitive) |
| `not-contains-text` | `text` | Must NOT contain this string |
| `required-row-keys` | `values: [...]` | First-column values that must all appear |
| `all-boxes-same-width` | `value: true` | All box borders have identical width |
| `starts-with` | `text` | First non-empty line starts with this |
| `ends-with` | `text` | Last non-empty line ends with this |
| `heading-exists` | `text` | Section heading must contain this text |
| `pattern` | `regex` | Content matches regex |

---

## Cross-File Consistency Groups

```toml
[[consistency-group]]
name = "package-hierarchy-references"
description = "All files that reference the package layer concept must agree"
uris = [
    "md://computing/01-PACKAGE.md#the-big-picture:figure.layer-stack:0",
    "md://computing/00-OVERVIEW.md#landscape:figure:0",
]
rules = ["same-box-count", "same-box-width"]
```

---

## CLI Commands

```bash
# Resolve — print the element + metadata
proof resolve "md://computing/01-PACKAGE.md#the-big-picture:0"

# Check a specific element against its registered invariants
proof check "md://computing/01-PACKAGE.md#the-big-picture:0"

# Pin as DaVinci — registers in proof.toml, generates invariants
proof pin "md://computing/01-PACKAGE.md#the-big-picture:0" \
    --id package-layer-stack \
    --template layer-stack \
    --protection error

# Generate invariants from current state (AI-assisted)
proof spec generate "md://computing/01-PACKAGE.md#the-big-picture:0"

# Generate a template YAML from an existing element
proof spec template "md://computing/01-PACKAGE.md#the-big-picture:0"

# List all registered DaVinci elements
proof pin list

# Scan a directory and suggest candidates for DaVinci registration
proof scan . --suggest-daVinci
```

---

## Error Codes

| Code | Condition |
|------|-----------|
| `md_file_not_found` | Path does not exist |
| `md_section_not_found` | Heading anchor matches no section |
| `md_element_not_found` | Index out of range |
| `md_label_not_found` | Label matches no element in section |
| `md_label_ambiguous` | Label matches more than one element |
| `md_invariant_violated` | DaVinci invariant check failed |
| `md_template_not_found` | Named template not registered |

---

## Design Decisions

**Why `md://` not `proof://`?**
The scheme names the resource type (markdown content), not the resolver tool.
Any editor, CI system, or AI agent can implement an `md://` resolver.
`proof` is the reference implementation of an open standard.

**Why heading anchors not line numbers?**
Line numbers change whenever content is added above the figure. Heading names
are stable. This makes `md://` addresses survive normal document editing.

**Why types and kinds?**
`figure.flowchart` is more useful than just `figure:0` because it lets
proof apply kind-specific validation (a flowchart must have connector arrows,
a layer-stack must have consistent box widths). Templates build on kinds.

**Why label substring matching?**
Exact matching is fragile for long labels. Normalized substring matching
makes labels forgiving while still precise enough for disambiguation.

**Why YAML for custom templates?**
YAML is human-readable, diffable, and versionable. Teams can publish template
packs as separate files. AI can generate template YAML from existing diagrams.

---

## Future Work

- `md://` resolver as a standalone library crate
- `proof diff md://A md://B` — diff two elements for consistency
- Cross-repo references: `md://repo:path#heading:type:index`
- Watch mode: `proof watch --daVinci .` — re-validate on file change
- Editor plugin: hover over a diagram to see its `md://` address and invariants
- Template registry: share templates across teams/projects
