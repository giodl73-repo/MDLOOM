# proof

**Write in markdown. Compile to anything.**

**Series:** [Tools & Infrastructure](https://github.com/giodl73-repo/giodl73-repo/blob/main/series/tools-infrastructure.md).

**Review roles:** This repo uses
[ROLES](https://github.com/giodl73-repo/ROLES), the `.roles` convention for
repository-local review panels.

proof is a full markdown compilation toolchain — LaTeX math, ASCII slide decks,
live dashboards, tree diagrams, sparkline charts, cross-referenced guides — all
authored in plain text, compiled to terminal-perfect output.

It powers a 2,700-file reference library (217 directories, 13 sections), a suite of
technical presentation decks, and a cross-linked documentation corpus. Every guide,
every chart, every diagram: authored in `.source.md`, compiled to `.md` by proof.

```
.source.md  →  proof compile  →  .md / .html
.md         →  proof backfill →  .source.md
```

Think of it as a static site generator for terminal-first content: markdown is
the primary artifact, HTML is the first human publish target, and **Pebbles**
(`pebble.v1`) are compact AI/context transfer artifacts. Richer
targets such as PPTX belong behind the same compile graph rather than a separate
workflow.

**What proof can render from a single `.source.md` file:**

```
$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$   →  ASCII math with fractions, integrals, Greek
proof:slide layout=title-content          →  80-column ASCII presentation canvas
proof:tree kind=org                       →  box-drawing tree diagrams  
proof:element kind=sparkline              →  ▂▃▅▇█ inline sparklines
proof:xref uri="md://api.md#auth"         →  *See: [Authentication](api.md#authentication)*
proof:toc section="API Reference"         →  scoped, auto-updating table of contents
[sym:checkmark] passed                    →  ✓ passed
```

proof also **checks** your markdown corpus — catching geometry errors in ASCII art,
broken `md://` heading references, missing required sections, and misaligned tables —
with file:line:col precision and did-you-mean suggestions.

---

## Install

proof and its URI library live in the same workspace. Clone and build:

```bash
git clone https://github.com/giodl73-repo/PROOF
git clone https://github.com/giodl73-repo/MDPATH   # sibling directory
cd PROOF
cargo build --release
```

Binary: `target/release/proof` (or `../../target/release/proof` from workspace root).

---

## Checking

```bash
proof check .                      # lint all markdown
proof check docs/ --errors-only    # errors only
proof check . --by-code            # group counts by diagnostic code
proof check . --deduplicate        # collapse repeated warnings into summary lines
```

proof validates:

- **ASCII art** — box widths, column separator alignment, connector continuity, flowchart geometry
- **Markdown structure** — H1 count, required headings, heading order, file length, H2 allowlists
- **Tables** — column count, required columns, required row keys, allowed values, separator dashes
- **Source tables** — inline pipe tables in `.source.md` files are flagged so durable row data can move to sidecar JSON/CSV or generated PROOF tables
- **Links** — prose `[text](path.md)` links exist on disk
- **Source documents** — broken `md://` references and missing heading paths caught before compile
- **DaVinci figures** — structural invariants verified on every `proof check --daVinci`

Every diagnostic includes file, line, column, code, and message. Did-you-mean
suggestions appear for common typos:

```
languages/08-TYPESCRIPT.md:34:1   error    ascii_box_width     bottom border 64, top 63
src/guides/math.source.md:45:1    error    md_broken_uri       Reference to 'fig.md' not found — did you mean 'figs.md'?
src/guides/api.source.md:18:1     error    md_broken_heading   Heading 'authenticaton' not found in 'api.md'
src/guides/math.source.md:22:1    warning  SYMBOL-001          Unknown symbol 'checkmar' — did you mean 'checkmark'?
src/slides/deck.source.md:9:1     warning  SLIDE-001           Slide has 5 bullets — reduce to 4 or fewer (30-second rule)
```

At corpus scale, `--deduplicate` collapses repeated warnings:

```
42x warning [SLIDE-001]: Slide has 5 bullets — reduce to 4 or fewer  in docs/slides/*.md
```

---

## Reverse dependency lookup

Find every source file that references a given `md://` URI — so you know what
breaks before renaming a heading or moving a figure:

```bash
proof depends md://api.md#authentication
proof depends md://figures/arch.md
```

---

## Compiling

Source files (`.source.md`) contain `proof:` directives. Compile resolves every
directive and writes the output `.md` file.

```bash
proof compile src/guides/          # compile directory → docs/guides/ (from proof.toml)
proof compile --watch              # watch all [[compile]] targets for changes
proof compile --progress           # show per-file progress at corpus scale
proof compile file.source.md -o out.md   # single file, explicit output
proof compile file.source.md --target html -o out.html
proof compile file.source.md --target pebble -o out.pebble.json
proof compile file.source.md --target json-report -o out.proof-report.json
proof compile src/guides/ --target site --output-dir site/
proof compile file.source.md --target pdf -o out.pdf
proof compile file.source.md --target docx -o out.docx
proof compile deck.slides.source.md --target pptx -o deck.pptx
```

Markdown is the default compile target. `--target html` resolves the same source
directives, strips source-only frontmatter, and writes a standalone HTML document
with common Markdown support for lists, tables, links, task lists, strikethrough,
and fenced code. Raw HTML is escaped by default. Successful compile runs also
write `.proof/artifacts.json`, a target-aware manifest of source files, output
paths, diagnostics, cache status, and resolved directive counts.

PROOF can use SLICE selectors over prepared artifact/report rows without moving
compile or render policy into SLICE; see
[`docs/specs/slice-selectors.md`](docs/specs/slice-selectors.md).

`--target pebble` writes a compact `pebble.v1` JSON document for agents,
retrieval, and transfer. Pebbles preserve stable section IDs, heading paths,
line numbers, resolved Markdown text, source path, and resolved dependency refs.
CROP can support the same schema for corpus slices, so a CROP view pack and a
PROOF compiled source can exchange small, provenance-bearing context chunks.

`--target json-report` writes `proof.publish.json_report.v1`: a stable
machine-readable compile bundle for CI, agents, and integrations. It includes
artifact summary, resolved Markdown, section summaries, source metadata,
dependency refs, diagnostics, and compile counts without replacing Pebble's
compact retrieval schema.

`--target site` compiles a source tree to static HTML pages and writes a
navigation `index.html` plus `proof-site.json` site manifest in the output
directory. It is a local static site artifact, not deployment, hosting, or search
ranking.

`--target pdf` renders the same resolved HTML publish output into a portable PDF
artifact. The first PDF backend is deterministic and dependency-free for CI; it
does not claim browser/print-engine pixel equivalence.

`--target docx` writes an editable Word-processing OOXML package from resolved
Markdown, including headings, paragraphs, native list numbering, tables, fenced
code text, links, and basic document metadata.

`--target pptx` writes an editable native PowerPoint OOXML deck from explicit
`.slides.source.md` inputs. It emits real text boxes, native bullets/numbering,
monospace code text, speaker notes, relationships, and manifest records; it does
not rasterize slides or embed HTML.

`.source.md` files may start with source-only frontmatter for corpus tagging.
The block is stripped from normal compiled output and surfaced by status/stats:

```yaml
---
tags: [ops, runbook]
ops: [lint, compile]
content_tags: [guide]
---
```

```bash
proof stats --by-tag src/guides/
proof check src/guides/ --tag publish --op lint
proof compile src/guides/ --tag publish --content-tag guide
proof status src/guides/
proof index --root docs/guides --output docs/INDEX.md
proof catalog --root docs/guides --output docs/CATALOG.md
```

Tag filters are opt-in and exact-match. Defaults remain inclusive; multiple
filters must all match the source frontmatter.

Configure targets in `proof.toml`:

```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

## Backfilling existing markdown

Projects that already have `.md` files can bootstrap proof sources without a
manual rewrite. The first pass is literal-first: preserve current markdown,
add provenance frontmatter, optionally compile the generated source, and report
round-trip fidelity.

```bash
proof backfill docs/ --output-source proof-source/ --literal-first --check-roundtrip
```

This creates `.source.md` candidates and `backfill-report.json`. The report
includes advisory block counts for prose, fences, markdown tables, ASCII table
candidates, chart-like blocks, diagram-like blocks, and ambiguous blocks.

To start a structured migration, add `--extract-tables`. Proof still preserves
the literal `.source.md` body, and writes high-confidence markdown pipe tables to
sidecar files like `proof-source/guide.tables.json`, recording each extraction in
the report.

See `docs/guides/14-backfill-migration.md` for a MAXIM-style staged migration
checklist.

---

## Directives

### LaTeX math — `$...$` and `proof:math`

Inline math expands anywhere in prose, bullets, and slide titles:

```
$\alpha + \beta = \gamma$  →  α + β = γ
$x^2 + y^2 = z^2$          →  x² + y² = z²
$\forall \epsilon > 0$      →  ∀ ε > 0
$\frac{n(n+1)}{2}$          →  n(n+1)/2
```

Display blocks render stacked fractions, integrals, matrices, cases:

````markdown
```proof:math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```
````

No LaTeX installation required. Pure Rust renderer — 60+ symbols, superscripts,
subscripts, √, primes, stacked fractions, integrals with limits, matrices, cases.

---

### ASCII presentations — `.slides.source.md`

````markdown
```proof:slide layout=title
title: "proof"
subtitle: "Markdown quality assurance"
```
---
```proof:slide layout=title-content
title: "What proof checks"
---
proof:bullets
- ASCII art geometry errors
[2] - Broken md:// references
[3] - Missing required sections
```
````

Six layouts: `title` · `title-content` · `two-column` · `section` · `stats` · `blank`

Body directives: `proof:bullets` · `proof:ol` (numbered list) · `proof:columns`
· `proof:callout` · `proof:divider` · `proof:quote` · `proof:centered` · `proof:right`
· `proof:stat` · `proof:notes`

**Progressive reveal**: bullets prefixed with `[N]` (N ≥ 2) assign that bullet to
reveal step N. Compile produces one canvas block per step — each page shows all
bullets with step ≤ current step (cumulative).

---

### Tree diagrams — `proof:tree`

````markdown
```proof:tree kind=org
root: proof workspace
- proof: CLI + compile pipeline
- proof-canvas: terminal char grid
- proof-math: LaTeX renderer
```
````

````markdown
```proof:tree kind=dirtree root=src max_depth=2
```
````

````markdown
```proof:tree kind=taxonomy source=md://src/data/features.md name=name parent=category
```
````

Kinds: `dirtree` · `org` · `taxonomy` · `dependency` · `outline`

---

### Data elements — `proof:element` and `proof:row`

Fixed-width data cells that compose into column-aligned dashboards:

````markdown
```proof:element kind=sparkline value="1,3,2,5,4,7,9" width=14
```
```proof:element kind=value value="99.9%" label="uptime" width=14
```
```proof:row source=md://src/data/metrics.md foreach=row separator=" │ "
proof:element kind=label field=name width=24
proof:element kind=badge field=status width=10
proof:element kind=sparkline field=trend width=14
```
````

Kinds: `value` · `delta` · `sparkline` · `mini-bar` · `label` · `badge`

---

### ASCII dashboards — `.dashboard.source.md`

Fixed-width canvas with named regions at exact x/y positions:

```yaml
---
dashboard:
  width: 80
  height: 20
  regions:
    header:  { x: 0, y: 0,  width: 80, height: 3  }
    metrics: { x: 0, y: 3,  width: 80, height: 14 }
    footer:  { x: 0, y: 17, width: 80, height: 3  }
---
```

Each region is a mini-document supporting any `proof:` directive.
DASHBOARD-006 warns if canvas width > 220 (standard terminal threshold).

---

### Table of contents — `proof:toc`

````markdown
```proof:toc max-depth=3 style=list
```
```proof:toc section="API Reference" max-depth=4 style=numbered
```
````

Auto-generates from headings in the current file or any `source=md://` file.
`section=` scopes the TOC to a subsection. Styles: `list` · `numbered` · `tree`

---

### Symbols — `[sym:name]` and `proof:symbol`

Named Unicode glyphs that expand in prose, bullets, and slide titles:

```
[sym:checkmark] done    →  ✓ done
[sym:star][sym:star][sym:star-empty]  →  ★★☆
[sym:warning] note      →  ⚠ note
[sym:arrow-right] next  →  → next
```

Built-in symbols: checkmark, x, warning, info, dot, diamond, star, arrow-*, triangle-*,
rule-thin, rule-double, and 30+ extended symbols. Custom symbols via `proof.toml`.

---

### Cross-references — `proof:xref`

Resolves a heading's text from another document at compile time:

````markdown
```proof:xref uri="md://api.md#authentication" format=note
```
````

Three formats:
- `inline` (default): `*See: [Authentication](api.md#authentication)*`
- `note`: `> **See also:** [Authentication](api.md#authentication)`
- `callout`: `→ [Authentication](api.md#authentication)`

Optional `label=` override. When the target heading is renamed, recompile
updates every `proof:xref` that pointed to it.

---

### Include and layout — `proof:include` and `proof:layout`

````markdown
```proof:include
md://figures/arch.md#:0
```

```proof:include pin=arch-diagram
md://figures/arch.md#:0
```

```proof:layout gap=4 labels="Before,After"
md://before.md#:0
md://after.md#:0
```
````

`pin=id` declares that this figure must be protected by a DaVinci invariant
pin with that ID. COMPILE-007 warns if no matching `[[davinci]]` entry exists,
prompting `proof pin <uri> --id <id>`.

---

## The md:// URI scheme

Every figure, table, and element in every markdown file has a stable named
address. proof uses `md://` URIs for cross-file references, figure pinning,
and error reporting:

```
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
md://src/data/metrics.md:table:0[row=Goroutine,col=Stack Size]
md://docs/math.md:math:pythagorean
```

URIs survive edits because they address content by name, not line number. The
resolver is the `mdpath` crate — see [mdpath](../mdpath/README.md).

`proof check` validates that `md://` URIs in source files point to files that
exist AND that the heading path (e.g., `#api-reference/authentication`) resolves
to real headings in the target document.

---

## DaVinci figure pinning

Lock a figure's structural invariants. Compile aborts if a future edit violates them:

```bash
proof spec-generate "md://figures/arch.md:figure:goroutine-scheduler"
# → paste suggested [[davinci]] block into proof.toml

proof pin "md://figures/arch.md:figure:goroutine-scheduler" --id goroutine-scheduler
proof check --daVinci .
```

---

## Fix pipeline

```bash
proof draft . -o draft-plan.json                    # generate a reviewable plan
proof fix --plan draft-plan.json --dry-run          # preview what changes
proof fix --plan draft-plan.json --min-confidence high
proof fix --plan draft-plan.json --min-confidence medium
proof --config proof.toml fix --plan draft-plan.json
```

`proof fix` verifies modified files with the same global `--config` path used by
other commands unless `--no-verify` is supplied. Each run writes
`.proof/last-fix.json` with applied/skipped counts, modified files, and
verification status.

---

## proof.toml

```toml
[files]
root = true

[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[ascii_box]
enabled = true
tolerance = 1

[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Summary", "Examples"]

[[section_schemas]]
paths = ["docs/guides/*.md"]
paths_exclude = ["00-OVERVIEW.md"]
required_h2_all = ["Usage", "Examples"]
optional_h2 = ["Background", "See Also"]   # closes the H2 allowlist — unexpected H2s warned

[[davinci]]
id = "goroutine-scheduler"
uri = "md://figures/arch.md:figure:goroutine-scheduler"
protection = "error"

  [[davinci.invariants]]
  check = "width"
  expected = 80
```

---

## Workspace

The proof repo contains three crates:

| Crate | Purpose |
|-------|---------|
| `proof` | CLI, linting, compile pipeline |
| `proof-canvas` | Fixed-width ASCII char grid (usable standalone in any TUI) |
| `proof-math` | LaTeX→terminal renderer (standalone library) |

`mdpath` lives in a sibling repo and handles `md://` URI parsing and resolution.

---

## Guides

Compiled guides live in `docs/guides/`. Rebuild with:

```bash
proof compile --watch
```

| Guide | Content |
|-------|---------|
| [Getting started](docs/guides/00-getting-started.md) | Install, first check, first compile |
| [Math](docs/guides/01-math.md) | LaTeX rendering — all tiers |
| [Symbols](docs/guides/02-symbols.md) | Symbol library and shapes |
| [Elements](docs/guides/03-elements.md) | Data cells and row compositor |
| [Slides](docs/guides/04-slides.slides.md) | Presentation layouts and reveal |
| [Trees](docs/guides/05-trees.md) | Tree diagrams |
| [Dashboard](docs/guides/06-dashboard.md) | Canvas regions |
| [Compile](docs/guides/07-compile.md) | Full directive reference |
| [Lint](docs/guides/08-lint.md) | Check rules and proof.toml |
| [Crates](docs/guides/09-crates.md) | proof-canvas and proof-math APIs |
| [CROP](docs/guides/12-crop.md) | Corpus-intelligence adapter |

---

## License

MIT — see [LICENSE](LICENSE).
