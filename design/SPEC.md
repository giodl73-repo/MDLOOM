# proof — Specification v0.3

A schema-driven document compiler, quality checker, and markdown-native
typesetting toolchain with an AI-assisted fix pipeline.

---

## Purpose

`proof` turns structured markdown source trees into validated, rendered,
reference-aware documentation systems. It is designed to feel like a modern
compiler and a LaTeX-style typesetter for technical corpora:

- authors write `.source.md` and `.md` sources with lightweight directives,
  `md://` references, ASCII figures, math, slides, dashboards, and metadata;
- proof resolves the corpus graph, compiles source documents, renders derived
  artifacts, and validates the result against project contracts;
- diagnostics are stable compiler messages that can drive CI, editors, AI
  review, and deterministic repair.

The first shipped surface is a fast markdown and ASCII-art checker because
markdown documents that contain ASCII diagrams — boxes, flowcharts, tables,
connector lines — have no automated validator. Authors introduce subtle
alignment errors that are invisible at writing time but render incorrectly or
look sloppy in final output. `proof` fills this gap as one phase of the larger
compiler.

Secondary purpose: enforce structural conventions on large guide libraries where
every file must follow a style contract (required sections, required content patterns,
heading limits).

At scale — a 2,000-file library like MAXIM — manual repair is impractical. proof
provides staged pipelines for both publishing and repair:

- **compile** — source directives and references become rendered markdown,
  HTML, figures, slides, dashboards, and other target artifacts;
- **check** — source and output are validated against schema, layout, reference,
  and corpus policy;
- **plan/fix** — rich diagnostics become reviewable fix plans and deterministic
  edits.

---

## Design Principles

1. **Compiler pipeline** — proof has explicit phases: parse source, resolve
   references, compile/render artifacts, check invariants, report diagnostics,
   and apply reviewed fixes. Commands are user-facing entrypoints into those
   phases, not unrelated scripts.

2. **Schema-driven** — no hard-coded opinions about structure. All rules come from a
   `proof.toml` schema file the author controls, cascading through the directory tree.

3. **Cascading config** — `proof.toml` files nest through directories. Lists are additive
   (parent + child both enforced); scalars use the nearest config's value.

4. **Source metadata is first-class** — source frontmatter tags, ops, and content
   tags are passive metadata today, but they are part of the source model and
   may drive future compile targets, policies, slices, and reports.

5. **Artifact-oriented typesetting** — `.source.md` is authoring input; compiled
   markdown, HTML, figures, slides, dashboards, and generated layouts are
   artifacts. Compile must be deterministic, explicit about stale or failed
   outputs, and safe for CI. PPTX is a future publish backend, not a separate
   source format.

6. **Registered diagnostics** — every emitted diagnostic code has one registry entry
   with owner family, default severity, and a human description. New source literals
   that look like diagnostic codes must be registered before tests pass.

7. **Precise error location** — every diagnostic reports `file:line:col` with enough
   context that both humans and AI can resolve it without reading the whole file.

8. **Three output modes for three audiences**:
   - `text` — human-readable, colored terminal output
   - `json` — compact machine-readable, for CI and editor integration
   - `rich` — structured with context blocks, for AI-assisted fix planning

9. **Separation of detection from judgment** — proof detects *where* errors are and
   *what* is wrong. Deciding *how* to fix an alignment error requires spatial judgment
   that belongs to AI or the author. proof never guesses the fix direction.

10. **Fix pipeline** — `proof check` → `proof draft` (AI) → `proof fix` — enables bulk
    repair of an entire library in one supervised pass.

11. **Wave/pulse execution record** — architecture and quality work is tracked
    through waves and pulses so changes have mission, gates, validation, and
    closeout history.

12. **Fast** — parallel file processing via rayon. A 2,000-file library completes in
    under 5 seconds on a modern machine. Config resolution is cached per directory.

---

## Compiler and Typesetter Model

proof treats a repository as a corpus, not a bag of independent markdown files.
The corpus contains source documents, figure files, configs, generated artifacts,
and historical quality records. A command may operate on one file, but the model
is graph-shaped.

### Source Layer

Authoring inputs are markdown-family files:

- `.source.md` — source documents that may contain proof directives and source
  frontmatter;
- `.md` — ordinary markdown documents and figure fragments;
- `proof.toml` — schema, compile, DaVinci, AI, and corpus policy;
- wave/pulse records — architecture history and quality execution notes.

`.source.md` frontmatter is a source metadata block. It may contain:

```toml
tags: [compiler, typesetter]
ops: [review, publish]
content_tags: [architecture, guide]
```

