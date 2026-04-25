# glint — Specification v0.2

A fast, schema-driven markdown and ASCII art linter with an AI-assisted fix pipeline.

---

## Purpose

Markdown documents that contain ASCII art diagrams — boxes, flowcharts, tables,
connector lines — have no automated validator. Authors introduce subtle alignment
errors that are invisible at writing time but render incorrectly or look sloppy in
final output. `glint` fills this gap.

Secondary purpose: enforce structural conventions on large guide libraries where
every file must follow a style contract (required sections, required content patterns,
heading limits).

At scale — a 2,000-file library like MAXIM — manual repair is impractical. glint
provides a three-stage pipeline: **detect**, **plan**, **fix** — where detection is
mechanical (Rust), planning is AI-assisted (Claude), and fixing is deterministic (Rust).

---

## Design Principles

1. **Schema-driven** — no hard-coded opinions about structure. All rules come from a
   `glint.toml` schema file the author controls, cascading through the directory tree.

2. **Cascading config** — `glint.toml` files nest through directories. Lists are additive
   (parent + child both enforced); scalars use the nearest config's value.

3. **Precise error location** — every diagnostic reports `file:line:col` with enough
   context that both humans and AI can resolve it without reading the whole file.

4. **Three output modes for three audiences**:
   - `text` — human-readable, colored terminal output
   - `json` — compact machine-readable, for CI and editor integration
   - `rich` — structured with context blocks, for AI-assisted fix planning

5. **Separation of detection from judgment** — glint detects *where* errors are and
   *what* is wrong. Deciding *how* to fix an alignment error requires spatial judgment
   that belongs to AI or the author. glint never guesses the fix direction.

6. **Fix pipeline** — `glint check` → `glint plan` (AI) → `glint fix` — enables bulk
   repair of an entire library in one supervised pass.

7. **Fast** — parallel file processing via rayon. A 2,000-file library completes in
   under 5 seconds on a modern machine. Config resolution is cached per directory.

---

## The Fix Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│  STAGE 1: glint check --format rich                         │
│                                                             │
│  Rust: fast, mechanical, parallel                           │
│  Output: rich.json — every error with surrounding context,  │
│  expected vs. actual column positions, box structure        │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  STAGE 2: AI review (fix-guide skill)                       │
│                                                             │
│  Claude reads rich.json + file content                      │
│  For each diagnostic, decides:                              │
│    - Direction of fix (add/remove char, which side)         │
│    - Confidence (high / medium / low)                       │
│    - The exact edit (old_string → new_string)               │
│  Output: plan.json — a fix plan file                        │
└─────────────────────────────┬───────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  STAGE 3: glint fix --plan plan.json                        │
│                                                             │
│  Rust: applies edits from plan.json to files                │
│  --dry-run: shows diff without writing                      │
│  --min-confidence high: skip medium/low confidence fixes    │
│  Re-runs check after applying to confirm zero errors        │
└─────────────────────────────────────────────────────────────┘
```

### Bulk workflow for the entire MAXIM library

```bash
# Stage 1: detect all errors, emit rich context
glint check --format rich --config glint.toml . > rich.json

# Stage 2: AI generates fix plan (Claude Code / fix-guide skill)
# Input: rich.json  Output: plan.json

# Stage 3: dry run first — review what will change
glint fix --plan plan.json --dry-run

# Stage 4: apply high-confidence fixes automatically
glint fix --plan plan.json --min-confidence high

