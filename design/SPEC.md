# glint — Specification v0.1

A fast, schema-driven markdown and ASCII art linter written in Rust.

---

## Purpose

Markdown documents that contain ASCII art diagrams — boxes, flowcharts, tables,
connector lines — have no automated validator. Authors introduce subtle alignment
errors that are invisible at writing time but render incorrectly or look sloppy in
final output. `glint` fills this gap.

Secondary purpose: enforce structural conventions on large guide libraries where
every file must follow a style contract (required sections, required content patterns,
heading limits).

---

## Design Principles

1. **Schema-driven** — the tool has no hard-coded opinions about structure. All rules
   come from a `glint.toml` schema file that the author controls.

2. **Cascading config** — `glint.toml` files nest through the directory tree. A root-level
   config sets defaults; per-directory configs inherit and extend. Lists are additive;
   scalars use the nearest config's value.

3. **Precise error location** — every diagnostic reports `file:line:col`, making errors
   actionable in any editor or CI system. No vague "something is wrong with your ASCII art."

4. **Two output modes** — `text` (default, colored, human-readable) and `json` (machine-readable,
   for editor integration). GitHub Actions output mode emits `::error file=...::` annotations.

5. **Report, then decide** — the tool reports errors precisely. Fixing most ASCII art errors
   requires human judgment (the author decides which character moved). The tool identifies
   WHERE the problem is; the author decides the fix.

6. **Fast** — parallel file processing via rayon. A 2,000-file library should complete in
   under 2 seconds on a modern machine.

---

## Config Cascade

### File Discovery

glint looks for config in this order, starting from the file being linted and walking up:

```
file.md
  ↑
  dir/glint.toml        ← nearest config
  ↑
  parent/glint.toml     ← grandparent config
  ↑
  root/glint.toml       ← root config (files.root = true stops cascade here)
```

### Merge Semantics

| Field type | Merge behavior |
|-----------|---------------|
| Lists (`required_h2_all`, `required_patterns`, custom rules) | **Additive** — parent + child both applied |
| Scalars (`tolerance`, `max_h1`, `enabled`) | **Child wins** — nearest config takes precedence |
| Optional scalars (`max_lines`, `max_h1`) | **Child wins if set** — falls back to parent if `None` |
| `files.include` / `files.exclude` | **Child replaces** — the nearest config controls file selection |

### Explicit Parent

```toml
# Override the auto-cascade and point to an explicit parent
extends = "../../schemas/shared.toml"
```

### Stop Cascade

```toml
[files]
root = true   # do not cascade above this directory
```

---

## Section Schemas

Per-directory schemas apply additional rules to files matching path globs. They are
additive on top of the base `[markdown]` config:

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
paths = ["computing/**", "os/**", "scripting/**"]
required_h2_all = ["The Big Picture", "Common Confusion Points"]
```

A file at `languages/08-TYPESCRIPT.md` sees:
- `max_h1 = 1` (from root)
- `required_h2_all = ["Decision Cheat Sheet", "Type System Snapshot", "Syntax Reference Card"]` (root + languages schema, unioned)

---

## Check Reference

### `ascii_box` — Box Alignment

Detects ASCII art boxes and validates alignment:

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_box_width` | error | A content row or bottom border has different visual width than the top border |
| `ascii_box_col` | error | A column separator (`\|` or `│`) is not aligned with the border junction above it |

