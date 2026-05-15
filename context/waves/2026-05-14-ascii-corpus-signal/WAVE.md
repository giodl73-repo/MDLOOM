---
wave: ascii-corpus-signal
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# ASCII Corpus Signal

## Mission

Reduce remaining MAXIM ASCII warning noise where proof is clearly over-reading
valid content, without turning a previously warning-only corpus into errors.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - PARSE/PIXEL review | DONE | Reviewed `ascii_box` and `ascii_flow` string slicing, visual columns, border heuristics, and fixture coverage. |
| 02 - Cell padding false positive | DONE | Math bars such as `|G|` inside single-cell boxes are no longer treated as cell delimiters. |
| 03 - Nested box experiment | DONE | A broader box pairing change reduced warnings but exposed 244 MAXIM errors, so it was not kept. |
| 04 - Corpus and validation gate | DONE | MAXIM remains `0` errors; full proof validation passes. |

## Gates

- Focused regression test for math pipes inside a single-cell box.
- Existing nested-box behavior remains warning-only carry-forward.
- MAXIM corpus check stays error-free.
- Full Rust test/build/diff validation passes.

## Closeout

See `CLOSE.md`.
