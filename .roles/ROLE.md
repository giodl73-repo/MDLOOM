# proof Review Roles

Thirteen perspectives on proof — the markdown quality assurance, compiler, and
typesetting system.
Each role has a pointed view and pulls against at least one other.

## The Thirteen Roles

```
─── Code / correctness roles ────────────────────────────────────────────

PIXEL    ASCII Art Analyst           ─── Alignment, visual rendering, Unicode edge cases
SIGNAL   False Positive Analyst      ─── Actionability, noise ratio, author experience
SCHEMA   Rule Design Reviewer        ─── Schema expressiveness, cascade, merge semantics
PARSE    Algorithm Correctness       ─── Parser edge cases, invariants, parallelism safety
BENCH    Test & Performance          ─── Coverage, benchmarks, regression safety
SOURCE   Source/Target Document      ─── Include system, compile pipeline, author UX
COMPOSE  Layout & Composition        ─── Visual arrangement, frame alignment, gap math
CACHE    Cache Correctness           ─── Key computation, invalidation, snapshot integrity
BACKFILL Reverse Adoption            ─── Existing .md migration, round-trip fidelity, cutover safety

─── Domain / publishing roles ───────────────────────────────────────────

PRESS    Word Processor Expert       ─── Authoring friction, publishing conventions, discoverability
PANEL    Dashboard Designer          ─── Information density, scan path, terminal UI conventions
BOOK     Technical Writer            ─── Corpus-scale documentation, toolchain trust, cross-references
STAGE    Presentation Designer       ─── Slide structure, visual rhythm, 30-second communication rule
```

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **PARSE**   — a wrong diagnostic is worse than a missing one
2. **CACHE**   — a stale cache producing wrong output is a silent correctness bug
3. **PIXEL**   — ASCII art detection is the core value proposition
4. **SOURCE**  — if authors can't use it, correctness doesn't matter
5. **SIGNAL**  — a tool with too much noise gets ignored
6. **SCHEMA**  — rule design governs what gets caught
7. **COMPOSE** — visual output must be correct but is less critical than data correctness
8. **BACKFILL** — migration must not lose existing corpus content
9. **BENCH**   — performance matters but correctness comes first
10. **BOOK**   — corpus-scale concerns come after individual correctness
11. **PRESS**  — authoring experience matters after the tool works correctly
12. **STAGE**  — presentation conventions after document correctness
13. **PANEL**  — dashboard aesthetics after functional correctness

## Core Tensions

### Code / correctness tensions

| Pulls | Against | Because |
|-------|---------|---------|
| PIXEL | SIGNAL | catching every misalignment generates noise |
| CACHE | SOURCE | complex key computation makes compilation feel slow |
| CACHE | COMPOSE | every new layout attribute needs a cache key change |
| SCHEMA | SIGNAL | powerful schemas produce more rules which can produce more noise |
| PARSE | BENCH | correctness under edge cases trades against speed |
| SOURCE | COMPOSE | simple directive syntax vs. expressive layout attributes |
| SIGNAL | PIXEL | filtering false positives risks hiding real errors |
| BENCH | PARSE | parallelism introduces non-determinism risk PARSE must police |
| BACKFILL | SOURCE | preserving existing markdown literally vs. promoting it into cleaner source directives |
| BACKFILL | SIGNAL | reporting every uncertain extraction vs. keeping migration reports actionable |

### Domain / publishing tensions

| Pulls | Against | Because |
|-------|---------|---------|
| PRESS | PARSE | learnable syntax vs. unambiguous grammar |
| PRESS | SIGNAL | author-friendly error messages vs. low noise |
| PANEL | COMPOSE | design intent vs. mathematical layout correctness |
| PANEL | SOURCE | higher-level grid model vs. pixel-coordinate power |
| BOOK | SIGNAL | zero false positives across a corpus vs. catching real issues |
| BOOK | SCHEMA | clarity of rules vs. expressiveness of rule language |
| STAGE | COMPOSE | visual density vs. geometric alignment |
| STAGE | PRESS | slide conventions vs. document conventions |

## Usage

Invoke any role when reviewing:

| Work type | Roles |
|-----------|-------|
| Detection algorithm changes | PARSE + PIXEL |
| New schema features | SCHEMA + SIGNAL |
| Test additions | BENCH + PARSE |
| Spec or design docs | SOURCE + SCHEMA + SIGNAL |
| Layout engine | COMPOSE + PARSE |
| Cache implementation | CACHE + BENCH |
| Compile pipeline | SOURCE + CACHE + SIGNAL |
| Backfill / reverse migration | BACKFILL + SOURCE + BOOK + SIGNAL |
| Artifact cutover plans | BACKFILL + BOOK + BENCH |
| Performance work | BENCH + PARSE + CACHE |
| New directive syntax | PRESS + SOURCE + PARSE |
| Dashboard features | PANEL + COMPOSE |
| Slide features | STAGE + PRESS + COMPOSE |
| Documentation / guides | BOOK + PRESS + SIGNAL |
| Error messages | BOOK + PRESS + SIGNAL |
| Authoring UX | PRESS + SOURCE + SIGNAL |
