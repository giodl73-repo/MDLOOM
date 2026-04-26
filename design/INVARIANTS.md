# proof Invariants

Invariants are properties that must hold for all inputs, at all times.
A change that breaks an invariant is a regression — fix the code, not the invariant.

Invariants are distinct from pitfalls: pitfalls describe how things *can go wrong*;
invariants describe what must *always be true*.

---

## Detection Invariants

### I-1: A file with no ASCII boxes produces no box diagnostics

**Claim:** If a file contains no border lines (`+---+` or `┌─┐` style), the
`ascii_box` check produces zero diagnostics.

**Why it matters:** Any false positive on prose that happens to contain `+` or `|`
destroys trust in the tool.

**Test:** `tests/integration_tests.rs::perfect_box_zero_diagnostics`

**Status:** HOLDS — verified by running against well-formatted prose

---

### I-2: A perfectly aligned box produces zero diagnostics

**Claim:** A box where all content rows have the same visual width as the border,
and all `|`/`│` characters appear at exactly the junction column positions, produces
zero `ascii_box_*` diagnostics regardless of cell content.

**Why it matters:** The detector must be correct on valid input before being trusted on invalid input.

**Test:** `tests/integration_tests.rs::perfect_box_zero_diagnostics`
         `src/checks/ascii_box::tests::perfect_box_no_errors`

**Status:** HOLDS

---

### I-3: Every diagnostic has a valid location

**Claim:** Every `Diagnostic` returned by any check has:
- `span.line ≥ 1` (1-based)
- `span.col ≥ 1` (1-based)
- `file` pointing to the file being checked

**Why it matters:** An editor integration that receives `line=0` will crash or display
the diagnostic in the wrong place.

**Test:** Not yet written — add assertion to integration tests that verify span validity
on all diagnostics from fixture files.

**Status:** BELIEVED — no known violations, but no explicit test.
**Action:** Write a test that asserts `d.span.line >= 1 && d.span.col >= 1` for all diagnostics.

---

### I-4: Linting is deterministic

**Claim:** Linting the same file twice, in the same process run, produces identical diagnostics
in identical order.

**Why it matters:** Non-determinism causes CI flakiness and makes debugging impossible.

**Note:** Parallel file processing (rayon) produces non-deterministic *file order* across
multiple files — that's acceptable. But linting a *single file* must be deterministic.

**Test:** Not yet written.

**Status:** BELIEVED — no mutable shared state in the per-file check path.

---

### I-5: Child config required sections are a superset of parent's

**Claim:** If the parent config requires `["Decision Cheat Sheet"]` and the child config
requires `["Type System Snapshot"]`, the effective config for a file under the child directory
requires `["Decision Cheat Sheet", "Type System Snapshot"]` — both.

**Why it matters:** The cascade's additive merge semantics must hold. A child config must
not be able to silently remove a parent's requirements.

**Test:** `tests/integration_tests.rs::markdown_required_section_missing` (partial)
**Action:** Write a cascade-specific test: load parent + child config, verify merged list.

**Status:** HOLDS by implementation in `merge_markdown()`

---

### I-6: Tolerance correctly bounds reported drift

**Claim:**
- `tolerance = 0` → any column drift ≥ 1 is reported
- `tolerance = N` → column drift ≤ N is suppressed; drift > N is reported

**Why it matters:** Tolerance is the primary mechanism for absorbing minor formatting
variation (trailing spaces, slightly different rendering). If it's off by one, authors
will either get false positives or miss real errors.

**Test:** Not yet written.

**Status:** BELIEVED — implemented via `c.abs_diff(expected_col) <= tolerance`

---

### I-7: Parallel and sequential produce the same diagnostic set

**Claim:** Running `runner.run()` (parallel, rayon) produces the same *set* of diagnostics
as running `runner.lint_file()` on each file sequentially. Order may differ; content must not.

**Why it matters:** rayon can introduce data races if any state is shared incorrectly.
The config cache uses `Arc<Mutex<...>>` — this must not cause missed diagnostics or
duplicates.

**Test:** Not yet written.

**Status:** BELIEVED — each file's lint path is independent.

