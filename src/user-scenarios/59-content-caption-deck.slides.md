<!-- proof:compiled from="proof:slides" count=3 -->
```slides
SLIDE 1 ─────────────────────────────────────────────────────────────────────── 1/3











                              Content-caption demo
                             One slide, two slides











SLIDE 2 ─────────────────────────────────────────────────────────────────────── 2/3
Architecture overview


────────────────────────────────────────────────────────────────────────────────
● Tier 1: parse cache — token streams keyed by content hash
● Tier 2: resolve cache — md:// URI → element content
● Tier 3: compile cache — rendered output keyed by source + figure hashes














────────────────────────────────────────────────────────────────────────────────
Figure 1 — three-tier cache, see CACHE-SNAPSHOTS.md

SLIDE 3 ─────────────────────────────────────────────────────────────────────── 3/3
Test results


────────────────────────────────────────────────────────────────────────────────
● Lib unit tests: 648
● Cache tests: 13 (added in v0.7)
● Integration tests across 5 test binaries: 145














────────────────────────────────────────────────────────────────────────────────
*All 793 tests green; zero build warnings*

```
<!-- /proof:compiled -->