Supported forms are scalar values, inline lists, and block lists. Compile strips
generic source frontmatter from ordinary `.source.md` outputs. Specialized slide
and dashboard frontmatter remains owned by those renderers.

### Resolve Layer

The resolver turns local document structure into stable addresses:

- `md://path.md#heading` points at files, headings, and resolved ranges;
- `proof:include`, `proof:layout`, figures, slides, dashboards, symbols, and
  charts declare dependencies;
- `proof depends` exposes reverse dependencies so authors can understand what a
  source object feeds.

The long-term contract is that every generated artifact can explain the source
objects and config that produced it.

### Compile and Typeset Layer

`proof compile` is the publishing phase. It resolves directives, renders
supported objects, and writes artifacts. The compiler must:

- be deterministic for the same source/config inputs;
- fail loudly on invalid explicit config or unresolved required dependencies;
- preserve stale outputs only when explicitly configured to do so;
- support validation-only runs via `--check`;
- support multi-target corpus compiles through config-defined compile targets.

Supported artifact families include ordinary markdown output, ASCII figures,
charts, math blocks, symbols, slides, dashboards, and layouts. New artifact
families should enter through this staged model rather than ad hoc command code.

### Check Layer

`proof check` is the compiler diagnostic phase. It validates both source and
artifact contracts:

- markdown structure and section schema;
- ASCII box/flow/tree/chart/table correctness;
- source links and `md://` addressability;
- compile-time embedded diagnostics;
- DaVinci pinned figure invariants;
- corpus hygiene such as unused figures.

Diagnostics are the durable interface between compiler phases, CI, editors, AI
review, and authors.

### Plan and Fix Layer

`proof draft` and `proof fix` are the repair stages. They do not replace the
compiler; they consume compiler diagnostics and produce audited edits:

- rich diagnostics provide enough context for AI and humans;
- draft plans group related diagnostics and precompute deterministic edits;
- fix plans are applied exactly, with `old_string` guards and dry-run support;
- verification re-runs the relevant check phase after writes unless explicitly
  disabled.

### Reverse and Backfill Layer

Existing corpora often start as hand-written `.md` files with no `.source.md`
history. MAXIM-style libraries need a reversible migration path: generated
source files should reproduce the current markdown first, then progressively
extract structure into better source directives.

`proof backfill` is the adoption bridge and reverse compiler phase. Its job is
to let an existing documentation system get value quickly: generate source
candidates, run checks, draft fixes, and improve quality without requiring a
manual rewrite into proof-native sources first.

```text
existing .md corpus
  -> inventory/classify blocks
  -> extract figures, tables, charts, and data candidates
  -> generate .source.md and sidecar data files
  -> compile generated source
  -> compare compiled output to original .md
  -> emit a backfill report and review plan
```

Backfill must be conservative. The first invariant is round-trip fidelity: a
generated `.source.md` should compile back to the original `.md` modulo explicitly
declared normalizations. Semantic extraction is layered on top of that invariant.

The product promise is **quickie adoption with a safe upgrade path**:

1. **Mirror** — create `.source.md` files that mostly preserve current markdown
   literally, add provenance frontmatter, and compile back to the same output.
2. **Inspect** — classify ASCII art, tables, charts, repeated structures, and
   ambiguous blocks with confidence scores.
3. **Improve** — extract high-confidence structures into proof directives or
   sidecar data so future edits become easier and fixes become automatable.
4. **Automate** — run `proof check`, `proof draft`, and `proof fix` on the
   generated source corpus to make quality repairs repeatable.
5. **Adopt** — once round-trip and review gates pass, treat `.source.md` as the
   owned source of truth and compiled `.md` as generated output.

Backfill classes:

| Class | Input pattern | Generated source direction |
|-------|---------------|----------------------------|
| Literal markdown | prose, headings, lists, callouts | preserve as source text |
| ASCII figure | fenced boxes, flowcharts, trees, diagrams | extract to figure block or referenced figure file, preserve original as fallback |
| ASCII table | grid tables or aligned tables | extract rows/columns to structured data plus a table/render directive |
| Markdown table | pipe table | extract to structured rows while preserving rendered table shape |
| Chart-like block | bars, sparklines, scales | extract data series and chart directive when confidence is high |
| Repeated section pattern | recurring headings, cheatsheets, cards | infer source template candidates and tags |
| Ambiguous block | mixed prose/art, low-confidence diagram | keep literal block and mark for review |

Backfill outputs:

- `.source.md` files next to or under a configured source root;
- optional sidecar data files for extracted tables/charts;
- source frontmatter with generated `tags`, `ops = ["backfill"]`, and
  `content_tags`;
- comments or metadata identifying low-confidence extracted blocks;
- a `backfill-report.json` with source path, generated path, confidence,
  block classification counts/evidence, extraction decisions, round-trip diff
  summary, and review items.

