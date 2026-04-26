---
name: review-spec
description: Review design/SPEC.md for internal consistency, completeness, and alignment with the current implementation. Uses SCHEMA (rule design) and SIGNAL (actionability) roles.
user_invocable: true
---

# Spec Review

The SPEC.md is the contract between proof's design and its implementation. This skill audits
the spec for drift, gaps, and inconsistencies.

## Steps

### 1. Read the spec

Read `design/SPEC.md` in full.

### 2. SCHEMA review — rule design consistency

Check every config field mentioned in the spec against `src/config.rs`:
- Does the field exist in the struct?
- Does the TOML key match the struct field name?
- Is the described merge behavior implemented in `merge_markdown()` and `merge()`?
- Are all check codes documented (ascii_box_width, ascii_box_col, ascii_cell_padding, etc.)?

Flag format:
```
**SCHEMA [DRIFT]:** SPEC says {claim} but config.rs has {actual}
Fix: {what to update}
```

### 3. SIGNAL review — actionability

For each check described:
- Is the error message in the spec specific enough that an author knows what to fix?
- Is the code output (ascii_box_width etc.) consistent with what the Rust code emits?
- Does the spec document the `tolerance` behavior correctly?

Flag format:
```
**SIGNAL [UNCLEAR]:** {section} — {what's ambiguous}
Fix: {suggested clarification}
```

### 4. Invariant audit

Check each invariant in the spec's Invariants table:
- Is there a test that would catch a violation?
- Is the invariant precisely stated (no wiggle room)?

Flag any invariant without a corresponding test.

### 5. Non-goals check

Verify non-goals haven't accidentally been implemented. If a feature is listed as a non-goal
but exists in the code, either update the non-goals list or remove the feature.

## Output

Produce a structured report:
- SCHEMA issues (spec vs. code drift)
- SIGNAL issues (message clarity)
- Invariants without tests
- Non-goals violations
- Summary: PASS / NEEDS REVISION
