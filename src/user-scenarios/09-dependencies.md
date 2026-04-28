# Platform Component Dependencies

## Repository structure

<!-- proof:compiled from="proof:tree kind=dirtree" uri="" -->
```dirtree
user-scenarios/
├── 07-canvas-tui/
├── 21-proof-math-demo/
├── 26-canvas-tui/
│   └── main.rs
├── 27-proof-math-binary/
│   └── main.rs
├── 29-fix-pipeline/
├── data/
│   └── models.md
├── 02-math-api.md
├── 02-math-api.source.md
├── 03-metrics-dashboard.dashboard.md
├── 03-metrics-dashboard.dashboard.source.md
├── 04-status-deck.slides.md
├── 04-status-deck.slides.source.md
├── 08-model-comparison.md
├── 08-model-comparison.source.md
├── 09-dependencies.source.md
├── 10-calculus-deck.slides.source.md
├── 12-blog-post.source.md
├── 14-ml-taxonomy.source.md
├── 15-rulebook.source.md
├── 17-pitch-deck.slides.source.md
├── 18-architecture.source.md
├── 19-problem-set.source.md
├── 22-status-board.dashboard.source.md
├── 23-adr-with-toc.source.md
├── 25-wip-guide.source.md
└── proof.toml
```
<!-- /proof:compiled -->

## Architecture hierarchy

proof:bullets
- platform
  - auth-service: user-db, cache, jwt-lib
  - api-gateway: auth-service, rate-limiter, router
  - data-pipeline: kafka, schema-registry, storage-client
  - ml-inference: model-store, data-pipeline, gpu-runtime
  - dashboard: api-gateway, websocket-server, chart-lib

## Crate dependencies

proof:bullets
- proof (CLI + lib)
  - proof-canvas: unicode-width
  - proof-math: unicode-width
  - mdpath: thiserror
- icelines
  - icelines-core: (no deps)
  - icelines-fetch: icelines-core
  - icelines-cli: all three above
