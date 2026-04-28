# Getting Started with proof

proof is a markdown quality-assurance and compilation system for terminal-first
documentation. It does two things: it **lints** markdown (catches alignment
errors in ASCII art, broken links, missing sections) and it **compiles** source
documents (resolves `proof:` directives into rendered output). Think of it as a
type-checker for your documentation — catching structural errors before they reach
readers, and doing the mechanical rendering work for you.

The two modes are independent. You can use `proof check` on any existing markdown
repository with no setup beyond a `proof.toml`. Compilation requires `.source.md`
files with `proof:` directives, but you adopt it incrementally — one file at a time.

---

## What proof does

```proof:tree kind=org
root: proof CLI
- proof check: Lint markdown and ASCII art
  - AsciiBoxCheck: Box border alignment
  - AsciiFlowCheck: Flow diagram nodes
  - MarkdownCheck: Headings and links
  - MarkdownTableCheck: Column alignment
  - SourceLinkCheck: Broken md:// references in source files
- proof compile: Resolve directives
  - proof:math: LaTeX → Unicode/ASCII art
  - proof:symbol: Named glyphs
  - proof:element: Numeric cells (sparklines, bars, values)
  - proof:row: Data rows from tables
  - proof:tree: Tree diagrams from data
  - proof:slide: Presentation slides
  - proof:region: Dashboard canvas regions
- proof fix: Auto-patch lint errors
- proof pin: Register figure invariants (DaVinci protection)
```

---

## Install

proof and its URI library (`mdpath`) live in a Cargo workspace. Clone both into
sibling directories, then build from the workspace root:

```bash
git clone https://github.com/giodl73-repo/PROOF
git clone https://github.com/giodl73-repo/MDPATH
cd ..                              # go to the parent directory
cargo build                        # builds both crates together
```

The binary is at `C:/src/target/debug/proof` (or `release/proof` for production).
On Windows: the same paths with `.exe`.

---

## First scan

Run `proof check` on any directory to see what proof finds:

```bash
proof check .
```

For a new repository this typically surfaces ASCII art alignment errors, missing
required heading sections, and broken internal links. Each diagnostic includes the
file, line, column, severity, and a short explanation:

```
languages/08-TYPESCRIPT.md:34:1  error  ascii_box_width  bottom border 64 chars, top 63
docs/api.md:112:1                 warn   md_missing_h2    required ## "Summary" absent
```

Start with errors (structural failures) before addressing warnings (style issues).

---

## Configuration

proof reads `proof.toml` from the directory being checked, cascading up to the
nearest file with `root = true`. A minimal root config:

```toml
[files]
root = true

[ascii_box]
enabled = true

[markdown]
enabled = true
required_h2_all = ["Summary", "Examples"]

[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"
```

The `[[compile]]` section tells proof where to find source files and where to
write compiled output — so `proof compile` and `proof compile --watch` work
without any extra flags.

---

## The source → output pipeline

Source files (`.source.md`) contain `proof:` directives that get resolved into
rendered markdown. The mental model: source is code, compiled output is the
artifact. Never edit the compiled `.md` files directly — edit the `.source.md`
and recompile.

```proof:tree kind=dependency
root: docs/guides/05-trees.md
- src/guides/05-trees.source.md: proof:tree directives resolved
- src/data/features.md: taxonomy source table
- src/data/diagnostic-codes.md: second taxonomy source
```

Compile a single file:

```bash
proof compile src/guides/math.source.md
```

Compile a whole directory to a separate output location:

```bash
proof compile src/guides/ --output-dir docs/guides/
```

Watch for changes and recompile automatically on save:

```bash
proof compile --watch   # reads [[compile]] targets from proof.toml
```

---

## Feature coverage

```proof:tree kind=taxonomy source=md://src/data/features.md name=name parent=category
```

---

## Next steps

- [Math guide](math.md) — LaTeX rendering for formulas and symbols
- [Symbols guide](symbols.md) — named glyph library and shape renderer
- [Elements guide](elements.md) — sparklines, bars, values, and data rows
- [Slides guide](slides.md) — ASCII presentation layouts
- [Trees guide](trees.md) — org charts, taxonomies, and dirtrees
- [Compile guide](compile.md) — full directive reference
- [Lint guide](lint.md) — check rules and proof.toml options
