# proof Trees — ASCII Tree Diagrams

`proof:tree` generates ASCII tree diagrams from either inline content or an
external data table. Trees are the right tool when hierarchy is the point —
org charts, taxonomy classifications, dependency graphs, numbered outlines,
and filesystem structure all have a clearer shape as a tree than as a table.

The five kinds cover the main hierarchy patterns. `dirtree` scans the
filesystem directly. The other four (`org`, `taxonomy`, `dependency`,
`outline`) either accept inline content in the directive body or read from a
markdown data table via `source=md://...`.

---

## dirtree — filesystem trees

`dirtree` is the only kind that reads from disk rather than a data table. It
walks the filesystem from a `root` path and renders the directory structure.
Use it to document project layout, show where configuration files live, or
provide orientation in a complex monorepo.

`max_depth` controls how deep to go. `exclude` takes comma-separated glob
patterns to skip — commonly used to hide `target/`, `node_modules/`, and
build artifacts.

<!-- proof:compiled from="proof:tree kind=dirtree" uri="" -->
```dirtree
src/
├── checks/
│   ├── ascii_barchart.rs
│   ├── ascii_box.rs
│   ├── ascii_char.rs
│   ├── ascii_flow.rs
│   ├── ascii_tree.rs
│   ├── markdown.rs
│   ├── markdown_table.rs
│   ├── mod.rs
│   └── source_links.rs
├── dashboard/
│   ├── canvas.rs
│   ├── mod.rs
│   └── region.rs
├── data/
│   ├── diagnostic-codes.md
│   ├── features.md
│   ├── slide-layouts.md
│   └── symbol-catalog.md
├── element/
│   ├── mini_bar.rs
│   ├── mod.rs
│   ├── row.rs
│   ├── sparkline.rs
│   └── value.rs
├── figure/
│   ├── dither.rs
│   ├── mod.rs
│   └── shape.rs
├── guides/
│   ├── 00-getting-started.source.md
│   ├── 01-math.source.md
│   ├── 02-symbols.source.md
│   ├── 03-elements.source.md
│   ├── 04-slides.slides.source.md
│   ├── 05-trees.source.md
│   ├── 06-dashboard.source.md
│   ├── 07-compile.source.md
│   └── 08-lint.source.md
├── math/
│   ├── fraction.rs
│   ├── integral.rs
│   ├── matrix.rs
│   ├── mod.rs
│   ├── render.rs
│   ├── superscript.rs
│   ├── symbols.rs
│   ├── tier2.rs
│   └── tokenizer.rs
├── slide/
│   ├── bullets.rs
│   ├── canvas.rs
│   ├── inline.rs
│   ├── layout.rs
│   ├── mod.rs
│   └── parser.rs
├── symbol/
│   ├── library.rs
│   ├── mod.rs
│   └── shape.rs
├── tree/
│   ├── dirtree.rs
│   ├── mod.rs
│   └── schema.rs
├── user-scenarios/
│   ├── 07-canvas-tui/
│   ├── 21-proof-math-demo/
│   ├── data/
│   │   └── models.md
│   ├── 02-math-api.source.md
│   ├── 03-metrics-dashboard.dashboard.source.md
│   ├── 04-status-deck.slides.source.md
│   ├── 08-model-comparison.source.md
│   ├── 09-dependencies.source.md
│   ├── 10-calculus-deck.slides.source.md
│   ├── 12-blog-post.source.md
│   ├── 14-ml-taxonomy.source.md
│   ├── 15-rulebook.source.md
│   ├── 17-pitch-deck.slides.source.md
│   ├── 18-architecture.source.md
│   ├── 19-problem-set.source.md
│   ├── 22-status-board.dashboard.source.md
│   ├── 23-adr-with-toc.source.md
│   ├── 25-wip-guide.source.md
│   └── proof.toml
├── baseline.rs
├── compile.rs
├── config.rs
├── davinci.rs
├── diagnostic.rs
├── draft.rs
├── fix.rs
├── layout.rs
├── lib.rs
├── main.rs
├── runner.rs
└── spec_gen.rs
```
<!-- /proof:compiled -->

With exclusions:

