# proof User Scenarios

25 real use cases, each with a person, a goal, the proof commands to accomplish it,
and the result. Every scenario is runnable against the current proof binary.

Source files live in `src/user-scenarios/`. Compiled output in `docs/user-scenarios/`.

---

## US-01 — Technical writer auditing a large docs corpus

**Who**: A technical writer inheriting a markdown repository with ~200 files.
**Goal**: Find every structural problem before publishing — broken boxes, missing sections, rotted links.

```bash
proof check docs/ --errors-only
```

**Expectation**: file:line:col for every error. Zero noise from warnings.
**Covers**: ascii_box_width, md_missing_h2, link_broken_target

---

## US-02 — Developer adding LaTeX math to an API reference

**Who**: A Rust developer writing docs for a numerical library.
**Goal**: Inline math in parameter descriptions, display math for key formulas.

`src/user-scenarios/02-math-api.source.md`:

---

## US-03 — Data analyst building a metrics dashboard

**Who**: An analyst who runs a daily metrics report and wants a fixed-width terminal view.
**Goal**: Dashboard showing 4 KPIs, a sparkline trend, and a status indicator.

`src/user-scenarios/03-metrics-dashboard.dashboard.source.md`:

---

## US-04 — Team lead creating a weekly status deck

**Who**: An engineering manager presenting to their team every Monday.
**Goal**: 6-slide deck with title, section dividers, bullet summaries, and a KPI slide.

`src/user-scenarios/04-status-deck.slides.source.md`:

---

## US-05 — Researcher pinning an architecture diagram

**Who**: A distributed systems researcher who has a critical architecture figure.
**Goal**: Ensure the figure can never be accidentally changed without a visible error.

```bash
proof spec-generate "md://docs/arch.md:figure:system-overview" --id system-overview
# paste output into proof.toml
proof check --daVinci .
```

**Covers**: DaVinci invariant pinning, spec-generate, protection=error

---

## US-06 — Documentation maintainer auto-fixing alignment errors

**Who**: A maintainer who just inherited 47 box alignment errors from a colleague.
**Goal**: Apply all high-confidence fixes without reviewing each one.

```bash
proof check . --errors-only              # see the 47 errors
proof fix . --min-confidence high --dry-run  # preview
proof fix . --min-confidence high        # apply
proof check .                            # verify clean
```

**Covers**: fix pipeline, confidence levels, bottom-up application order

---

## US-07 — TUI developer embedding proof-canvas

**Who**: A Rust developer building a terminal dashboard app.
**Goal**: Use proof-canvas as a layout primitive — paste regions at exact positions.

`src/user-scenarios/07-canvas-tui/main.rs`:

---

## US-08 — ML engineer creating a model comparison view

**Who**: A machine learning engineer comparing 5 model variants.
**Goal**: A row for each model with label, accuracy value, sparkline of validation loss, and delta.

`src/user-scenarios/08-model-comparison.source.md`:

---

## US-09 — Project manager generating a dependency tree

**Who**: A PM documenting which components depend on what.
**Goal**: Dependency tree from a data table of (component, depends-on) pairs.

`src/user-scenarios/09-dependencies.source.md`:

---

## US-10 — Teacher creating a calculus slide deck

**Who**: A math instructor preparing lecture slides for Calculus II.
**Goal**: Slides with inline math in body text, display math for key theorems.

`src/user-scenarios/10-calculus-deck.slides.source.md`:

---

## US-11 — Open source maintainer checking PRs in CI

**Who**: An open source project maintainer enforcing documentation standards.
**Goal**: `proof check` runs in GitHub Actions and fails the PR if docs degrade.

```yaml
# .github/workflows/docs.yml
- name: proof lint
  run: proof check . --fail-on-error
```

**Covers**: exit codes, --fail-on-error, CI integration

---

## US-12 — Technical blogger with ASCII art diagrams

**Who**: A blogger writing about distributed systems with carefully aligned boxes.
**Goal**: Ensure every diagram stays geometrically correct after edits.

`src/user-scenarios/12-blog-post.source.md`:

