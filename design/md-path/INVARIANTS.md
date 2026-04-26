# md-path Invariants

Properties that must hold for all inputs, at all times. A change that
violates an invariant is a regression.

---

## I-1: Determinism — same URI always resolves to same element

**Claim:** Resolving the same `md://` URI twice against the same file content
always returns the same `ResolvedElement`. Resolution is a pure function of
(uri, file_content, root_dir).

**Why it matters:** If resolution is non-deterministic, DaVinci invariant checks
become unreliable — a passing check on run 1 could fail on run 2 with identical
files.

**Test:** Resolve the same URI 100 times against the same content, assert all results are identical.

---

## I-2: Named over numeric — proof never emits a numeric URI when a name exists

**Claim:** When proof generates an `md://` URI (for error output, `proof pin`,
fix plans), it always uses the named selector form if the element has a label.
A numeric URI is only emitted when no label can be detected.

**Why it matters:** Numeric URIs break when content is added above the element.
Named URIs are stable. Automatically emitting named URIs is the primary stability
mechanism.

**Test:** Pin a labeled figure → emitted URI contains the label. Pin an unlabeled
figure → emitted URI contains `:N`. Add label later → `proof pin --update` emits named form.

---

## I-3: Section uniqueness — ambiguous heading path is always an error

**Claim:** If two sections in a file normalize to the same heading anchor, ANY
URI that references that anchor produces `md_section_ambiguous`. The resolver
never silently picks one.

**Why it matters:** Silent resolution of ambiguous sections is a class of bug where
the wrong DaVinci invariant is enforced. Failing loudly is safer.

**Test:** File with two `## Overview` sections → any URI with `#overview` errors.

---

## I-4: Integer selectors are always 0-based

**Claim:** `:0` is always the first element, `:1` the second, etc. There is no
configuration or mode that changes this. The `:0` shorthand for "first figure"
is always equivalent to `:figure:0`.

**Why it matters:** A resolver that uses 1-based indexing in some contexts breaks
all numeric URIs from other tools.

**Test:** Resolve `:0` on a section with 3 figures — always returns the first one (line N).

---

## I-5: Percent-encoding is symmetric

**Claim:** Any `md://` URI produced by proof can be parsed back by the resolver
to the same (file, heading, type, kind, selector, sub-selector) tuple.
Encode(Decode(uri)) == uri for all valid URIs.

**Why it matters:** Round-trip stability is required for URIs stored in proof.toml
DaVinci entries, fix plans, and error output to remain valid across sessions.

**Test:** Generate URIs for elements with special chars in labels, encode them,
parse them, resolve them — must return same element as the unencoded form.

---

## I-6: Label detection priority is always inline → preceding → numeric

**Claim:** The three label detection rules are always applied in priority order:
(1) inline label inside fence with no language string, (2) preceding bold or
short text line before fence, (3) numeric fallback. A figure can never be
addressed by rule 2 if rule 1 applies.

**Why it matters:** If priority is inconsistent, the same figure gets different
URIs depending on when/how it's resolved. The URI must be stable.

**Test:** Figure with BOTH an inline label AND a preceding bold label → rule 1
wins, URI uses inline label, preceding label is ignored.

---

## I-7: Resolution is read-only

**Claim:** Resolving an `md://` URI never modifies any file. The resolver is a
pure read operation against the filesystem.

**Why it matters:** If resolution had side effects, running `proof check` or
`proof resolve` would corrupt documents. This must be guaranteed at the type level
— the resolver takes `&Path`, not `&mut Path`.

**Test:** Verify no files are modified after resolving any URI against any fixture.

---

## I-8: All md:// URIs are absolute from proof root

**Claim:** The `path` component in `md://path` is always relative to the proof
root (where proof.toml lives), never relative to the current working directory
or the file being checked. Two invocations of proof from different working
directories, pointing at the same root, resolve the same URIs identically.

**Why it matters:** URIs stored in proof.toml DaVinci entries must work regardless
of where proof is invoked from.

**Test:** Run `proof resolve md://computing/01-PACKAGE.md#...` from C:\, C:\src,
and C:\src\maxim — all return the same result when --config points to same root.

---

## I-9: Label pure-digit restriction

**Claim:** A label that is purely digits (matches `^\d+$`) is never registered
as a figure label. Labels must contain at least one non-digit character.
This eliminates the integer/string selector ambiguity (MP-04).

**Why it matters:** If a figure can be labeled "0", then `:figure:0` is ambiguous
between "first figure" and "figure named zero". The restriction makes `:0` always
an integer.

**Test:** Code block with inline label "0" → no label detected, numeric fallback used.
Code block with inline label "0-start" → label registered (contains non-digit).

---

## I-10: Sub-selector validity per type

**Claim:** The following composition matrix is enforced — invalid combinations
produce `md_invalid_subkey`:

| Type | Valid sub-selectors |
|------|---------------------|
| `figure` | `[box=...]`, `[row=N]` (line access) |
| `table` | `[row=...]`, `[col=...]`, `[row=...,col=...]` |
| `chart` | `[bar=...]` |
| `text` | *(none)* |
| `heading` | *(none)* |

**Why it matters:** Applying `[row=X]` to a text paragraph is semantically wrong
and will produce garbage results. The matrix must be enforced at parse time,
not discovered at runtime.

**Test:** `md://file.md#section:text:0[row=X]` → `md_invalid_subkey` error.
`md://file.md#section:heading[box=Y]` → `md_invalid_subkey` error.

---

## I-11: Query parameters apply only to collection types

**Claim:** `?select`, `?filter`, `?top`, `?skip` are only valid on `table` and
`chart` types. Applying them to `figure`, `text`, or `heading` produces
`md_invalid_query_on_type`. Exception: `?count` is valid on all types and returns
the count of matching elements.

**Why it matters:** `?filter=X eq Y` has no meaningful interpretation on a prose
paragraph or ASCII art diagram. Accepting it silently would produce confusing
behavior.

**Test:** `md://file.md#section:figure:0?filter=X eq Y` → `md_invalid_query_on_type`.
`md://file.md#section:figure?count` → integer count of figures.

---

## I-12: Consistency group members resolve independently

**Claim:** Resolving each URI in a `[[consistency-group]]` never affects the
resolution of any other URI in the same group. Consistency rules are validated
AFTER all members are independently resolved.

**Why it matters:** If member resolution was order-dependent or had side effects,
a consistency failure could be reported for the wrong element.

**Test:** Consistency group with 3 URIs where member 2 has a box-width mismatch —
members 1 and 3 must still resolve correctly before the comparison rule fires.