# Stage 5: verify
glint check --config glint.toml .
```

---

## `--format rich` Output

The `rich` format extends the `json` format by adding a `context` block to each
diagnostic. This is the format intended for AI consumption.

```json
[
  {
    "file": "languages/08-TYPESCRIPT.md",
    "line": 42,
    "col": 7,
    "severity": "error",
    "code": "ascii_box_col",
    "message": "column separator at col 7, expected col 8 — off by 1 (box opened at line 38)",
    "context": {
      "box_opens_at": 38,
      "border_line": "+-------+-------+",
      "expected_cols": [1, 9, 17],
      "actual_cols_this_line": [1, 7, 17],
      "lines": {
        "37": "```",
        "38": "+-------+-------+",
        "39": "| good  | good  |",
        "40": "| good  | good  |",
        "41": "| good  | good  |",
        "42": "| bad |  bad   |",
        "43": "+-------+-------+",
        "44": "```"
      }
    }
  }
]
```

**What the context block gives the AI:**
- The border that defined the box — AI knows what the box is supposed to look like
- `expected_cols` vs. `actual_cols_this_line` — exact column positions, no arithmetic needed
- Surrounding lines including code fence — full structure visible in one block
- The AI can immediately see: *"first cell has 4 chars, needs 6 — add two spaces"*

---

## Fix Plan Format

A fix plan is a JSON file generated by AI (via the `fix-guide` skill) and consumed
by `glint fix`. It is a machine-readable, reviewable audit trail of every intended edit.

```json
{
  "schema_version": "1",
  "generated_at": "2026-04-25T14:30:00Z",
  "generated_by": "fix-guide",
  "source_report": "rich.json",
  "summary": {
    "total_fixes": 47,
    "high_confidence": 41,
    "medium_confidence": 5,
    "low_confidence": 1,
    "files_affected": 12
  },
  "fixes": [
    {
      "id": "fix-001",
      "file": "languages/08-TYPESCRIPT.md",
      "diagnostic": {
        "code": "ascii_box_col",
        "line": 42,
        "col": 7
      },
      "description": "Add one space before 'bad' in first cell — | at col 7 needs to be at col 9",
      "confidence": "high",
      "reasoning": "Border expects | at col 9 (7 dash cells). Content row has 4 chars before |. Needs 6. Adding ' b' → '  b' shifts | right by 2.",
      "edit": {
        "line": 42,
        "old_string": "| bad |  bad   |",
        "new_string": "|  bad |  bad  |"
      }
    },
    {
      "id": "fix-002",
      "file": "computing/01-PACKAGE.md",
      "diagnostic": {
        "code": "ascii_box_width",
        "line": 18,
        "col": 1
      },
      "description": "Bottom border is 1 char wider than top — remove trailing +",
      "confidence": "high",
      "reasoning": "Top border width: 63. Bottom border width: 64. The extra char is a trailing + that shouldn't be there.",
      "edit": {
        "line": 18,
        "old_string": "+------+------++",
        "new_string": "+------+------+"
      }
    }
  ]
}
```

### Confidence levels

| Level | Meaning | Default action |
|-------|---------|---------------|
| `high` | One unambiguous fix — extra char, clear direction | Auto-apply |
| `medium` | Fix direction clear but edit touches multiple lines | Apply with review |
| `low` | Ambiguous — box may need structural redesign | Skip, flag for human |

### Fix application rules

- Each fix is applied in **reverse line order** (bottom of file first) so earlier line
  numbers stay valid after edits to later lines.
- `old_string` must match exactly in the file at the specified line — if it doesn't
  match (file changed since plan was generated), the fix is skipped and logged.
- After all fixes are applied, `glint check` is re-run automatically. Any remaining
  errors are reported — the plan did not fully resolve them.

---

## Config Cascade

### File Discovery

glint looks for config starting from the file's directory and walking up:

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
extends = "../../schemas/shared.toml"
```

### Stop Cascade

```toml
[files]
root = true   # do not cascade above this directory
```

---

## Section Schemas

Per-directory schemas apply additional rules to files matching path globs, additive
on top of the base `[markdown]` config:

```toml
[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]

[[section_schemas]]
paths = ["languages/**"]
required_h2_all = ["Type System Snapshot", "Syntax Reference Card"]

[[section_schemas]]
paths = ["computing/**", "os/**"]
required_h2_all = ["The Big Picture", "Common Confusion Points"]
```

---

## Check Reference

### `ascii_box` — Box Alignment

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_box_width` | error | A content row or bottom border has different visual width than the top border |
| `ascii_box_col` | error | A column separator (`\|` or `│`) is not aligned with the border junction above it |

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
| `ascii_cell_padding` | warning | Cell content flush against `\|` delimiter — no whitespace buffer |
| `ascii_arrow_gap` | warning | Gap (space) inside a horizontal arrow body (`── ─▶`) |
| `ascii_connector_drift` | warning | Vertical connector `│` drifts column between consecutive lines |

**Config:**
```toml
[ascii_flow]
enabled = true
check_arrow_alignment = true
check_cell_padding = true
min_cell_padding = 1
```

### `markdown` — Structure Validation

| Code | Severity | Description |
|------|----------|-------------|
| `md_h1_count` | warning | File has more H1 headings than `max_h1` allows |
| `md_missing_section` | warning | A required `## Heading` is absent |
| `md_missing_pattern` | error/warning | A required content pattern is not found |
| `md_file_length` | warning | File exceeds `max_lines` |

### Custom Rules

```toml
[[custom_rules]]
name = "no_todo"
pattern = "TODO|FIXME"
negate = true
severity = "warning"
```

---

## CLI Reference