Backfill gates:

1. **No silent loss** — if a block cannot be confidently extracted, preserve it
   literally and flag it.
2. **Round-trip before abstraction** — generated source must compile to the
   current artifact before reviewers accept higher-level table/chart extraction.
3. **Reviewable diffs** — all differences between original markdown and compiled
   backfill output are reported with file/line context.
4. **Stable provenance** — generated source records the original artifact path
   so future compiles can explain where the source came from.
5. **Idempotence** — running backfill again should not churn accepted source
   unless the original artifact or extraction policy changed.

For MAXIM, the intended migration is:

```bash
# Day 1: safe mirror with reports, no semantic risk
proof backfill maxim/ --output-source maxim-source/ --literal-first --report backfill-report.json
proof compile maxim-source/ --output-dir maxim-roundtrip/
proof backfill --check-roundtrip maxim/ --output-source maxim-source/

# Day 2+: promote confident structures, then use normal proof automation
proof backfill maxim/ --output-source maxim-source/ --report backfill-report.json
proof compile maxim-source/ --output-dir maxim-roundtrip/
proof check maxim-source/ maxim-roundtrip/ --format rich
proof draft maxim-source/ -o draft-plan.json
proof fix --plan draft-plan.json --dry-run
```

The first pass creates source candidates that reproduce existing guides. Later
passes can promote extracted ASCII tables into data-backed table directives,
chart-looking blocks into chart directives, and recurring guide structure into
templates or section schemas.

Backfill should support an intentionally low-commitment mode for teams that only
want automation on top of their current system:

- leave original `.md` files in place;
- generate source files in a separate directory;
- write reports and proposed source ownership metadata;
- run checks/fixes against generated source or original artifacts depending on
  confidence;
- require an explicit cutover before generated `.md` replaces existing files.

### Architecture Contract

The CLI shell follows the compiler boundary:

```text
main.rs -> cli parser -> dispatch context -> command adapters -> proof_lib phases
```

- `main.rs` stays a thin binary shell.
- `cli.rs` owns clap parsing and parser-only dispatch input.
- `dispatch.rs` owns routing context and command selection.
- `cmd_*` modules own command arguments and command-specific adapters.
- `proof_lib` owns reusable compiler/check/fix behavior.

This boundary is part of the spec. New features should prefer reusable library
phases and small command adapters over growing the CLI shell.

---

## The Fix Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│  STAGE 1: proof check --format rich                         │
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
│  STAGE 3: proof fix --plan plan.json                        │
│                                                             │
│  Rust: applies edits from plan.json to files                │
│  --dry-run: shows diff without writing                      │
│  --min-confidence high: skip medium/low confidence fixes    │
│  Re-runs check after applying to confirm zero errors        │
└─────────────────────────────────────────────────────────────┘
```

### Bulk workflow for the entire MAXIM library

**Option A: draft workflow (recommended)** — proof pre-populates the plan, AI annotates:

```bash
# Stage 1: generate draft plan (groups errors, pre-computes deterministic fixes)
proof draft --config proof.toml . -o draft-plan.json

# Stage 2: AI opens draft-plan.json and fills in:
#   - new_string for each non-auto fix
#   - decision (prose explanation) per group
#   - confidence for each group
# Groups with auto=true are already done — skip them.
# One decision per group (not per line).

# Stage 3: preview
proof fix --plan draft-plan.json --dry-run

# Stage 4: apply auto fixes first (confidence=high, auto=true)
proof fix --plan draft-plan.json --min-confidence high

# Stage 5: verify
proof check --config proof.toml .
```

**Option B: rich workflow** — AI reads full rich report and generates plan from scratch:

```bash
# Stage 1: detect all errors, emit rich context
proof check --format rich --config proof.toml . > rich.json

# Stage 2: AI (fix-guide skill) reads rich.json → writes plan.json
# Input: rich.json  Output: plan.json

# Stage 3: dry run first — review what will change
proof fix --plan plan.json --dry-run

# Stage 4: apply high-confidence fixes automatically
proof fix --plan plan.json --min-confidence high

# Stage 5: verify
proof check --config proof.toml .
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
      "actual_cols": [1, 7, 17],
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
- `expected_cols` vs. `actual_cols` — exact column positions, no arithmetic needed
- Surrounding lines including code fence — full structure visible in one block
- The AI can immediately see: *"first cell has 4 chars, needs 6 — add two spaces"*

Current rich context is strongest for ASCII box diagnostics. Other diagnostic
families may emit ordinary JSON without a typed rich block until their context
schema is promoted into the registry contract.

---

