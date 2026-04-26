# proof Review Roles

Eight perspectives on proof — the markdown quality assurance and compilation system.
Each role has a pointed view and pulls against at least one other.

## The Eight Roles

```
PIXEL    ASCII Art Analyst           ─── Alignment, visual rendering, Unicode edge cases
SIGNAL   False Positive Analyst      ─── Actionability, noise ratio, author experience
SCHEMA   Rule Design Reviewer        ─── Schema expressiveness, cascade, merge semantics
PARSE    Algorithm Correctness       ─── Parser edge cases, invariants, parallelism safety
BENCH    Test & Performance          ─── Coverage, benchmarks, regression safety
SOURCE   Source/Target Document      ─── Include system, compile pipeline, author UX
COMPOSE  Layout & Composition        ─── Visual arrangement, frame alignment, gap math
CACHE    Cache Correctness           ─── Key computation, invalidation, snapshot integrity
```

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **PARSE**   — a wrong diagnostic is worse than a missing one
2. **CACHE**   — a stale cache producing wrong output is a silent correctness bug
3. **PIXEL**   — ASCII art detection is the core value proposition
4. **SOURCE**  — if authors can't use it, correctness doesn't matter
5. **SIGNAL**  — a tool with too much noise gets ignored
6. **SCHEMA**  — rule design governs what gets caught
7. **COMPOSE** — visual output must be correct but is less critical than correctness of data
8. **BENCH**   — performance matters but correctness comes first

## Core Tensions

| Pulls | Against | Because |
|-------|---------|---------|
| PIXEL | SIGNAL | catching every misalignment generates noise |
| CACHE | SOURCE | complex key computation makes compilation feel slow |
| CACHE | COMPOSE | every new layout attribute needs a cache key change |
| SCHEMA | SIGNAL | powerful schemas produce more rules which can produce more noise |
| PARSE | BENCH | correctness under edge cases (Unicode, empty files) trades against speed |
| SOURCE | COMPOSE | simple directive syntax vs. expressive layout attributes |
| SIGNAL | PIXEL | filtering false positives risks hiding real errors |
| BENCH | PARSE | parallelism introduces non-determinism risk PARSE must police |

## Usage

Invoke any role when reviewing:
- Detection algorithm changes → PARSE + PIXEL
- New schema features → SCHEMA + SIGNAL
- Test additions → BENCH + PARSE
- Spec or design docs → SOURCE + SCHEMA + SIGNAL
- Layout engine → COMPOSE + PARSE
- Cache implementation → CACHE + BENCH
- Compile pipeline → SOURCE + CACHE + SIGNAL
- Performance work → BENCH + PARSE + CACHE
