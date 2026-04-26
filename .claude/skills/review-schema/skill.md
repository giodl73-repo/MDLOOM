---
name: review-schema
description: Review a proof.toml schema file (or the cascade implementation) for correctness, merge semantics, and usability. Uses SCHEMA and SIGNAL roles. Pass a file path or "cascade" to review the implementation.
user_invocable: true
---

# Schema Review

Reviews a `proof.toml` for correct syntax, sensible defaults, and merge behavior,
OR reviews the cascade implementation in `src/config.rs`.

## Input

Specify what to review:
- `<path/to/proof.toml>` — review a specific schema file
- `cascade` — review the cascade implementation in src/config.rs
- `maxim` — review schemas/reference.toml (the maxim library schema)

## Steps for Schema File Review

### 1. Syntax check

Parse the TOML mentally:
- Are all section headers valid (`[ascii_box]`, `[[section_schemas]]`, etc.)?
- Are all fields recognized by the spec?
- Are glob patterns syntactically valid?

Flag format:
```
**SCHEMA [SYNTAX]:** line {n} — {problem}
Fix: {correction}
```

### 2. SIGNAL review — will this produce useful diagnostics?

For each rule:
- If `required_h2_all = ["Decision Cheat Sheet"]` — would an author know which file is missing it?
- If `tolerance = 0` — would trailing spaces cause false positives on this content?
- Are `required_patterns` patterns specific enough to not false-positive?

### 3. Cascade correctness

If multiple proof.toml files exist in a hierarchy, trace the effective config for a sample file:
- Which proof.toml files apply?
- What is the merged `required_h2_all`?
- What is the effective `tolerance`?
- Is `files.root = true` set at the right level?

### 4. Section schema coverage

For `[[section_schemas]]` entries:
- Do the `paths` globs correctly match the intended directories?
- Are any directories not covered that should be?
- Are the required sections realistic — do actual files in those directories have them?

## Steps for Cascade Implementation Review

Read `src/config.rs`:
- Does `collect_configs_up` walk in the right direction (file → root)?
- Does `merge()` correctly apply additive semantics for lists?
- Does `merge_markdown()` produce a superset for `required_h2_all`?
- Is `files.root = true` correctly handled as a cascade stop?
- Does the config cache in `runner.rs` key on directory (not file)?

## Output

- Syntax issues
- SIGNAL issues (rules that will produce noise or be ignored)
- Cascade trace (for a sample file if reviewing a hierarchy)
- Coverage gaps (directories not covered by any section_schema)
- Summary: VALID / NEEDS FIXES
