# Database Guide

## Query Optimization

For most queries, a covering index is sufficient. The query planner uses
statistics to decide between index scans and table scans.

```proof:xref uri="md://src/user-scenarios/49-xref-note.source.md#indexing-strategy" format=note
```

## Connection Pooling

Pool size should be set to `CPU cores × 2 + disk spindles`. Larger pools
create contention, not throughput.

```proof:xref uri="md://src/user-scenarios/49-xref-note.source.md#performance-tuning" format=callout
```

## Indexing Strategy

Composite indexes should order columns by selectivity (most selective first),
then by query access pattern.

## Performance Tuning

Start with connection pool sizing, then look at slow query logs before
adding indexes. Most "slow database" problems are missing indexes.
