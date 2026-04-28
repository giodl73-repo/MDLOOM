# proof

**Markdown quality assurance and compilation for terminal-first documentation.**

proof does two things well. It **checks** markdown — catching ASCII art geometry
errors, broken links, missing required sections, and misaligned table columns
with file:line:col precision. And it **compiles** source documents — resolving
`proof:` directives into rendered LaTeX math, ASCII presentations, data
dashboards, tree diagrams, and more.

The mental model: `.source.md` is source code. `.md` is the compiled artifact.
proof is the compiler.

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
proof check . --fail-on-error      # CI mode: non-zero exit on errors
```

proof validates:

- **ASCII art** — box widths, column separator alignment, connector continuity
- **Markdown structure** — required headings, heading order, file length
- **Tables** — column count, required columns, required row keys, allowed values
- **Links** — targets exist on disk
- **Source documents** — broken `md://` references caught before compile time

Every diagnostic includes file, line, column, code, and message:

```
languages/08-TYPESCRIPT.md:34:1  error    ascii_box_width   bottom border 64, top 63
docs/api.md:112:1                warning  md_missing_h2     required ## "Summary" absent
src/guides/math.source.md:45:1   error    md_broken_uri     md:// URI references missing file
```

---

## Compiling

Source files (`.source.md`) contain `proof:` directives. Compile resolves every
directive and writes the output `.md` file.

```bash
proof compile src/guides/          # compile directory → docs/guides/ (from proof.toml)
proof compile --watch              # watch all [[compile]] targets for changes
proof compile file.source.md -o out.md   # single file, explicit output
```

Configure targets in `proof.toml`:

```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

---

## Directives

### LaTeX math — `$...$` and `proof:math`

Inline math expands anywhere in prose, bullets, and slide titles:

```
$\alpha + \beta = \gamma$  →  α + β = γ
$x^2 + y^2 = z^2$          →  x² + y² = z²
$\forall \epsilon > 0$      →  ∀ ε > 0
```

Display blocks render stacked fractions, integrals, matrices:

````markdown
```proof:math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```
````

No LaTeX installation required. Pure Rust renderer covering 60+ symbols,
superscripts, subscripts, √, primes, stacked fractions, integrals with limits,
matrices, and cases environments.

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
- Broken md:// references
- Missing required sections
- Table schema violations
```
````

Six layouts: `title` · `title-content` · `two-column` · `section` · `stats` · `blank`

Body directives: `proof:bullets` · `proof:ol` · `proof:callout` · `proof:divider`
· `proof:quote` · `proof:centered` · `proof:right` · `proof:stat` · `proof:notes`

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
    header: { x: 0, y: 0, width: 80, height: 3 }
    metrics: { x: 0, y: 3, width: 80, height: 14 }
    footer:  { x: 0, y: 17, width: 80, height: 3 }
---
```

Each region is a mini-document supporting any `proof:` directive.

---

### Table of contents — `proof:toc`

````markdown
```proof:toc max-depth=3 style=list
```
````

Auto-generates from headings in the current file or any `source=md://` file.

---

### Symbols — `[sym:name]` and `proof:symbol`

Named Unicode glyphs that expand in prose, bullets, and slide titles:

```
[sym:checkmark] done  →  ✓ done
[sym:star][sym:star][sym:star][sym:star-empty][sym:star-empty]  →  ★★★☆☆
[sym:warning] check this  →  ⚠ check this
```

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

---

## DaVinci figure pinning

Lock a figure's structural invariants. Compile aborts if a future edit violates them:

```bash
proof spec-generate "md://figures/arch.md:figure:goroutine-scheduler"
# → paste suggested [[davinci]] block into proof.toml

proof pin "md://figures/arch.md:figure:goroutine-scheduler"
proof check --daVinci .
```

---

## Fix pipeline

```bash
proof fix . --dry-run                  # preview what changes
proof fix . --min-confidence high      # apply safe auto-fixes
proof fix . --min-confidence medium    # apply heuristic fixes too
```

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
required_h2_all = ["Summary", "Examples"]

[[section_schemas]]
paths = ["docs/guides/*.md"]
required_h2_all = ["Usage", "Examples"]
paths_exclude = ["00-OVERVIEW.md"]
```

---

## Workspace

The proof repo contains three crates:

| Crate | Purpose |
|-------|---------|
| `proof` | CLI, linting, compile pipeline |
| `proof-canvas` | Fixed-width ASCII char grid (usable in any TUI) |
| `proof-math` | LaTeX→terminal renderer (standalone library) |

`mdpath` lives in a sibling repo and handles `md://` URI parsing and resolution.

---

## Guides

Compiled guides live in `docs/guides/`. Rebuild with:

```bash
bash scripts/build-guides.sh           # compile all
bash scripts/build-guides.sh --check   # validate without writing
proof compile --watch                  # watch mode
```

| Guide | Content |
|-------|---------|
| [Getting started](docs/guides/00-getting-started.md) | Install, first check, first compile |
| [Math](docs/guides/01-math.md) | LaTeX rendering — all tiers |
| [Symbols](docs/guides/02-symbols.md) | Symbol library and shapes |
| [Elements](docs/guides/03-elements.md) | Data cells and row compositor |
| [Slides](docs/guides/04-slides.slides.md) | Presentation layouts |
| [Trees](docs/guides/05-trees.md) | Tree diagrams |
| [Dashboard](docs/guides/06-dashboard.md) | Canvas regions |
| [Compile](docs/guides/07-compile.md) | Full directive reference |
| [Lint](docs/guides/08-lint.md) | Check rules and proof.toml |
| [Crates](docs/guides/09-crates.md) | proof-canvas and proof-math standalone library APIs |

---

## License

MIT — see [LICENSE](LICENSE).
