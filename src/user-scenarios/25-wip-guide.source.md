# Feature Guide (Work in Progress)

This document is being drafted. Some referenced data files don't exist yet.
In a future release, `stub=true` on directives will compile without error
when the source is missing. For now, we use placeholder text.

## Feature Overview

Our feature taxonomy (data file coming in next sprint — will use
`proof:tree kind=taxonomy source=md://src/data/features-v2.md`):

proof:bullets
- Category: math
  - LaTeX inline rendering
  - Display block rendering
- Category: slides
  - 6 layout types
  - Full directive suite
- Category: dashboard
  - Fixed-canvas regions
  - Multi-region compositor

## Performance Comparison

Benchmark results (data file pending — will use `proof:row source=md://...`):

| Scenario | Ops/sec | Improvement |
|----------|---------|-------------|
| (pending) | — | — |

## What's implemented today

proof:bullets
- [sym:checkmark] Core algorithm — complete
- [sym:checkmark] Basic API — complete
- [sym:arrow-right] Performance tuning — in progress
- [sym:cross] Benchmarks — blocked on test data
- [sym:cross] Feature taxonomy — blocked on categorization work
