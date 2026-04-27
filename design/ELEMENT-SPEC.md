# proof element — Micro-Element Primitives

> **Status**: Design — not yet implemented.

---

## What it is

`proof:element` is the smallest embeddable visual unit in a compiled document.
Unlike `proof:chart` or `proof:tree` (which produce full figures), an element
renders a **single primitive** — a number, a sparkline, a bar, a delta — with
an exact character budget and zero chrome by default.

Elements are designed to embed inside GFM table cells, dashboard regions, and
`proof:row` compositors. The `no-chrome` flag strips ALL framing so the output
is raw characters only.

---

## Element kinds

| Kind | Output example | Min width | Source |
|------|---------------|-----------|--------|
| `value` | `138.0` | 1 | single scalar from table cell |
| `delta` | `+12.3` or `−4.1` | 1 | scalar, rendered with sign |
| `sparkline` | `▁▂▅▇█▆▄` | 1 | series column from table |
| `mini-bar` | `████░░░` | 1 | scalar + max → filled/empty bars |
| `label` | `Connor McDavid` | 1 | string from table cell |
| `badge` | `UFA` | 1 | enumerated string, styled |

---

## Directive syntax

```
```proof:element kind=sparkline width=10 no-chrome
md://stats/2025.md#mcdavid:table:0?select=season,ppg
```
```

```
```proof:element kind=value field=pace_82 format="{:.1}" width=6 no-chrome
md://stats/2025.md#mcdavid:table:0[row=0]
```
```

```
```proof:element kind=mini-bar field=pts_82 max=200 width=20 no-chrome
md://stats/2025.md#mcdavid:table:0[row=0]
```
```

---

## Attributes

| Attribute | Kinds | Default | Description |
|-----------|-------|---------|-------------|
| `kind` | all | required | `value`, `delta`, `sparkline`, `mini-bar`, `label`, `badge` |
| `width` | all | auto | Exact character budget. Element MUST fit in N chars |
| `height` | all | 1 | Line budget (micro-elements default to 1 line) |
| `no-chrome` | all | false | Strip all framing: no fence, label, axis, title. Output is raw chars |
| `field` | value, delta, mini-bar, label, badge | — | Column/key to extract from source row |
| `format` | value, delta | `{}` | Rust-style format string for numerics (`{:.1}`, `{:+.2}`) |
| `align` | all | left | `left`, `right`, `center` within width budget |
| `max` | mini-bar | auto | Scale reference for bar fill (default: max across all rows) |
| `fill` | mini-bar, sparkline | `█`, `░` | Fill and empty characters |
| `value` | label, badge | — | Inline literal value (alternative to source URI) |

---

## Kind specifications

### `value`

Extracts a single numeric or string scalar. Formatted with `format=` and padded/truncated to `width`.

```
proof:element kind=value field=pts_82 format="{:.1}" width=6 align=right
→ " 138.0"
```

### `delta`

Like `value` but always renders with a sign prefix. Positive = `+`, negative = `−` (U+2212).

```
proof:element kind=delta field=improvement format="{:+.2}" width=6
→ "+0.19"
```

### `sparkline`

Renders a series of values as 8-level block characters `▁▂▃▄▅▆▇█`.
Source must resolve to a sequence of numeric values (a column of a table or a series).

```
proof:element kind=sparkline width=10 field=ppg
→ "▁▂▅▇█▆▄▃▂▄"
```

Encoding: min → `▁`, max → `█`, linear interpolation for intermediate values.
If `width < series_length`, values are aggregated (mean per bucket).
If `width > series_length`, values are repeated to fill.

### `mini-bar`

Renders a horizontal bar proportional to `field / max`. Fill chars: `█` for filled, `░` for empty.

```
proof:element kind=mini-bar field=pts_82 max=200 width=20 no-chrome
→ "█████████████░░░░░░░"
```

### `label`

Extracts a string value, truncated to `width` with `…` if needed.

