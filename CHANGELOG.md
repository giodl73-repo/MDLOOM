# Changelog

All notable changes to **proof** (formerly **glint**), in [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. This project follows semantic versioning.

The throughline: a tool that began as an ASCII-box width checker has grown into a four-stage document quality system — **detect → plan → fix → compile** — with stable figure addressing, invariant pinning, and a math/diagram/slide rendering pipeline on top.

```
v0.5  ┌──────────────────────────────────────────────────────────────┐
      │ math · watch · multi-target compile · guides · source-link    │
      ├──────────────────────────────────────────────────────────────┤
v0.4  │ slides · dashboard · figures · symbols · elements             │
      ├──────────────────────────────────────────────────────────────┤
v0.3  │ compile pipeline · md:// URI scheme · DaVinci pinning         │
      ├──────────────────────────────────────────────────────────────┤
v0.2  │ fix pipeline · draft · baseline                               │
      ├──────────────────────────────────────────────────────────────┤
v0.1  │ check · ASCII box / flow / tree · markdown rules              │
      └──────────────────────────────────────────────────────────────┘
```

---

## [0.5.0] — 2026-04-27 — *the rendering release*

The shift from "compiles figures" to "compiles documents." A `.source.md` file can now embed real math, render trees and charts from data, and compose slide decks — all to ASCII output that survives any monospace pipeline. Multi-target watch builds and the `mdpath` Classifier extension make `proof` usable as a live build tool for any docs site, not just a CI gate.

### Added

#### Math module — `proof:math`

A complete ASCII math renderer. Inline `$...$` and display `proof:math` blocks expand to centered ASCII with real geometric layout — no LaTeX, no MathJax, no fonts.

- **Tokenizer + symbol table** — Greek letters, operators, relations, set-theory symbols, arrows, calligraphic and blackboard letters. Hundreds of tokens map to single Unicode glyphs.
- **Superscripts and subscripts** — `x^2` renders with real superscript digits; `H_2O` uses subscript digits. Multi-character exponents stack above the baseline.
- **Fractions** — numerator and denominator centered above and below a horizontal bar, width auto-computed from operand widths.
- **Integrals, sums, products** — large operators with bounds positioned above and below the symbol; the integrand sits flush to the right.
- **Matrices and vectors** — `pmatrix`, `bmatrix`, `vmatrix` with column-aligned cells and proper bracket characters that scale to row count.
- **Square roots** — radical with a horizontal bar that extends across the radicand.
- **Tier 2 layouts** — limits, piecewise functions, accents, multi-line equations.
- **Render targets** — display math (centered block) and inline (single-line); both unicode-width-aware.

#### `proof compile` — multi-target + watch

- **`[[compile]]` config blocks** — declare any number of source/output directory pairs in `proof.toml`. Each pair can have its own `source_dir`, `output_dir`, and optional filters. `proof compile` with no args reads the table and compiles every target.
- **`--watch`** — file watcher across all `[[compile]]` targets. Saves to `.source.md` files retrigger compile to the paired `output_dir`. Edits to a referenced figure retrigger every dependent file via the cache's reverse-dependency index.
- **`--output-dir` / `-o-dir`** — single-flag override for ad-hoc output directory at the CLI. Mutually exclusive with `-o` (single-file output).
- **Default output resolution** — CLI flag wins; otherwise the first matching `[[compile]]` target's `output_dir`; otherwise the source directory.
- **`.slides.source.md` → `.slides.md`** wiring — the SLIDE renderer is now part of the compile pipeline, dispatched on filename suffix.

#### Source link checking

- New checks `source_link_broken`, `source_link_missing` validate links inside `.source.md` files against the **resolved output paths**, not the source paths. A link to `../guides/01-math.md` in a source file now correctly checks against `docs/guides/01-math.md` after compile resolution.
- Source-side link checks integrate into the `proof check` pipeline; CI can now catch broken cross-document links before compile.

#### `mdpath` Classifier extension

- The `mdpath` library now ships a **Classifier** trait that lets consumers extend element-kind detection without forking. `proof` registers classifiers for math blocks, slide regions, dashboard regions, and trees, so `md://` URIs with `:figure.math:`, `:figure.slide:`, `:figure.tree:` selectors resolve correctly.
- Classifier registration is composable — multiple classifiers can claim non-overlapping kinds; conflicts are reported as `MDPATH-005`.

#### New directives (full implementations, not just specs)

- **`proof:tree`** — directory tree, taxonomy tree, or reference tree from a YAML/JSON source. Validators T-1 through T-8 enforce structure (no orphan children, consistent indent, balanced branches). 4 implementation waves complete.
- **`proof:chart`** — ASCII bar chart, sparkline, histogram, and 5 more kinds. Three explicit categories (categorical, distribution, time-series). Reads from `[[mapping]]` data sources.
- **`proof:slide`** — one slide per block in a `.slides.source.md` deck. Layout renderers handle title, two-column, image-with-caption, code-with-output, and bulleted forms. Wave 4 wires the deck-level compile.
- **`proof:dashboard`** + **`proof:region`** — multi-region dashboard composition. Wave 3 region compositor places regions on a grid and equalizes row heights.
- **`proof:element`** — named ASCII element library (boxes, banners, callouts) with image import via `image`/`resvg`. 99 tests.
- **`proof:symbol`** — `[sym:name]` inline expansion engine and core symbol library. 39 tests.
- **`proof:figure`** — named ASCII art figures with optional image import.
- **`[[mapping]]`** — shared data-binding system used by `proof:row`, `proof:tree`, `proof:chart`. One mapping table, multiple consumers.

#### Guides infrastructure

- **`docs/guides/`** — first-class user guides authored as `.source.md` and compiled by `proof` itself (eat your own dog food). Topics: `00-getting-started`, `01-math`, `02-symbols`, `03-elements`, `04-slides`, `05-trees`, `06-dashboard`, `07-compile`, `08-lint`.
- The guides directory is wired as a `[[compile]]` target in the repo's own `proof.toml`. Editing a guide source recompiles to `docs/guides/`.

#### Workspace setup

- `proof` and `mdpath` now live as siblings under one parent (`C:/src/proof`, `C:/src/mdpath`) with `proof` consuming `mdpath` via path dependency. Cargo workspace config aligns versions and shares a target directory for faster incremental builds.
- README and TUTORIAL document the two-repo clone-side-by-side install.

#### Other

- **`proof spec-generate`** — given a figure, suggests structural invariants (box count, required labels, minimum row count) suitable for a `[[davinci]]` block. Bootstraps pinning for a large existing corpus.
- **`mdpath` BatchResolver** — resolve multiple `md://` URIs against the same file without reparsing. `proof compile` uses this for per-file resolution passes.
- **31 spec scenarios** hand-simulated with findings resolved across compile, layout, cache, and snapshot specs (`design/SCENARIOS.md`).
- **403+ tests** across SLIDE waves, **99** for element, **73** integration tests for L1 coverage gaps.
- **Diagnostics**: `COMPILE-001..007`, `MDPATH-001..005`, `MATH-001..008`, `TREE-001..008`, `CHART-001..006`.

### Changed

- **Renamed `fig://` → `md://`** throughout — all specs, source code, tests, and config examples.
- **Removed all `glint` references from source** — binary, library, config file, and emitted output. Naming history retained at the bottom of this file for reference.
- **Cargo description** updated to reflect full scope: figures, tables, links, ASCII art, and source compilation.
- **Bottom-border tolerance** now correctly applied (was previously skipped); blank line between boxes no longer breaks box boundary detection; tree-diagram false positives suppressed.
- **Auto-fix range extended** to ±4 box offsets; cell padding now auto-fixes for single-column boxes.

### Fixed

- 5 issues from architectural review (`16fec28`).
- 6 review pipeline findings + BENCH coverage gaps (`a6fd65c`).
- Three intentional-content config escape hatches added so legitimate patterns stop being flagged (`c212a5e`).

### What it enables

A docs site authored as `.source.md` files compiles to render-ready `.md` with correct math, validated figure references, alignment-checked ASCII art, and broken-link detection — all in a single watch loop. The MAXIM library (2,170 files, ~14,000 pages) is built end-to-end with `proof compile --watch`.

---

## [0.4.0] — 2026-04-26 — *the figure release*

The shift from "compile pipeline exists" to "compile pipeline has things to compose." A library of named, addressable, image-importable figure primitives — slides, dashboards, elements, symbols, figures themselves — each with its own spec, implementation waves, and test fixtures. By the end of v0.4 the directive vocabulary covered everything a real docs corpus needs to render: prose, math (designed), trees, charts, slides, dashboards, and named elements.

### Added

- **SLIDE** (`.slides.source.md` decks), **DASHBOARD** (multi-region grids), **FIGURE** (named ASCII figures with image import), **SYMBOL** (`[sym:name]` expansion), and **ELEMENT** (boxes, banners, callouts) subsystems — each shipped as a SPEC, an IMPL-PLAN, and at least one implementation wave.
- **MAPPING-SPEC** — shared data-binding mechanism used by every directive that reads from a data source.
- **`image` and `resvg` dependencies** — figure import from PNG/SVG.
- Spec review roles: SOURCE, COMPOSE, CACHE — added under `.roles/`.

### What it enables

Docs that need a slide deck, a dashboard, or a callout no longer drop down to ASCII art by hand. Each block is a directive backed by a renderer that knows its own invariants.

---

## [0.3.0] — 2026-04-25 — *the addressing release*

The shift from "linter that finds problems" to "document quality system with stable handles for figures." Renamed `glint` → `proof`. Introduced the `md://` URI scheme, the `proof compile` pipeline, and DaVinci invariant pinning.

### Added

- **`md://` URI scheme** — every figure (box, flowchart, table, chart) gets a stable handle of the form `md://path#heading:figure.kind:label`. Section-qualified addresses survive line shifts. Implemented in the `mdpath` standalone crate (56+ passing tests). Sub-selectors (`[row=X]`, `[col=Y]`, `[box=Z]`), OData query parameters (`?select`, `?filter`, `?top`, `?skip`, `?count`).
- **`proof compile`** — markdown compiler that resolves `proof:include` and `proof:layout` directives in `.source.md`, validates DaVinci invariants on each included figure, and writes compiled output. `--check`, `--cache-status`, `--no-cache`, snapshot save/restore/diff/list/prune/deploy.
- **`proof layout`** — ASCII collage composer. N figures arranged side-by-side with height equalization, gap insertion, unicode-width-aware columns, multi-row wrapping, label centering, top/center/bottom alignment, optional borders. Invariants L-1 through L-9.
- **`proof resolve`** — print element content, file path, line range, label, kind for any `md://` URI.
- **`proof pin`** + **`proof pin-list`** — register a figure with DaVinci invariants in `proof.toml`. Protection levels `warn` / `error` / `lock`. Invariant rules: `box-count`, `contains-text`, more.
- **Three-tier cache** (`THREE-TIER-CACHE.md`) and **cache snapshots** (`CACHE-SNAPSHOTS.md`) — content hash → resolution hash → render hash.

### Changed

- Renamed binary `glint` → `proof`, library `glint_lib` → `proof_lib`, config `glint.toml` → `proof.toml`. Old config filename auto-migrates on first run.
- README reframed: from "ASCII art linter" to "Document quality assurance for markdown corpora."

### What it enables

A diagram in `computing/01-PACKAGE.md § The Big Picture` is no longer addressed as "line 47" — it has a stable handle that survives content shifts, can be referenced from other files, and carries invariants enforced at compile time.

---

## [0.2.0] — 2026-04-25 — *the fix release*

v0.1 told you what was wrong. v0.2 fixed it. Detection is mechanical (Rust); fixing is mechanical too — but the *judgment* between them (which border is the authority, which direction to shift a column) is delegated to AI working off rich structured context.

### Added

- **`proof check --format rich`** — diagnostics carry surrounding code blocks, expected vs. actual widths, adjacent lines. Designed as input for AI fix planners.
- **`proof draft`** — pre-populated fix plan with errors grouped by file/region. Auto-fixable groups carry `decision: auto`; ambiguous groups carry `decision: needs_review` with rich context for AI triage.
- **`proof fix --plan plan.json`** — applies a structured fix plan to the working tree. `--dry-run`, `--min-confidence high|medium|low`, `--no-verify`, `--no-signal-check`.
- **Bottom-up application order** — fixes apply highest-line-first so earlier line numbers stay valid. Stale-anchor detection skips and logs rather than corrupting.
- **Signal-loss guard** — refuses fixes that remove non-whitespace content unless explicitly allowed.
- **Three deterministic auto-fixes**: `link_directory` (bare text → markdown link), `box_col_pm1` (column off by one), `nested_box_col` (inner box edges aligned to outer frame). Pattern B and Pattern C detection.
- **GFM table schema validator** — `[[markdown_table.table_schemas]]` blocks declare `required_columns`, `required_row_keys`, `min_body_rows`, `allowed_values`. Diagnostics: `table_missing_column`, `table_missing_row`, `table_min_rows`, `table_bad_value`.
- **Link validation** — `link_columns` + `verify_link_targets` resolve every link cell to disk. Diagnostics: `link_bare_text`, `link_broken_target`, `link_missing`, `md_table_missing_link`, `md_broken_link`.
- **Heading + style checks** — `md_h1_count`, `md_missing_section`, `md_duplicate_heading`, `md_heading_order`, `md_missing_pattern`, `md_file_length`. `ascii_barchart` validates horizontal bar chart geometry.
- **Tab expansion + wide-character detection** — `char_wide`, `char_fullwidth` flag CJK ideographs, em-dashes, presentation forms.
- **`paths_exclude`** for section schemas — schemas can scope to `*.md` while excluding `00-OVERVIEW.md`.
- **E2E test pipeline** — `check → rich → plan → fix → verify` runs in CI on every push.
- **Invariants I-11..I-13** — formal properties of fix application (idempotence, no signal loss, position-stable on partial application).

### Fixed

- **GFM `parse_row` for escaped pipes and code spans** — single fix eliminated 817 false positives.
- **`md_heading_format`** false positive on `C#` and `F#` language names.

### What it enables

Bulk repair with a safety net. The MAXIM library went from "manual repair impractical" to "fixable in one supervised afternoon."

---

## [0.1.0] — 2026-04-25 — *the foundation*

The seed. A fast, schema-driven Rust linter that parsed every code block in a markdown file as potential ASCII art and reported geometric defects with `file:line:col` precision.

### Added

- **`proof check`** — lint files and report diagnostics. Three output formats: `text`, `json`, `rich` (planned).
- **ASCII box / flow / tree validation** — `ascii_box_width`, `ascii_box_col`, `ascii_cell_padding`, `ascii_arrow_gap`, `ascii_connector_drift`. Borders that don't add up, columns that drift, missing whitespace inside cells, broken arrow bodies.
- **Markdown structural rules** — H1 count, required H2s, duplicate-heading detection, heading order.
- **Schema-driven, cascading `glint.toml`** — root config sets defaults; per-directory configs inherit and extend (lists additive, scalars use nearest). Effective config inspection via `proof config <path>`.
- **Parallel file processing** via `rayon` — 2,000-file library completes in under 5 seconds.
- **68 unit + integration tests**, fixtures for every check class.
- **`design/SPEC.md`, `design/INVARIANTS.md`, `design/STYLE-GUIDE.md`** — designed-first, then implemented. Invariants I-01..I-10 specify what a "valid" ASCII box is at the parser level.

### What it enables

Catches silent ASCII art errors that render correctly in a monospace editor but corrupt in MkDocs, GitHub web view, or any rendering pipeline that disagrees with the author's font metrics about character widths.

---

## Naming history

| Period | Binary | Library | Config |
|--------|--------|---------|--------|
| v0.1 — v0.2 | `glint` | `glint_lib` | `glint.toml` |
| v0.3+ | `proof` | `proof_lib` | `proof.toml` |

The rename reflects the scope expansion: `glint` lints; `proof` certifies and compiles.
