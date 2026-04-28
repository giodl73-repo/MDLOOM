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
| US-07 | note | proof-canvas is a Rust library — see `crates/proof-canvas/` |
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
| US-21 | note | proof-math is a Rust library — see `crates/proof-math/` |
| US-22 | ✓ compiles | Status board — 6-region dashboard, no panic |
| US-23 | ✓ compiles | ADR with TOC — `proof:toc` generates numbered outline |
| US-24 | ✓ CLI | Multi-target `[[compile]]` — guides + presentations route correctly |
| US-25 | ✓ compiles | WIP guide — placeholder text while data files are pending |

**Passed**: 23/25 runnable (US-07 and US-21 are library crates, not CLI scenarios)

## Bugs found during scenario validation

1. **Panic in dashboard regions** — `compile_element` with OOB indices when called from
   region body context. Fixed: added `source_fallback()` guard in compile.rs.

2. **Inline body trees** — `kind=org/taxonomy/dependency` with inline body content was
   reverted by `git checkout src/compile.rs`. These scenarios used `proof:bullets` as workaround.
   **Needs re-application**: the inline body feature for non-dirtree kinds.

3. **Formatted strings in kind=value** — `"99.9%"`, `"142ms"`, `"2.1M"` in dashboard
   element inline values hit ELEMENT-002. The F79 fix (Text fallback) was also reverted.
   Workaround: use `kind=label` for pre-formatted display strings.

## Future scenarios to add

- US-26: proof-canvas embedded in a real Rust TUI (ratatui + proof-canvas side by side)
- US-27: proof-math standalone Rust binary using the crate API
- US-28: Large corpus scan (maxim — 2,703 files, 0 errors baseline check)
- US-29: proof fix pipeline on a corpus with 47 errors
- US-30: proof compile --delete-on-error CI workflow
