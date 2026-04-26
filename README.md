# proof

Document quality assurance for markdown corpora — ASCII art, table schemas, link integrity, and heading structure, with an AI-assisted fix pipeline.

---

## What it does

Markdown documentation at scale accumulates drift: boxes whose borders don't add up, tables missing required rows, internal links that rotted, headings in the wrong order. `proof check` catches all of it with file:line:col precision. `proof draft` sends failures to an AI that writes a structured fix plan. `proof fix` applies the plan.

```
$ proof check --config proof.toml languages/08-TYPESCRIPT.md

languages/08-TYPESCRIPT.md:34:1  error    ascii_box_width      bottom border 64 chars, top border 63
languages/08-TYPESCRIPT.md:34:1  warning  ascii_cell_padding   col 2 row 3: content flush against delimiter
languages/08-TYPESCRIPT.md:112:1 warning  md_missing_section   required ## "Type System Snapshot" absent
languages/08-TYPESCRIPT.md:198:1 error    table_missing_row    "Memory model" row required in Type System Snapshot

4 diagnostics (2 errors, 2 warnings)
```

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

Checks: `ascii_box_width`, `ascii_box_col`, `ascii_cell_padding`, `ascii_arrow_gap`, `ascii_connector_drift`

### GFM table schemas

Tables carry structure that matters. Declare what you require in `proof.toml` and every file is held to it:

```toml
[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = ["Binding", "Typing", "Strength", "Type system", "Type inference", "Memory model"]
min_body_rows = 4
```

Checks: `table_missing_column`, `table_missing_row`, `table_min_rows`, `table_bad_value`, `table_link_required`

### Link integrity

```toml
link_columns = ["Directory", "Entry Point"]
verify_link_targets = true
link_auto_fix = "directory"
```

Every cell in a link column must contain a markdown link. `verify_link_targets = true` resolves each path on disk. `proof fix` can auto-fill bare directory names with the correct link syntax.

Checks: `link_bare_text`, `link_broken_target`, `link_missing`

### Heading structure

```toml
[markdown]
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet", "Type System Snapshot"]
```

Checks: `md_h1_count`, `md_missing_section`, `md_duplicate_heading`, `md_heading_order`

### Character safety

Wide and fullwidth Unicode characters (CJK, em-dashes in the wrong encoding, presentation forms) break ASCII art column alignment silently. `proof` flags them before they corrupt diagrams.

Check: `char_wide`, `char_fullwidth`

---

## The fix pipeline

```
proof check --format rich --config proof.toml . -o rich.json
     │
     │  rich format: each diagnostic includes the surrounding code block,
     │  expected vs actual column widths, and adjacent lines — everything
     │  an AI needs to reason about the fix without reading the whole file
     ▼
proof draft --input rich.json -o plan.json
     │
     │  calls the configured AI (Claude by default) with the rich context
     │  AI writes a structured plan: one fix per diagnostic, with reasoning
     │  and confidence level (high / medium / low)
     ▼
proof fix --plan plan.json --dry-run        # review what will change
proof fix --plan plan.json --min-confidence high   # apply high-confidence fixes
     │
     ▼
proof check .                               # verify: should be clean
```

The plan format is JSON and human-readable — review it before applying:

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

## proof.toml

Schema-driven, cascading. A root `proof.toml` sets library-wide defaults. Per-directory `proof.toml` files inherit and extend — they never replace the root.

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

# languages/proof.toml (child — inherits root, adds more)
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

A file at `languages/08-TYPESCRIPT.md` is checked against both the root rules and the `languages/` section schema. Required section lists are additive — child schemas extend, never override.

Effective config for any file: `proof config path/to/file.md`

---

## The fig:// URI scheme (coming)

`proof` is building `fig://` — a URI scheme for addressing specific diagrams within markdown files by name rather than by line number.

```
fig://computing/01-PACKAGE.md#the-big-picture:0
      └─ file ─────────────┘  └─ section ────┘ └ index
```

This is the address of the first code block inside `## The Big Picture` in that file. Line numbers change as content evolves; section-qualified figure addresses are stable.

`fig://` enables the DaVinci protection tier: diagrams pinned with invariants that `proof` checks on every run. A pinned figure carries a contract — if the box structure changes in a way that violates the invariant, `proof` reports it as an error regardless of other rules.

---

## Check codes

| Code | Sev | Description |
|------|-----|-------------|
| `ascii_box_width` | error | Row or border width mismatch |
| `ascii_box_col` | error | Column separator misaligned |
| `ascii_cell_padding` | warning | Missing whitespace inside cell delimiter |
| `ascii_arrow_gap` | warning | Gap inside horizontal arrow body |
| `ascii_connector_drift` | warning | Vertical connector shifts column between lines |
| `char_wide` | error | Wide/fullwidth character breaks column alignment |
| `md_h1_count` | warning | Too many H1 headings |
| `md_missing_section` | warning | Required `## Heading` absent |
| `md_duplicate_heading` | warning | Same heading appears more than once |
| `md_heading_order` | warning | Heading level skips (H1 → H3 with no H2) |
| `md_missing_pattern` | warning | Required content pattern absent |
| `md_file_length` | warning | File exceeds `max_lines` |
| `table_missing_column` | error | Required column absent from table |
| `table_missing_row` | error | Required row key absent from table |
| `table_min_rows` | warning | Table has fewer body rows than `min_body_rows` |
| `table_bad_value` | error | Cell value not in `allowed_values` list |
| `link_bare_text` | error | Link column cell contains plain text, not a markdown link |
| `link_broken_target` | error | Link target does not exist on disk |
| `link_missing` | warning | Expected link is absent from table cell |

---

## Install

Build from source:

```bash
git clone https://github.com/your-org/proof
cd proof
cargo build --release
./target/release/proof --version
```

`cargo install proof` — coming once the crate is published.

---

## Design

- [SPEC.md](design/SPEC.md) — full specification
- [INVARIANTS.md](design/INVARIANTS.md) — invariants with test traceability
- [design/pitfalls/](design/pitfalls/) — detection and schema failure mode catalog
- [.roles/](.roles/) — review roles: PIXEL, SIGNAL, SCHEMA, PARSE, BENCH

---

## License

MIT — see [LICENSE](LICENSE).
