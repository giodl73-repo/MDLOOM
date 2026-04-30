# Changelog

All notable changes to **proof** (formerly **glint**), in [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. This project follows semantic versioning.

The throughline: a tool that began as an ASCII-box width checker has grown into a four-stage document quality system — **detect → plan → fix → compile** — with stable figure addressing, invariant pinning, and a math/diagram/slide rendering pipeline on top.

```
v0.7  ┌──────────────────────────────────────────────────────────────┐
      │ spec-honesty · 8 chart kinds · md:// query params · snapshots │
      ├──────────────────────────────────────────────────────────────┤
v0.6  │ xref · chart · reveal · AI CLI · author experience            │
      ├──────────────────────────────────────────────────────────────┤
v0.5  │ math · watch · multi-target compile · guides · source-link    │
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

## [0.7.0] — 2026-04-30 — *the spec-honesty release*

A push to close every overpromised "✅ Implemented" status against the actual code. Every item below was either claimed in spec but a placeholder, or labeled deferred. Now they ship.

### Added

- **Multi-line directives inside `proof:region` bodies** — `proof:chart`, `proof:tree`, `proof:row`, `proof:element`, `proof:symbol`, `proof:shape`, `proof:math` all render correctly when nested inside a dashboard region (fenceless syntax). Previously the inner directive header was kept but its body lines were dropped, so anything with inline data silently rendered as a placeholder. (Closed issue #6.)
- **`proof:tree kind=outline` numbered-bullet inline mode** — `1. / 1.1 / 1.1.1` lines auto-indent by dot depth; trailing periods normalize at depth ≥ 1.
- **Slide layout `content-caption`** — title + body + caption strip from `subtitle:`. Was previously a fallback to `title-content`.
- **Slide layout `comparison`** — 2×2 quadrant grid via `## q:tl/tr/bl/br` markers, with optional `## axis:x` / `## axis:y` labels. Was previously a fallback.
- **`proof:tree kind=decision`** — DFS with Yes/No branch labels, leaf labels for unknown targets, cycle guard. Was listed in TREE-SPEC but had no implementation.
- **DaVinci `regex` invariant rule** — alongside the existing substring `pattern` rule. Powered by the `regex` crate.
- **Quarter-block dither mode for figure import** — full 16-glyph 2×2 quadrant table; previously fell back to full-block. Also unbroke the `--features figure` build (37 → 0 errors).
- **ASCII tree T-4 children-shape lint** — detects continuation `│` lines that imply a child but where the next real node sits at the same or shallower depth.
- **md:// URI query parameters** — `?select=cols`, `?filter=col=val|col!=val|col>val|col<val`, `?count`, `?top=N`, `?skip=N`. Threaded through both URI resolution paths so every directive that reads md:// honors them.
- **Eight new chart kinds** — `area`, `stacked-bar`, `waterfall`, `scatter`, `heatmap`, `candlestick`, `gantt`, `timeline`. `ChartPoint` extended with `extras: Vec<f64>` for multi-value kinds.
- **Cache snapshots subsystem** — `proof cache snapshot {save|restore|list|diff|prune|deploy}`. Named compile states with integrity hash; restore is rejected with `COMPILE-004` if the manifest was tampered with.

### Changed

- `parse_md_table` now accepts both bounded (`| a | b |`) and unbounded (`a | b`) pipe-table forms; mdpath returns the unbounded form when extracting an addressed table.
- DASHBOARD-SPEC, SLIDE-SPEC, TREE-SPEC, FIGURE-IMPL-PLAN, MAPPING-SPEC, CHART-SPEC, CACHE-SNAPSHOTS status lines all updated to match what ships.

### Removed

- **`proof:chart kind=sankey` removed from CHART-SPEC scope.** Proportional flow widths quantize poorly to fixed-width character cells. Authors who need flow visualizations should use `kind=stacked-bar` for level transitions or embed an SVG via `proof:include`.

### Tests

93 new unit and integration tests across the changes. 793 total pass; zero build warnings.

---

## [0.6.0] — 2026-04-28 — *the author experience release*

The focus shifts from "can proof render this?" to "does proof help you author well?" v0.6 closes the authoring loop: cross-references that update themselves, diagnostics that suggest fixes, AI-assisted invariant generation, and a full corpus-scale toolset. The slide system matures from a renderer into a presentation platform.

### Added

#### New compile directives

- **`proof:chart`** — bar and line charts rendered to ASCII from a markdown table source. Supports axis labels, title, configurable width. Used inside any `.source.md` document or dashboard region.
- **`proof:xref`** — cross-reference directive that resolves the target heading text at compile time. `uri="md://api.md#authentication"` renders as `*See: [Authentication](api.md#authentication)*`. Three formats: `inline`, `note`, `callout`. When a heading is renamed, recompile updates every `proof:xref` automatically.
- **`proof:blockquote`** — prose document block quote with a left margin bar (`│`). Distinct from `proof:quote` (slide-only, centered); `proof:blockquote` is for document context with optional `attribution=` and `style=` (`bar` | `indented` | `double`).
- **`proof:include pin=id`** — declare the expected DaVinci invariant ID inline on an include directive. Emits COMPILE-007 warning when no matching `[[davinci]]` entry exists in `proof.toml`, prompting `proof pin <uri> --id <id>`. When the pin exists, invariant validation runs as before.

#### Slide system — presentation platform

- **Progressive reveal** — bullets prefixed `[N]` (N ≥ 2) are assigned to reveal step N. `proof compile` produces one canvas block per step, cumulative. The `[N]` syntax works inside any `proof:bullets` block in a `.slides.source.md` file.
- **Slide footer** — `footer: true` in front-matter stamps author, date, and deck title on the last row of every slide canvas. `footer-text: "Custom"` overrides the auto format.
- **`layout=agenda`** — auto-generates a bullet list of all `layout=section` slide titles from the deck. The agenda lists section slides that appear *after* it — no manual maintenance.
- **Slide progress bar** — `progress-bar: true` emits a `████░░░ N/M` proportional bar between the SLIDE separator and the canvas content. Outside the canvas (SL-1 invariant still holds).
- **Two-column default ratio 60:40** — changed from 50:50. Presentation best practice; `ratio=50:50` in existing source files still works unchanged.

#### Corpus-scale tools

- **`proof status`** — one-screen corpus health summary: source count, compiled count, stale files, last compile time, cached error/warning counts, config summary. `proof check` now writes `.proof/last-check.json` after every run so `proof status` can display live diagnostic counts.
- **`proof depends`** — reverse dependency lookup: `proof depends md://api.md#authentication` lists every `.source.md` file that references that URI. Find everything that breaks before renaming a heading or moving a figure.
- **`proof check --unused`** — find `.md` figures that no `.source.md` references via `proof:include`, `proof:layout`, or `source=md://...`. Emits `unused_figure` warnings. Off by default (full corpus walk); enable with `--unused`.
- **`proof check --deduplicate`** — at corpus scale, collapses repeated identical diagnostics into `42x warning [SLIDE-001]: ... in docs/slides/*.md`. Singletons still render normally.

#### AI-assisted authoring

- **`[ai]` config block** — configures any external AI CLI for `proof spec-generate --ai` and future commands. `command` + `args` with `{prompt}` substitution. Default: `claude -p "{prompt}"` (Claude Code). Works with `llm`, `ollama`, `aichat`, or any CLI that reads a prompt and writes a response.

```toml
[ai]
command = "claude"
args    = ["-p", "{prompt}"]
```

- **`proof spec-generate --ai`** — calls the configured AI CLI with the figure content and asks it to suggest `[[davinci]]` invariants. Without `--ai`, the existing static heuristic analysis runs with no dependencies.

#### Schema — section rules

- **`optional_h2`** in `[markdown]` and `[[section_schemas]]` — H2 headings that are allowed but not required. When any of `required_h2`, `required_h2_all`, or `optional_h2` is non-empty, H2s not in any list emit `md_unexpected_section` (H2 allowlist).
- **`forbidden_h2`** in `[markdown]` and `[[section_schemas]]` — H2 sections that must NOT appear. Emits `md_forbidden_section`. Use to keep authoring scaffolds (`## Draft`, `## TODO`) out of production guides.

#### Diagnostic improvements

- **Did-you-mean for symbols** — `Unknown symbol 'checkmar' — did you mean 'checkmark'?` Levenshtein distance search across names + aliases.
- **Did-you-mean for `md://` URIs** — `Reference to 'fig.md' not found — did you mean 'figs.md'?` Filesystem walk for closest match within edit distance 3.
- **`md://` heading path validation in `proof check`** — verifies that heading slugs in URIs (e.g. `md://api.md#authentication`) resolve to real headings in the target file. Emits `md_broken_heading` when the heading doesn't exist. Previously only the file's existence was checked.
- **`SLIDE-001` message** — now reads `"Slide has 6 bullets — reduce to 4 or fewer (30-second rule)"` with actionable count and threshold. Default `max_bullets` changed from 6 → 4.
- **Bullet continuation paragraphs** — indented prose under a bullet item renders with the parent bullet's content-column indent and no glyph. Does not count toward `max_bullets`.

#### Dashboard

- **DASHBOARD-006** — warning when canvas `width` exceeds 220 columns (standard terminal threshold). Emitted at compile time.

### Changed

- `proof spec-generate` signature: now accepts `--ai` flag and reads `[ai]` config from `proof.toml`.
- `proof:toc section=` parameter (already in v0.5) documented and tested with 7 dedicated tests.
- All design spec status lines updated from "not yet implemented" to reflect actual implementation state.

### What it enables

Authors can now write a 50-slide deck, reference 300 figures by name, and know that every cross-reference is alive, every invariant is enforced, and any heading rename ripples through automatically on the next compile. The `proof status` + `proof depends` + `proof check --unused` triad gives full corpus visibility without running the full check pipeline. The `[ai]` block turns any installed AI CLI into a first-class authoring assistant — no API keys in config, no SDK dependencies.

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
- **Prose link target verification** — `MarkdownCheck` now resolves every `[text](path.md)` link against the runner root (or file parent) and emits `link_broken_target` for missing files. Skips `http(s)://`, `mailto:`, `md://`, and `#anchor` links; ignores links inside fenced code blocks and backtick code spans. Toggle via `[markdown] check_links = false`.

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