```
COMMANDS
  check   Lint files and report diagnostics (default)
  fix     Apply a fix plan generated by AI
  config  Print the effective config for a path
  init    Write a glint.toml to the current directory
  stats   Summary statistics only (no per-file output)

CHECK OPTIONS
  glint check [PATHS]...
    -c, --config <FILE>           Use this config file (skips auto-cascade)
    -f, --format <FMT>            text (default) | json | rich | github
    -e, --errors-only             Suppress warnings
        --no-fail                 Exit 0 even when errors found
    -o, --output <FILE>           Write output to file instead of stdout
        --progress                Show progress bar (auto-enabled for >100 files)

FIX OPTIONS
  glint fix --plan <FILE>
        --plan <FILE>             Fix plan JSON file (required)
        --dry-run                 Show diff without writing any files
        --min-confidence <LVL>    Skip fixes below this level: high | medium | low
        --no-verify               Skip re-running check after applying fixes
    -o, --output <FILE>           Write application log to file

STATS OPTIONS
  glint stats [PATHS]...
        --by-directory            Break down counts by directory
        --by-code                 Break down counts by error code
```

---

## Output Formats

### `text` (default)
```
languages/08-TYPESCRIPT.md:42:7: error [ascii_box_col]: column separator at col 7, expected col 9
  note: box opened at line 38
```

### `json`
```json
[{"file":"...","line":42,"col":7,"severity":"error","code":"ascii_box_col","message":"..."}]
```

### `rich`
Extended json with `context` block — see **`--format rich` Output** section above.

### `github`
```
::error file=languages/08-TYPESCRIPT.md,line=42,col=7::[ascii_box_col] column separator at col 7, expected col 9
```

---

## Invariants

| # | Invariant | Has Test |
|---|-----------|----------|
| I-1 | A file with no ASCII boxes produces zero `ascii_box_*` diagnostics | yes |
| I-2 | A perfectly aligned box produces zero diagnostics regardless of content | yes |
| I-3 | Every diagnostic has `span.line ≥ 1` and `span.col ≥ 1` | no — add |
| I-4 | Linting the same file twice produces identical diagnostics | no — add |
| I-5 | Child config `required_h2_all` is a superset of parent's | partial |
| I-6 | `tolerance = N` suppresses drift ≤ N; reports drift > N | no — add |
| I-7 | Parallel and sequential execution produce the same diagnostic set | no — add |
| I-8 | `--format json` and `--format rich` output are always valid JSON arrays | yes (json) |
| I-9 | Exit code 0 iff zero error-severity diagnostics (or `--no-fail`) | yes |
| I-10 | Unicode boxes treated identically to ASCII boxes | yes |
| I-11 | `glint fix` with `old_string` that doesn't match the file skips that fix and logs it | no — add |
| I-12 | `glint fix --dry-run` makes zero writes to disk | no — add |
| I-13 | Fix application in reverse line order — later line edits never invalidate earlier line numbers | no — add |

---

## What Remains to Build

### Rust (glint itself)

| Item | Priority | Description |
|------|----------|-------------|
| `--format rich` | P0 | Add `context` block to each diagnostic in JSON output |
| `glint fix --plan` | P0 | Apply fix plan: parse JSON, apply edits in reverse line order, verify |
| `--dry-run` for fix | P0 | Show unified diff without writing |
| `--min-confidence` for fix | P1 | Filter fixes below threshold |
| Re-verify after fix | P1 | Auto re-run `check` after `glint fix`, report residual errors |
| `glint stats` | P2 | Summary by directory and error code |
| Progress bar | P2 | `--progress` flag for large runs |
| Fix application log | P2 | Structured log of what was applied, skipped, failed |

### AI Skills (`.claude/skills/`)

| Skill | Priority | Description |
|-------|----------|-------------|
| `fix-guide` | P0 | Read rich.json + files → generate plan.json |
| `fix-review` | P1 | Review a plan.json before applying — flag low-confidence fixes |

### Documentation

| Item | Priority | Description |
|------|----------|-------------|
| `README.md` | P1 | Public-facing project README |
| `.github/workflows/ci.yml` | P1 | CI: cargo test + cargo build |

### Tests

| Item | Priority | Description |
|------|----------|-------------|
| Invariant I-3 (valid spans) | P1 | Assert all diagnostics have line ≥ 1, col ≥ 1 |
| Invariant I-6 (tolerance bounds) | P1 | Verify tolerance=N suppresses ≤N, reports >N |
| Invariant I-7 (parallel = sequential) | P1 | Run both, diff the diagnostic sets |
| Fix plan application | P0 | Integration test: write plan.json → glint fix → verify file contents |
| Fix --dry-run | P0 | Assert no disk writes after dry-run |
| Fix with stale old_string | P1 | Assert skipped, not panicked |
| CRLF line endings | P1 | Fixture with \\r\\n — should not cause false width mismatches |

---

## Non-Goals

- **Custom check plugins** — use `custom_rules` for simple patterns; native plugins are future work.
- **Non-markdown files** — glint only reads `.md` files.
- **HTML rendering correctness** — glint validates source structure, not rendered output.
- **Fully automatic fix without review** — `--dry-run` exists for a reason. Bulk fixes
  across 2,000 files should be reviewed before applying.
