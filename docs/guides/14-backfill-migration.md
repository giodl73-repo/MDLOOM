# Backfill Migration Guide

`proof backfill` is the bridge from an existing markdown corpus to proof-owned
source. It is designed for MAXIM-style libraries: large, valuable `.md` trees
where preserving current content is more important than guessing perfect
semantics on the first pass.

---

## Migration principle

Backfill is literal-first. The first generated `.source.md` should compile back
to the original markdown, carry provenance, and produce a report that tells a
reviewer what can be promoted later.

<!-- proof:compiled from="proof:tree kind=dependency" uri="" -->
```dependency
Backfill migration
├── Inventory: scan existing .md files
├── Literal source: generate .source.md without losing content
├── Round trip: compile generated source and compare output
├── Report: classify prose, fences, tables, charts, diagrams, ambiguous blocks
└── Promote: extract high-confidence structures only after review
```
<!-- /proof:compiled -->

---

## First pass: mirror the corpus

Start with a separate output source tree so existing documents remain untouched.

```bash
proof backfill docs/ --output-source proof-source/ --literal-first --check-roundtrip
```

Expected outputs:

- generated `.source.md` files in `proof-source/`
- provenance frontmatter with `ops = ["backfill"]`
- `backfill-report.json`
- round-trip status for each file when `--check-roundtrip` is set

Treat a failed round trip as a blocker. Do not accept extracted source ownership
until literal output is trustworthy.

---

## Second pass: classify before extracting

The report is the review surface. It should tell BACKFILL and SIGNAL reviewers
where the corpus has obvious structure and where the migration needs human
judgment.

Look for:

| Report signal | Meaning |
|---------------|---------|
| `prose` | Safe literal markdown body |
| `fence` | Existing code or rendered artifact that should not be guessed |
| `markdown_table` | Candidate for table sidecar extraction |
| `ascii_table` | Possible table, but spacing may carry semantic meaning |
| `chart_like` | Candidate for chart extraction after review |
| `diagram_like` | Usually keep literal until a directive is obvious |
| `ambiguous` | Needs human review before promotion |

---

## Third pass: extract high-confidence tables

When the literal pass is clean, extract obvious markdown pipe tables into
sidecar data while preserving the source body.

```bash
proof backfill docs/ --output-source proof-source/ --extract-tables --check-roundtrip
```

This should write sidecar files such as:

```text
proof-source/
  guide.source.md
  guide.tables.json
  backfill-report.json
```

The source stays readable; the sidecar gives future waves a stable data source
for `proof:table`, `proof:row`, `proof:tree`, and `proof:chart` promotion.

---

## Cutover checklist

Before replacing hand-authored markdown with proof-owned source:

1. The literal pass round-trips.
2. The backfill report has no unreviewed high-risk ambiguous blocks.
3. Extracted tables have sidecars and report entries.
4. The generated source compiles in CI.
5. The compiled `.md` output is the artifact people read.
6. Future edits happen in `.source.md`, not the compiled `.md`.

For a MAXIM-style repo, cut over one directory or section at a time. A good first
slice is a guide directory with mostly prose and markdown tables, not a directory
full of hand-tuned diagrams.
