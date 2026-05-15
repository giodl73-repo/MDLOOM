---
wave: schema-corpus-signal
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
source: context/waves/2026-05-14-architecture-quality-review-rail/CLOSE.md
---

# Schema and Corpus Signal

## Mission

Turn the remaining review carry-forwards into explicit decisions: schema merge
compatibility, MAXIM warning signal, sibling mdpath warnings, and fix-pipeline
line-ending guarantees.

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Schema disable compatibility decision | DONE | child `markdown.enabled=false` test and merge change |
| 02 - MAXIM `md_unexpected_section` signal reduction | DONE | corpus classified by code; schema policy carry-forward |
| 03 - Sibling mdpath warning cleanup decision | DONE | tracked as sibling-repo carry-forward, not proof worktree mutation |
| 04 - Fix CRLF preservation test | DONE | `fix::tests::apply_fix_preserves_crlf_line_endings` |

## Done Criteria

- Schema merge behavior has an accepted compatibility path.
- MAXIM warning volume is reduced or explicitly classified by schema policy.
- mdpath warnings are either fixed in the sibling repo or tracked there.
- `proof fix` has CRLF preservation coverage or a documented blocker.

## Non-Goals

- Do not mutate MAXIM content before schema policy is accepted.
- Do not edit sibling `mdpath` without making that repo the active worktree.

## Closeout

See `CLOSE.md`.
