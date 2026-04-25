---
name: review-ascii
description: Review ASCII art detection algorithms in src/checks/ascii_box.rs and ascii_flow.rs. Uses PIXEL (detection correctness) and PARSE (algorithm safety) roles.
user_invocable: true
---

# ASCII Art Detection Review

Reviews the core detection logic for correctness, edge case safety, and Unicode handling.

## Steps

### 1. PARSE review — safety

Read `src/checks/ascii_box.rs` and `src/checks/ascii_flow.rs`.

Check every string slice operation:
- Is it on a character boundary? (`line[n..]` must start on a char boundary)
- Could `inner_end <= first_len` cause a panic? (`split_cells` must guard this)
- Does `visual_width()` use `UnicodeWidthChar` consistently — no `len()` leaks?
- Does any function panic on an empty input? (Empty file, empty code block, single-char line)
- Does `vertical_columns()` and `junction_columns()` handle tabs correctly?

Flag format:
```
**PARSE [PANIC RISK]:** {function}:{line} — {what could panic and why}
Fix: {guard to add}
```

### 2. PIXEL review — detection correctness

For each detection heuristic, verify it does what it claims:

**Border detection (`is_border_line`):**
- Does it require ≥ 2 junction chars?
- Would it fire on `| Option A | Option B |` (prose table)? (It should not — no junction chars.)
- Would it fire on `+--` (partial border)? (Edge case — document the behavior.)

**Column extraction (`junction_columns`, `vertical_columns`):**
- Are positions 1-based visual columns, not 0-based byte offsets?
- For a line `│ foo │ bar │`, are the `│` positions at the right visual columns?

**Box region detection (`find_boxes`):**
- Does it correctly pair top and bottom borders?
- If two consecutive border lines appear (like in a row separator `+---+---+\n+---+---+`),
  does it handle both correctly?

Flag format:
```
**PIXEL [DETECTION]:** {function} — {what it misses or fires on incorrectly}
Fix: {correction}
```

### 3. Fixture audit

For each fixture in `tests/fixtures/`:
- Manually count the | positions in 'misaligned' fixtures — are they actually misaligned?
- Does `perfect_box.md` produce zero diagnostics? (Run the test to verify.)
- Is there a fixture for: single-char-line panic, CRLF endings, tab characters, deeply nested boxes?

### 4. Test coverage gaps

List behaviors that have no test:
- CRLF line endings
- Tabs as indentation before box
- A box inside a box (nested)
- Unicode wide characters (CJK) in border or content
- Empty code block

## Output

- PARSE issues (panic risks, byte/char boundary bugs)
- PIXEL issues (detection errors, false positives, missed cases)
- Fixture issues (fixtures claiming defects that aren't present)
- Coverage gaps (behaviors with no test)
- Summary: SAFE / NEEDS FIXES
