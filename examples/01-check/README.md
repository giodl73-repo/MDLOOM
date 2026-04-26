# Example 01 — Basic linting with `proof check`

**What you'll learn:** Run `proof check`, read diagnostic output, create a `proof.toml` schema.

## Setup

```bash
cd examples/01-check
proof check .
```

You'll see errors in `guide.md` — a misaligned box, a table missing a required row,
a heading out of order. These are intentional.

## Expected output

```
guide.md:12:1: error [ascii_box_width]: row width 45 ≠ box width 44 (box opened at line 9)
guide.md:18:1: error [ascii_box_col]: column separator at col 22 (expected col 21) — off by 1 (box opened at line 16)
guide.md:35:1: warning [md_missing_section]: required ## "Decision Cheat Sheet" absent
guide.md:47:1: error [table_missing_row]: required row key "Memory model" absent in Type System Snapshot
guide.md:60:1: warning [md_h1_count]: 2 H1 headings found; max is 1

FAIL — 1 files checked, 3 errors, 2 warnings
```

## Add a schema

`proof.toml` is already here. Look at it — it requires specific headings and table rows.

```bash
proof config guide.md   # show effective rules for this file
proof check . --errors-only   # suppress warnings, focus on errors
proof check . -f json   # machine-readable output
proof stats . --by-code   # count by error code
```

## Fix the errors

```bash
proof draft . -o draft.json   # generate a fix plan
proof fix --plan draft.json --dry-run   # preview changes
proof fix --plan draft.json   # apply
proof check .   # should be clean now
```
