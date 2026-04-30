---
slides:
  width: 80
  height: 24
  theme: minimal
---

```proof:slide layout=title title="Content-caption demo" subtitle="One slide, two slides"
```

---

```proof:slide layout=content-caption title="Architecture overview" subtitle="Figure 1 — three-tier cache, see CACHE-SNAPSHOTS.md"
proof:bullets
- Tier 1: parse cache — token streams keyed by content hash
- Tier 2: resolve cache — md:// URI → element content
- Tier 3: compile cache — rendered output keyed by source + figure hashes
```

---

```proof:slide layout=content-caption title="Test results" subtitle="*All 793 tests green; zero build warnings*"
proof:bullets
- Lib unit tests: 648
- Cache tests: 13 (added in v0.7)
- Integration tests across 5 test binaries: 145
```