## Fix Plan Format

A fix plan is a JSON file generated by AI (via the `fix-guide` skill) and consumed
by `proof fix`. It is a machine-readable, reviewable audit trail of every intended edit.

```json
{
  "schema_version": "1",
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
- After all fixes are applied, `proof check` is re-run automatically. Any remaining
  errors are reported — the plan did not fully resolve them.

---

## Config Cascade

### File Discovery

proof looks for config starting from the file's directory and walking up:

```
file.md
  ↑
  dir/proof.toml        ← nearest config
  ↑
  parent/proof.toml     ← grandparent config
  ↑
  root/proof.toml       ← root config (files.root = true stops cascade here)
```

### Merge Semantics

| Field type | Merge behavior |
|-----------|---------------|
| Lists (`required_h2_all`, `required_patterns`, custom rules) | **Additive** — parent + child both applied |
| Scalars (`tolerance`, non-markdown check configs) | **Child wins** — nearest config takes precedence |
| Optional scalars (`max_lines`, `max_h1`) | **Child wins if set** — falls back to parent if `None` |
| `files.include` | **Child replaces** when set — the nearest config controls inclusion |
| `files.exclude` | **Additive** — child exclusions are added to parent exclusions |
| `markdown.enabled` | **Explicit child wins** — if a child TOML sets `enabled`, that value overrides the parent; absent child value inherits |

Implementation note: proof keeps TOML explicitness in an internal loaded-config
layer for fields where default values and absence have different cascade
meanings (`markdown.enabled`, `files.include`). The public `GlintConfig` remains
the effective runtime config and does not expose parser-only explicitness
markers.

### Explicit Parent

```toml
extends = "../../schemas/shared.toml"
```

When `extends` appears, proof loads the explicit parent relative to the current
config file and stops automatic ancestor discovery for that branch. The effective
config is `extends` parent first, then the extending child.

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
| `ascii_box_col` | error/warning | Content columns miss expected border junctions; bottom-border warnings only compare comparable column structures, not row separators, spanning rows, connector ports, or top-border connector anchors |
| `ascii_unclosed_fence` | warning | A fenced code block was opened but not closed |

**Config:**
```toml
[ascii_box]
enabled = true
tolerance = 0          # columns of allowed drift (0 = exact)
code_blocks_only = true
check_unicode = true
tab_width = 4
check_col_separators = true
```

### `ascii_flow` — Flowchart and Cell Padding

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_cell_padding` | warning | Cell content flush against a real box delimiter when the declared cell width has room for padding |
| `ascii_arrow_gap` | warning | Gap (space) inside a horizontal arrow body (`── ─▶`); multi-space layout gaps and bidirectional scale rulers are ignored |
| `ascii_connector_drift` | warning | Vertical connector `│` drifts column between consecutive connector-only lines |

**Config:**
```toml
[ascii_flow]
enabled = true
check_arrow_alignment = true
check_cell_padding = true
min_cell_padding = 1
```

### `ascii_barchart` — Bar Chart Validation

Applies to unlabeled/plain-text diagram fences only (empty info string or
`text`/`txt`/`ascii`/`diagram`/`chart`/`barchart`), not typed programming
fences. Boxed multi-panel diagrams, adjacent pattern fills, equation operators,
axis-attached bars, and multi-run texture rows are not treated as charts.
Stacked bars may mix default fill characters (`█`/`░`) without char-consistency
warnings. Duration values are recognized only when the duration suffix follows a
numeric value (`250ms`, `1.5s`, `3m`).

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_barchart_char` | warning | A row uses a different bar character from the first row |
| `ascii_barchart_pad` | warning | Missing minimum padding between label/bar or bar/value |
| `ascii_barchart_value` | warning | Value formats differ across rows |
| `ascii_barchart_align` | warning | Value column starts at inconsistent visual columns |
| `ascii_barchart_scale` | warning | Bar width is disproportionate to its numeric value |

### `ascii_char` — Alignment-Safe Character Ranges

| Code | Severity | Description |
|------|----------|-------------|
| `ascii_char_range` | error/warning | A code-block character is outside safe alignment ranges; wide characters are errors by default and suppressed when `ascii_char.error_on_wide = false` |

### `ascii_tree` — Tree Structure

Tree diagnostics use `TREE-001` through `TREE-008` for connector grammar,
indentation, duplicate entries, slash policy, and path verification.

### `markdown` — Structure Validation

| Code | Severity | Description |
|------|----------|-------------|
| `md_h1_count` | warning | File has more H1 headings than `max_h1` allows |
| `md_missing_section` | warning | A required `## Heading` is absent |
| `md_missing_pattern` | error/warning | A required content pattern is not found |
| `md_file_length` | warning | File exceeds `max_lines` |
| `md_forbidden_section` | warning | A configured forbidden H2 appears |
| `md_unexpected_section` | warning | An H2 is outside the active allowlist; `optional_h2` activates closed-world H2 checking |
| `md_heading_format` | warning | Heading spacing or trailing hash style is invalid |
| `md_empty_heading` | warning | Heading marker has no content |
| `md_heading_hierarchy` | warning | Heading levels skip |
| `md_duplicate_heading` | warning | Duplicate heading text appears at the same level |
| `md_break_style` | warning | Thematic break style differs from `thematic_break_style` |
| `md_blockquote_spacing` | warning | Block quote marker is missing the configured following space |
| `link_broken_target` | warning | A relative Markdown link target does not exist; inline math/function notation such as `[X](t)` is ignored |

