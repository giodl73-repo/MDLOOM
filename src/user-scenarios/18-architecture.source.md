# mdloom Codebase Architecture

## Repository structure

```mdloom:tree kind=dirtree root=src max_depth=2 exclude=target
```

## Team organization

mdloom:bullets
- Core: compile pipeline, lint checks, fix system
  - mdloom-math: LaTeX renderer crate
  - mdloom-canvas: char grid crate
- Integrations: mdpath URI scheme and resolver
- Documentation: guides, scenarios, spec clarifications

## Module dependency graph

mdloom:bullets
- mdloom binary
  - compile.rs: math, symbol, element, slide, dashboard, tree, layout
  - runner.rs: checks, config
  - checks: ascii_box, ascii_flow, ascii_tree, markdown, markdown_table, source_links
  - dashboard: canvas (mdloom-canvas), region
  - slide: parser, canvas, layout, bullets, inline
  - element: value, delta, sparkline, mini_bar, row
  - symbol: library, shape
  - tree: dirtree, schema
