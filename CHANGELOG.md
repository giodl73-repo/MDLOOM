# Changelog

All notable changes to **proof** (formerly **glint**), traced from initial release through current state.

The throughline: a tool that began as an ASCII-box width checker has grown into a four-stage document quality system — **detect → plan → fix → pin**. Each release added one tier on top of the last, never replacing what came before.

```
v0.5  ┌────────────────────────────────────────────┐
      │ mdpath + compile + layout + resolve/pin     │  ← source doc compilation
      ├────────────────────────────────────────────┤
v0.4  │ md:// addressing + DaVinci protection      │  ← named, pinnable figures
      ├────────────────────────────────────────────┤
v0.3  │ Table schemas + link validation + draft     │  ← document structure as data
      ├────────────────────────────────────────────┤
v0.2  │ Fix pipeline (rich → plan → fix)            │  ← AI-assisted bulk repair
      ├────────────────────────────────────────────┤
v0.1  │ ASCII box check + cascading config          │  ← detection foundation
      └────────────────────────────────────────────┘
```

---

## [0.5.0] — 2026-04-26 — *the compilation release*

The shift from "linter with pinnable figures" to "document compilation system." Source documents (`.source.md`) now reference figures by stable `md://` URI and are compiled to output `.md` files by a pipeline that validates DaVinci invariants before writing. The `md://` resolver ships as a standalone library (`mdpath`) so editors and CI tools can adopt the addressing scheme without depending on proof.

### Added

- **`mdpath` library** — standalone Rust crate implementing the `md://` URI scheme. Fully implemented with 56+ passing tests covering URI parsing (all forms), section navigation, element detection, label matching (exact → starts-with → substring hierarchy), sub-selectors (`[row=X]`, `[col=Y]`, `[box=Z]`), query parameters (`?select`, `?filter`, `?top`, `?skip`, `?count`), round-trip stability, and error cases. Ships at `https://github.com/giodl73-repo/MDPATH`.

- **`proof compile`** — markdown compiler that resolves `proof:include` and `proof:layout` fenced directives in `.source.md` files, validates DaVinci invariants on each included figure, and writes compiled `.md` output. Default output path drops `.source.` in-place (`foo.source.md` → `foo.md`). Flags: `--watch`, `--check`, `--cache-status`, `--no-cache`. Cache snapshot commands: `proof cache snapshot save/restore/diff/list/prune/deploy`.

- **`proof layout`** — ASCII art collage composer. Takes N figures (via `md://` URIs or file paths) and arranges them side-by-side with correct alignment. Handles height equalization, gap insertion, unicode-width-aware column measurement, multi-row wrapping (`--cols`), label centering (`--labels`), top/center/bottom alignment (`--align`), and optional borders (`--border`). 31 passing tests covering all layout invariants L-1 through L-9.

- **`proof resolve`** — resolve an `md://` URI and print element content, file path, line range, label, section heading, element type, and detected kind.

- **`proof pin`** — register a figure with DaVinci invariants. Writes a `[[davinci]]` block to `proof.toml` with the URI, protection level (`error` or `warn`), and invariant rules.

- **`proof pin-list`** — list all pinned DaVinci figures with their URIs and invariants.

- **Figure validation before embed** — `proof compile` checks each `md://` figure against its DaVinci pin (if one exists) before embedding. Emits `COMPILE-001` (error) or `COMPILE-003` (warn) on invariant violation. Emits `COMPILE-007` (dirty figure warning) when a figure's content has changed since the last pin snapshot.

- **Compile diagnostics**: `COMPILE-001` (DaVinci error violation), `COMPILE-002` (URI resolve failure), `COMPILE-003` (DaVinci warn), `COMPILE-007` (dirty figure warning).

- **`BatchResolver` in mdpath** — resolve multiple `md://` URIs in the same file without re-reading it. `proof compile` uses this for per-file resolution passes.

- **Design documents**: `COMPILE-SPEC.md`, `LAYOUT-SPEC.md`, `THREE-TIER-CACHE.md`, `CACHE-SNAPSHOTS.md`, `SCENARIOS.md` (31 spec findings resolved across all new subsystems).

- **Review roles**: SOURCE (source document format and compile pipeline UX), COMPOSE (layout algorithm correctness and unicode-width accounting), CACHE (cache invalidation and snapshot consistency). Added to `.roles/`.

### Changed

- **Renamed `fig://` → `md://`** throughout — all specs, source code, tests, configuration examples, and documentation. The old scheme name does not appear anywhere in the codebase.
- **Removed all `glint` references from source** — any remaining `glint` references in source files, comments, or generated output have been cleared. The naming history table in this file remains for reference.
- **`proof.toml` description updated** — Cargo.toml description reflects the full scope: figures, tables, links, ASCII art, and compilation.

### What it enables

A figure in `languages/10-GO.md § Concurrency Model` is not just named — it is *compiled into* any document that references `md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler`. Change the source figure; recompile dependent documents. The DaVinci invariants are the contract between the figure and its consumers: if the scheduler diagram loses its "M:N multiplexing" label, every compile that includes it fails, loudly, before the change ships.

