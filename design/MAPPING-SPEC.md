# proof field mapping — Shared Data Binding System

> **Status**: ✅ Implemented. `FieldMap`, `parse_md_table`, `parse_json_source` live in `src/tree/schema.rs`. URI query parameters all live: `?select=cols`, `?filter=col=val|col!=val|col>val|col<val` (compose with AND when repeated), `?count` (single-cell row-count synthetic table), `?top=N` and `?skip=N` (compose for paging — skip then top).

---

## What it is

Field mapping is the system by which proof directives bind to source data.
Every directive that consumes external data — `proof:chart`, `proof:tree`,
`proof:element`, `proof:row`, `proof:region` — uses the same field mapping
primitives described here.

Individual specs declare **which roles** they need (e.g. `tree/org` needs a
`name` role and a `parent` role). This spec defines how those roles are resolved
from any source format.

---

## Source formats

| Format | How to address | Detection |
|--------|----------------|-----------|
| GFM markdown table | `md://path.md#section:table:N` | default when content has `\|` header |
| JSON array of objects | `md://path.json` or `format=json` | `.json` extension or explicit `format=json` |
| YAML sequence | `md://path.yaml` (future) | `.yaml` extension |
| CSV | `md://path.csv` (future) | `.csv` extension |
| Inline values | `value="literal"` attribute | `value=` present |

The source format is auto-detected from the URI extension. Override with
`format=json`, `format=table`, etc. in the directive attributes.

---

## Field reference syntax

### Explicit field mapping

Declare which source column/key maps to each role using the directive attribute:

```
proof:tree kind=org name="Employee" parent="Manager" label="Title"
```

```
proof:element kind=value field="Goals" format="{:.0}"
```

```
proof:chart kind=bar item="Team" value="Goals"
```

The role name (left side) is defined by the directive spec. The field name
(right side in quotes) is the exact column header or JSON key in the source.

### Auto-detection

When no explicit mapping is provided, proof tries candidate names
(case-insensitive, first match wins):

| Role | Candidates tried (in order) |
|------|----------------------------|
| `name` | `name`, `label`, `title`, `id`, `key` |
| `parent` | `parent`, `parent_id`, `reports_to`, `belongs_to`, `manager` |
| `value` | `value`, `val`, `score`, `count`, `amount`, `total` |
| `item` | `item`, `name`, `label`, `category`, `key` |
| `date` | `date`, `time`, `timestamp`, `period`, `month`, `year` |
| `series` | `series`, `group`, `category`, `type` |

**Override always wins.** If you declare `name="Employee"`, auto-detection is
skipped for that role.

### Row binding (`proof:row source=`)

In `proof:row`, `field=X` refers to the column `X` of the **current row** in
the `proof:row` loop:

```
proof:row source=md://stats.md#edm:table:0
  proof:element kind=label field=name width=24
  proof:element kind=value field=pts_82 width=6
```

Each iteration binds the current row as an implicit context. `field=name` extracts
the value of the `name` column in the current row.
`field=` always refers to the current iteration's row — no variable name needed.

---

## Query parameters

`md://` URIs support query parameters for filtering and column selection:

| Parameter | Syntax | Effect |
|-----------|--------|--------|
| `?select=A,B,C` | column list | Return only the specified columns |
| `?filter=col op val` | predicate | Filter rows (see operators below) |
| `?top=N` | integer | Return first N rows |
| `?skip=N` | integer | Skip first N rows |
| `?sort=col` | column name | Sort ascending by column |
| `?sort=col desc` | column + desc | Sort descending |
| `?count` | flag | Return row count as scalar |

**Filter operators:**

| Operator | Example | Meaning |
|----------|---------|---------|
| `eq` / `=` | `?filter=type eq UFA` | Equality |
| `ne` / `!=` | `?filter=status ne injured` | Not equal |
| `gt` / `>` | `?filter=pts gt 80` | Greater than |
| `lt` / `<` | `?filter=pts lt 20` | Less than |
| `gte` / `>=` | `?filter=age gte 30` | Greater than or equal |
| `lte` / `<=` | `?filter=age lte 25` | Less than or equal |
| `contains` | `?filter=name contains McDavid` | Substring match |
| `startswith` | `?filter=team startswith EDM` | Prefix match |

Multiple filters: `?filter=pts gt 80&filter=status eq active`

---

## Row selectors

After the URI, `[row=N]` or `[row=label]` selects a single row for scalar extraction:

```
md://stats.md#edm:table:0[row=0]           # first row (0-indexed)
md://stats.md#edm:table:0[row=McDavid]     # row where key column = "McDavid"
md://stats.md#edm:table:0[row=-1]          # last row
```

Combined with `field=` to extract a single scalar:

```
proof:element kind=value field=pts_82
md://stats.md#edm:table:0[row=McDavid]
```

This resolves `pts_82` from the row where the key column equals "McDavid".

---

## Root markers (for hierarchical kinds)

