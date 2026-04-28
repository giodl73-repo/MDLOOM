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

# Watch for changes and recompile on save
proof compile --watch            # reads [[compile]] targets from proof.toml

# Validate directives without writing output
proof compile --check src/guides/
```

---

## Complete directive reference

Every `proof:` directive uses a fenced code block with the directive name as
the info string. Attributes go on the opening fence line; the block body
provides the directive's content.

```proof:tree kind=org
root: proof directives
- Data directives
  - proof:element: Single data cell (value, sparkline, bar, label, badge)
  - proof:row: Column-aligned data rows from a table
  - proof:tree: ASCII tree (org, taxonomy, dependency, outline, dirtree)
- Math and symbols
  - proof:math: LaTeX display math block
  - proof:symbol: Named symbol rendered as ASCII art block
  - proof:shape: Geometric shape (banner, badge, ribbon)
- Prose document directives
  - proof:blockquote: Indented block quote with optional attribution (markdown `>` or boxed)
  - proof:numbered-list (alias: proof:ol): Ordered (numbered) list with decimal sub-numbering
  - proof:toc: Auto-generate table of contents from headings
- Slide body directives
  - proof:bullets: Nested bullet list
  - proof:callout: Bordered callout box with style
  - proof:divider: Horizontal rule
  - proof:quote: Attributed block quote (slide-only — centered with curly quotes)
  - proof:centered: Centered text
  - proof:stat: KPI stat cell
  - proof:notes: Speaker notes (excluded from slide output)
  - proof:right: Right-align a block of text (complement to proof:centered)
- Compositor directives
  - proof:slide: Full slide declaration in a .slides.source.md file
  - proof:region: Named region in a .dashboard.source.md file
- Include directives
  - proof:include: Inline content from an md:// URI
  - proof:table: Render a data table from an md:// URI
```

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
```proof:math
\frac{n(n+1)}{2}
```
````

Attributes: `width` (columns, 0=auto), `align` (left/center/right), `no-chrome` (omit the compiled wrapper).

---

## proof:element and proof:row

`proof:element` renders one data cell; `proof:row` renders a whole column-aligned
table from a data source. The `source=md://...` attribute points to any markdown
table. The `foreach=row` attribute sets the iteration variable.

````markdown
```proof:row source=md://src/data/features.md foreach=row separator=" │ "
proof:element kind=label field=name width=30
proof:element kind=badge field=status width=10
```
````

---

## proof:tree

Trees accept either inline body content or a `source=md://...` data table.
For inline content, use `root:` for the root node and `- child` with indentation.
For data-driven trees, specify `name=` and `parent=` column names.

````markdown
```proof:tree kind=org
root: My Project
- Frontend: React
- Backend: Rust
```
````

---

## proof:toc — Table of Contents

Generate a TOC from the headings of any markdown source. With no `source=`,
the TOC is built from the surrounding compiled file. Use `style=` for the
visual form, `max-depth=` to cap heading levels, and `section=` to scope
the TOC to a single subsection — only the descendants of the heading whose
text matches `section=` are listed (the anchor heading itself is omitted).

Attributes: `source` (md:// URI, optional), `max-depth` (default 3),
`style` (`list` | `tree` | `numbered`), `section` (heading text — case-insensitive match).

````markdown
```proof:toc max-depth=2 style=numbered
```

```proof:toc section="API Reference" max-depth=4
```
````

The second example produces a TOC of every heading nested under
`## API Reference`, stopping at the next H2 sibling. Combine with
`style=tree` for a navigable visual, or `style=numbered` to call out
"we are on item 3.2" during a walkthrough.

---

## Cache behavior

proof caches compilation results using content-addressed hashes. A file is
only recompiled when its source or any of its dependencies change. The cache
lives in `.proof-cache/` at the project root.

```proof:tree kind=taxonomy
root: Cache tiers
- Tier 1 (parse): Source file parse result
  - Invalidated by any source file change
- Tier 2 (resolve): md:// URI resolution
  - Invalidated by source or target file change
- Tier 3 (compile): Full rendered output
  - Invalidated when any input changes
```

---

## Diagnostic codes produced by compile

```proof:tree kind=taxonomy source=md://src/data/diagnostic-codes.md name=code parent=module
```
