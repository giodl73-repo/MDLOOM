# mdloom Compile — Directive Reference

`mdloom compile` is mdloom's documentation compiler. It reads `.source.md` files,
resolves every `mdloom:` directive into rendered output, and writes the result.
The mental model: source documents are like source code — they reference external
data, render math, generate trees. Compiled documents are the artifact that gets
committed, published, or read.

Never edit compiled `.md` files directly. Edit the source and recompile. The
compiled output has `<!-- mdloom:compiled ... -->` markers that prove it was
generated, not hand-written.

---

## File naming conventions

mdloom routes files to different compilation pipelines based on their suffix.
The output path is derived automatically by stripping `.source.`:

| Source suffix | Output suffix | Compilation route |
|---------------|---------------|-------------------|
| `.source.md` | `.md` | General compile — resolves all directives |
| `.slides.source.md` | `.slides.md` | Slide compositor — layouts and body directives |
| `.dashboard.source.md` | `.dashboard.md` | Canvas compositor — fixed-position regions |

Route source files to a different output directory without renaming them:

```bash
mdloom compile src/guides/ --output-dir docs/guides/
```

---

## Compile commands

```bash
# Compile one file (output next to source)
mdloom compile src/guides/math.source.md

# Compile one file to a specific output path
mdloom compile src/guides/math.source.md -o docs/guides/math.md

# Compile a whole directory, output to docs/
mdloom compile src/guides/ --output-dir docs/guides/

# Publish one source file as standalone HTML
mdloom compile src/guides/math.source.md --target html -o docs/guides/math.html

# Compile one source file into a compact AI/context transfer artifact
mdloom compile src/guides/math.source.md --target mdport -o context/math.mdport.json

# Compile one source file into a machine-readable report bundle
mdloom compile src/guides/math.source.md --target json-report -o reports/math.mdloom-report.json

# Compile a source tree into a local static site
mdloom compile src/guides/ --target site --output-dir site/

# Publish one source file as a portable PDF
mdloom compile src/guides/math.source.md --target pdf -o docs/guides/math.pdf

# Publish one source file as an editable Word document
mdloom compile src/guides/math.source.md --target docx -o docs/guides/math.docx

# Publish an explicit slide source as an editable PowerPoint deck
mdloom compile src/decks/status.slides.source.md --target pptx -o decks/status.pptx

# Watch for changes and recompile on save
mdloom compile --watch            # reads [[compile]] targets from mdloom.toml

# Validate directives without writing output
mdloom compile --check src/guides/
```

`--target html` resolves the same source directives as the default Markdown
target, then renders the resolved Markdown as a standalone HTML document with a
small built-in stylesheet. The HTML backend supports common Markdown blocks
including headings, lists, tables, links, task lists, strikethrough, and fenced
code. Raw HTML in source Markdown is escaped rather than passed through, so the
publish backend stays safe by default. Watch mode remains Markdown-only until
target-aware watch manifests are modeled.

`--target mdport` writes **Mdports**: compact `mdport.v1` JSON
documents optimized for agents, retrieval, and transfer rather than visual
presentation. A mdport contains the source path, title, resolved dependency refs,
and section chunks with stable IDs, heading paths, line numbers, and resolved
Markdown text. The schema is intentionally CROP-friendly: CROP can emit the same
shape for view packs or corpus slices, while MDLOOM emits it for compiled source
documents.

`--target json-report` writes `mdloom.publish.json_report.v1`: a stable
machine-readable compile bundle for CI, agents, and integrations. It includes
artifact summary, resolved Markdown, section summaries, source metadata,
dependency refs, diagnostics, and compile counts. It is intentionally more
verbose than Mdport and is not a replacement for Mdport's compact retrieval
format.

`--target site` compiles a source tree to static HTML pages and writes a
navigation `index.html` plus `mdloom-site.json` site manifest in the output
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

`--target pptx` writes a native editable PowerPoint OOXML deck from explicit
`.slides.source.md` inputs. It supports title/content slides, native
bullets/numbering, editable monospace code text, and speaker notes from
`mdloom:notes`; arbitrary prose sources are rejected so deck generation stays
intentional.

---

## Complete directive reference

Every `mdloom:` directive uses a fenced code block with the directive name as
the info string. Attributes go on the opening fence line; the block body
provides the directive's content.

