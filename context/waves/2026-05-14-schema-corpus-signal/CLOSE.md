---
wave: schema-corpus-signal
date_close: 2026-05-14
status: complete
---

# Schema and Corpus Signal - Closeout

## Delivered

| Area | Result |
|---|---|
| Schema disable | Child TOML can now explicitly set `[markdown] enabled = false` to disable inherited Markdown checks. |
| Fix line endings | `proof fix` preserves CRLF files when applying line edits. |
| MAXIM signal | Current MAXIM run is warning-only; largest code is `md_unexpected_section`. |
| mdpath warnings | Sibling `mdpath` warnings are explicitly carried forward instead of hidden in proof. |

## MAXIM Diagnostic Snapshot

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Count |
|---|---:|
| Files | 2739 |
| Errors | 0 |
| Warnings | 30658 |
| `md_unexpected_section` | 17335 |
| `ascii_cell_padding` | 4918 |
| `ascii_box_col` | 3753 |
| `ascii_char_range` | 3056 |

## Decisions

1. MAXIM content should not be bulk-edited until schema policy is accepted.
2. `md_unexpected_section` is a schema allowlist/design signal, not a guide
   content defect by default.
3. Sibling `mdpath` warning cleanup should happen with `C:\src\mdpath` as the
   active worktree or a dedicated cross-repo wave.

## Carry-Forwards

1. Add explicit unset-vs-explicit tracking for additional Markdown booleans only
   when a real child-disable use case appears.
2. Design a MAXIM schema policy wave that chooses between stricter per-directory
   allowlists, softer optional H2 defaults, or a warning budget by corpus area.
3. Clean `mdpath` warnings in the sibling repository.