### `markdown_table` — GFM Pipe Tables

| Code | Severity | Description |
|------|----------|-------------|
| `md_table_separator_invalid` | warning | Separator cells do not meet dash/alignment syntax; a blank row-label corner may use a compact two-dash separator |
| `md_table_col_mismatch` | error | Header, separator, or body row column counts differ |
| `md_table_cell_padding` | warning | Cell padding is below the configured minimum when the row has the expected columns and the cell has room |
| `md_table_schema` | warning | A configured table schema requirement is unmet |
| `md_table_missing_link` | warning | A configured link column contains bare text |
| `md_broken_link` | warning | A configured table link target does not exist |
| `md_table_empty_header` | warning | A table header cell is empty, except an intentional blank top-left row-label corner in comparison matrices |
| `md_table_too_wide` | warning | A table exceeds configured maximum columns |
| `source_inline_table` | warning | A `.source.md` file contains an inline pipe table; durable row data should live in sidecar JSON/CSV or generated PROOF tables |
| `md_missing_table` | warning | A required table is absent |

### `source_links` — Source Document Links

| Code | Severity | Description |
|------|----------|-------------|
| `md_missing_source` | warning | A source document is missing required source-link declarations |
| `md_broken_uri` | warning | A source-link `md://` target cannot be resolved |
| `md_broken_heading` | warning | A source-link heading target does not exist |

### Compile, Figure, Slide, Dashboard, Math, and Symbol Diagnostics

The compile/render toolchain emits registered prefixed codes rather than
Markdown-style names:

| Family | Codes | Purpose |
|--------|-------|---------|
| Compile | `COMPILE-001`, `COMPILE-002`, `COMPILE-003`, `COMPILE-004`, `COMPILE-007` | Directive resolution/rendering, compile-time DaVinci checks, embedded lint failures |
| Figure | `FIGURE-001`, `FIGURE-003`, `FIGURE-006` | Figure rendering, bounds, clipping/lossy placement |
| Chart | `CHART-001`, `CHART-002` | Chart rendering and invalid chart source/data |
| Dashboard | `DASHBOARD-001` through `DASHBOARD-006` | Dashboard geometry, missing regions, overflow, width policy |
| Element | `ELEMENT-001` through `ELEMENT-005` | Element directive data, source, and row-width errors |
| Math | `MATH-001`, `MATH-003`, `MATH-004`, `MATH-006` | Math rendering, clipping, tokenization/syntax |
| Slide | `SLIDE-001`, `SLIDE-002`, `SLIDE-006`, `SLIDE-007` | Slide bullet limits, layout ratio, range, depth |
| PPTX | `PPTX-001`, `PPTX-002` | PPTX source boundary and native deck parsing |
| Symbol | `SYMBOL-001`, `SYMBOL-003` | Symbol and shape lookup failures |
| DaVinci | `fig_invariant_violated` | Pinned figure invariant failure |
| Corpus ops | `unused_figure` | Markdown figure file has no source-document reference |
| Runner | `io_error` | Input file could not be read |

`src/diagnostic_registry.rs` is the implementation source of truth for code
owner family, default severity, and descriptions. `SPEC.md` intentionally
documents family-level purpose for the larger render toolchain rather than every
message variant.

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
  backfill
          Generate source candidates from existing markdown artifacts
  draft   Generate a pre-populated fix plan — AI annotates inline
  fix     Apply a fix plan (from proof draft or AI-generated)
  compile Compile .source.md documents by resolving proof directives
  resolve Resolve an md:// URI and print its target content/range
  depends List source documents that reference an md:// URI
  layout  Compose figures side-by-side or vertically
  pin     Register a DaVinci pinned figure in proof.toml
  pin-list
          List pinned DaVinci figures
  tree    Generate or validate ASCII tree diagrams
  status  Show cached corpus health/status
  spec-generate
          Suggest DaVinci invariants for a figure
  config  Print the effective config for a path
  init    Write a proof.toml to the current directory
  stats   Summary statistics only (no per-file output)

