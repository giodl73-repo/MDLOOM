# proof Codebase Architecture

## Repository structure

```proof:tree kind=dirtree root=src max_depth=2 exclude=target
```

## Team organization

proof:bullets
- Core: compile pipeline, lint checks, fix system
  - proof-math: LaTeX renderer crate
  - proof-canvas: char grid crate
- Integrations: mdpath URI scheme and resolver
- Documentation: guides, scenarios, spec clarifications

## Module dependency graph

proof:bullets
- proof binary
  - compile.rs: math, symbol, element, slide, dashboard, tree, layout
  - runner.rs: checks, config
  - checks: ascii_box, ascii_flow, ascii_tree, markdown, markdown_table, source_links
  - dashboard: canvas (proof-canvas), region
  - slide: parser, canvas, layout, bullets, inline
  - element: value, delta, sparkline, mini_bar, row
  - symbol: library, shape
  - tree: dirtree, schema