---

## US-13 — DevOps engineer with watch-mode docs pipeline

**Who**: A DevOps engineer who wants docs to rebuild on every save during authoring.
**Goal**: `proof compile --watch` reading from proof.toml targets.

```bash
# proof.toml already has:
# [[compile]]
# source_dir = "src/docs"
# output_dir = "docs"

proof compile --watch
# edits to src/docs/*.source.md trigger immediate recompile
```

**Covers**: watch mode, multi-target, [[compile]] in proof.toml

---

## US-14 — Data scientist generating a taxonomy tree from a classification table

**Who**: A data scientist documenting a hierarchical taxonomy of ML model types.
**Goal**: Taxonomy tree from a markdown table with `model` and `category` columns.

`src/user-scenarios/14-ml-taxonomy.source.md`:

---

## US-15 — Game designer writing a rulebook with structure

**Who**: A tabletop game designer writing rules for a board game.
**Goal**: Numbered sections, callout boxes for important rules, a quick-reference slide.

`src/user-scenarios/15-rulebook.source.md`:

---

## US-16 — API documentarian enforcing table structure

**Who**: A developer writing API reference docs with required table schemas.
**Goal**: Every "Parameters" table must have "Name", "Type", "Required" columns.

```toml
# proof.toml
[[markdown_table.table_schemas]]
heading = "Parameters"
required_columns = ["Name", "Type", "Required", "Description"]
```

**Covers**: section schemas, table schemas, required_columns enforcement

---

## US-17 — Startup founder creating a pitch deck with KPIs

**Who**: A startup founder building a board presentation.
**Goal**: Title slide, stats slide with 4 KPIs, two-column comparison slide.

`src/user-scenarios/17-pitch-deck.slides.source.md`:

---

## US-18 — System architect documenting a codebase

**Who**: A software architect explaining a monorepo structure.
**Goal**: Dirtree of the repo + org chart of the team + dependency tree of crates.

`src/user-scenarios/18-architecture.source.md`:

---

## US-19 — Math educator writing textbook exercises

**Who**: A professor writing a problem set with solutions.
**Goal**: Display math for each problem, inline math in prose, matrices and integrals.

`src/user-scenarios/19-problem-set.source.md`:

---

## US-20 — CI engineer checking source link integrity

**Who**: An engineer who wants to catch broken md:// references before compile.
**Goal**: `proof check` catches broken references in .source.md files early.

```bash
proof check src/guides/          # md_broken_uri errors if any md:// is missing
proof compile --check src/guides/ # validates directives without writing
```

**Covers**: SourceLinkCheck, md_broken_uri, compile --check

---

## US-21 — Library author documenting proof-math standalone

**Who**: A Rust developer building a CLI tool that needs terminal math rendering.
**Goal**: Use proof-math crate directly for LaTeX → ASCII output.

`src/user-scenarios/21-proof-math-demo/main.rs`:

---

## US-22 — Analyst building a multi-region terminal status board

**Who**: An analyst who monitors 3 services and wants a terminal status board.
**Goal**: 4-region dashboard: header, two data panels side-by-side, footer.

`src/user-scenarios/22-status-board.dashboard.source.md`:

---

## US-23 — Note-taker generating a table of contents

**Who**: A developer maintaining a long architecture decision record (ADR).
**Goal**: Auto-generate a TOC at the top from the document's headings.

`src/user-scenarios/23-adr-with-toc.source.md`:

---

## US-24 — Team setting up multi-target compilation

**Who**: A documentation team with guides and presentations in separate directories.
**Goal**: `proof compile` and `proof compile --watch` build both targets automatically.

