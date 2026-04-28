# Implementation Plan — Scenario Findings

Actionable code changes from scenarios 09-60. Spec clarifications are handled
separately in each spec file. This plan covers only findings that require
code changes.

---

## Priority 1 — Correctness bugs

### F42 — `\frac` inline downgrade: add parens for complex numerators

**Finding:** `\frac{x+y}{z}` inline → renders as `x+y/z` (wrong precedence).
Should render as `(x+y)/z`.

**Fix:** In `proof-math/src/lib.rs` `expand_command()` frac handler, when
numerator contains operators (`+`, `-`, `*`, `/`, `\pm` etc.), wrap in parens:

```rust
// current
format!("{}/{}", num, den)

// fixed
let num_needs_parens = num.contains(['+', '-', '×', '÷', '·']);
let num_display = if num_needs_parens { format!("({})", num) } else { num };
format!("{}/{}", num_display, den)
```

**File:** `proof/crates/proof-math/src/lib.rs` → `expand_command` for `"frac"`
**Tests:** Add L0 test: `\frac{x+y}{z}` inline → `(x+y)/z`; `\frac{a}{b}` → `a/b`

---

### F76 — Sparkline constant series divide-by-zero

**Finding:** When all series values are equal (min=max), normalization
`(v - min) / (max - min)` divides by zero.

**Fix:** In `proof/src/element/sparkline.rs`, guard the normalization:

```rust
let range = max - min;
let normalized = if range == 0.0 {
    0.5 // constant series → mid-height block ▄
} else {
    (v - min) / range
};
```

**File:** `proof/src/element/sparkline.rs`
**Tests:** Add L0 test: `value="5,5,5,5,5"` → all `▄` blocks, no panic

---

### F90 — Cycle detection in dependency trees

**Finding:** Circular references in `proof:tree kind=dependency` (A parent=B,
B parent=A) cause infinite recursion in `build_dfs_tree`.

**Fix:** In `proof/src/tree/schema.rs` `dfs_children()`, pass a `visited`
`HashSet` (already present) and early-return when a node is already in the set:

```rust
if visited.contains(child_name) {
    // emit COMPILE-002 warning: cycle detected
    continue;
}
visited.insert(child_name.to_string());
```

**File:** `proof/src/tree/schema.rs` → `dfs_children`
**Tests:** Add L0 test: table with A→B, B→A parent cycle → renders without panic, emits warning

---

### F123 — Unicode-safe `clip_to_width`

**Finding:** `clip_to_width` splits wide Unicode chars (2-column CJK) at the
boundary, leaving half a character.

**Fix:** In `proof/src/slide/layout.rs` and `proof-canvas/src/lib.rs`:

```rust
pub fn clip_to_width(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        let ch_w = char_visual_width(ch);
        if w + ch_w > width.saturating_sub(1) {
            out.push('…');
            break;
        }
        out.push(ch);
        w += ch_w;
    }
    out
}
```

Use `visual_width` per character, not `chars().count()`.

**Files:** `proof/src/slide/layout.rs`, `proof/src/slide/bullets.rs`,
`proof-canvas/src/lib.rs`
**Tests:** Add L0 test: CJK string clipped at boundary → no half-char

---

### F128 — Wide char at last canvas column

**Finding:** A 2-column wide char placed at the last column (col = width-1)
writes col and tries to write col+1 which is out-of-bounds. Current code
checks `col + 1 < self.width` before writing the placeholder — this is correct.
Verify the first half IS written even when the second can't be.

**Fix:** Audit `proof-canvas/src/lib.rs` paste loop — current code:
```rust
self.buf[row * self.width + col] = ch; // written
if ch_w >= 2 && col + 1 < self.width {
    self.buf[row * self.width + col + 1] = ' '; // only if room
}
col += ch_w;
```
This is already correct — the char is written, placeholder only if room.
Add a test to confirm.

**Files:** `proof-canvas/src/lib.rs`
**Tests:** Add L0 test: paste 2-wide char at col=width-1 → char written, no panic, next row unaffected

---

## Priority 2 — Missing features

### F42b — `proof:row` missing source attr surfaced at check time

**Finding:** Missing `source=` on `proof:row` is only caught at compile time.
`proof check` should catch it earlier via `SourceLinkCheck`.

**Fix:** Extend `SourceLinkCheck` in `proof/src/checks/source_links.rs` to
detect `proof:row` fences without a `source=md://` attribute and emit a
new diagnostic `md_missing_source`:

```rust
if fence_info.starts_with("proof:row") {
    let has_source = info_after.contains("source=md://");
    if !has_source {
        diags.push(Diagnostic::error(
            path.to_path_buf(), line_no, 1,
            "md_missing_source",
            "proof:row requires source=md://... attribute"
        ));
    }
}
```