For `tree/org`, `tree/taxonomy`, and similar kinds where one row is the root,
the root is identified by a special parent value. Auto-detected markers:

`—`, `-`, `null`, `none`, `` (empty), `0`, `root`

Override: `root-marker="ROOT"` in the directive attributes.

---

## Numeric formatting

The `format=` attribute uses Rust `std::fmt` specifiers:

| Format | Example output | Use case |
|--------|---------------|----------|
| `{}` | `138` | Default (integer-like) |
| `{:.1}` | `138.0` | 1 decimal place |
| `{:.2}` | `138.04` | 2 decimal places |
| `{:+.2}` | `+0.19` or `-0.12` | Delta with sign |
| `{:>6.1}` | `  138.0` | Right-aligned, width 6 |
| `{:06.1}` | `0138.0` | Zero-padded |
| `{:.0}%` | `72%` | Percentage (no decimal) |

---

## Type coercion

When a field value is extracted, proof coerces it:

| Target kind | Coercion |
|------------|---------|
| `value` / `delta` | Parse as `f64`. If unparseable, emit `MAPPING-003`. |
| `label` / `badge` | Treat as string. No coercion. |
| `sparkline` series | Split on `,` or read multi-row column. Parse each as `f64`. |
| `mini-bar` | Parse as `f64`. |

---

## Mapping in each spec

Each directive spec declares which roles it uses and their semantics:

| Spec | Roles | Required | Optional |
|------|-------|----------|---------|
| `chart/bar` | `item`, `value`, `max` | `item`, `value` | `max` |
| `chart/line` | `x`, `y`, `series`, `label` | `x`, `y` | `series`, `label` |
| `chart/scatter` | `x`, `y`, `series`, `label` | `x`, `y` | `series`, `label` |
| `chart/heatmap` | row headers, column headers, value | all from matrix | — |
| `chart/timeline` | `date`, `event`, `label` | `date`, `event` | `label` |
| `chart/gantt` | `task`, `start`, `end`, `status` | `task`, `start`, `end` | `status` |
| `tree/org` | `name`, `parent`, `label` | `name`, `parent` | `label` |
| `tree/taxonomy` | `name`, `parent`, `level` | `name`, `parent` | `level` |
| `tree/dependency` | `name`, `parent`, `version` | `name`, `parent` | `version` |
| `tree/decision` | `node`, `yes`, `no` | `node`, `yes`, `no` | `label` |
| `element/value` | `field` | `field` | — |
| `element/delta` | `field` | `field` | — |
| `element/sparkline` | `field` (series) | `field` | — |
| `element/mini-bar` | `field`, `max` | `field` | `max` |
| `element/label` | `field` | `field` | `style` (default\|badge) |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `MAPPING-001` | error | Required field/role not found in source — specify explicitly with `field="ColName"` |
| `MAPPING-002` | error | Source URI resolved to empty or missing table |
| `MAPPING-003` | error | Field value cannot be coerced to numeric (for value/delta/mini-bar/sparkline) |
| `MAPPING-004` | warning | Multiple columns matched auto-detection — using first match; specify `field=` to be explicit |
| `MAPPING-005` | error | Row selector `[row=X]` matched no rows |
| `MAPPING-006` | warning | Filter matched 0 rows — source resolved to empty |
| `MAPPING-007` | error | `proof:row source=` loop source has no rows — nothing to render |

---

## Implementation

### Existing (Wave 3, tree/schema.rs)

- `FieldMap` struct with explicit + auto-detect
- `parse_md_table()` and `parse_json_source()`
- Per-kind `auto_detect_*()` functions
- `DEFAULT_ROOT_MARKERS` for hierarchical kinds

### Needed (future waves)

- `MappingContext` — unified field resolution for all directive types
- Query parameter parser (`?select=`, `?filter=`, `?top=`, `?sort=`)
- Row selector parser (`[row=N]`, `[row=label]`)
- Type coercion (`f64`, string, series)
- row source binding for `proof:row`

### File layout

```
src/mapping/
  mod.rs        — pub re-exports
  source.rs     — parse_md_table(), parse_json_source(), parse_csv()
  field.rs      — FieldMap, auto_detect_*, role resolution
  query.rs      — ?filter=, ?select=, ?sort=, ?top=, ?skip= parsing
  selector.rs   — [row=N], [row=label] parsing
  coerce.rs     — type coercion (string → f64, etc.)
  row.rs      — source binding + field resolution for proof:row
```

---

## See also

- [Chart Spec](./chart-spec.md) — field roles for chart kinds
- [Tree Spec](./tree-spec.md) — field roles for tree kinds (org, taxonomy, etc.)
- [Element Spec](./element-spec.md) — field roles for element kinds
- [Dashboard Spec](./dashboard-spec.md) — proof:row source= and region content
- [mdpath](../../mdpath/design/SPEC.md) — URI scheme (the addressing layer above mapping)