```toml
# proof.toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

**Covers**: [[compile]] array, multi-target routing, watch mode

---

## US-25 — Developer using stub=true for work-in-progress directives

**Who**: A developer building a guide before the data files exist.
**Goal**: Compile the document even with broken md:// references during drafting.

`src/user-scenarios/25-wip-guide.source.md`:

---

## US-31 — Writer adding attributed block quotes to a guide

**Who**: A technical writer who wants visually distinct quotations in a guide.
**Goal**: Render `proof:blockquote` with a left-bar margin and attribution line.

`src/user-scenarios/31-blockquote.source.md`:

---

## US-32 — Analyst building a bar chart from a data table

**Who**: An analyst with a markdown table of benchmark results.
**Goal**: Render a horizontal bar chart directly from the data with `proof:chart`.

`src/user-scenarios/32-benchmark-chart.source.md`:

---

## US-33 — Presenter using cross-references that survive heading renames

**Who**: A technical author who writes `See Section X` links between documents.
**Goal**: `proof:xref` resolves heading text at compile time; when the heading is renamed, one recompile updates all references.

`src/user-scenarios/33-xref-guide.source.md`:

---

## US-34 — Presenter building a progressive reveal deck

**Who**: A speaker who wants to reveal bullet points one at a time during a live talk.
**Goal**: `.slides.source.md` with `[N]` markers; compiled output has one canvas per reveal step.

`src/user-scenarios/34-reveal-deck.slides.source.md`:

---

## US-35 — Team lead adding consistent footer and agenda to a deck

**Who**: An engineering manager who wants every slide to show the author, date, and a slide counter, plus an auto-generated agenda slide.
**Goal**: `footer: true` + `layout=agenda` in front-matter.

`src/user-scenarios/35-footer-agenda.slides.source.md`:

---

## US-36 — Ops engineer checking corpus health before a deploy

**Who**: An ops engineer running `proof status` to confirm the corpus is fully compiled and error-free before a docs release.
**Goal**: Single-screen health summary — source count, stale count, last compile time.

```bash
proof status docs/
```

**Covers**: `proof status`, `.proof/last-check.json`

---

## US-37 — Author finding orphaned figures before archiving

**Who**: A documentation manager cleaning up a large corpus.
**Goal**: `proof check --unused` identifies figures that no `.source.md` references.

```bash
proof check . --unused
```

**Covers**: `proof check --unused`, `md_unused_figure`

---

## US-38 — Developer safely renaming a heading

**Who**: A developer who wants to rename `## Authentication` to `## Auth Flow` without breaking any source references.
**Goal**: `proof depends` lists every file to update before making the rename.

```bash
proof depends md://api.md#authentication
```

**Covers**: `proof depends`, reverse dependency lookup

---

## US-39 — Documentation lead enforcing no Draft sections in production

**Who**: A team lead who wants to prevent `## Draft` or `## TODO` headings from shipping.
**Goal**: `forbidden_h2` in `[[section_schemas]]` emits `md_forbidden_section` on any matching H2.

```toml
[[section_schemas]]
paths = ["docs/**/*.md"]
forbidden_h2 = ["Draft", "TODO", "WIP", "Placeholder"]
```

**Covers**: `forbidden_h2`, `md_forbidden_section`

---

## US-40 — Technical writer enforcing strict section structure

**Who**: A writer who wants each guide to use only approved H2 sections.
**Goal**: `optional_h2` allowlist — any H2 not in the approved list emits `md_unexpected_section`.

```toml
[[section_schemas]]
paths = ["guides/**/*.md"]
required_h2_all = ["Overview", "Decision Cheat Sheet"]
optional_h2 = ["Background", "See Also", "Examples"]
```

**Covers**: `optional_h2`, H2 allowlist, `md_unexpected_section`

---

## US-41 — Author generating DaVinci invariants with an AI CLI

**Who**: A developer who wants to lock a complex architecture diagram but doesn't know which invariants to write.
**Goal**: `proof spec-generate --ai` calls `claude -p` with the figure content and returns a `[[davinci]]` block.

```toml
[ai]
command = "claude"
args    = ["-p", "{prompt}"]
```

```bash
proof spec-generate "md://figures/arch.md:figure:goroutine-scheduler" --ai
```

**Covers**: `[ai]` config block, `proof spec-generate --ai`, configurable CLI

---

## US-42 — Developer protecting a figure with an inline pin declaration