CHECK OPTIONS
  proof check [PATHS]...
    -c, --config <FILE>           Use this config file (skips auto-cascade)
    -f, --format <FMT>            text (default) | json | rich | github
    -e, --errors-only             Suppress warnings
        --no-fail                 Exit 0 even when errors found
    -o, --output <FILE>           Write output to file instead of stdout
        --deduplicate             Collapse repeated diagnostics by code/directory
        --unused                  Report unreferenced markdown figures
        --daVinci                 Validate pinned DaVinci figure invariants
        --tag <TAG>               Only check source files with this tag
        --op <OP>                 Only check source files with this operation tag
        --content-tag <TAG>       Only check source files with this content tag

Explicit `--config` is authoritative: if the file is missing or invalid, proof
exits with an error instead of falling back to auto-discovered config.

BACKFILL OPTIONS
  proof backfill [PATHS]...
        --output-source <DIR>       Write generated .source.md files under DIR
        --report <FILE>             Write extraction and round-trip report
        --literal-first             Prefer exact source mirroring over semantic extraction
        --extract-tables            Extract markdown tables into sidecar JSON
        --check-roundtrip           Compile generated sources and compare to originals

  Backfill is a reverse compiler. It starts from existing `.md` artifacts and
  generates `.source.md` candidates plus optional sidecar data, preserving
  ambiguous blocks literally and reporting every non-identical round-trip diff.
  It is also the quick adoption path: teams can keep their current `.md` files,
  generate sources elsewhere, and use proof automation before deciding to cut
  over to generated artifacts.

  The MVP report includes per-file block counts for prose, fenced blocks,
  markdown tables, ASCII table candidates, chart-like blocks, diagram-like
  blocks, and ambiguous blocks. These classifications are advisory only until
  extraction flags are enabled.

  With `--extract-tables`, high-confidence markdown pipe tables outside fenced
  code blocks are written to sibling sidecar files named
  `<stem>.tables.json`. The generated `.source.md` body remains literal-first;
  the report records extraction kind, line, row/column counts, confidence, and
  sidecar path so teams can review before replacing markdown tables with
  structured directives.

  Planned follow-up flags include `--extract-charts`, `--literal-only`,
  `--min-confidence`, and `--cutover-plan`.

DRAFT OPTIONS
  proof draft [PATHS]...
    -o, --output <FILE>           Output file (default: draft-plan.json)

  Generates a pre-populated plan file:
    - Errors grouped by source object (same box, table, chart → one group)
    - Deterministic fixes pre-computed (barchart scale, separator dashes)
      with auto=true and confidence=high — no AI review needed
    - Judgment calls pre-templated with old_string filled in;
      AI writes new_string + decision + confidence for each group
    - One decision per group (not per line)

FIX OPTIONS
  proof fix --plan <FILE>
        --plan <FILE>             Fix plan JSON file (required)
        --dry-run                 Show diff without writing any files
        --min-confidence <LVL>    Skip fixes below this level: high | medium | low
        --no-verify               Skip re-running check after applying fixes
        --no-signal-check         Allow non-whitespace removals after explicit review

  `proof fix` accepts root global options such as `--config`. Verification uses
  the same explicit config override and checks the files modified by the applied
  plan. Every run writes `.proof/last-fix.json` with schema version, plan path,
  dry-run flag, confidence threshold, applied/skipped counts, modified files,
  and verification status (`passed`, `failed`, or `skipped`).

