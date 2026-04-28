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

```proof:tree kind=dirtree root=src max_depth=2
```

With exclusions:

```proof:tree kind=dirtree root=src max_depth=3 exclude=target,*.lock
```

---

## org — org chart / hierarchy

`org` renders a tree from parent-child relationships. Use it for command
hierarchies, system architecture, feature breakdowns, or any "A contains B
contains C" structure. Write the hierarchy inline in the directive body using
`root:` for the top node and `- child` with indentation for children.

The inline format is convenient for static hierarchies. For hierarchies derived
from data (e.g., an org chart from a people table), use `source=md://...` with
`name=` and `parent=` column names.

```proof:tree kind=org
root: proof CLI
- proof check: Lint markdown and ASCII art
  - AsciiBoxCheck: Box border alignment
  - AsciiFlowCheck: Flow diagram nodes
  - MarkdownCheck: Headings and links
  - MarkdownTableCheck: Column alignment
- proof compile: Resolve directives
  - proof:math: LaTeX → Unicode/ASCII art
  - proof:symbol: Named glyphs
  - proof:element: Numeric cells
  - proof:row: Data rows
  - proof:tree: Tree diagrams
  - proof:slide: Presentations
  - proof:region: Dashboard canvas
- proof fix: Auto-patch errors
- proof pin: Register figure invariants
```

From a data table (parent/child columns):

```proof:tree kind=org source=md://src/data/features.md name=name parent=category
```

---

## taxonomy — classification trees

`taxonomy` is like `org` but oriented toward classification hierarchies where
nodes at the same level represent peers within a category. The difference is
visual: taxonomy renders with clear category-level breaks. Use it for
knowledge organization, feature matrices, or content classification.

```proof:tree kind=taxonomy
root: Math rendering
- Tier 1: Unicode substitution
  - Greek letters
  - Operators
  - Arrows
  - Set theory
  - Logic
- Tier 2: Single-line ASCII
  - Superscripts
  - Subscripts
  - Square roots
  - Inline fractions
  - Primes
- Tier 3: Multi-line display
  - Stacked fractions
  - Integrals with limits
  - Sum and product
  - Matrices
  - Cases
```

---

## dependency — dependency graphs

`dependency` renders a tree where each node's children are its dependencies.
Use it to document what a module, package, or system relies on. The visual
shape makes it immediately clear which components are shared (appear multiple
times) and which are leaf dependencies.

```proof:tree kind=dependency
root: compile output
- math module: tokenizer, symbols, superscript, tier2, fraction, integral, matrix, render
- symbol module: library, shape, mod
- element module: value, delta, sparkline, mini_bar, label, badge, row
- slide module: parser, canvas, layout, bullets, inline
- dashboard module: canvas, region
- compile.rs: math module, symbol module, element module, slide module, dashboard module
```

---

## outline — numbered outlines

`outline` renders a numbered hierarchical outline. Use it for document
structure, table of contents drafts, or project plans where numbering conveys
sequence and hierarchy together.

```proof:tree kind=outline
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

---

## Tree from external data

When your hierarchy lives in a data table, use `source=md://path/to/table.md`
with `name=` and `parent=` to identify which columns drive the structure.
proof synthesizes parent nodes automatically when they appear as parent values
but aren't themselves rows in the table — so a flat table with a `category`
column works without needing explicit category rows.

```proof:tree kind=taxonomy source=md://src/data/diagnostic-codes.md name=code parent=module
```

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
