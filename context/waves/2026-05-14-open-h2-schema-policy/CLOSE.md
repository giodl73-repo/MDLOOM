---
wave: open-h2-schema-policy
date_close: 2026-05-14
status: complete
---

# Open H2 Schema Policy - Closeout

## Delivered

| Area | Result |
|---|---|
| Required H2 policy | `required_h2` and `required_h2_all` now enforce required sections without activating `md_unexpected_section`. |
| Explicit allowlist | `optional_h2` remains the switch that closes the H2 allowlist and warns on unlisted H2s. |
| Documentation | README and SPEC now describe `optional_h2` as the explicit closed-world H2 policy. |
| Tests | Unit coverage proves required-only open-world behavior and optional allowlist behavior. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before | After |
|---|---:|---:|
| Files | 2739 | 2740 |
| Errors | 0 | 0 |
| Warnings | 30658 | 13323 |
| `md_unexpected_section` | 17335 | 0 |

## Carry-Forwards

1. Remaining MAXIM signal is dominated by ASCII layout diagnostics:
   `ascii_cell_padding`, `ascii_box_col`, and `ascii_char_range`.
2. Sibling `mdpath` warnings remain external to proof and should be cleaned in
   the `C:\src\mdpath` worktree or a dedicated cross-repo wave.
