# proof Codebase Architecture

## Repository structure

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
