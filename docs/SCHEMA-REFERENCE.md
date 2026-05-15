# proof.toml — Schema Reference

Complete reference for every field in `proof.toml`. Grouped by config block in source order. Each section gives a one-paragraph description of what the block controls, then a table of fields with type, default, and description. A complete annotated example sits at the end.

---

## File discovery

`proof` looks for a config file using these names, in order: `proof.toml`, `.proof.toml`, `.proof/config.toml`. The first match wins; `--config <path>` overrides discovery.

When checking `path/to/file.md`, configs **cascade up** the directory tree from the file's location toward the project root. Each `proof.toml` found contributes rules; a config marked `files.root = true` stops the cascade. Use `extends = "../shared.toml"` (top-level) to pull in an explicit parent config that lives outside the cascade chain.

---

## Cascade & merge semantics

| Field type | Merge rule |
|-----------|-----------|
| `[meta]` name/description | Child wins if set, else parent |
| `[files] include` | Child wins if non-empty, else parent |
| `[files] exclude` | **Additive** — children cannot un-exclude what a parent excluded |
| `[files] root` | Logical OR — either side can mark the stop point |
| `[ascii_*]` blocks | Whole struct: **child wins** (scalars don't merge field-by-field) |
| `[markdown_table]` | Whole struct: **child wins** (table schemas are per-directory) |
| `[markdown]` lists (`required_h2*`, `required_patterns`) | **Additive** |
| `[markdown]` scalars (`max_h1`, `max_lines`, style checks) | Child's value wins |
| `[[section_schemas]]`, `[[custom_rules]]`, `[[davinci]]` | **Additive** |
| `[[compile]]` | Child wins if it declares any; else inherits parent's |

**Path prefixing:** in a directory-level `proof.toml`, `paths` and `paths_exclude` under `[[section_schemas]]` are auto-prefixed with that directory's path relative to root. Write `paths = ["02-*.md"]`, not `paths = ["languages/02-*.md"]`.

---

## `[meta]`

Project metadata — purely informational, no validation effect. Use this to name the config so `proof config` output and diagnostics carry a recognizable label.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string? | none | Human-readable project name |
| `description` | string? | none | One-line project description |

---

## `[files]`

Controls which files `proof check .` discovers. `include` is a child-wins glob list; `exclude` is additive across cascade (children cannot un-exclude). Setting `root = true` is the equivalent of `tsconfig`'s `root` — proof stops walking up looking for parent configs at this level. Use this when you want to be sure no surprise grandparent config silently changes the rules.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `include` | `Vec<string>` (globs) | `["**/*.md"]` | Files to check |
| `exclude` | `Vec<string>` (globs) | `[]` | Files to skip even if included |
| `root` | bool | `false` | Stop the cascade at this directory |

Glob semantics: `**` (any depth), `*` (one segment), `?` (one char). Patterns are tested against the path **relative to this config file's directory**.

---

## `[ascii_box]`

Validates ASCII-art boxes — both `+---+ | | +---+` and Unicode `┌─┐ │ │ └─┘`. Catches: top/bottom border width mismatch, drifting `|` column separators across rows, missing inside-cell padding. Bottom-border column checks ignore row separators, spanning rows, connector ports, and incoming connector anchors so flowchart ports are not treated as table columns. Use this when guides include layered diagrams that must align column-perfect across many rows.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `tolerance` | usize | `0` | Columns of slop allowed in border alignment (`0` = exact) |
| `code_blocks_only` | bool | `true` | Only validate inside fenced code blocks (avoids prose false positives) |
| `check_unicode` | bool | `true` | Also validate `┌─┐ │ └─┘` style boxes |
| `tab_width` | usize | `4` | Tab stop width for visual column calculation |
| `check_col_separators` | bool | `true` | Verify `│` column positions are consistent across rows; disable for spatial side-by-side diagrams |

Set `check_col_separators = false` when the directory's diagrams legitimately place independent boxes at different column positions in the same code block. Width mismatches are still fully checked.

---

## `[ascii_flow]`

Validates flowcharts — boxes connected by arrows (`-->`, `──▶`, `→`) and vertical pipes. Catches arrow gaps, drifting connector-only vertical lines, and inconsistent cell padding inside flow boxes. Multi-space layout gaps between separate arrows and bidirectional scale rulers are not treated as broken arrow bodies.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `check_arrow_alignment` | bool | `true` | Detect arrows; verify they form straight lines without gaps |
| `check_cell_padding` | bool | `true` | Verify text in flow boxes has consistent padding both sides |
| `min_cell_padding` | usize | `1` | Minimum padding spaces inside a cell |

---

## `[ascii_barchart]`

Validates inline ASCII bar charts — labeled rows where bar length encodes a numeric value. Catches misaligned values, mixed value formats (`%` vs raw integer), bar lengths that don't match their numeric values (e.g. a 78% bar filling 100% of the chart width), and missing label/value padding. Runs only in plain-text diagram fences: empty info string, `text`, `txt`, `ascii`, `diagram`, `chart`, or `barchart`. Multi-run texture rows, equation operators, axis-attached bars, and boxed panels are ignored; stacked bars may mix default fill characters such as `█` and `░`.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `min_bar_width` | usize | `3` | Minimum consecutive bar chars to count as a bar |
| `min_chart_rows` | usize | `2` | Minimum consecutive bar rows to count as a chart |
| `bar_chars` | `Vec<string>` | `[]` (uses `█▓▒░#=`) | Custom bar-body characters; empty = defaults |
| `min_label_padding` | usize | `1` | Minimum spaces between label and bar start |
| `min_value_padding` | usize | `1` | Minimum spaces between bar end and value |
| `check_value_format` | bool | `true` | Warn when value formats differ across rows |
| `require_value_alignment` | bool | `true` | Warn when values don't form a clean column |
| `alignment_tolerance` | usize | `1` | Columns of slop in value alignment |
| `check_proportionality` | bool | `true` | Warn when bar widths don't match the encoded values |
| `proportionality_tolerance` | usize | `2` | Bar-character slop for proportionality (rounding errors) |

Use `bar_chars = ["*"]` if your charts use ASCII-only bars instead of the default block characters.

---

## `[ascii_char]`

Wide / fullwidth Unicode characters (CJK, fullwidth ASCII variants, some emoji) consume two visual columns but one source character — they can silently break ASCII-art alignment. This check flags them by default. Set `error_on_wide = false` when the directory legitimately contains wide content as guide examples (typography, world-languages, culinary-history, status checklists) — the `[ascii_box]` checker still uses correct visual width to validate alignment, so a misdrawn box around wide characters is still caught.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `error_on_wide` | bool | `true` | Error on 2-column chars in alignment-sensitive positions; set false to suppress intentional wide content |
| `warn_unusual` | bool | `false` | Also warn on narrow chars outside the safe Unicode ranges (high false-positive rate) |

---

## `[ascii_tree]`

Validates tree-structured code blocks — directory listings, taxonomies, org charts. Reads the fence info string to decide what to validate: `dirtree` (filesystem hierarchy with `/` slashes), `tree`, `org`, `taxonomy`, etc. Catches indentation drift, duplicate sibling names, missing `/` on directory entries, and (opt-in) paths that don't exist on disk.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `indent_width` | usize | `4` | Spaces per indentation level |
| `kind` | string? | none | Restrict to one fence kind (e.g. `"dirtree"`); `None` validates all tree kinds |
| `check_dir_slash` | bool | `true` | In `dirtree`: directories must end with `/`, files must not |
| `check_duplicates` | bool | `true` | Flag duplicate entry names under the same parent |
| `verify_paths` | bool | `false` | Resolve each `dirtree` path against disk and flag missing entries |
| `verify_root` | string? | none | Filesystem root for `verify_paths`; defaults to the directory containing `proof.toml` |

Use `verify_paths = true` on a section landing page that catalogs an actual directory layout — the check then becomes a live "is the README still accurate?" guard.

---

## `[markdown]`

Document structure: headings, required content, file length. **Must set `enabled = true`** to activate any markdown checks (the default is `false` so a bare `[markdown]` block alone does nothing). Heading-quality checks default on; document-style checks default off.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Master switch — required `true` for any check to run |
| `max_h1` | usize? | none | Maximum H1s per file (typically `1`) |
| `required_h2` | `Vec<string>` | `[]` | At least **one** of these H2s must appear |
| `required_h2_all` | `Vec<string>` | `[]` | **All** of these H2s must appear |
| `optional_h2` | `Vec<string>` | `[]` | Allowed-but-not-required H2s. Non-empty activates H2 allowlist. |
| `forbidden_h2` | `Vec<string>` | `[]` | H2s that must NOT appear — emits `md_forbidden_section` |
| `required_patterns` | `Vec<RequiredPattern>` | `[]` | Substring/regex patterns that must appear |
| `max_lines` | usize? | none | File length cap |
| `check_heading_format` | bool | `true` | Warn on `##heading` (missing space) |
| `check_empty_headings` | bool | `true` | Warn on `## ` (no content) |
| `check_heading_hierarchy` | bool | `true` | Warn when heading levels skip (H1 → H3) |
| `check_duplicate_headings` | bool | `false` | Warn on identical heading text at the same level |
| `thematic_break_style` | string? | none | Enforce `"---"`, `"***"`, `"___"`, or `""` (any) |
| `check_blockquote_spacing` | bool | `false` | Warn on `>text` (missing space after `>`) |
| `check_links` | bool | `true` | Verify cross-document `[text](path.md)` links resolve to a real file. Skips `http(s)://`, `mailto:`, `md://`, `#anchor` links, and inline math/function notation like `[X](t)`. Emits `link_broken_target`. |

### `RequiredPattern`

Substring or regex required to appear somewhere in the file.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `pattern` | string | required | Substring or Rust-regex syntax |
| `description` | string | required | What this pattern represents (shown in diagnostics) |
| `severity` | enum | `"error"` | `"error"` or `"warning"` |

---

## `[markdown_table]`

GFM pipe-table validation: separator format, cell padding, named per-table schemas. Use `ignore_extra_body_cols = true` for math/code-heavy guides where `|` appears legitimately in content (`|G|` group order, regex alternation, bitwise OR) and gets miscounted as extra columns. A blank top-left header is allowed when it serves as the row-label corner of a comparison matrix; its corner separator may use two dashes.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch |
| `min_separator_dashes` | usize | `3` | Minimum `-` count per separator cell (GFM ≥ 3) |
| `check_cell_padding` | bool | `true` | Verify cells have padding both sides |
| `min_cell_padding` | usize | `1` | Minimum spaces inside cell delimiters |
| `required_tables` | usize? | none | Minimum number of tables per file |
| `table_schemas` | `Vec<TableSchema>` | `[]` | Named schemas — see below |
| `check_empty_headers` | bool | `true` | Warn when a header cell is empty, except a row-label corner in comparison matrices |
| `max_columns` | usize | `0` | Warn over this many columns (`0` = no limit) |
| `ignore_extra_body_cols` | bool | `false` | Don't flag rows with MORE columns than header (rows with FEWER are still flagged) |

---

## `[[markdown_table.table_schemas]]`

A schema applied to a specific named table (matched by H2 heading) or to all tables in a file. Use `link_columns` + `verify_link_targets = true` to make a table a live navigation index — bare text in a link column is flagged, and broken paths are caught.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `heading` | string? | none | H2 text the table must follow (without `##`); `None` = applies to all tables |
| `required_columns` | `Vec<string>` | `[]` | All these column headers must be present (exact match) |
| `required_columns_any` | `Vec<string>` | `[]` | At least one of these column headers must be present |
| `min_body_rows` | usize? | none | Minimum body rows |
| `required_row_keys` | `Vec<string>` | `[]` | Values that must appear in the first (key) column |
| `column_allowed_values` | `HashMap<string, Vec<string>>` | `{}` | Per-column whitelist of cell values |
| `link_columns` | `Vec<string>` | `[]` | Columns where every body cell must contain a `[text](url)` link |
| `link_auto_fix` | string | `""` | `"directory"`, `"file"`, or `""` — strategy for repairing bare-text cells |
| `verify_link_targets` | bool | `false` | Resolve link paths and check existence on disk |

`link_auto_fix` strategies: `"directory"` turns `computing/` → `[computing/](../computing/00-OVERVIEW.md)`; `"file"` turns `01-PKG.md` → `[01-PKG.md](../dirname/01-PKG.md)`; `""` reports without fixing.

---

## `[[section_schemas]]`

Per-glob rule overrides, layered additively on top of the root `[markdown]` block. Use this when one section needs stricter rules than the rest (e.g. landing pages must have a navigation table, language guides need a Type System Snapshot). Each `[[section_schemas]]` entry contributes additional requirements; if a file matches multiple entries, ALL their requirements apply.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `paths` | `Vec<string>` | required | Globs — files matching ANY count as candidates |
| `paths_exclude` | `Vec<string>` | `[]` | Globs that exclude files even if matched by `paths` |
| `required_h2_all` | `Vec<string>` | `[]` | Additional H2s — all must be present |
| `required_h2` | `Vec<string>` | `[]` | Additional H2s — at least one must be present |
| `optional_h2` | `Vec<string>` | `[]` | H2s that are allowed but not required. When any H2 list is non-empty, H2s not in any list emit `md_unexpected_section` |
| `forbidden_h2` | `Vec<string>` | `[]` | H2s that must NOT appear — emits `md_forbidden_section`. Use to keep authoring scaffolds (`Draft`, `TODO`) out of production |
| `required_patterns` | `Vec<RequiredPattern>` | `[]` | Additional patterns |
| `max_lines` | usize? | none | Override `[markdown]` `max_lines` for matched files |

**H2 allowlist behaviour:** when `optional_h2`, `required_h2`, or `required_h2_all` is non-empty in the effective config, any H2 heading not in any of those lists triggers `md_unexpected_section`. Leave all three empty to allow any H2.

Use `paths_exclude` to carve exceptions out of a broad `paths = ["*.md"]` instead of fighting the additive union of multiple schemas.

---

## `[[custom_rules]]`

Free-form regex rules. Each rule is applied to every file (or a glob subset via `only_in`) and reports on match or non-match. Use `negate = true` for "this pattern should NOT appear" rules (TODOs, leftover review tags) — that's almost always what you actually want.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Identifier shown in diagnostics |
| `description` | string | required | Human-readable purpose |
| `pattern` | string (regex) | required | Rust `regex` syntax |
| `negate` | bool | `false` | If `true`, **warn when pattern IS found** (inverse match) |
| `severity` | string | `"warning"` | `"error"` or `"warning"` |
| `only_in` | `Vec<string>` (globs) | `[]` | Restrict to matching files (empty = all files) |

---

## `[[compile]]`

Declares source/output directory pairs for `proof compile`. Each entry maps one source directory containing `.source.md` files to one output directory of compiled `.md` files. Multiple `[[compile]]` blocks declare multiple targets, all built by a single `proof compile` invocation. Paths are relative to the proof root.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `source_dir` | string? | none | Source directory containing `.source.md` files |
| `output_dir` | string? | none | Output directory for compiled files |

```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

---

## `[[davinci]]`

Pin a specific figure to an `md://` URI and attach invariants that must hold across edits. Protects canonical diagrams from silent drift when guide content gets refactored. Register entries via `proof pin "md://..." --id <name>` (recommended) or write directly in `proof.toml`. DaVinci entries are additive across cascade — a root config can establish library-wide pins, and per-directory configs add their own.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | required | Stable identifier (used in `proof pin list` and diagnostics) |
| `uri` | string | required | `md://path#heading:selector` — resolved via mdpath |
| `description` | string | `""` | Human-readable purpose |
| `template` | string? | none | Template name to inherit base invariants from |
| `protection` | enum | `"warn"` | `"warn"` (warning), `"error"` (fails check), `"lock"` (reserved for future hard-block) |
| `invariants` | `Vec<Invariant>` | `[]` | One or more invariant rules — see below |

---

## `[[davinci.invariant]]`

A single invariant rule on a pinned element. Many parameters are mutually relevant per rule (e.g. `box-width` uses `min`/`max`, `contains-text` uses `text`); only the relevant fields need to be set.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `rule` | string | required | Rule name — see catalog below |
| `text` | string? | none | String parameter (`contains-text`, `equals`, `starts-with`, `ends-with`, `pattern`) |
| `min` | usize? | none | Lower bound (`box-width.min`, `row-count.min`, etc.) |
| `max` | usize? | none | Upper bound (`box-width.max`, `row-count.max`, etc.) |
| `value` | usize? | none | Exact count (`box-count`, `column-count`) |
| `values` | `Vec<string>`? | none | List parameter (`required-row-keys`) |
| `tolerance` | usize? | none | Slop allowed (`bar-proportional`, `all-boxes-same-width`) |

**Rule catalog:**

| Rule | Parameters | Description |
|------|-----------|-------------|
| `box-width` | `min`, `max` | Visual width of the box border in range |
| `box-count` | `value`, `min`, `max` | Number of detected boxes |
| `column-count` | `value` | Column separators per content row |
| `row-count` | `min`, `max`, `value` | Body rows in a table figure |
| `contains-text` | `text` | Figure must contain this string |
| `starts-with` | `text` | First non-empty line begins with this |
| `ends-with` | `text` | Last non-empty line ends with this |
| `equals` | `text` | Figure content (trimmed) equals this exactly |
| `pattern` | `text` | Figure must contain this substring/regex |
| `required-row-keys` | `values` | Table figure must contain all these keys in column 1 |
| `all-boxes-same-width` | `tolerance` | All detected boxes share a width (within tolerance) |
| `bar-proportional` | `tolerance` | Bar widths in a chart are proportional to their values |

---

## Annotated example — full `proof.toml`

```toml
# Top-level: optional explicit parent. Path is relative to this file's dir.
extends = "../shared-rules.toml"

[meta]
name        = "Reference Library"
description = "Schema for content guides"

[files]
include = ["**/*.md"]
exclude = [
    "TRACKER.md", "VOLUMES.md", "CLAUDE.md",   # library management
    "_archive/**", "sections/**",              # not content guides
    "*/00-OVERVIEW.md",                        # landing pages — separate schema
]
root = true                                    # stop cascade here

# ── Compile pipeline ────────────────────────────────────────────────────────

[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

# ── ASCII validators (whole-struct child-wins on cascade) ───────────────────

[ascii_box]
enabled              = true
tolerance            = 2          # ±2 col drift
check_col_separators = false      # spatial diagrams in this directory

[ascii_flow]
enabled               = true
check_arrow_alignment = true
min_cell_padding      = 1

[ascii_barchart]
enabled                   = true
min_bar_width             = 3
check_proportionality     = true
proportionality_tolerance = 2

[ascii_char]
error_on_wide = false             # CJK examples appear in this section

[ascii_tree]
enabled       = true
indent_width  = 4
kind          = "dirtree"
verify_paths  = false             # set true to validate paths exist on disk

# ── Markdown structure (lists are additive on cascade) ──────────────────────

[markdown]
enabled                 = true    # required to activate any markdown check
max_h1                  = 1
required_h2_all         = ["Decision Cheat Sheet"]
max_lines               = 800
check_heading_hierarchy = true

[[markdown.required_patterns]]
pattern     = "```"
description = "must contain at least one code block"
severity    = "warning"

# ── Pipe tables (whole struct child-wins on cascade) ────────────────────────

[markdown_table]
enabled                = true
required_tables        = 1
ignore_extra_body_cols = true     # math/code uses | freely

[[markdown_table.table_schemas]]
heading            = "Decision Cheat Sheet"
min_body_rows      = 2

[[markdown_table.table_schemas]]
heading              = "Directories"
required_columns     = ["Directory", "Entry Point"]
min_body_rows        = 3
link_columns         = ["Directory", "Entry Point"]
link_auto_fix        = "directory"
verify_link_targets  = true

# ── Per-glob overrides (additive across cascade) ────────────────────────────

[[section_schemas]]
paths           = ["computing/**", "ai-engineering/**"]
required_h2_all = ["The Big Picture", "Common Confusion Points"]

# In a directory-level proof.toml, paths auto-prefix with that directory:
# [[section_schemas]]
# paths           = ["*.md"]              # → languages/*.md
# paths_exclude   = ["00-OVERVIEW.md"]
# required_h2_all = ["Type System Snapshot"]

# ── Custom regex rules ──────────────────────────────────────────────────────

[[custom_rules]]
name        = "no_editor_tags"
description = "@editor review tags should be resolved before publication"
pattern     = "@editor\\["
negate      = true                # warn when pattern IS found
severity    = "warning"

# ── Pinned figures with invariants ──────────────────────────────────────────

[[davinci]]
id          = "package-layer-stack"
uri         = "md://computing/01-PACKAGE.md#the-big-picture:0"
description = "Canonical 5-level package manager hierarchy"
protection  = "error"             # fails `proof check --davinci`

  [[davinci.invariant]]
  rule = "box-width"
  min  = 68
  max  = 72

  [[davinci.invariant]]
  rule = "contains-text"
  text = "SYSTEM / OS LAYER"

  [[davinci.invariant]]
  rule = "box-count"
  min  = 5
```

---

## `[ai]`

Configures the external CLI used by `proof spec-generate --ai` and future AI-assisted commands. proof shells out to any CLI that accepts a prompt and writes its response to stdout — no API client code, no SDK. Configure once, use everywhere.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `command` | string | `"claude"` | CLI binary name (must be on PATH or absolute path) |
| `args` | `Vec<string>` | `["-p", "{prompt}"]` | Argument list. `{prompt}` is replaced with the prompt text at call time. If no arg contains `{prompt}`, the prompt is written to stdin. |

**Common configurations:**

```toml
# Claude Code (default — works if `claude` is installed)
[ai]
command = "claude"
args    = ["-p", "{prompt}"]

# Simon Willison's llm tool
[ai]
command = "llm"
args    = ["-m", "gpt-4o", "{prompt}"]

# Ollama (local model)
[ai]
command = "ollama"
args    = ["run", "llama3", "{prompt}"]

# aichat
[ai]
command = "aichat"
args    = ["{prompt}"]
```

Usage:

```bash
proof spec-generate "md://figures/arch.md:figure:goroutine-scheduler" --ai
```

Without `--ai`, `proof spec-generate` uses static heuristic analysis (no CLI required).

---

## See also

- `proof config <file>` — print effective merged config for any file (single source of truth when debugging cascade)
- `schemas/default.toml` — minimal starter (run `proof init` to copy)
- `schemas/reference.toml` — full real-world example
- `design/COMPILE-SPEC.md` — `proof compile` pipeline and source-file directives
- `design/STYLE-GUIDE.md` — style rules referenced by checks (S-01 wide chars, etc.)
