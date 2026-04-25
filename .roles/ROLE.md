# glint Review Roles

Five perspectives on a markdown and ASCII art linter. Each role has a pointed view
and pulls against at least one other. Named after optical and signal concepts —
because glint is about seeing what others miss.

## The Five Roles

```
PIXEL    ASCII Art Analyst        ─── Character alignment, visual rendering, Unicode edge cases
SIGNAL   False Positive Analyst   ─── Actionability, noise ratio, author experience
SCHEMA   Rule Design Reviewer     ─── Schema expressiveness, cascade correctness, merge semantics
PARSE    Algorithm Correctness    ─── Parser edge cases, invariants, parallelism safety
BENCH    Test & Performance       ─── Coverage, benchmarks, regression safety
```

## Tiebreaker Ranking

When roles conflict, earlier roles govern:

1. **PARSE**   — a wrong diagnostic is worse than a missing one
2. **PIXEL**   — ASCII art detection is the core value proposition
3. **SIGNAL**  — a tool with too much noise gets ignored
4. **SCHEMA**  — rule design governs what gets caught
5. **BENCH**   — performance matters but correctness comes first

## Core Tensions

| Pulls | Against | Because |
|-------|---------|---------|
| PIXEL | SIGNAL | catching every misalignment generates noise; authors tune out noisy tools |
| SCHEMA | SCOPE | powerful schemas add complexity that makes the tool harder to adopt |
| PARSE | BENCH | correctness under edge cases (Unicode, empty files) trades against speed |
| SIGNAL | PIXEL | filtering false positives risks hiding real errors |
| BENCH | PARSE | parallelism introduces non-determinism risk that PARSE must police |

## Usage

Invoke any role when reviewing:
- Code changes to detection algorithms → PARSE + PIXEL
- New schema features → SCHEMA + SIGNAL
- Test additions → BENCH + PARSE
- Spec or design documents → SCHEMA + SIGNAL
- Performance work → BENCH + PARSE