**Detection rules:**
- A border line must contain ≥ 2 junction chars (`+`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┬`, `┴`, `┼`) with fill chars (`-`, `─`) between them.
- Content rows are any line between two detected border rows.
- Visual width is measured using `unicode-width` (correct for multi-byte chars).
- Only scans inside fenced code blocks when `code_blocks_only = true` (default).

**Config:**
```toml
[ascii_box]
enabled = true
tolerance = 0          # columns of allowed drift (0 = exact)
code_blocks_only = true
check_unicode = true
```

### `ascii_flow` — Flowchart and Cell Padding

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_cell_padding` | warning | Cell content is flush against a `\|` delimiter (no whitespace buffer) |
| `ascii_arrow_gap` | warning | A gap (space) detected inside a horizontal arrow body (`── ─▶`) |
| `ascii_connector_drift` | warning | A vertical connector `│` drifts column between consecutive lines |

**Config:**
```toml
[ascii_flow]
enabled = true
check_arrow_alignment = true
check_cell_padding = true
min_cell_padding = 1   # minimum spaces on each side of cell content
```

### `markdown` — Structure Validation

| Code | Severity | Description |
|------|----------|-------------|
| `md_h1_count` | warning | File has more H1 headings than `max_h1` allows |
| `md_missing_section` | warning | A required `## Heading` is absent |
| `md_missing_pattern` | error/warning | A required content pattern is not found |
| `md_file_length` | warning | File exceeds `max_lines` |

**Config:**
```toml
[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]
required_h2 = []          # at least one of these (OR)
max_lines = 800

[[markdown.required_patterns]]
pattern = "```"
description = "must contain at least one code block"
severity = "warning"
```

### Custom Rules

Regex-based rules applied to file content:

```toml
[[custom_rules]]
name = "no_todo"
description = "TODO comments should not remain in published guides"
pattern = "TODO|FIXME"
negate = true        # warn when pattern IS found
severity = "warning"
only_in = []         # restrict to specific globs (empty = all files)
```

---

## CLI Reference

```
glint [OPTIONS] [PATHS]...
glint check [PATHS]...    # explicit subcommand (same as default)
glint config              # show effective config
glint init                # write glint.toml to current directory

Options:
  -c, --config <FILE>     Use this config file (skips auto-cascade)
  -f, --format <FMT>      Output format: text (default), json, github
  -e, --errors-only       Suppress warnings
      --no-fail           Exit 0 even when errors found
```

---

## Output Formats

### text (default)
```
languages/08-TYPESCRIPT.md:42:8: error [ascii_box_width]: row width 17 ≠ box width 16 (box opened at line 38)
languages/08-TYPESCRIPT.md:38:1: warning [md_missing_section]: missing required section: "Type System Snapshot"
```

### json
```json
[
  {"file":"languages/08-TYPESCRIPT.md","line":42,"col":8,"severity":"error","code":"ascii_box_width","message":"..."},
  ...
]
```

### github
```
::error file=languages/08-TYPESCRIPT.md,line=42,col=8::[ascii_box_width] row width 17 ≠ box width 16
```

---

## Invariants

These properties must hold at all times. Any change that breaks an invariant is a regression.

| # | Invariant |
|---|-----------|
| I-1 | A file with zero ASCII boxes produces zero `ascii_box_*` diagnostics |
| I-2 | A perfectly aligned box produces zero diagnostics regardless of content |
| I-3 | Every diagnostic contains a valid file path, 1-based line, and 1-based column |
| I-4 | The same file linted twice produces identical diagnostics (deterministic) |
| I-5 | A child config's `required_h2_all` is always a superset of the parent's (additive merge) |
| I-6 | `tolerance = 0` reports any alignment drift ≥ 1 column; `tolerance = N` suppresses drift ≤ N |
| I-7 | Parallel (rayon) execution produces the same diagnostic set as sequential execution |
| I-8 | `--format json` output is always a valid JSON array |
| I-9 | Exit code is 0 if and only if error-severity diagnostics = 0 (or `--no-fail` is set) |
| I-10 | A Unicode box using `┌─┐│└─┘` is treated identically to an ASCII box using `+-+\|+-+` |

---

## Non-Goals (v0.1)

- **Auto-fix** — ASCII art alignment requires author judgment. glint reports; humans fix.
- **Custom check plugins** — use `custom_rules` for simple patterns; full plugins are future work.
- **Non-markdown files** — glint only reads `.md` files.
- **HTML rendering correctness** — glint validates source structure, not rendered output.