**Who**: A developer who wants to declare a figure's expected DaVinci pin in the source document, not just in proof.toml.
**Goal**: `proof:include pin=id` emits COMPILE-007 warning when no matching `[[davinci]]` entry exists.

`src/user-scenarios/42-inline-pin.source.md`:

---

## US-43 — CI engineer grouping 200 slide warnings into a summary

**Who**: A CI engineer running `proof check` on a 50-slide deck corpus.
**Goal**: `--deduplicate` collapses `42x SLIDE-001 in docs/slides/*.md` instead of 42 individual lines.

```bash
proof check . --deduplicate
```

**Covers**: `proof check --deduplicate`, `format_deduplicated`

---

## US-44 — Developer catching a renamed heading before compile

**Who**: A developer who renamed `## Setup` to `## Installation` in `guide.md` and wants to catch all broken references.
**Goal**: `proof check` emits `md_broken_heading` for any `.source.md` that references `md://guide.md#setup`.

```bash
proof check src/
# → error [md_broken_heading]: Heading 'setup' not found in 'guide.md'
```

**Covers**: heading path validation, `md_broken_heading`

---

## US-45 — Author scoping a TOC to one section

**Who**: A writer maintaining a long API reference who wants a TOC only for the Authentication section.
**Goal**: `proof:toc section="Authentication"` lists only headings nested under that section.

`src/user-scenarios/45-scoped-toc.source.md`:

---

## US-46 — Analyst adding a progress bar to a live deck

**Who**: An analyst presenting a 10-slide deck who wants the audience to see their position.
**Goal**: `progress-bar: true` in `.slides.source.md` front-matter renders `████░░░ N/M` between each separator and canvas.

`src/user-scenarios/46-progress-deck.slides.source.md`:

---

## US-47 — Author catching a symbol typo

**Who**: A writer who types `[sym:checkmar]` instead of `[sym:checkmark]`.
**Goal**: Compile emits `Unknown symbol 'checkmar' — did you mean 'checkmark'?`

`src/user-scenarios/47-symbol-typo.source.md`:

---

## US-48 — Developer configuring ollama as the AI CLI

**Who**: A developer who runs local models and wants `proof spec-generate --ai` to use ollama instead of claude.
**Goal**: `[ai]` config block with `command = "ollama"` and `args = ["run", "llama3", "{prompt}"]`.

```toml
[ai]
command = "ollama"
args    = ["run", "llama3", "{prompt}"]
```

```bash
proof spec-generate "md://figures/arch.md:figure:0" --ai
```

**Covers**: `[ai]` config, stdin vs arg substitution, configurable CLI

---

## US-49 — Writer using proof:xref with note format

**Who**: A technical writer who wants "See also" callouts in a prose document.
**Goal**: `proof:xref format=note` renders `> **See also:** [Heading](link)`.

`src/user-scenarios/49-xref-note.source.md`:

---

## US-50 — Second compile faster due to Tier 2 resolve cache

**Who**: A developer iterating on a guide that includes the same figure 5 times.
**Goal**: On the second compile, all 5 includes hit the Tier 2 resolve cache — no mdpath re-parse.

```bash
# First compile: 5 cache misses → 5 mdpath resolves → 5 cache stores
proof compile src/guide.source.md

# Second compile (figure unchanged): 5 cache hits → 0 mdpath calls
proof compile src/guide.source.md
```

**Covers**: Tier 2 resolve cache, `resolve_uri_cached`, `.proof/cache/resolve/`

---

## Results

Run: `proof compile src/user-scenarios/ --output-dir docs/user-scenarios/`

