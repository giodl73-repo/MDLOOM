---
wave: wide-character-policy
date_close: 2026-05-14
status: complete
---

# Wide Character Policy - Closeout

## Delivered

| Area | Result |
|---|---|
| Config behavior | `ascii_char.error_on_wide = false` now suppresses intentional wide-character diagnostics. |
| Strict default | `error_on_wide = true` still emits `ascii_char_range` errors for CJK/fullwidth/emoji in code blocks. |
| Documentation | SPEC and schema reference now describe suppression semantics. |
| Tests | Added `error_on_wide_false_suppresses_intentional_wide_chars`. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 10903 | 7847 |
| `ascii_char_range` | 3056 | 0 |

## Carry-Forwards

1. Remaining MAXIM warning signal is now dominated by `ascii_box_col`,
   `ascii_cell_padding`, and `ascii_connector_drift`.
2. Nested/side-by-side box detection still needs a dedicated design wave.
3. Sibling `mdpath` warnings remain external to proof.
