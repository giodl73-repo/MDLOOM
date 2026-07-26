# Getting Started with mdloom

mdloom is a markdown quality-assurance and compilation system for terminal-first
documentation. It does two things: it **lints** markdown (catches alignment
errors in ASCII art, broken links, missing sections) and it **compiles** source
documents (resolves `mdloom:` directives into rendered output). Think of it as a
type-checker for your documentation — catching structural errors before they reach
readers, and doing the mechanical rendering work for you.

The two modes are independent. You can use `mdloom check` on any existing markdown
repository with no setup beyond a `mdloom.toml`. Compilation requires `.source.md`
files with `mdloom:` directives, but you adopt it incrementally — one file at a time.

---

## What mdloom does

<!-- mdloom:compiled from="mdloom:tree kind=org" uri="" -->
```org
mdloom CLI
├── mdloom check: Lint markdown and ASCII art
├── AsciiBoxCheck: Box border alignment
├── AsciiFlowCheck: Flow diagram nodes
├── MarkdownCheck: Headings and links
├── MarkdownTableCheck: Column alignment
├── SourceLinkCheck: Broken md:// references in source files
├── mdloom compile: Resolve directives
├── mdloom:math: LaTeX → Unicode/ASCII art
├── mdloom:symbol: Named glyphs
├── mdloom:element: Numeric cells (sparklines, bars, values)
├── mdloom:row: Data rows from tables
├── mdloom:tree: Tree diagrams from data
├── mdloom:slide: Presentation slides
├── mdloom:region: Dashboard canvas regions
├── mdloom fix: Auto-patch lint errors
└── mdloom pin: Register figure invariants (DaVinci protection)
```
<!-- /mdloom:compiled -->

---

## Install

mdloom and its URI library (`mdpath`) live in a Cargo workspace. Clone both into
sibling directories, then build from the workspace root:

```bash
git clone https://github.com/giodl73-repo/MDLOOM
git clone https://github.com/giodl73-repo/MDPATH
cd ..                              # go to the parent directory
cargo build                        # builds both crates together
```

The binary is at `C:/src/target/debug/mdloom` (or `release/mdloom` for production).
On Windows: the same paths with `.exe`.

---

## First scan

Run `mdloom check` on any directory to see what mdloom finds:

```bash
mdloom check .
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

mdloom reads `mdloom.toml` from the directory being checked, cascading up to the
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

The `[[compile]]` section tells mdloom where to find source files and where to
write compiled output — so `mdloom compile` and `mdloom compile --watch` work
without any extra flags.

---

## The source → output pipeline

Source files (`.source.md`) contain `mdloom:` directives that get resolved into
rendered markdown. The mental model: source is code, compiled output is the
artifact. Never edit the compiled `.md` files directly — edit the `.source.md`
and recompile.

<!-- mdloom:compiled from="mdloom:tree kind=dependency" uri="" -->
```dependency
docs/guides/05-trees.md
├── src/guides/05-trees.source.md: mdloom:tree directives resolved
├── src/data/features.md: taxonomy source table
└── src/data/diagnostic-codes.md: second taxonomy source
```
<!-- /mdloom:compiled -->

Compile a single file:

```bash
mdloom compile src/guides/math.source.md
```

Compile a whole directory to a separate output location:

```bash
mdloom compile src/guides/ --output-dir docs/guides/
```

Watch for changes and recompile automatically on save:

```bash
mdloom compile --watch   # reads [[compile]] targets from mdloom.toml
```

---

## Feature coverage

<!-- mdloom:compiled from="mdloom:tree kind=taxonomy" uri="md://src/data/features.md" -->
```taxonomy
math
├── LaTeX math inline
└── LaTeX math display
symbols
├── Symbol expansion
├── Symbol block
└── Shape renderer
elements
├── Element value
├── Element delta
├── Element sparkline
├── Element mini-bar
├── Element label
├── Element badge
└── Row compositor
slides
├── Slide title
├── Slide title-content
├── Slide two-column
├── Slide section
├── Slide stats
├── Slide blank
├── Slide bullets
├── Slide callout
├── Slide divider
├── Slide quote
└── Slide centered
dashboard
└── Dashboard canvas
trees
├── Tree dirtree
├── Tree org
├── Tree taxonomy
├── Tree dependency
└── Tree outline
figures
├── Figure import
└── DaVinci pin
linting
├── Lint check
└── Auto-fix
compile
└── Compile pipeline
```
<!-- /mdloom:compiled -->

---

## Next steps

- [Math guide](math.md) — LaTeX rendering for formulas and symbols
- [Symbols guide](symbols.md) — named glyph library and shape renderer
- [Elements guide](elements.md) — sparklines, bars, values, and data rows
- [Slides guide](slides.md) — ASCII presentation layouts
- [Trees guide](trees.md) — org charts, taxonomies, and dirtrees
- [Compile guide](compile.md) — full directive reference
- [Lint guide](lint.md) — check rules and mdloom.toml options