COMPILE OPTIONS
  proof compile [PATHS]...
        --check                   Validate without writing output files
        --watch                   Recompile on source/dependency changes
        --delete-on-error         Remove stale output when compile fails
        --progress                Show a running compiled/total count
        --output-dir <DIR>        Write compiled files under DIR
        --root <DIR>              Root directory for md:// URI resolution
        --target <TARGET>         md (default) | html | pebble | json-report | site | pdf | docx | pptx
        --tag <TAG>               Only compile source files with this tag
        --op <OP>                 Only compile source files with this operation tag
        --content-tag <TAG>       Only compile source files with this content tag
    -o, --output <FILE>           Explicit output path for one source file

  `md` is the canonical compile target and preserves proof's terminal-first
  renderer contract. `html` is the first human publish target: proof resolves
  source directives to markdown, strips source frontmatter, then emits a
  standalone HTML document. The HTML backend supports common Markdown blocks
  including headings, lists, tables, links, task lists, strikethrough, and fenced
  code; raw HTML is escaped rather than passed through.

  `pebble` writes Pebbles (`pebble.v1`): compact JSON context
  transfer artifacts optimized for agents rather than human presentation. A
  pebble records source path, title, format, resolved dependency refs, and
  section chunks with stable IDs, heading paths, source line numbers, and resolved
  Markdown text. CROP may emit the same schema for view/corpus slices so PROOF
  and CROP can share provenance-bearing context packs. `--watch` currently
  supports only `--target md`.

  `json-report` writes `proof.publish.json_report.v1`: a stable machine-readable
  compile/report bundle for CI, agents, and integrations. It records source path,
  title, artifact summary, source metadata, resolved Markdown, section summaries,
  dependency refs, diagnostics, and compile counts. It is intentionally more
  verbose than Pebble and does not replace Pebble's compact retrieval schema.

  `site` compiles source trees to static HTML pages, a navigation `index.html`,
  and a `proof-site.json` manifest with page/source/output/diagnostic metadata.
  It is a local static site artifact, not deployment, hosting, search ranking, or
  target-aware watch mode.

  `pdf` renders the same resolved HTML publish output into a portable PDF
  artifact. The first backend is deterministic and dependency-free for CI. It
  does not claim exact browser or print-engine layout equivalence.

  `docx` writes a native editable Office Open XML Word-processing package from
  resolved Markdown. It supports headings, paragraphs, native bullet/numbered
  lists, tables, fenced code text, links, and basic metadata; tracked changes,
  comments, corporate templates, and full Word style customization are out of
  scope for the first backend.

  `pptx` writes a native editable Office Open XML PowerPoint deck from explicit
  `.slides.source.md` inputs. It preserves the slide-source boundary, emits real
  text boxes with native bullets/numbering, monospace code text, notes slide
  parts for `proof:notes`, and package relationships/content types inspectable
  in CI. It does not infer decks from arbitrary prose, rasterize slides, embed
  HTML, or implement animations, rich themes, charts, media, or brand templates.

  LaTeX is deferred until after these publish backends.

  Non-watch compile runs write `.proof/artifacts.json` with schema version,
  config root, generation timestamp, and one artifact entry per source. Each
  entry records source path, output path, target (`md`, `html`, `pebble`,
  `json-report`, `site`, `pdf`, future publish backends), status (`written`,
  `cached`, `up_to_date`, `error`), resolved
  directive count, cache usage, and diagnostics. The manifest is provenance for
  status, stale-output checks, cutover planning, and future PPTX/site/PDF
  backends; it is not a replacement for the content cache.

SOURCE FRONTMATTER
  `.source.md` files may begin with a YAML-style `---` block containing:
    tags = general corpus tags
    ops = workflow/operation tags
    content_tags/content = content classification tags

  Compile strips source frontmatter from ordinary `.source.md` output. Slides and
  dashboards keep their specialized frontmatter parsers.

  `proof check`, `proof compile`, and `proof stats` accept opt-in exact-match
  filters: `--tag`, `--op`, and `--content-tag`. Filters are additive: if more
  than one filter is supplied, a source must match all requested fields. Without
  filters, tags never exclude content.

STATS OPTIONS
  proof stats [PATHS]...
        --by-directory            Break down counts by directory
        --by-code                 Break down counts by error code
        --by-tag                  Break down source frontmatter tags, ops, and content tags
        --tag <TAG>               Only include source files with this tag
        --op <OP>                 Only include source files with this operation tag
        --content-tag <TAG>       Only include source files with this content tag

CONFIG OPTIONS
  proof config [PATH]
        Prints the resolved effective config as TOML. Without `--config`, PATH
        is resolved through normal cascade. With explicit `--config`, the
        supplied config is printed with defaults and auto-cascade is skipped.
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
Extended json with a `rich` block — see **`--format rich` Output** section above.

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
| I-3 | Every diagnostic has `span.line ≥ 1` and `span.col ≥ 1` | yes |
| I-4 | Linting the same file twice produces identical diagnostics | yes |
| I-5 | Child config `required_h2_all` is a superset of parent's | yes |
| I-6 | `tolerance = N` suppresses drift ≤ N; reports drift > N | yes |
| I-7 | Parallel and sequential execution produce the same diagnostic set | yes |
| I-8 | `--format json` and `--format rich` output are always valid JSON arrays | yes |
| I-9 | Exit code 0 iff zero error-severity diagnostics (or `--no-fail`) | yes |
| I-10 | Unicode boxes treated identically to ASCII boxes | yes |
| I-11 | `proof fix` with `old_string` that doesn't match the file skips that fix and logs it | yes |
| I-12 | `proof fix --dry-run` makes zero writes to disk | yes |
| I-13 | Fix application in reverse line order — later line edits never invalidate earlier line numbers | yes |
| I-14 | Every diagnostic-like source literal is present in the diagnostic registry | yes |

