# proof

Document quality assurance for markdown corpora. `proof` catches ASCII art geometry defects, GFM table schema violations, broken links, and heading structure errors with file:line:col precision — then fixes them with an AI-assisted pipeline.

---

## What it does

Markdown documentation at scale accumulates drift: boxes whose borders don't add up, tables missing required rows, internal links that rotted, headings in the wrong order. `proof check` catches all of it. `proof draft` sends failures to an AI that writes a structured fix plan. `proof fix` applies the plan.

```
$ proof check languages/08-TYPESCRIPT.md

languages/08-TYPESCRIPT.md:34:1  error    ascii_box_width      bottom border 64 chars, top border 63
languages/08-TYPESCRIPT.md:34:1  warning  ascii_cell_padding   col 2 row 3: content flush against delimiter
languages/08-TYPESCRIPT.md:112:1 warning  md_missing_section   required ## "Type System Snapshot" absent
languages/08-TYPESCRIPT.md:198:1 error    table_missing_row    "Memory model" row required in Type System Snapshot

4 diagnostics (2 errors, 2 warnings)
```

---

## Commands

| Command | Purpose |
|---------|---------|
| `proof check [paths]` | Lint markdown files |
| `proof draft [paths]` | Generate a pre-populated fix plan |
| `proof fix --plan plan.json` | Apply a fix plan to the working tree |
| `proof compile [paths]` | Compile `.source.md` files (resolves `proof:include` / `proof:layout`) |
| `proof layout [uris/files]` | Compose N figures side-by-side as an ASCII collage |
| `proof resolve "md://..."` | Resolve an `md://` URI, print content and metadata |
| `proof pin "md://..."` | Register a figure with DaVinci invariants |
| `proof pin-list` | List all pinned DaVinci figures |
| `proof init` | Create a `proof.toml` in the current directory |
| `proof config [path]` | Show effective config for a file |
| `proof stats` | Error counts by directory and code |

---

## What it validates

### ASCII art

Every code block that contains a box, flowchart, or bar chart is parsed as structured art:

```
+------------------+     +------------------+
| token stream     | --> | AST              |     ← proof validates:
+------------------+     +------------------+     • border widths match top/bottom
                                                   • cell padding ≥ 1 space each side
+----------+----------+                            • column separators aligned
| col A    | col B    |                            • arrow bodies unbroken
| value    | value    |                            • connector lines don't drift
+----------+----------+
```

### GFM table schemas

Tables carry structure that matters. Declare what you require in `proof.toml` and every file is held to it:

```toml
[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = ["Binding", "Typing", "Strength", "Type system", "Type inference", "Memory model"]
min_body_rows = 4
```

### Link integrity

```toml
link_columns = ["Directory", "Entry Point"]
verify_link_targets = true
link_auto_fix = "directory"
```

Every cell in a link column must contain a markdown link. `verify_link_targets = true` resolves each path on disk.

### Heading structure

```toml
[markdown]
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet", "Type System Snapshot"]
```

### Character safety

Wide and fullwidth Unicode characters (CJK, em-dashes in the wrong encoding, presentation forms) break ASCII art column alignment silently. `proof` flags them before they corrupt diagrams.

---

## The fix pipeline

```
proof check --format rich . -o rich.json
     │
     │  rich format: each diagnostic includes the surrounding code block,
     │  expected vs actual column widths, and adjacent lines — everything
     │  an AI needs to reason about the fix without reading the whole file
     ▼
proof draft [paths] -o plan.json
     │
     │  generates a pre-populated fix plan
     │  auto-fixable errors carry decision: auto
     │  ambiguous cases carry decision: needs_review for human or AI triage
     ▼
proof fix --plan plan.json --dry-run          # review what will change
proof fix --plan plan.json --min-confidence high   # apply high-confidence fixes
     │
     ▼
proof check .                                 # verify: should be clean
```

The plan format is JSON and human-readable — review before applying:

```json
{
  "schema_version": "1",
  "fixes": [
    {
      "id": "fix-001",
      "file": "languages/08-TYPESCRIPT.md",
      "confidence": "high",
      "description": "Remove extra char from bottom border of type table",
      "reasoning": "Top border width: 63. Bottom width: 64. Extra trailing + has no matching top.",
      "edit": {
        "line": 34,
        "old_string": "+------+------++",
        "new_string": "+------+------+"
      }
    }
  ]
}
```