---

## [0.4.0] — 2026-04-25 — *the addressing release*

The shift from "linter that finds problems in files" to "document quality system with stable handles for figures." Renamed `glint` → `proof` to reflect the broader scope: this is no longer just a linter, it is an evidentiary tool — every diagnostic, every fix, every pinned figure is *proof* that the document holds together.

### Added

- **`fig://` URI scheme** (design complete, implementation in progress) — every figure (box, flowchart, bar chart, table) gets a stable, human-readable address of the form `fig://path#heading:index`. Line numbers change as content evolves; section-qualified figure addresses survive.
- **DaVinci protection tier** (designed) — pinned figures carry invariants (column count, row count, named labels). If a future edit violates the invariant, `proof` reports it as an error regardless of whether the box still parses cleanly. Figures graduate from "must be syntactically valid" to "must remain semantically the figure they were."
- **`design/FIG-SPEC.md`** — open specification for the addressing scheme. Any tool (editors, CI, agents) can implement a resolver; `proof` is the reference implementation.

### Changed

- Renamed binary `glint` → `proof`, library `glint_lib` → `proof_lib`, config file `glint.toml` → `proof.toml`. Old config filename auto-migrates on first run.
- Cargo manifest version bumped to `0.2.0` (the published crate version lags the conceptual milestone numbering — semver vs. release-train naming).
- `README.md` reframed: from "ASCII art linter" to "Document quality assurance for markdown corpora — figures, tables, links, and ASCII art."

### What it enables

A diagram in `computing/01-PACKAGE.md § The Big Picture` is no longer addressed as "line 47" — it is addressed as `fig://computing/01-PACKAGE.md#the-big-picture:0`. That handle survives content shifts, can be referenced from other files, and can carry invariants. The foundation for cross-file consistency checks and figure-level CI gates.

---

## [0.3.0] — 2026-04-25 — *the schema release*

Where v0.2 made fixes possible, v0.3 made the rules expressive enough to enforce real document contracts. The library style guide (every guide must contain a "Type System Snapshot" with these four required rows; every "Directory" cell must be a working markdown link) becomes data, not prose.

### Added

- **GFM table schema validator** (`4ad6c4f`) — declare `[[markdown_table.table_schemas]]` blocks in `proof.toml` with `required_columns`, `required_row_keys`, `min_body_rows`, `allowed_values`. Every matching table is held to its schema. Checks: `table_missing_column`, `table_missing_row`, `table_min_rows`, `table_bad_value`. (Style guide rules S-09..S-14)
- **Link validation** (`4e76e7b`) — `link_columns` and `verify_link_targets = true` resolve every link cell to disk. Checks: `link_bare_text`, `link_broken_target`, `link_missing`, `md_table_missing_link`, `md_broken_link`.
- **Auto-fix engine** for links and box-column drift (`2af8a1d`, `3592d7a`, `3686bfd`) — three deterministic transforms: `link_directory` (bare text → markdown link with inferred path), `box_col_pm1` (column off by one — shift to nearest valid alignment), `nested_box_col` (inner box edges aligned to outer frame).
- **Pattern B detection + signal-loss quality guard** (`d94c7d9`) — Pattern B is the asymmetric-vtable diagram class. The signal-loss guard refuses to apply a fix that would remove non-whitespace content unless `--no-signal-check` is passed.
- **`proof draft` subcommand** (`26ee0c4`) — pre-populated fix plan with errors grouped by file/region. Auto-fixable groups carry `decision: auto`; ambiguous groups carry `decision: needs_review` for AI or human triage.
- **Rich context in `DraftFix`** (`38fbe76`) — every draft entry now includes border widths, column positions, and surrounding lines inline. The AI never has to re-read the source file to reason about a fix.
- **Heading + style checks** (`045099d`) — `md_h1_count`, `md_missing_section`, `md_duplicate_heading`, `md_heading_order`, `md_missing_pattern`, `md_file_length` (S-15..S-20). `ascii_barchart` validates horizontal bar chart geometry.
- **Tab expansion + wide-character detection** (`58b10d6`) — `char_wide` and `char_fullwidth` flag CJK ideographs, em-dashes in the wrong encoding, and presentation forms before they corrupt ASCII art column alignment silently.
- **`paths_exclude` for `section_schemas`** (`7b899a9`) — schemas can scope to `*.md` while excluding `00-OVERVIEW.md`. SC-04 pitfall added.
- **E2E test pipeline** (`b9cd0d1`) — full `check → rich → plan → fix → verify` loop runs in CI on every push.

### Fixed

- **GFM `parse_row` for escaped pipes and code spans** (`b23b150`) — single fix eliminated 817 false positives. The parser now correctly treats `` `|` `` and `\|` as literal pipes, not column separators.
- **`md_heading_format`** false positive on `C#` and `F#` language names (`8e1ccf9`).
- **Pattern C** (multi-row box headers) auto-fix (`ca97d89`) — fixture coverage expanded; invariants I-14/I-15 added.