```
proof:element kind=label field=name width=24 align=left
→ "Connor McDavid          "
```

### `badge`

Like `label` but for short enumerated strings (`UFA`, `RFA`, `ELC`, `LTIR`, `NTC`).
Right-padded to `width` with spaces. Future: color coding via ANSI when terminal supports it.

```
proof:element kind=badge field=expiry_type width=5
→ "UFA  "
```

---

## `no-chrome` mode

When `no-chrome` is set, the output is:
- No enclosing ` ``` ` fence
- No `<!-- proof:compiled from=... -->` traceability comment
- No labels, axes, titles, or padding beyond `width`
- Raw characters only — suitable for embedding inside a GFM table cell or `proof:row`

Without `no-chrome` (default): the element is wrapped in a fenced code block with a traceability comment, same as `proof:include`.

---

## Embedding in GFM table cells

With `no-chrome`, elements can appear inline. In a `.source.md`:

```markdown
| Player | Pts/82 | Trend | Contract |
|--------|--------|-------|---------|
| ```proof:element kind=label field=name width=20 no-chrome\nmd://stats.md#:0[row=0]\n``` | ```proof:element kind=value field=pts_82 format="{:.1}" width=6 no-chrome\nmd://stats.md#:0[row=0]\n``` | ... | ... |
```

In practice, `proof:row` is a cleaner way to build table rows — see DASHBOARD-SPEC.md.

---

## `proof:row` — horizontal compositor

`proof:row` renders a single horizontal line of elements side-by-side with column pinning.

```
```proof:row foreach=player in md://stats/2025.md#edm:table:0 separator=" "
proof:element kind=label field=name width=24 align=left
proof:element kind=value field=pts_82 format="{:.1}" width=6 align=right
proof:element kind=mini-bar field=pts_82 max=200 width=20 no-chrome
proof:element kind=sparkline width=10 no-chrome field=career_arc
proof:element kind=badge field=expiry_type width=5
proof:element kind=delta field=improvement format="{:+.2}" width=6
```
```

- `foreach` — iterate over rows of a source table, emit one line per row
- Each element gets its exact `width` budget — no overflow permitted
- `separator` — character(s) between elements (default: single space)
- Column positions are pinned: element N always starts at the sum of widths 1..N-1

**Invariant R-1**: sum of element widths + separators must equal the declared row width.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| E-1 | Output character count = `width` (padded or truncated) |
| E-2 | `kind=value` / `kind=delta` resolve to a scalar, not a list |
| E-3 | `kind=sparkline` min → `▁`, max → `█`, 8 levels total |
| E-4 | `kind=mini-bar` fill proportion = `field / max` within ±1 char |
| E-5 | `no-chrome` output contains no fence lines and no HTML comments |
| E-6 | `align=right` right-aligns within `width`; `align=center` centers (tie-break: extra space on right) |
| R-1 | `proof:row` total column widths + separators = declared row width |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `ELEMENT-001` | error | `width` exceeded — output would be wider than declared budget |
| `ELEMENT-002` | error | `kind=value` resolved to non-scalar (list or empty) |
| `ELEMENT-003` | warning | `kind=sparkline` series has fewer values than `width` — values repeated |
| `ELEMENT-004` | error | `proof:row` column widths + separators ≠ declared row width |
| `ELEMENT-005` | error | `field` not found in source table |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/element/mod.rs` | Element rendering engine |
| `src/element/sparkline.rs` | 8-level block char encoding |
| `src/element/mini_bar.rs` | Proportional bar rendering |
| `src/element/row.rs` | proof:row compositor |
| `src/compile.rs` | proof:element + proof:row directive handling |

---

## See also

- [Dashboard Spec](./dashboard-spec.md) — canvas compositor using proof:element primitives
- [Chart Spec](./chart-spec.md) — full-figure charts (use proof:element for inline micro-charts)
- [Compile Spec](./compile-spec.md) — compilation pipeline