---

## Current Backlog

### Rust (proof itself)

| Item | Priority | Description |
|------|----------|-------------|
| Full raw/effective config model | P2 | Promote all TOML sections to optional raw structs before resolving into complete effective config |
| Fix application log | P1 | Structured log of what was applied, skipped, failed |
| Corpus compile graph | P1 | Materialize source dependencies and compile targets as a graph so stale checks, watch mode, and artifact provenance share one model |
| Reverse/backfill command | P1 | Generate `.source.md` candidates and extraction reports from existing `.md` corpora with round-trip gates |
| Backfill extraction classifiers | P1 | Classify literal markdown, ASCII figures, ASCII tables, markdown tables, chart-like blocks, templates, and ambiguous blocks |
| Backfill cutover plans | P1 | Produce reviewable source-ownership plans so teams can adopt proof without immediately replacing existing `.md` files |
| Tag-driven operations | P1 | Let source frontmatter tags/ops/content tags select compile/check/report slices without hard-coding directory layouts |
| Artifact manifest | P1 | Record generated outputs, source inputs, config, and diagnostics for reproducible typesetting runs |
| Warning cleanup | P2 | Keep the proof workspace warning-clean; sibling `mdpath` warnings are tracked separately |
| Command module split | done | Extract command handlers from `main.rs` into focused modules (`cmd_check`, `cmd_compile`, `cmd_pin`, etc.) |
| CLI compiler boundary | done | Keep parser, dispatch context, command adapters, and reusable library phases separated |
| Source frontmatter tags | done | Parse source tags, ops, and content tags; strip generic frontmatter from compiled ordinary source output |
| Compile directive module split | P2 | Split `compile.rs` directive parsing/rendering by family while keeping one stable compile facade |

### AI Skills (`.claude/skills/`)

| Skill | Priority | Description |
|-------|----------|-------------|
| `fix-guide` | P0 | Read rich.json + files → generate plan.json |
| `proof-wave` / `proof-pulse` / `proof-plan` | P1 | Keep architecture and quality work on explicit waves and pulses |
| `fix-review` | P2 | Review a plan.json before applying — flag low-confidence fixes |
| `publish-review` | P2 | Review compile/type-set artifacts for stale outputs, missing source metadata, and graph inconsistencies |
| `backfill-review` | P1 | Review backfill reports, extraction confidence, and round-trip diffs before accepting generated source |

### Documentation

| Item | Priority | Description |
|------|----------|-------------|
| `.github/workflows/ci.yml` | P1 | CI: cargo test + cargo build |
| Wave closeouts | P2 | Bridge significant architecture waves into `CHANGELOG.md` without rewriting history |
| Compiler bible examples | P1 | Add source-to-artifact examples that demonstrate source frontmatter, md:// references, compile targets, and quality gates together |
| Backfill migration guide | P1 | Document the MAXIM-style path from existing `.md` to generated `.source.md`, extracted data, and accepted source ownership |

### Tests

| Item | Priority | Description |
|------|----------|-------------|
| CLI config failure path | done | `--config missing.toml` and invalid explicit config files fail loudly |
| Loaded config explicitness | done | Parser-only explicitness for `markdown.enabled` and `files.include` is kept out of effective runtime config |
| CRLF preservation in `proof fix` | done | Applying fixes preserves CRLF line endings where present |
| Typed rich contexts | P1 | Add registry-backed context contracts for table, link, chart, compile, and markdown diagnostics |
| Compiler/type-setter golden tests | P1 | Lock source-to-artifact output for representative figures, slides, dashboards, math, and frontmatter-tagged sources |
| Backfill round-trip golden tests | P1 | Ensure generated source compiles back to original markdown for literal blocks, ASCII figures, tables, charts, and ambiguous blocks |

---

## Non-Goals

- **Custom check plugins** — use `custom_rules` for simple patterns; native plugins are future work.
- **General-purpose build system** — proof may manage document artifact graphs, but it is not a replacement for `make`, Cargo, npm, or CI orchestrators.
- **Arbitrary binary assets** — proof may reference external assets, but the core compiler operates on markdown-family source and text-based artifacts.
- **Browser layout equivalence** — proof validates source structure and proof-owned renderers; it does not guarantee pixel-identical HTML/PDF output across browsers.
- **Perfect semantic reverse engineering** — backfill must preserve and report
  uncertainty; it should not pretend every ASCII block or table can be converted
  into high-level data without review.
- **Fully automatic fix without review** — `--dry-run` exists for a reason. Bulk fixes
  across 2,000 files should be reviewed before applying.
