# proof Compile — Directive Reference

`proof compile` is proof's documentation compiler. It reads `.source.md` files,
resolves every `proof:` directive into rendered output, and writes the result.
The mental model: source documents are like source code — they reference external
data, render math, generate trees. Compiled documents are the artifact that gets
committed, published, or read.

Never edit compiled `.md` files directly. Edit the source and recompile. The
compiled output has `<!-- proof:compiled ... -->` markers that prove it was
generated, not hand-written.

---

## File naming conventions

proof routes files to different compilation pipelines based on their suffix.
The output path is derived automatically by stripping `.source.`:

| Source suffix | Output suffix | Compilation route |
|---------------|---------------|-------------------|
| `.source.md` | `.md` | General compile — resolves all directives |
| `.slides.source.md` | `.slides.md` | Slide compositor — layouts and body directives |
| `.dashboard.source.md` | `.dashboard.md` | Canvas compositor — fixed-position regions |

Route source files to a different output directory without renaming them:

```bash
proof compile src/guides/ --output-dir docs/guides/
```

---

## Compile commands

```bash
# Compile one file (output next to source)
proof compile src/guides/math.source.md

# Compile one file to a specific output path
proof compile src/guides/math.source.md -o docs/guides/math.md

# Compile a whole directory, output to docs/
proof compile src/guides/ --output-dir docs/guides/

# Publish one source file as standalone HTML
proof compile src/guides/math.source.md --target html -o docs/guides/math.html

# Compile one source file into a compact AI/context transfer artifact
proof compile src/guides/math.source.md --target pebble -o context/math.pebble.json

# Compile one source file into a machine-readable report bundle
proof compile src/guides/math.source.md --target json-report -o reports/math.proof-report.json

# Compile a source tree into a local static site
proof compile src/guides/ --target site --output-dir site/

# Publish one source file as a portable PDF
proof compile src/guides/math.source.md --target pdf -o docs/guides/math.pdf

# Publish one source file as an editable Word document
proof compile src/guides/math.source.md --target docx -o docs/guides/math.docx

# Watch for changes and recompile on save
proof compile --watch            # reads [[compile]] targets from proof.toml

# Validate directives without writing output
proof compile --check src/guides/
```

`--target html` resolves the same source directives as the default Markdown
target, then renders the resolved Markdown as a standalone HTML document with a
small built-in stylesheet. The HTML backend supports common Markdown blocks
including headings, lists, tables, links, task lists, strikethrough, and fenced
code. Raw HTML in source Markdown is escaped rather than passed through, so the
publish backend stays safe by default. Watch mode remains Markdown-only until
target-aware watch manifests are modeled.

`--target pebble` writes **Pebbles**: compact `pebble.v1` JSON
documents optimized for agents, retrieval, and transfer rather than visual
presentation. A pebble contains the source path, title, resolved dependency refs,
and section chunks with stable IDs, heading paths, line numbers, and resolved
Markdown text. The schema is intentionally CROP-friendly: CROP can emit the same
shape for view packs or corpus slices, while PROOF emits it for compiled source
documents.

`--target json-report` writes `proof.publish.json_report.v1`: a stable
machine-readable compile bundle for CI, agents, and integrations. It includes
artifact summary, resolved Markdown, section summaries, source metadata,
dependency refs, diagnostics, and compile counts. It is intentionally more
verbose than Pebble and is not a replacement for Pebble's compact retrieval
format.

`--target site` compiles a source tree to static HTML pages and writes a
navigation `index.html` plus `proof-site.json` site manifest in the output
directory. It is intended for local/static documentation publishing; hosting,
deployment, search ranking, and target-aware watch mode are out of scope.

`--target pdf` renders the same resolved HTML publish output into a portable PDF
artifact. The first backend is deterministic and dependency-free for CI, with
reasonable text output and metadata. It does not claim exact browser or print
engine layout equivalence.

`--target docx` writes a native editable Word-processing OOXML package from
resolved Markdown. It supports headings, paragraphs, native bullets/numbering,
tables, fenced code text, links, and basic metadata without requiring Microsoft
Word during CI.

---

## Complete directive reference

Every `proof:` directive uses a fenced code block with the directive name as
the info string. Attributes go on the opening fence line; the block body
provides the directive's content.