<!-- proof:compiled from="proof:tree kind=dirtree" uri="" -->
```dirtree
src/
├── checks/
│   ├── ascii_barchart.rs
│   ├── ascii_box.rs
│   ├── ascii_char.rs
│   ├── ascii_flow.rs
│   ├── ascii_tree.rs
│   ├── markdown.rs
│   ├── markdown_table.rs
│   ├── mod.rs
│   └── source_links.rs
├── dashboard/
│   ├── canvas.rs
│   ├── mod.rs
│   └── region.rs
├── data/
│   ├── diagnostic-codes.md
│   ├── features.md
│   ├── slide-layouts.md
│   └── symbol-catalog.md
├── element/
│   ├── mini_bar.rs
│   ├── mod.rs
│   ├── row.rs
│   ├── sparkline.rs
│   └── value.rs
├── figure/
│   ├── dither.rs
│   ├── mod.rs
│   └── shape.rs
├── guides/
│   ├── 00-getting-started.source.md
│   ├── 01-math.source.md
│   ├── 02-symbols.source.md
│   ├── 03-elements.source.md
│   ├── 04-slides.slides.source.md
│   ├── 05-trees.source.md
│   ├── 06-dashboard.source.md
│   ├── 07-compile.source.md
│   └── 08-lint.source.md
├── math/
│   ├── fraction.rs
│   ├── integral.rs
│   ├── matrix.rs
│   ├── mod.rs
│   ├── render.rs
│   ├── superscript.rs
│   ├── symbols.rs
│   ├── tier2.rs
│   └── tokenizer.rs
├── slide/
│   ├── bullets.rs
│   ├── canvas.rs
│   ├── inline.rs
│   ├── layout.rs
│   ├── mod.rs
│   └── parser.rs
├── symbol/
│   ├── library.rs
│   ├── mod.rs
│   └── shape.rs
├── tree/
│   ├── dirtree.rs
│   ├── mod.rs
│   └── schema.rs
├── user-scenarios/
│   ├── 07-canvas-tui/
│   ├── 21-proof-math-demo/
│   ├── data/
│   │   └── models.md
│   ├── 02-math-api.source.md
│   ├── 03-metrics-dashboard.dashboard.source.md
│   ├── 04-status-deck.slides.source.md
│   ├── 08-model-comparison.source.md
│   ├── 09-dependencies.source.md
│   ├── 10-calculus-deck.slides.source.md
│   ├── 12-blog-post.source.md
│   ├── 14-ml-taxonomy.source.md
│   ├── 15-rulebook.source.md
│   ├── 17-pitch-deck.slides.source.md
│   ├── 18-architecture.source.md
│   ├── 19-problem-set.source.md
│   ├── 22-status-board.dashboard.source.md
│   ├── 23-adr-with-toc.source.md
│   ├── 25-wip-guide.source.md
│   └── proof.toml
├── baseline.rs
├── compile.rs
├── config.rs
├── davinci.rs
├── diagnostic.rs
├── draft.rs
├── fix.rs
├── layout.rs
├── lib.rs
├── main.rs
├── runner.rs
└── spec_gen.rs
```
<!-- /proof:compiled -->

---

## org — org chart / hierarchy

`org` renders a tree from parent-child relationships. Use it for command
hierarchies, system architecture, feature breakdowns, or any "A contains B
contains C" structure. Write the hierarchy inline in the directive body using
`root:` for the top node and `- child` with indentation for children.

The inline format is convenient for static hierarchies. For hierarchies derived
from data (e.g., an org chart from a people table), use `source=md://...` with
`name=` and `parent=` column names.

<!-- proof:compiled from="proof:tree kind=org" uri="" -->
```org
proof CLI
├── proof check: Lint markdown and ASCII art
├── AsciiBoxCheck: Box border alignment
├── AsciiFlowCheck: Flow diagram nodes
├── MarkdownCheck: Headings and links
├── MarkdownTableCheck: Column alignment
├── proof compile: Resolve directives
├── proof:math: LaTeX → Unicode/ASCII art
├── proof:symbol: Named glyphs
├── proof:element: Numeric cells
├── proof:row: Data rows
├── proof:tree: Tree diagrams
├── proof:slide: Presentations
├── proof:region: Dashboard canvas
├── proof fix: Auto-patch errors
└── proof pin: Register figure invariants
```
<!-- /proof:compiled -->

From a data table (parent/child columns):