---

### I-8: JSON output is always valid JSON

**Claim:** `proof --format json` always produces output that is parseable as a JSON array,
even when there are zero diagnostics (in which case it produces `[]`).

**Why it matters:** Any tool that consumes proof's JSON output will break if the output
is malformed.

**Test:** `tests/integration_tests.rs::binary_json_output_is_parseable`

**Status:** HOLDS — verified by E2E test

---

### I-9: Exit code reflects error presence

**Claim:**
- Exit code 0 → zero error-severity diagnostics (warnings don't count)
- Exit code 1 → at least one error-severity diagnostic
- `--no-fail` → always exit 0 regardless

**Test:** `tests/integration_tests.rs::binary_exits_zero_on_clean_file`
         `tests/integration_tests.rs::binary_exits_nonzero_on_errors`

**Status:** HOLDS

---

### I-10: Unicode and ASCII boxes are treated equivalently

**Claim:** A Unicode box `┌─┐\n│ x │\n└─┘` produces the same diagnostic behavior as
the equivalent ASCII box `+--+\n| x |\n+--+`. Alignment errors are detected in both;
correct boxes produce zero diagnostics in both.

**Why it matters:** Authors mix styles. The tool must not have a blind spot for either.

**Test:** `src/checks/ascii_box::tests::unicode_box_detected`
         `tests/integration_tests.rs::perfect_box_zero_diagnostics` (includes unicode box fixture)

**Status:** HOLDS

---

---

### I-14: A `└──┘` bottom-close border cannot be the top of a new box

**Claim:** A line whose first junction character is a bottom-left corner (`└`, `╚`, `╰`)
is never treated as the opening border of a new box. Stacked flowchart diagrams with
linear `└──┘ → ┌──┐` structure produce zero false phantom-box diagnostics.

**Why it matters:** Multi-box flowcharts are the primary diagram structure across the
entire reference library. Every landscape diagram uses stacked boxes. Without this
invariant, a guide's main diagram generates hundreds of false alignment errors,
drowning out real errors.

**Test:** `stacked_boxes_no_phantom_box_errors`, `bottom_close_border_not_treated_as_box_top`,
`bottom_left_corner_cannot_open_box` in `tests/integration_tests.rs`

**Status:** HOLDS — implemented via `can_open_box()` guard in `find_boxes()`

---

### I-15: Annotation text after a closing `│` is detected as a width error

**Claim:** If a content row contains text after the closing `│` delimiter
(e.g., `│ content │  ← annotation`), the row's visual width exceeds the box border
width and is reported as `ascii_box_width`. The annotation is not silently ignored.

**Why it matters:** Inline annotations are a common authoring error. The rule going
forward: annotations must appear outside the code block or in a dedicated cell with
a proper delimiter. proof enforces this mechanically.

**Test:** `annotation_after_closing_bar_detected` in `tests/integration_tests.rs`

**Status:** HOLDS — natural consequence of width comparison

---

## Invariant Health

| Invariant | Status | Has Test? |
|-----------|--------|-----------|
| I-1: No boxes → no box diagnostics | HOLDS | yes |
| I-2: Perfect box → zero diagnostics | HOLDS | yes |
| I-3: Valid span location | HOLDS | yes |
| I-4: Determinism | HOLDS | yes |
| I-5: Additive merge superset | HOLDS | yes |
| I-6: Tolerance bounds | HOLDS | yes |
| I-7: Parallel = sequential | HOLDS | yes |
| I-8: JSON validity | HOLDS | yes |
| I-9: Exit code | HOLDS | yes |
| I-10: Unicode = ASCII | HOLDS | yes |
| I-11: old_string mismatch skips fix | HOLDS | yes |
| I-12: dry-run makes zero writes | HOLDS | yes |
| I-13: Fix reverse line order | HOLDS | yes |
| I-14: Bottom-close border cannot open box | HOLDS | yes |
| I-15: Annotation after `│` detected as width error | HOLDS | yes |

**Action items:** Write tests for I-3, I-4, I-6, I-7. Strengthen I-5 with a cascade test.