### What it enables

Schema validation moves the style contract out of CLAUDE.md prose ("every guide must have a Decision Cheat Sheet") and into machine-checkable `proof.toml` blocks. The library's editorial conventions become CI failures, not honor-system pleas. Combined with the auto-fix engine, a single command can repair an entire 2,000-file library in one supervised pass — and *report* what it changed.

---

## [0.2.0] — 2026-04-25 — *the fix release*

v0.1 told you what was wrong. v0.2 fixed it. The conceptual leap: detection is mechanical (Rust), fixing is mechanical too — but the *judgment* between them (which direction to shift a misaligned column, which border is the authority) is delegated to AI working off structured context.

### Added

- **`proof check --format rich`** — third output format alongside `text` and `json`. Each diagnostic carries the surrounding code block, expected vs. actual column widths, and adjacent lines. Designed as input for AI fix planners.
- **`proof fix --plan plan.json`** — applies a structured fix plan to the working tree. Flags: `--dry-run` (show diff without writing), `--min-confidence high|medium|low` (gate by AI confidence level), `--no-verify` (skip post-fix re-check), `--no-signal-check` (allow fixes that remove non-whitespace).
- **Fix plan schema** — JSON with `fixes[]`: each entry is `{id, file, confidence, description, reasoning, edit: {line, old_string, new_string}}`. Plans are human-readable; review before applying.
- **Bottom-up application order** — fixes are applied highest-line-number first so earlier line numbers stay valid after later edits. If `old_string` no longer matches the current file (drift between plan generation and fix application), the fix is skipped and logged. No silent corruption.
- **Invariants I-11/I-12/I-13** — formal properties of fix application (idempotence on clean files, no signal loss, position-stable on partial application).
- **`design/SPEC.md` v0.2** — full pipeline design doc.
- **Public README, CI workflow, Cargo metadata** — first release with a public face. CI runs `test + clippy + fmt + E2E smoke` on every push.
- **`fix-guide` skill** — `.claude/skills/fix-guide/skill.md` for AI agents driving the pipeline.

### What it enables

Bulk repair with a safety net. The pre-v0.2 workflow was "run check, read 1,500 errors, fix them by hand." Post-v0.2: `check --format rich` → AI writes a plan → review the plan → `fix --min-confidence high` applies the safe ones, leaves the rest for a second AI pass at lower confidence. The MAXIM library went from "manual repair impractical" to "fixable in one supervised afternoon."

---

## [0.1.0] — 2026-04-25 — *the foundation*

The seed. A fast, schema-driven Rust linter that parsed every code block in a markdown file as potential ASCII art and reported geometric defects with file:line:col precision.

### Added

- **`proof check`** (default subcommand) — lint files and report diagnostics.
- **ASCII box validation** — `ascii_box_width`, `ascii_box_col`, `ascii_cell_padding`, `ascii_arrow_gap`, `ascii_connector_drift`. Borders that don't add up, columns that drift, missing whitespace inside cells, broken arrow bodies, vertical connectors that wander.
- **Schema-driven, cascading `glint.toml`** — root config sets library-wide defaults; per-directory configs inherit and extend (lists additive, scalars use nearest). `paths_exclude` for selectively skipping files. Effective config inspection via `proof config <path>`.
- **Three output formats** — `text` (human, colored), `json` (machine, compact for editors/CI), with `rich` planned for v0.2.
- **Parallel file processing** via `rayon` — 2,000-file library completes in under 5 seconds. Per-directory config resolution cached.
- **Six initial diagnostic codes** + the foundational error/warning/info severity system.
- **68 unit + integration tests**, fixtures for every check class (perfect box, width mismatch, col misalignment, cell padding, arrow gap, complex diagram).
- **`design/SPEC.md`, `design/INVARIANTS.md`, `design/STYLE-GUIDE.md`** — designed-first, then implemented. Invariants I-01..I-10 specify what a "valid" ASCII box is at the parser level, independent of any specific check.

### What it enables

Catches the silent class of ASCII art errors that render correctly in a monospace editor but corrupt in MkDocs, GitHub web view, or any rendering pipeline that disagrees with the author's font metrics about character widths. The first time anyone could ask "is every box in this 2,000-file library geometrically sound?" and get a precise, machine-readable answer in seconds.

---

## Naming history

| Period | Binary | Library | Config |
|--------|--------|---------|--------|
| v0.1 — v0.3 | `glint` | `glint_lib` | `glint.toml` |
| v0.4+ | `proof` | `proof_lib` | `proof.toml` |

The rename reflects the scope expansion: `glint` (v0.1) lints; `proof` (v0.4) certifies.

---

## Versioning policy

Conceptual milestones are tracked here as `0.1` through `0.5`. The Cargo crate version is now aligned at `0.5.0` following the v0.5 release. Future versions will continue to align semver with milestone numbers.