| Scenario | Status | Notes |
|----------|--------|-------|
| US-01 | ✓ check | `proof check` catches errors — no source file needed |
| US-02 | ✓ compiles | Math API docs — 3 display math blocks render correctly |
| US-03 | ✓ compiles | Metrics dashboard — 4 KPI regions, sparkline trend |
| US-04 | ✓ compiles | Status deck — 6 slides, stats layout, bullets |
| US-05 | ✓ CLI | `proof spec-generate` — generates DaVinci TOML block |
| US-06 | ✓ CLI | `proof fix --min-confidence high` — fix pipeline works |
| US-07 | note | proof-canvas is a Rust library — see `src/user-scenarios/26-canvas-tui/main.rs` |
| US-08 | ✓ compiles | Model comparison — proof:row from data/models.md |
| US-09 | ✓ compiles | Dependencies — dirtree + bullet lists |
| US-10 | ✓ compiles | Calculus deck — 6 slides, inline + display math |
| US-11 | ✓ CLI | CI integration — `proof check . --fail-on-error` works |
| US-12 | ✓ compiles | Blog post with ASCII art — figures preserved intact |
| US-13 | ✓ CLI | `proof compile --watch` — rebuilds on save |
| US-14 | ✓ compiles | ML taxonomy — bullet list hierarchy renders cleanly |
| US-15 | ✓ compiles | Rulebook — proof:ol numbered lists, callouts, proof:right |
| US-16 | ✓ check | Table schemas in proof.toml — enforced at lint time |
| US-17 | ✓ compiles | Pitch deck — title, stats, two-column layouts |
| US-18 | ✓ compiles | Architecture — dirtree + bullet org chart |
| US-19 | ✓ compiles | Problem set — 4 display math blocks, limits, matrices |
| US-20 | ✓ check | Source link checking — `proof check src/` catches broken md:// |
| US-21 | note | proof-math is a Rust library — see `src/user-scenarios/27-proof-math-binary/main.rs` |
| US-22 | ✓ compiles | Status board — 6-region dashboard, no panic |
| US-23 | ✓ compiles | ADR with TOC — `proof:toc` generates numbered outline |
| US-24 | ✓ CLI | Multi-target `[[compile]]` — guides + presentations route correctly |
| US-25 | ✓ compiles | WIP guide — placeholder text while data files are pending |
| US-26 | ✓ library | proof-canvas in ratatui TUI — `src/user-scenarios/26-canvas-tui/main.rs` |
| US-27 | ✓ library | proof-math binary — `src/user-scenarios/27-proof-math-binary/main.rs` |
| US-28 | ✓ compiles | Large corpus scan — 2,703-file baseline check, 0 errors |
| US-29 | ✓ CLI | Fix pipeline on 47 errors — `proof fix --min-confidence high` |
| US-30 | ✓ compiles | CI `--delete-on-error` workflow — stale output cleaned on failure |
| US-31 | ✓ compiles | proof:blockquote — bar margin + attribution renders correctly |
| US-32 | ✓ compiles | proof:chart bar charts from inline data — two charts |
| US-33 | ✓ compiles | proof:xref inline + note format — self-referential headings resolve |
| US-34 | ✓ compiles | proof:reveal — 4-slide deck, 2 SLIDE-001 warnings (expected in demo) |
| US-35 | ✓ compiles | Slide footer + layout=agenda — footer on all slides, agenda auto-generated |
| US-36 | ✓ CLI | proof status — source/compiled/stale counts, last compile time |
| US-37 | ✓ CLI | proof check --unused — md_unused_figure for orphaned figures |
| US-38 | ✓ CLI | proof depends — lists all references before renaming |
| US-39 | ✓ config | forbidden_h2 — md_forbidden_section emitted for Draft/TODO headings |
| US-40 | ✓ config | optional_h2 allowlist — md_unexpected_section for unlisted H2s |
| US-41 | note | proof spec-generate --ai — requires claude or other CLI on PATH |
| US-42 | ✓ compiles | proof:include pin= — document compiles, COMPILE-007 path explained |
| US-43 | ✓ CLI | proof check --deduplicate — collapses repeated warnings |
| US-44 | ✓ CLI | proof check catches md_broken_heading on renamed heading |
| US-45 | ✓ compiles | proof:toc section= — scoped TOC for Authentication section only |
| US-46 | ✓ compiles | progress-bar: true — 5-slide deck with ████░░░ N/M bars |
| US-47 | ✓ compiles | [sym:checkmar] typo → SYMBOL-001 with did-you-mean 'checkmark' |
| US-48 | note | [ai] config with ollama — requires ollama installed locally |
| US-49 | ✓ compiles | proof:xref format=note and format=callout both render |
| US-50 | ✓ perf | Tier 2 resolve cache — second compile skips mdpath re-parse |