**File:** `proof/src/checks/source_links.rs`
**Tests:** Add L1 test: `.source.md` with `proof:row` missing source → `md_missing_source` error

---

### F86 — Root detection when `root:` absent in inline tree body

**Finding:** `render_inline_tree` in `compile.rs` requires a `root:` prefix.
If absent, the tree fails silently.

**Fix:** In `render_inline_tree`, treat the first non-indented non-empty line
as the root if no `root:` prefix is found:

```rust
if let Some(rest) = trimmed.strip_prefix("root:") {
    nodes.push((0, rest.trim().to_string()));
} else if leading == 0 && nodes.is_empty() {
    // First non-indented line is implicitly the root
    nodes.push((0, trimmed.to_string()));
}
```

**File:** `proof/src/compile.rs` → `render_inline_tree`
**Tests:** Add L0 test: inline tree without `root:` → first line becomes root node

---

### F96 — Symbol case-insensitive lookup (verify)

**Finding:** Spec says symbol lookup is case-insensitive. Verify this is
implemented and add a test.

**Check:** `proof/src/symbol/mod.rs` — `resolve()` should lowercase the name.
Already implemented per session notes. Verify test exists.

**Tests:** Confirm test `symbol_resolve_case_insensitive` in integration_tests.rs

---

### F103 — proof:toc style=numbered

**Finding:** `proof:toc` currently supports `style=list` and `style=tree`.
`style=numbered` (with sequential numbers) is missing.

**Fix:** In `proof/src/compile.rs` `generate_toc()`, add numbered style:

```rust
"numbered" => {
    let mut counters: Vec<usize> = vec![0; max_depth + 1];
    // Generate "1.", "1.1.", "2." etc.
}
```

**File:** `proof/src/compile.rs` → `generate_toc`
**Tests:** Add L0 test: `style=numbered` → "1. Title", "  1.1. Section" format

---

### F116 — Stub attribute for work-in-progress directives

**Finding:** Authors building documents incrementally need a way to mark
`proof:tree source=md://not-yet-created.md` as intentionally incomplete
without generating an error.

**Fix:** Add `stub=true` attribute to tree, row, element, and include directives.
When present, resolve failures emit a warning instead of an error and write
the source block through unchanged.

**Files:** `proof/src/compile.rs` → each directive's parse and dispatch
**Tests:** Add L1 test: `proof:tree stub=true source=md://missing.md` → warning, written=true

---

## Priority 3 — Robustness and polish

### F83 — Tree exclude: basename vs glob clarification (test)

Current behavior: basename match at any depth. Add test to confirm:

```rust
// test: exclude=target skips C:/src/proof/crates/proof-math/target/
// but NOT C:/src/proof/src/dashboard/ (doesn't contain "target")
```

**File:** `proof/src/tree/dirtree.rs` tests

---

### F75 — Sparkline: series shorter than width (verify ELEMENT-003)

**Finding:** ELEMENT-003 warning fires when series length < width. Verify
the warning message is actionable and the rendering (repeat values) is correct.

**Check:** `proof/src/element/sparkline.rs` — confirm repeat logic and test.

---

### F119 — Stale output: add `--delete-on-error` flag

**Finding:** When compile fails, leaving stale output is confusing. Add opt-in
deletion.

**Fix:** Add `--delete-on-error` flag to `proof compile`:
- When set: if `has_errors`, delete the output file if it exists
- Default: false (current behavior — leave stale)

**File:** `proof/src/main.rs` → Compile command, `cmd_compile`
**Tests:** Add L2 CLI test: `--delete-on-error` removes output on error

---

## Summary

| Priority | Finding | Description | Complexity |
|----------|---------|-------------|------------|
| P1 | F42 | frac inline parens for complex numerators | small |
| P1 | F76 | sparkline constant series divide-by-zero | trivial |
| P1 | F90 | cycle detection in dependency trees | small |
| P1 | F123 | unicode-safe clip_to_width | small |
| P1 | F128 | wide char at last canvas column (verify) | trivial |
| P2 | F42b | proof:row missing source caught at check time | small |
| P2 | F86 | root detection without root: prefix | small |
| P2 | F96 | symbol case-insensitive lookup (verify) | trivial |
| P2 | F103 | proof:toc style=numbered | small |
| P2 | F116 | stub=true attribute for WIP directives | medium |
| P3 | F83 | tree exclude basename test | trivial |
| P3 | F75 | sparkline short series verify | trivial |
| P3 | F119 | --delete-on-error flag | small |
