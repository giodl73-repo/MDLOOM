# glint

A fast, schema-driven markdown and ASCII art linter with an AI-assisted fix pipeline.

```
+--------+--------+        glint check --format rich
| cell   | cell   |   →    detects misalignments with precise file:line:col
|  oops  | cell   |        
+--------+--------+        AI reviews context → generates fix plan → glint fix applies
```

---

## What it does

Markdown documents containing ASCII art diagrams — boxes, flowcharts, tables, connector lines — have no automated validator. `glint` catches:

- **Box alignment errors** — a `|` at column 8 when column 9 is expected; bottom border a character wider than the top
- **Cell padding violations** — content flush against a `|` delimiter with no whitespace buffer
- **Arrow gaps** — a space inside a horizontal arrow body (`── ─▶`)
- **Vertical connector drift** — a `│` connector that shifts columns between lines
- **Structural requirements** — required headings, required content patterns, H1 count limits

Rules come from a `glint.toml` schema file you control. Configs cascade through the directory tree — a root schema sets defaults, per-directory schemas add section-specific rules.

---

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/giodl73-repo/glint
cd glint
cargo build --release
```

---

## Quick start

```bash
# Lint the current directory
glint check .

# Lint a specific file
glint check guide.md

# Create a starter schema
glint init
```

---

## The fix pipeline

For bulk repair of a large library (hundreds or thousands of files):

```bash
# Stage 1: detect errors, emit rich context for AI
glint check --format rich --config glint.toml . -o rich.json

# Stage 2: AI (Claude / fix-guide skill) reads rich.json → writes plan.json
# Each diagnostic gets a context block with the box structure, expected columns,
# actual columns, and surrounding lines — everything the AI needs to decide direction.

# Stage 3: preview
glint fix --plan plan.json --dry-run

# Stage 4: apply high-confidence fixes
glint fix --plan plan.json --min-confidence high

# Stage 5: verify
glint check .
```

---

## Config cascade

Place `glint.toml` at your project root. Add per-directory `glint.toml` files to layer
section-specific rules:

```toml
# root glint.toml
[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]

[[section_schemas]]
paths = ["languages/**"]
required_h2_all = ["Type System Snapshot", "Syntax Reference Card"]

[[section_schemas]]
paths = ["computing/**"]
required_h2_all = ["The Big Picture", "Common Confusion Points"]
```

A file at `languages/08-TYPESCRIPT.md` gets all three required sections. Lists are additive — child schemas extend parent schemas, never replace them.

---

## CLI reference

```
glint check [PATHS]   Lint and report  (-f text | json | rich | github)
glint fix --plan F    Apply a fix plan (--dry-run, --min-confidence high|medium|low)
glint stats [PATHS]   Summary counts   (--by-code, --by-directory)
glint init            Write glint.toml to current directory
glint config          Show effective config for current directory
```

---

## Output formats

| Format | Use for |
|--------|---------|
| `text` | Terminal — colored, human-readable |
| `json` | CI / editor integration |
| `rich` | AI-assisted fix planning — includes context blocks |
| `github` | GitHub Actions annotations |

---

## Check codes

| Code | Sev | Description |
|------|-----|-------------|
| `ascii_box_width` | error | Row or border width mismatch |
| `ascii_box_col` | error | Column separator misaligned |
| `ascii_cell_padding` | warning | Missing whitespace inside cell |
| `ascii_arrow_gap` | warning | Gap inside horizontal arrow body |
| `ascii_connector_drift` | warning | Vertical connector shifts column |
| `md_h1_count` | warning | Too many H1 headings |
| `md_missing_section` | warning | Required `## Heading` absent |
| `md_missing_pattern` | error/warn | Required content pattern missing |
| `md_file_length` | warning | File exceeds `max_lines` |

---

## Fix plan format

AI-generated `plan.json` — reviewed then applied by `glint fix`:

```json
{
  "schema_version": "1",
  "fixes": [
    {
      "id": "fix-001",
      "file": "computing/01-PACKAGE.md",
      "confidence": "high",
      "description": "Remove extra trailing + from bottom border",
      "reasoning": "Top border width: 63. Bottom width: 64. Extra char is trailing +.",
      "edit": {
        "line": 18,
        "old_string": "+------+------++",
        "new_string": "+------+------+"
      }
    }
  ]
}
```

Fixes are applied in reverse line order (bottom of file first) so earlier line numbers stay valid after later edits. If `old_string` doesn't match the current file content, the fix is skipped and logged.

---

## Design

- [SPEC.md](design/SPEC.md) — full specification
- [INVARIANTS.md](design/INVARIANTS.md) — 13 invariants with test traceability
- [pitfalls/](design/pitfalls/) — detection and schema failure mode catalog
- [.roles/](\.roles/) — review roles: PIXEL, SIGNAL, SCHEMA, PARSE, BENCH

---

## License

MIT — see [LICENSE](LICENSE).
