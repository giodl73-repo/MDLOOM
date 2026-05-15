# US-68 — Directory tree with excludes

Render the proof source tree, hiding compiled artifacts and the cache
directory. Demonstrates the `exclude` glob list.

<!-- proof:compiled from="proof:tree kind=dirtree" uri="" -->
```dirtree
src/
├── chart/
│   ├── area.rs
│   ├── bar.rs
│   ├── candlestick.rs
│   ├── gantt.rs
│   ├── heatmap.rs
│   ├── line.rs
│   ├── mod.rs
│   ├── render.rs
│   ├── scatter.rs
│   ├── stacked_bar.rs
│   ├── timeline.rs
│   └── waterfall.rs
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
│   ├── 08-lint.source.md
│   ├── 09-crates.source.md
│   ├── 10-query-params.source.md
│   └── 11-cache-snapshots.source.md
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
│   ├── 26-canvas-tui/
│   │   └── main.rs
│   ├── 27-proof-math-binary/
│   │   └── main.rs
│   ├── 29-fix-pipeline/
│   │   ├── before.md
│   │   └── proof.toml
│   ├── data/
│   │   └── models.md
│   ├── 02-math-api.source.md
│   ├── 03-metrics-dashboard.dashboard.source.md
│   ├── 04-status-deck.slides.source.md
│   ├── 08-model-comparison.md
│   ├── 08-model-comparison.source.md
│   ├── 09-dependencies.md
│   ├── 09-dependencies.source.md
│   ├── 10-calculus-deck.slides.md
│   ├── 10-calculus-deck.slides.source.md
│   ├── 100-blockquote-styles.md
│   ├── 100-blockquote-styles.source.md
│   ├── 101-lint-t3-irregular.md
│   ├── 101-lint-t3-irregular.source.md
│   ├── 102-lint-t4-dangling.md
│   ├── 102-lint-t4-dangling.source.md
│   ├── 103-lint-symbol-cluster.md
│   ├── 103-lint-symbol-cluster.source.md
│   ├── 104-lint-davinci-regex.source.md
│   ├── 105-lint-forbidden-section.md
│   ├── 105-lint-forbidden-section.source.md
│   ├── 106-integration-chart-region-deck.dashboard.md
│   ├── 106-integration-chart-region-deck.dashboard.source.md
│   ├── 107-integration-deck-many-kinds.slides.md
│   ├── 107-integration-deck-many-kinds.slides.source.md
│   ├── 108-integration-query-chart.md
│   ├── 108-integration-query-chart.source.md
│   ├── 109-integration-decision-from-data.source.md
│   ├── 110-integration-full-doc.md
│   ├── 110-integration-full-doc.source.md
│   ├── 12-blog-post.md
│   ├── 12-blog-post.source.md
│   ├── 14-ml-taxonomy.md
│   ├── 14-ml-taxonomy.source.md
│   ├── 15-rulebook.md
│   ├── 15-rulebook.source.md
│   ├── 17-pitch-deck.slides.md
│   ├── 17-pitch-deck.slides.source.md
│   ├── 18-architecture.md
│   ├── 18-architecture.source.md
│   ├── 19-problem-set.md
│   ├── 19-problem-set.source.md
│   ├── 22-status-board.dashboard.md
│   ├── 22-status-board.dashboard.source.md
│   ├── 23-adr-with-toc.md
│   ├── 23-adr-with-toc.source.md
│   ├── 25-wip-guide.md
│   ├── 25-wip-guide.source.md
│   ├── 28-large-corpus-scan.source.md
│   ├── 30-delete-on-error.source.md
│   ├── 31-blockquote.source.md
│   ├── 32-benchmark-chart.source.md
│   ├── 33-xref-guide.source.md
│   ├── 34-reveal-deck.slides.source.md
│   ├── 35-footer-agenda.slides.source.md
│   ├── 42-inline-pin.source.md
│   ├── 45-scoped-toc.source.md
│   ├── 46-progress-deck.slides.source.md
│   ├── 47-symbol-typo.source.md
│   ├── 49-xref-note.source.md
│   ├── 51-area-revenue.md
│   ├── 51-area-revenue.source.md
│   ├── 52-stacked-bar-team.md
│   ├── 52-stacked-bar-team.source.md
│   ├── 53-waterfall-budget.md
│   ├── 53-waterfall-budget.source.md
│   ├── 54-scatter-team-velocity.md
│   ├── 54-scatter-team-velocity.source.md
│   ├── 55-heatmap-availability.md
│   ├── 55-heatmap-availability.source.md
│   ├── 56-candlestick-stock.md
│   ├── 56-candlestick-stock.source.md
│   ├── 57-gantt-release.md
│   ├── 57-gantt-release.source.md
│   ├── 58-timeline-milestones.md
│   ├── 58-timeline-milestones.source.md
│   ├── 59-content-caption-deck.slides.md
│   ├── 59-content-caption-deck.slides.source.md
│   ├── 60-comparison-matrix.slides.md
│   ├── 60-comparison-matrix.slides.source.md
│   ├── 61-bar-with-axes.md
│   ├── 61-bar-with-axes.source.md
│   ├── 62-line-data-driven.md
│   ├── 62-line-data-driven.source.md
│   ├── 63-bar-single-point.md
│   ├── 63-bar-single-point.source.md
│   ├── 64-bar-all-zero.md
│   ├── 64-bar-all-zero.source.md
│   ├── 65-waterfall-mostly-negative.md
│   ├── 65-waterfall-mostly-negative.source.md
│   ├── 66-decision-deploy.md
│   ├── 66-decision-deploy.source.md
│   ├── 67-outline-deep-numbered.md
│   ├── 67-outline-deep-numbered.source.md
│   ├── 68-dirtree-excludes.md
│   ├── 68-dirtree-excludes.source.md
│   ├── 69-org-from-table.md
│   ├── 69-org-from-table.source.md
│   ├── 70-dependency-dedup.md
│   ├── 70-dependency-dedup.source.md
│   ├── 71-stats-three.slides.md
│   ├── 71-stats-three.slides.source.md
│   ├── 72-two-column-70-30.slides.md
│   ├── 72-two-column-70-30.slides.source.md
│   ├── 73-section-divider.slides.md
│   ├── 73-section-divider.slides.source.md
│   ├── 74-agenda-auto.slides.md
│   ├── 74-agenda-auto.slides.source.md
│   ├── 75-box-theme.slides.md
│   ├── 75-box-theme.slides.source.md
│   ├── 76-dashboard-chart-region.dashboard.md
│   ├── 76-dashboard-chart-region.dashboard.source.md
│   ├── 77-dashboard-multiregion.dashboard.md
│   ├── 77-dashboard-multiregion.dashboard.source.md
│   ├── 78-dashboard-overflow.dashboard.md
│   ├── 78-dashboard-overflow.dashboard.source.md
│   ├── 79-dashboard-symbol-mix.dashboard.md
│   ├── 79-dashboard-symbol-mix.dashboard.source.md
│   ├── 80-dashboard-tree-region.dashboard.md
│   ├── 80-dashboard-tree-region.dashboard.source.md
│   ├── 81-query-select-tree.md
│   ├── 81-query-select-tree.source.md
│   ├── 82-query-filter-chain.md
│   ├── 82-query-filter-chain.source.md
│   ├── 83-query-count-element.md
│   ├── 83-query-count-element.source.md
│   ├── 84-query-paging.md
│   ├── 84-query-paging.source.md
│   ├── 85-query-error-bad-column.source.md
│   ├── 86-math-fractions.md
│   ├── 86-math-fractions.source.md
│   ├── 87-math-matrix.md
│   ├── 87-math-matrix.source.md
│   ├── 88-math-integral.md
│   ├── 88-math-integral.source.md
│   ├── 89-math-cases.md
│   ├── 89-math-cases.source.md
│   ├── 90-math-no-chrome.md
│   ├── 90-math-no-chrome.source.md
│   ├── 91-element-sparkline-data.md
│   ├── 91-element-sparkline-data.source.md
│   ├── 92-element-mini-bar.md
│   ├── 92-element-mini-bar.source.md
│   ├── 93-symbol-typo.md
│   ├── 93-symbol-typo.source.md
│   ├── 94-shape-roster.md
│   ├── 94-shape-roster.source.md
│   ├── 95-row-many-elements.source.md
│   ├── 96-xref-section.md
│   ├── 96-xref-section.source.md
│   ├── 97-toc-scoped.md
│   ├── 97-toc-scoped.source.md
│   ├── 98-include-pin.source.md
│   ├── 99-layout-collage.md
│   ├── 99-layout-collage.source.md
│   └── proof.toml
├── ai.rs
├── baseline.rs
├── cache.rs
├── compile.rs
├── config.rs
├── davinci.rs
├── depends.rs
├── diagnostic.rs
├── draft.rs
├── fix.rs
├── layout.rs
├── lib.rs
├── main.rs
├── runner.rs
├── spec_gen.rs
└── unused.rs
```
<!-- /proof:compiled -->
