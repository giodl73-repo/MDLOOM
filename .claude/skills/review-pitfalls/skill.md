---
name: review-pitfalls
description: Audit design/pitfalls/ for completeness and traceability. Every pitfall should have a status, a structural solution, and a test. Uses PARSE and BENCH roles.
user_invocable: true
---

# Pitfalls Review

Pitfalls are structural failure modes, not bugs. A pitfall is a category of error that
is easy to stumble into and hard to notice. This skill audits the pitfall documentation.

## Steps

### 1. Read all pitfall files

Read every file in `design/pitfalls/`. Current files:
- `pitfalls-ascii-detection.md` (AD-01..AD-06)
- `pitfalls-schema.md` (SC-01..SC-03)

### 2. PARSE review — structural solution quality

For each pitfall:
- Is the **Structural solution** actually structural (prevents the whole class, not just the instance)?
- Is the **Status** current — SOLVED, PARTIAL, or OPEN?
- If SOLVED, does the cited code location still exist and still implement the fix?
- If OPEN, is there a workaround documented?

Flag format:
```
**PARSE [STALE]:** {pitfall ID} — {what's out of date}
Fix: {update}
```

### 3. BENCH review — test traceability

For each pitfall:
- Is a test cited in the **Test** field?
- Does that test actually exist (grep for it)?
- Does the test specifically cover the pitfall pattern — not just the happy path?
- OPEN pitfalls should note "Test: not yet written" so the gap is visible.

Flag format:
```
**BENCH [MISSING TEST]:** {pitfall ID} — {what has no test}
Fix: {test to add}
```

### 4. Coverage gaps

Review the codebase for failure modes NOT documented in pitfalls:
- Any panic-prone code paths without an AD- entry?
- Any schema merge edge case without an SC- entry?
- Any new check (ascii_flow, markdown) without its own pitfalls file?

### 5. New pitfall candidates

Based on recent session history, identify behaviors that burned time or caused a failed test
and propose new pitfall entries.

## Output

- Stale pitfalls (status wrong, code location moved)
- Missing tests (pitfalls claimed SOLVED with no test)
- Coverage gaps (failure modes with no pitfall)
- New pitfall candidates
- Summary: COMPLETE / GAPS FOUND
