# proof.toml — Schema Reference

Complete reference for every field in `proof.toml`. Grouped by section in the order they appear in `src/config.rs`. Every field lists: type, default, what it does, what it catches, and a real example.

---

## File discovery

`proof` looks for a config file in the current directory using these names, in order:

```
proof.toml
.proof.toml
.proof/config.toml
```

The first match wins. `--config <path>` overrides discovery.

When checking a file at `path/to/file.md`, configs **cascade up** the directory tree from the file's location toward the project root. Each `proof.toml` found contributes rules; a config marked `files.root = true` stops the cascade.

---

## Cascade & merge semantics

Knowing how merge works is the difference between a clean schema and a confusing one.

| Field type | Merge rule |
|-----------|-----------|
| `[meta]` name/description | Child wins if set, else parent |
| `[files] include` | Child wins if non-empty, else parent (a directory knows its own subtree best) |
| `[files] exclude` | **Additive** — children cannot un-exclude what a parent excluded |
| `[files] root` | Logical OR — either side can mark the stop point |
| `[ascii_box]`, `[ascii_flow]`, `[ascii_barchart]`, `[ascii_char]` | Whole struct: **child wins** (scalars don't merge field-by-field) |
| `[markdown_table]` | Whole struct: **child wins** (table schemas are per-directory, not additive) |
| `[markdown] required_h2_all`, `required_h2`, `required_patterns` | **Additive** — both parent and child requirements must hold |
| `[markdown] max_h1`, `max_lines` | Child's value if set, else parent (`Option::or`) |
| `[markdown]` style scalars (`check_*`, `thematic_break_style`) | Child wins |
| `[[section_schemas]]` | **Additive** — both parent's and child's schemas apply |
| `[[custom_rules]]` | **Additive** — both parent's and child's rules apply |

**Path prefixing:** `paths` and `paths_exclude` in a directory-level `proof.toml` are automatically prefixed with that directory's path relative to the project root. So `languages/proof.toml` can write `paths = ["02-*.md"]` instead of `paths = ["languages/02-*.md"]`.

**`extends`** (top-level): explicit parent reference, overrides auto-cascade. Path is relative to this config file's directory.

```toml
extends = "../shared-rules.toml"
```

---

## `[meta]`

Project metadata — informational only, no validation effect.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string (optional) | none | Human-readable project name |
| `description` | string (optional) | none | One-line project description |

```toml
[meta]
name = "MAXIM Reference Library"
description = "Universal rules for all 2,170 reference guides"
```

---

## `[files]`

Controls which files `proof check .` will discover and check.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `include` | `Vec<string>` (globs) | `["**/*.md"]` | Files to check, by glob |
| `exclude` | `Vec<string>` (globs) | `[]` | Files to skip even if included |
| `root` | bool | `false` | Stop cascade — like `tsconfig`'s `root = true` |

**Catches:** none directly. This section just defines the file set; per-file checks come from the other sections.

**Glob semantics:** standard `**` (any depth), `*` (one path segment), `?` (one character). Tested against the path **relative to the config file's directory**.

```toml
[files]
include = ["**/*.md"]
exclude = [
    "TRACKER.md",          # library management — not a content guide
    "_archive/**",         # archived material
    "atlas/**",            # companion project, different rules
    "*/00-OVERVIEW.md",    # directory landing pages
]
root = true                # this is the project root; do not cascade higher
```

**Common pitfall:** `include` is **child wins**, but `exclude` is **additive**. A child config cannot remove an exclusion set by the root.

---

## `[ascii_box]`

Validates ASCII-art boxes — `+---+ | text | +---+` and Unicode equivalents (`┌─┐ │ │ └─┘`).

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `tolerance` | usize | `0` | Columns of slop allowed in border alignment (`0` = exact) |
| `code_blocks_only` | bool | `true` | Only check inside fenced code blocks (recommended; avoids prose false positives) |
| `check_unicode` | bool | `true` | Also validate `┌─┐ │ └─┘` style boxes |
| `tab_width` | usize | `4` | Tab stop width for visual column calculation (CommonMark default: 4) |

**Catches:**

| Check | Trigger |
|-------|---------|
| `ascii_box_width` | Top border width ≠ bottom border width (e.g. `+---+` vs `+----+`) |
| `ascii_box_col` | Internal `\|` column separators don't line up across rows |
| `ascii_cell_padding` | Content butts against `\|` with no breathing space (`\|text\|` vs `\| text \|`) |

```toml
[ascii_box]
enabled = true
tolerance = 0
code_blocks_only = true
check_unicode = true
tab_width = 4
```

**Common pitfall:** Setting `code_blocks_only = false` triggers false positives on prose paragraphs that contain `+` or `|` (e.g. mathematical expressions, regex examples). Keep it `true` unless you have a specific reason.

---

## `[ascii_flow]`

Validates ASCII flowcharts — boxes connected by arrows (`-->`, `──▶`, `→`) and vertical pipes.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `check_arrow_alignment` | bool | `true` | Detect arrows and verify they form straight horizontal/vertical lines without skips |
| `check_cell_padding` | bool | `true` | Verify text inside box cells has consistent padding both sides |
| `min_cell_padding` | usize | `1` | Minimum spaces of padding inside a cell (`\| text \|` = 1) |

**Catches:**

| Check | Trigger |
|-------|---------|
| `ascii_arrow_gap` | Horizontal arrow has a gap or break in the body |
| `ascii_connector_drift` | Vertical connector line shifts column between rows |

```toml
[ascii_flow]
enabled = true
check_arrow_alignment = true
check_cell_padding = true
min_cell_padding = 1
```

---

## `[ascii_barchart]`

Validates inline ASCII bar charts — labeled rows where bar length encodes a numeric value.

```
TypeScript  ████████████████████████  78%
Python      ███████████████████████   76%
Go          ██████████████            48%
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `min_bar_width` | usize | `3` | Minimum consecutive bar characters to count as a bar |
| `min_chart_rows` | usize | `2` | Minimum consecutive bar rows to count as a chart |
| `bar_chars` | `Vec<string>` | `[]` (uses `█▓▒░#=`) | Characters that form the bar body. Empty = use defaults |
| `min_label_padding` | usize | `1` | Minimum spaces between label text and bar start |
| `min_value_padding` | usize | `1` | Minimum spaces between bar end and value |
| `check_value_format` | bool | `true` | Warn when value formats differ across rows (`%` vs integer vs float) |
| `require_value_alignment` | bool | `true` | Warn when value column is not aligned across rows |
| `alignment_tolerance` | usize | `1` | Columns of slop allowed in value alignment |
| `check_proportionality` | bool | `true` | Warn when bar widths don't match the numeric values they encode |
| `proportionality_tolerance` | usize | `2` | Bar-character slop for proportionality (rounding errors) |

**Catches:**

| Check | Trigger |
|-------|---------|
| `barchart_value_misaligned` | Values don't form a clean column |
| `barchart_value_format` | One row uses `%`, another uses raw integer |
| `barchart_disproportionate` | Bar at "78%" fills 100% of the max bar width |
| `barchart_label_padding` | Label butts against bar with no spaces |

```toml
[ascii_barchart]
enabled = true
min_bar_width = 3
min_chart_rows = 2
check_proportionality = true
proportionality_tolerance = 2     # tolerate ±2 char rounding error
```

**Common pitfall:** If your charts use `*` or `:` as bar characters, set `bar_chars = ["*"]` — the empty default only matches `█▓▒░#=`.

---

## `[ascii_char]`

Character-range safety check (Style Guide rule **S-01**). Wide and fullwidth Unicode characters (CJK, em-dashes in the wrong encoding, presentation forms) silently break ASCII-art column alignment because they consume two visual columns but one source character. This check flags them.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `error_on_wide` | bool | `true` | Error on wide/fullwidth chars (2-col) — almost always recommended |
| `warn_unusual` | bool | `false` | Also warn on narrow chars outside the safe Unicode ranges |

**Catches:**

| Check | Trigger |
|-------|---------|
| `char_wide` | A 2-column character (CJK, em-dash variants, fullwidth ASCII) appears in a position where it breaks alignment |
| `char_fullwidth` | A fullwidth ASCII variant (`Ａ` instead of `A`) — usually a paste artifact |
| `char_unusual` | Narrow char outside the safe set (only fired if `warn_unusual = true`) |

```toml
[ascii_char]
enabled = true
error_on_wide = true
warn_unusual = false              # opt-in; high false-positive rate on intentional symbols
```

**Reference:** see `specs/unicode-east-asian-width.md` for the exact width table proof uses.

---

## `[markdown]`

Document structure: headings, required content patterns, line limits.

### Heading rules

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Master switch — **must be set `true`** to activate any markdown checks |
| `max_h1` | usize (optional) | none | Maximum H1 headings per file (typically `1`) |
| `required_h2` | `Vec<string>` | `[]` | At least **one** of these H2s must appear |
| `required_h2_all` | `Vec<string>` | `[]` | **All** of these H2s must appear (in any order) |
| `required_patterns` | `Vec<RequiredPattern>` | `[]` | Substring/regex patterns that must appear |
| `max_lines` | usize (optional) | none | File length cap |

### Heading quality

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `check_heading_format` | bool | `true` | Warn on `##heading` (missing space after `#`) |
| `check_empty_headings` | bool | `true` | Warn on `## ` (no content) |
| `check_heading_hierarchy` | bool | `true` | Warn when levels skip (H1 → H3 with no H2) |
| `check_duplicate_headings` | bool | `false` | Warn on the same heading text appearing twice at the same level |

### Document style

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `thematic_break_style` | string (optional) | none | Enforce a single style: `"---"`, `"***"`, `"___"`, or `""` (any) |
| `check_blockquote_spacing` | bool | `false` | Warn on `>text` (missing space after `>`) |

**Catches:**

| Check | Trigger |
|-------|---------|
| `md_h1_count` | More than `max_h1` H1 headings |
| `md_missing_section` | Required H2 absent |
| `md_duplicate_heading` | Same heading repeated at the same level |
| `md_heading_order` | Heading level skips (H1 → H3) |
| `md_heading_format` | Missing space after `#` |
| `md_empty_heading` | Heading with no text |
| `md_missing_pattern` | Required content pattern absent |
| `md_file_length` | File exceeds `max_lines` |
| `md_thematic_break_style` | `***` used where `---` is required |
| `md_blockquote_spacing` | `>text` instead of `> text` |

```toml
[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]
max_lines = 800
check_heading_hierarchy = true
check_duplicate_headings = true

[[markdown.required_patterns]]
pattern = "```"
description = "must contain at least one code block (landscape diagram)"
severity = "warning"
```

### `RequiredPattern`

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `pattern` | string | required | Substring or regex to find in the file |
| `description` | string | required | What this pattern represents (shown in diagnostic) |
| `severity` | enum | `"error"` | `"error"` or `"warning"` |

---

## `[markdown_table]`

GFM pipe-table validation: cell padding, separator format, named schemas.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `min_separator_dashes` | usize | `3` | Minimum `-` count in each separator cell (GFM requires ≥ 3) |
| `check_cell_padding` | bool | `true` | Verify cells have padding both sides |
| `min_cell_padding` | usize | `1` | Minimum spaces inside cell delimiters |
| `required_tables` | usize (optional) | none | Minimum number of pipe tables per file |
| `table_schemas` | `Vec<TableSchema>` | `[]` | Named schemas — see below |
| `check_empty_headers` | bool | `true` | Warn when a column header cell is empty |
| `max_columns` | usize | `0` | Warn when a table has more than this many columns (`0` = no limit) |

**Catches:**

| Check | Trigger |
|-------|---------|
| `table_separator_short` | Fewer than `min_separator_dashes` in a separator cell |
| `table_cell_padding` | Cell content has insufficient padding |
| `table_required_count` | File has fewer tables than `required_tables` |
| `table_empty_header` | Header cell is empty |
| `table_too_many_cols` | Column count exceeds `max_columns` |

### `[[markdown_table.table_schemas]]` — `TableSchema`

A schema applied to a specific named table (matched by H2 heading) or to all tables in a file.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `heading` | string (optional) | none | H2 text the table must follow (without `##`). If absent, applies to all tables |
| `required_columns` | `Vec<string>` | `[]` | All these column headers must be present (exact match) |
| `required_columns_any` | `Vec<string>` | `[]` | At least one of these column headers must be present |
| `min_body_rows` | usize (optional) | none | Minimum body rows (excluding header + separator) |
| `required_row_keys` | `Vec<string>` | `[]` | Values that must appear in the first (key) column |
| `column_allowed_values` | `HashMap<string, Vec<string>>` | `{}` | Per-column whitelist of allowed cell values |
| `link_columns` | `Vec<string>` | `[]` | Columns where every body cell must contain a markdown link |
| `link_auto_fix` | string | `""` | `"directory"`, `"file"`, or `""` — auto-fix strategy for bare-text cells |
| `verify_link_targets` | bool | `false` | Resolve link paths and verify they exist on disk |

**Catches (per-schema):**

| Check | Trigger |
|-------|---------|
| `table_missing_column` | Required column absent from this table |
| `table_missing_column_any` | None of the `required_columns_any` columns appear |
| `table_missing_row` | A `required_row_keys` value doesn't appear in the key column |
| `table_min_rows` | Table has fewer body rows than `min_body_rows` |
| `table_bad_value` | Cell value is not in the column's `column_allowed_values` list |
| `md_table_missing_link` | Cell in a `link_columns` column contains bare text, not `[text](url)` |
| `md_broken_link` | Link target does not exist on disk (only with `verify_link_targets = true`) |

### `link_auto_fix` strategies

| Strategy | Behavior | Example |
|----------|----------|---------|
| `"directory"` | Bare directory name → link to its `00-OVERVIEW.md` | `computing/` → `[computing/](../computing/00-OVERVIEW.md)` |
| `"file"` | Bare filename → link to file under sibling directory | `01-PKG.md` → `[01-PKG.md](../dirname/01-PKG.md)` |
| `""` | No auto-fix; report only | — |

```toml
[markdown_table]
enabled = true
required_tables = 1                    # every guide must have at least one table

# Six-row Type System Snapshot for every language guide
[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = [
    "Binding", "Typing", "Strength",
    "Type system", "Type inference", "Memory model",
]
min_body_rows = 4

# Decision Cheat Sheet — must exist and have content (column names vary)
[[markdown_table.table_schemas]]
heading = "Decision Cheat Sheet"
min_body_rows = 2

# Section landing pages: navigation table with verified links
[[markdown_table.table_schemas]]
heading = "Directories"
required_columns = ["Directory", "Entry Point"]
min_body_rows = 3
link_columns = ["Directory", "Entry Point"]
link_auto_fix = "directory"
verify_link_targets = true

# Memory model column has a fixed vocabulary
[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
column_allowed_values = { "Memory model" = ["GC", "Manual", "ARC", "Region", "Borrow checker"] }
```

**Common pitfall:** `markdown_table` is **not additive across cascade**. A child's `markdown_table` block replaces the parent's entirely. To layer table schemas, define each schema under the proof.toml that owns it (typically the directory that contains the relevant files).

---

## `[[section_schemas]]`

Per-glob rule overrides, applied additively on top of the root `[markdown]` block. Each entry targets a set of files by glob and contributes additional requirements.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `paths` | `Vec<string>` | required | Glob patterns — files matching ANY count as candidates |
| `paths_exclude` | `Vec<string>` | `[]` | Globs that exclude files even if they matched `paths` |
| `required_h2_all` | `Vec<string>` | `[]` | Additional H2s that ALL must be present |
| `required_h2` | `Vec<string>` | `[]` | Additional H2s where AT LEAST ONE must be present |
| `required_patterns` | `Vec<RequiredPattern>` | `[]` | Additional content patterns |
| `max_lines` | usize (optional) | none | Override the root `max_lines` for these files |

**Path prefixing reminder:** in a directory-level `proof.toml`, both `paths` and `paths_exclude` are auto-prefixed with that directory. Write `paths = ["02-*.md"]` not `paths = ["languages/02-*.md"]`.

**Catches:** same set as `[markdown]` (`md_missing_section`, `md_missing_pattern`, `md_file_length`) — but only fires for files matched by `paths` AND not by `paths_exclude`.

```toml
# Root proof.toml — applies one rule to a section by full glob
[[section_schemas]]
paths = ["computing/**", "ai-engineering/**"]
required_h2_all = ["The Big Picture", "Common Confusion Points"]

# languages/proof.toml — directory-local: paths auto-prefix with "languages/"
[[section_schemas]]
paths = ["*.md"]                       # → effectively languages/*.md
paths_exclude = ["00-OVERVIEW.md", "01-CHEATSHEET.md", "STATUS.md"]
required_h2_all = [
    "Type System Snapshot",
    "Syntax Reference Card",
    "What Makes It Distinct",
]

# Carve-out: 00-OVERVIEW.md gets a different schema
[[section_schemas]]
paths = ["00-OVERVIEW.md"]
required_h2_all = ["Language Genealogy"]
```

**Common pitfall:** if a file matches **multiple** `[[section_schemas]]`, ALL their requirements apply additively. Use `paths_exclude` to carve out exceptions instead of fighting the union.

---

## `[[custom_rules]]`

Free-form regex rules. Each rule is applied to every file (or a glob subset) and reports on match or non-match.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Identifier shown in diagnostics |
| `description` | string | required | Human-readable purpose |
| `pattern` | string (regex) | required | Rust `regex` syntax |
| `negate` | bool | `false` | If `true`, **warn when pattern IS found** (inverse match) |
| `severity` | string | `"warning"` | `"error"` or `"warning"` |
| `only_in` | `Vec<string>` (globs) | `[]` | Restrict to files matching these globs (empty = all files) |

**Catches:** custom check code = the rule's `name` field, used as the diagnostic code in output.

```toml
# Reject leftover review tags in published guides
[[custom_rules]]
name = "no_editor_tags"
description = "@editor review tags should be resolved before a guide is considered complete"
pattern = "@editor\\["
negate = true                         # report when found
severity = "warning"

# Forbid TODO markers
[[custom_rules]]
name = "no_todo"
description = "TODO/FIXME markers should not remain in published guides"
pattern = "TODO|FIXME|HACK"
negate = true
severity = "warning"
only_in = ["computing/**", "languages/**"]
```

**Common pitfall:** `negate = true` is the opt-in behavior most projects want (warn when a forbidden pattern appears). With `negate = false` (default), the rule warns when the pattern is **absent** — useful for required boilerplate but counter-intuitive.

---

## `[[davinci]]` — figure invariant pinning

Pin a specific figure to an `md://` URI and attach invariants that must hold across edits. Protects canonical diagrams from silent drift.

**Register via CLI** (recommended):

```bash
proof pin "md://computing/01-PACKAGE.md#the-big-picture:0" --id package-layer-stack --protection error
```

**Or write directly in `proof.toml`:**

```toml
[[davinci]]
id = "package-layer-stack"
uri = "md://computing/01-PACKAGE.md#the-big-picture:0"
description = "Canonical package manager hierarchy — 5-level stack diagram"
protection = "error"   # "warn" | "error" | "lock"

  [[davinci.invariant]]
  rule = "box-width"
  min = 68
  max = 72

  [[davinci.invariant]]
  rule = "contains-text"
  value = "SYSTEM / OS LAYER"

  [[davinci.invariant]]
  rule = "box-count"
  min = 5
```

**Verify pins:**

```bash
proof check --daVinci .
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Stable handle for the figure (used in `proof pin-list`) |
| `uri` | string | `md://path#heading:selector` — resolved via mdpath |
| `description` | string | Human-readable purpose (defaults to figure label if omitted) |
| `protection` | string | `"warn"`, `"error"`, or `"lock"` |
| `[[davinci.invariant]]` | array of tables | One invariant per entry; see rule list below |

**Protection tiers:**

| Tier | Behavior |
|------|---------|
| `warn` | `proof check --daVinci` emits a warning; `proof compile` continues |
| `error` | `proof check --daVinci` emits an error; `proof compile` aborts |
| `lock` | Same as `error`; reserved for future `proof fix` hard-block |

**Built-in invariant rules:**

| Rule | Parameters | Description |
|------|-----------|-------------|
| `box-width` | `min`, `max` | Visual width of the box border must be in range |
| `box-count` | `value`, `min`, `max` | Number of detected boxes in the figure |
| `column-count` | `value` | Number of column separators per content row |
| `contains-text` | `value` | Figure must contain this string (case-insensitive) |
| `not-contains-text` | `value` | Figure must NOT contain this string |
| `line-count` | `min`, `max`, `value` | Number of lines in the code block |
| `starts-with` | `value` | First non-empty line must start with this string |
| `ends-with` | `value` | Last non-empty line must end with this string |
| `pattern` | `value` | Figure must contain this substring |
| `required-row-keys` | `values` | Table figure must contain all of these row key strings |
| `equals` | `value` | Figure content must exactly equal this string (trimmed) |
| `heading-exists` | `value` | Content must contain a heading with this text |

---

## What this reference does NOT cover

A few labels appear in informal docs but have no dedicated config section in code:

- **`[markdown_links]`** — link validation lives on `TableSchema` (`link_columns`, `verify_link_targets`, `link_auto_fix`). There is no separate top-level `[markdown_links]` section.
- **`[[consistency-group]]`** — cross-file figure consistency groups are specified in `design/SCENARIOS.md` but not yet implemented.
- **CLI flags** — see `proof --help`. Config governs *what is checked*; CLI flags govern *how output is formatted* (`--format rich|json|text`, `--output`, `--min-confidence`, etc.).

---

## Effective config inspection

To see the resolved config for any file (after cascade and merge):

```bash
proof config languages/08-TYPESCRIPT.md
```

This is the single source of truth for "what rules apply to this file" — preferred over reading individual `proof.toml` files when debugging.

---

## See also

- `schemas/default.toml` — minimal starter config (run `proof init` to copy)
- `schemas/reference.toml` — full real-world example (the MAXIM library's root schema)
- `design/COMPILE-SPEC.md` — `proof compile` pipeline and `proof:include`/`proof:layout` directives
- `design/LAYOUT-SPEC.md` — `proof layout` algorithm and invariants
- `design/STYLE-GUIDE.md` — style rules (S-01 wide chars, etc.)