<!-- mdloom:compiled from="mdloom:tree kind=org" uri="" -->
```org
mdloom directives
├── Data directives
├── mdloom:element: Single data cell (value, sparkline, bar, label, badge)
├── mdloom:row: Column-aligned data rows from a table
├── mdloom:tree: ASCII tree (org, taxonomy, dependency, outline, dirtree)
├── Math and symbols
├── mdloom:math: LaTeX display math block
├── mdloom:symbol: Named symbol rendered as ASCII art block
├── mdloom:shape: Geometric shape (banner, badge, ribbon)
├── Slide body directives
├── mdloom:bullets: Nested bullet list
├── mdloom:callout: Bordered callout box with style
├── mdloom:divider: Horizontal rule
├── mdloom:quote: Attributed block quote
├── mdloom:centered: Centered text
├── mdloom:stat: KPI stat cell
├── mdloom:notes: Speaker notes (excluded from slide output)
├── mdloom:right: Right-align a block of text (complement to mdloom:centered)
├── mdloom:ol: Ordered (numbered) list with decimal sub-numbering
├── mdloom:toc: Auto-generate table of contents from headings
├── Compositor directives
├── mdloom:slide: Full slide declaration in a .slides.source.md file
├── mdloom:region: Named region in a .dashboard.source.md file
├── Include directives
├── mdloom:include: Inline content from an md:// URI
└── mdloom:table: Render a data table from an md:// URI
```
<!-- /mdloom:compiled -->

---

## The md:// URI scheme

`md://` is the stable addressing scheme for content within a mdloom project.
Every directive that reads external data (`source=`, `mdloom:include`,
`mdloom:row`) uses `md://` URIs. The path is always relative to the mdloom root
(the directory containing `mdloom.toml`).

```
md://src/data/features.md          ← whole file content
md://src/data/features.md#section  ← content of one section
md://languages/10-GO.md#concurrency:figure:goroutine-scheduler
                                    ← named figure in a section
```

mdloom checks `md://` URIs during `mdloom check` — broken references surface as
`md_broken_uri` errors before you even run compile. This means an AI session
can catch missing references with a simple lint run, not just a compile.

---

## mdloom:math — LaTeX display block

Use `mdloom:math` for multi-line math that needs the stacked fraction, integral,
matrix, or sum-with-limits rendering. Inline `$...$` is for single-line
expressions in prose; `mdloom:math` is for equations that deserve their own block.

````markdown
<!-- mdloom:compiled from="mdloom:math" -->
```
n(n+1)
──────
  2   
```
<!-- /mdloom:compiled -->
````

Attributes: `width` (columns, 0=auto), `align` (left/center/right), `no-chrome` (omit the compiled wrapper).

---

## mdloom:element and mdloom:row

`mdloom:element` renders one data cell; `mdloom:row` renders a whole column-aligned
table from a data source. The `source=md://...` attribute points to any markdown
table. The `foreach=row` attribute sets the iteration variable.

````markdown
<!-- mdloom:compiled from="mdloom:row" uri="md://src/data/features.md" -->
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
<!-- /mdloom:compiled -->
````

---

## mdloom:tree

Trees accept either inline body content or a `source=md://...` data table.
For inline content, use `root:` for the root node and `- child` with indentation.
For data-driven trees, specify `name=` and `parent=` column names.

````markdown
<!-- mdloom:compiled from="mdloom:tree kind=org" uri="" -->
```org
My Project
├── Frontend: React
└── Backend: Rust
```
<!-- /mdloom:compiled -->
````

---

## Cache behavior

mdloom caches compilation results using content-addressed hashes. A file is
only recompiled when its source or any of its dependencies change. The cache
lives in `.mdloom-cache/` at the project root.

<!-- mdloom:compiled from="mdloom:tree kind=taxonomy" uri="" -->
```taxonomy
Cache tiers
├── Tier 1 (parse): Source file parse result
├── Invalidated by any source file change
├── Tier 2 (resolve): md:// URI resolution
├── Invalidated by source or target file change
├── Tier 3 (compile): Full rendered output
└── Invalidated when any input changes
```
<!-- /mdloom:compiled -->

---

## Diagnostic codes produced by compile

<!-- mdloom:compiled from="mdloom:tree kind=taxonomy" uri="md://src/data/diagnostic-codes.md" -->
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
<!-- /mdloom:compiled -->