<!-- proof:compiled from="proof:tree kind=org" uri="" -->
```org
proof directives
├── Data directives
├── proof:element: Single data cell (value, sparkline, bar, label, badge)
├── proof:row: Column-aligned data rows from a table
├── proof:tree: ASCII tree (org, taxonomy, dependency, outline, dirtree)
├── Math and symbols
├── proof:math: LaTeX display math block
├── proof:symbol: Named symbol rendered as ASCII art block
├── proof:shape: Geometric shape (banner, badge, ribbon)
├── Slide body directives
├── proof:bullets: Nested bullet list
├── proof:callout: Bordered callout box with style
├── proof:divider: Horizontal rule
├── proof:quote: Attributed block quote
├── proof:centered: Centered text
├── proof:stat: KPI stat cell
├── proof:notes: Speaker notes (excluded from slide output)
├── proof:right: Right-align a block of text (complement to proof:centered)
├── proof:ol: Ordered (numbered) list with decimal sub-numbering
├── proof:toc: Auto-generate table of contents from headings
├── Compositor directives
├── proof:slide: Full slide declaration in a .slides.source.md file
├── proof:region: Named region in a .dashboard.source.md file
├── Include directives
├── proof:include: Inline content from an md:// URI
└── proof:table: Render a data table from an md:// URI
```
<!-- /proof:compiled -->

---

## The md:// URI scheme

`md://` is the stable addressing scheme for content within a proof project.
Every directive that reads external data (`source=`, `proof:include`,
`proof:row`) uses `md://` URIs. The path is always relative to the proof root
(the directory containing `proof.toml`).

```
md://src/data/features.md          ← whole file content
md://src/data/features.md#section  ← content of one section
md://languages/10-GO.md#concurrency:figure:goroutine-scheduler
                                    ← named figure in a section
```

proof checks `md://` URIs during `proof check` — broken references surface as
`md_broken_uri` errors before you even run compile. This means an AI session
can catch missing references with a simple lint run, not just a compile.

---

## proof:math — LaTeX display block

Use `proof:math` for multi-line math that needs the stacked fraction, integral,
matrix, or sum-with-limits rendering. Inline `$...$` is for single-line
expressions in prose; `proof:math` is for equations that deserve their own block.

````markdown
<!-- proof:compiled from="proof:math" -->
```
n(n+1)
──────
  2   
```
<!-- /proof:compiled -->
````

Attributes: `width` (columns, 0=auto), `align` (left/center/right), `no-chrome` (omit the compiled wrapper).

---

## proof:element and proof:row

`proof:element` renders one data cell; `proof:row` renders a whole column-aligned
table from a data source. The `source=md://...` attribute points to any markdown
table. The `foreach=row` attribute sets the iteration variable.

````markdown
<!-- proof:compiled from="proof:row" uri="md://src/data/features.md" -->
```
LaTeX math inline              │ stable    
LaTeX math display             │ stable    
Symbol expansion               │ stable    
Symbol block                   │ stable    
Shape renderer                 │ stable    
Element value                  │ stable    
Element delta                  │ stable    
Element sparkline              │ stable    
Element mini-bar               │ stable    
Element label                  │ stable    
Element badge                  │ stable    
Row compositor                 │ stable    
Slide title                    │ stable    
Slide title-content            │ stable    
Slide two-column               │ stable    
Slide section                  │ stable    
Slide stats                    │ stable    
Slide blank                    │ stable    
Slide bullets                  │ stable    
Slide callout                  │ stable    
Slide divider                  │ stable    
Slide quote                    │ stable    
Slide centered                 │ stable    
Dashboard canvas               │ stable    
Tree dirtree                   │ stable    
Tree org                       │ stable    
Tree taxonomy                  │ stable    
Tree dependency                │ stable    
Tree outline                   │ stable    
Figure import                  │ beta      
DaVinci pin                    │ beta      
Lint check                     │ stable    
Auto-fix                       │ stable    
Compile pipeline               │ stable    
```
<!-- /proof:compiled -->
````

---

## proof:tree

Trees accept either inline body content or a `source=md://...` data table.
For inline content, use `root:` for the root node and `- child` with indentation.
For data-driven trees, specify `name=` and `parent=` column names.

````markdown
<!-- proof:compiled from="proof:tree kind=org" uri="" -->
```org
My Project
├── Frontend: React
└── Backend: Rust
```
<!-- /proof:compiled -->
````

---

## Cache behavior

proof caches compilation results using content-addressed hashes. A file is
only recompiled when its source or any of its dependencies change. The cache
lives in `.proof-cache/` at the project root.

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Cache tiers
├── Tier 1 (parse): Source file parse result
├── Invalidated by any source file change
├── Tier 2 (resolve): md:// URI resolution
├── Invalidated by source or target file change
├── Tier 3 (compile): Full rendered output
└── Invalidated when any input changes
```
<!-- /proof:compiled -->

---

## Diagnostic codes produced by compile

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