Fixes apply bottom-up (highest line number first) so earlier line numbers stay valid after later edits. If `old_string` doesn't match current file content, the fix is skipped and logged — no silent corruption.

---

## proof compile

Source documents use `proof:include` and `proof:layout` fenced directives to reference figures by `md://` URI. The compiler resolves each reference, validates DaVinci invariants, and writes the compiled `.md` file.

**Mental model**: source markdown is source code. Compiled markdown is the artifact. DaVinci invariants are types. The compiler enforces types before output ships.

### Source document format

````markdown
## Concurrency Model

Intro text here.

```proof:include
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
```

Compare Go and Rust concurrency:

```proof:layout gap=4 labels="Go,Rust"
md://languages/10-GO.md#concurrency-model:0
md://languages/09-RUST.md#ownership-model:0
```
````

Compile it:

```bash
proof compile languages/10-GO.source.md
# Output: languages/10-GO.md  (drops .source. in-place)

proof compile src/*.source.md          # batch
proof compile . --check                # validate without writing
proof compile . --watch                # recompile on change
```

### Figure files

Figure files are standalone `.md` files whose figures are marked with `<!-- proof:figure -->` HTML comments immediately preceding each code fence. The comment is hidden in rendered output but gives the following code block a stable named identity:

```markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
│  OS Thread (M)                      │
│  ┌──────┐ ┌──────┐ ┌──────┐        │
│  │  G   │ │  G   │ │  G   │  ...   │
│  └──────┘ └──────┘ └──────┘        │
└─────────────────────────────────────┘
```
```

### Compile diagnostics

| Code | Meaning |
|------|---------|
| `COMPILE-001` | DaVinci invariant violation — compile aborted (error protection) |
| `COMPILE-002` | URI resolve failure — `md://` address not found |
| `COMPILE-003` | DaVinci invariant violation — warning only (warn protection) |
| `COMPILE-007` | Figure validation warning — figure content changed since last pin |

---

## proof layout

Compose N figures side-by-side as a single aligned ASCII art collage. Figures are fetched by `md://` URI or file path; the engine handles height equalization, gap insertion, and unicode-width-aware column alignment.

```bash
# Two figures, 4-space gap, labelled
proof layout \
    "md://languages/10-GO.md#concurrency-model:0" \
    "md://languages/09-RUST.md#ownership-model:0" \
    --gap 4 \
    --labels "Go,Rust"

# Three files, wrap into 2 columns
proof layout fig1.md fig2.md fig3.md --gap 3 --cols 2

# Output to file
proof layout fig1.md fig2.md --gap 4 -o layout.md
```

Example output (gap=4, labels="Go,Rust"):

```
Go                      Rust
┌────────────────────┐    ┌────────────────────┐
│  goroutines (M:N)  │    │  ownership system  │
│  ┌──┐ ┌──┐ ┌──┐   │    │  ┌──────────────┐  │
│  │G │ │G │ │G │   │    │  │  borrow ck   │  │
│  └──┘ └──┘ └──┘   │    │  └──────────────┘  │
└────────────────────┘    └────────────────────┘
```

Layout invariants guaranteed: all frames in a row equalized to the same height (L-2), visual gap exactly N spaces (L-4), unicode box-drawing chars measured at 1 column (L-5), labels centered over frame visual width (L-7).

---

## DaVinci pinning

Pin a figure with invariants it must always satisfy. If a future edit violates an invariant, `proof compile` reports it as an error.

```bash
proof pin "md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler" \
    --id goroutine-scheduler
```

This writes a `[[davinci]]` entry to `proof.toml`:

```toml
[[davinci]]
id = "goroutine-scheduler"
uri = "md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler"
protection = "error"

  [[davinci.invariant]]
  rule = "contains-text"
  value = "M:N multiplexing"

  [[davinci.invariant]]
  rule = "box-count"
  min = 2
```

List pinned figures:

```bash
proof pin-list
```

```
goroutine-scheduler  md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
                     invariants: contains-text("M:N multiplexing"), box-count(min=2)
                     protection: error
```

---

## proof resolve

Resolve an `md://` URI and print the element content and metadata:

```bash
proof resolve "md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler"
```