<!-- proof:compiled from="proof:tree kind=org" uri="md://src/data/features.md" -->
```org
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
<!-- /proof:compiled -->

---

## taxonomy — classification trees

`taxonomy` is like `org` but oriented toward classification hierarchies where
nodes at the same level represent peers within a category. The difference is
visual: taxonomy renders with clear category-level breaks. Use it for
knowledge organization, feature matrices, or content classification.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Math rendering
├── Tier 1: Unicode substitution
├── Greek letters
├── Operators
├── Arrows
├── Set theory
├── Logic
├── Tier 2: Single-line ASCII
├── Superscripts
├── Subscripts
├── Square roots
├── Inline fractions
├── Primes
├── Tier 3: Multi-line display
├── Stacked fractions
├── Integrals with limits
├── Sum and product
├── Matrices
└── Cases
```
<!-- /proof:compiled -->

---

## dependency — dependency graphs

`dependency` renders a tree where each node's children are its dependencies.
Use it to document what a module, package, or system relies on. The visual
shape makes it immediately clear which components are shared (appear multiple
times) and which are leaf dependencies.

<!-- proof:compiled from="proof:tree kind=dependency" uri="" -->
```dependency
compile output
├── math module: tokenizer, symbols, superscript, tier2, fraction, integral, matrix, render
├── symbol module: library, shape, mod
├── element module: value, delta, sparkline, mini_bar, label, badge, row
├── slide module: parser, canvas, layout, bullets, inline
├── dashboard module: canvas, region
└── compile.rs: math module, symbol module, element module, slide module, dashboard module
```
<!-- /proof:compiled -->

---

## outline — numbered outlines

`outline` renders a numbered hierarchical outline. Use it for document
structure, table of contents drafts, or project plans where numbering conveys
sequence and hierarchy together.

<!-- proof:compiled from="proof:tree kind=outline" uri="" -->
```outline
1. Installation
   1.1 From source
   1.2 From crates.io
2. Configuration
   2.1 proof.toml basics
   2.2 Section schemas
   2.3 Compile settings
3. Linting
   3.1 ASCII art checks
   3.2 Markdown checks
   3.3 Custom rules
4. Compilation
   4.1 Directives reference
   4.2 md:// URI scheme
   4.3 Cache behavior
```
<!-- /proof:compiled -->

---

## Tree from external data

When your hierarchy lives in a data table, use `source=md://path/to/table.md`
with `name=` and `parent=` to identify which columns drive the structure.
proof synthesizes parent nodes automatically when they appear as parent values
but aren't themselves rows in the table — so a flat table with a `category`
column works without needing explicit category rows.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="md://src/data/diagnostic-codes.md" -->
```taxonomy
ascii_box
├── ascii_box_width
├── ascii_box_col
└── ascii_box_open
ascii_flow
├── ascii_flow_node
└── ascii_flow_edge
ascii_tree
├── ascii_tree_indent
└── ascii_tree_root
ascii_barchart
└── ascii_barchart_scale
markdown
├── markdown_h1
├── markdown_h2
└── markdown_link
math
├── MATH-001
├── MATH-002
├── MATH-003
├── MATH-004
├── MATH-005
└── MATH-006
symbol
└── SYMBOL-001
compile
├── COMPILE-001
├── COMPILE-002
└── COMPILE-003
dashboard
├── DASHBOARD-001
├── DASHBOARD-002
└── DASHBOARD-003
```
<!-- /proof:compiled -->

---

## Validation

Trees are validated as they compile. proof checks that the tree has exactly
one root, that indentation is consistent, and that there are no circular
references. Run `proof tree validate` to check an existing rendered tree:

```bash
proof tree validate docs/guides/trees.md
```

---

## Tree attributes

| Attribute | Default | Description |
|-----------|---------|-------------|
| `kind` | required | `dirtree`, `org`, `taxonomy`, `dependency`, `outline` |
| `source` | — | `md://` URI of data table (or inline body) |
| `name` | auto | Column for node label |
| `parent` | auto | Column for parent relationship |
| `root` | — | Root path for dirtree |
| `max_depth` | unlimited | Maximum nesting depth |
| `exclude` | — | Comma-separated patterns to skip (dirtree) |
| `indent_width` | 4 | Spaces per indentation level |