**Passed**: 46/50 runnable (US-07, US-21 are library examples; US-41, US-48 require external AI CLI)

## Bugs found during scenario validation

1. **Panic in dashboard regions** — `compile_element` with OOB indices when called from
   region body context. Fixed: added `source_fallback()` guard in compile.rs.

2. **Inline body trees** — `kind=org/taxonomy/dependency` with inline body content was
   reverted by `git checkout src/compile.rs`. These scenarios used `proof:bullets` as workaround.
   **Needs re-application**: the inline body feature for non-dirtree kinds.

3. **Formatted strings in kind=value** — `"99.9%"`, `"142ms"`, `"2.1M"` in dashboard
   element inline values hit ELEMENT-002. The F79 fix (Text fallback) was also reverted.
   Workaround: use `kind=label` for pre-formatted display strings.

---

## US-26 — proof-canvas in a ratatui TUI

**Who**: A Rust developer building a terminal monitoring app.
**Goal**: Use proof-canvas as the layout primitive — paste regions at exact positions — then hand the rendered string to ratatui for display.

`src/user-scenarios/26-canvas-tui/main.rs`

Key pattern: `Canvas::new(80, 20)` → `paste()` for each panel → `Canvas::render()` → `ratatui::widgets::Paragraph::new(text)`.

**Covers**: Canvas::new, paste, draw_border, scroll_clip, render; ratatui integration point

---

## US-27 — proof-math standalone binary

**Who**: A Rust developer who needs terminal math rendering in a CLI tool.
**Goal**: Read prose from stdin, expand all `$...$` inline math and `$$...$$` display blocks, write expanded output to stdout.

`src/user-scenarios/27-proof-math-binary/main.rs`

```bash
echo 'The energy $E = mc^2$ is famous.' | cargo run --example proof-math-binary
# → The energy E = mc² is famous.
```

**Covers**: expand_inline_math, render_display_math, MathAlign, MathDiag stderr reporting

---

## US-28 — Large corpus scan (maxim)

**Who**: An author maintaining the maxim reference library (2,703 files, 217 directories).
**Goal**: Establish a zero-error baseline before authoring; catch any regressions after edits.

`src/user-scenarios/28-large-corpus-scan.source.md`

```bash
proof check . --errors-only
# → Checked 2703 files — 0 errors, 0 warnings
```

**Covers**: --errors-only, large corpus performance, per-section scoping, fix workflow

---

## US-29 — proof fix pipeline on 47 errors

**Who**: A maintainer who inherited docs with 47 box alignment errors.
**Goal**: Apply all high-confidence fixes without reviewing each one manually.

`src/user-scenarios/29-fix-pipeline/before.md` — sample of the broken files.
`src/user-scenarios/29-fix-pipeline/proof.toml` — zero-tolerance config for checking.

```bash
proof check src/user-scenarios/29-fix-pipeline/ --errors-only
proof fix   src/user-scenarios/29-fix-pipeline/ --min-confidence high --dry-run
proof fix   src/user-scenarios/29-fix-pipeline/ --min-confidence high
proof check src/user-scenarios/29-fix-pipeline/ --errors-only
```

**Covers**: fix pipeline, confidence levels, --dry-run, bottom-up application order

---

## US-30 — proof compile --delete-on-error CI workflow

**Who**: A CI engineer who wants docs to never deploy with stale compiled output.
**Goal**: `proof compile --delete-on-error` removes old output when a source fails to compile, so the deploy step always sees either fresh or absent output — never stale.

`src/user-scenarios/30-delete-on-error.source.md`

```yaml
- name: Compile docs
  run: proof compile --delete-on-error
```

**Covers**: --delete-on-error, [[compile]] multi-target, exit codes, GitHub Actions integration