```
uri:             md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
file:            languages/10-GO.md
lines:           47–61
element_type:    figure
kind:            flowchart
label:           GOROUTINE SCHEDULER — M:N multiplexing
section_heading: Concurrency Model

--- content ---
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
...
```

---

## Check codes

### ASCII art

| Code | Sev | Description |
|------|-----|-------------|
| `ascii_box_width` | error | Row or border width mismatch |
| `ascii_box_col` | error | Column separator misaligned |
| `ascii_cell_padding` | warning | Missing whitespace inside cell delimiter |
| `ascii_char_range` | warning | Character outside expected ASCII range in art block |
| `ascii_barchart_align` | warning | Horizontal bar chart column misaligned |

### Markdown structure

| Code | Sev | Description |
|------|-----|-------------|
| `md_h1_count` | warning | Too many H1 headings |
| `md_missing_section` | warning | Required `## Heading` absent |
| `md_duplicate_heading` | warning | Same heading appears more than once |
| `md_heading_order` | warning | Heading level skips (H1 → H3 with no H2) |
| `md_missing_pattern` | warning | Required content pattern absent |
| `md_file_length` | warning | File exceeds `max_lines` |

### Table schemas

| Code | Sev | Description |
|------|-----|-------------|
| `table_missing_column` | error | Required column absent from table |
| `table_missing_row` | error | Required row key absent from table |
| `table_min_rows` | warning | Table has fewer body rows than `min_body_rows` |
| `table_bad_value` | error | Cell value not in `allowed_values` list |

### Link integrity

| Code | Sev | Description |
|------|-----|-------------|
| `link_bare_text` | error | Link column cell contains plain text, not a markdown link |
| `link_broken_target` | error | Link target does not exist on disk |
| `link_missing` | warning | Expected link is absent from table cell |

---

## proof.toml

Schema-driven and cascading. A root `proof.toml` sets library-wide defaults. Per-directory `proof.toml` files inherit and extend — required lists are additive, scalars use nearest.

```toml
# root proof.toml
[meta]
name = "My Library"

[files]
include = ["**/*.md"]
exclude = ["CHANGELOG.md", "_archive/**"]
root = true

[ascii_box]
enabled = true
tolerance = 0
code_blocks_only = true

[markdown]
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]

[[markdown.required_patterns]]
pattern = "```"
description = "every guide must contain at least one code block"
severity = "warning"

# languages/proof.toml — inherits root, adds more
[[section_schemas]]
paths = ["*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Type System Snapshot", "Syntax Reference Card"]

[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = ["Binding", "Typing", "Strength"]
min_body_rows = 3
```

A file at `languages/08-TYPESCRIPT.md` is checked against both the root rules and the `languages/` child schema. Effective config for any file: `proof config languages/08-TYPESCRIPT.md`.

---

## Install

Build from source:

```bash
git clone https://github.com/giodl73-repo/PROOF
cd PROOF
cargo build --release
./target/release/proof --version
```

`cargo install proof` — available once the crate is published to crates.io.

The `md://` URI resolver is a separate library crate (`mdpath`) at [github.com/giodl73-repo/MDPATH](https://github.com/giodl73-repo/MDPATH). `proof` depends on it via path in the workspace; no separate install step needed.

---

## Design

- [design/SPEC.md](design/SPEC.md) — full specification
- [design/COMPILE-SPEC.md](design/COMPILE-SPEC.md) — compile pipeline specification
- [design/LAYOUT-SPEC.md](design/LAYOUT-SPEC.md) — layout composer specification
- [design/THREE-TIER-CACHE.md](design/THREE-TIER-CACHE.md) — caching architecture
- [design/CACHE-SNAPSHOTS.md](design/CACHE-SNAPSHOTS.md) — cache snapshot system
- [design/SCENARIOS.md](design/SCENARIOS.md) — 31 resolved spec findings
- [design/INVARIANTS.md](design/INVARIANTS.md) — invariants with test traceability
- [design/FIG-SPEC.md](design/FIG-SPEC.md) — `md://` URI addressing specification
- [design/pitfalls/](design/pitfalls/) — detection and schema failure mode catalog
- [.roles/](.roles/) — review roles: PIXEL, SIGNAL, SCHEMA, PARSE, BENCH, SOURCE, COMPOSE, CACHE

---

## License

MIT — see [LICENSE](LICENSE).
