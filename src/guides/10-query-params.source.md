# proof md:// Query Parameters — Filter, Slice, Project Tables

Every proof directive that pulls data from a markdown table — `proof:chart`,
`proof:tree`, `proof:element`, `proof:row`, `proof:table`, `proof:include` —
accepts a query string on the URI to transform the resolved content before
the directive sees it. The query string follows the standard `?key=val&key=val`
form and applies after mdpath element extraction.

```
md://data.md#:table:0?filter=pos=F&top=3&select=name,goals
              ─────────              ──────                ─────────
              addressing             transforms            projection
```

The transforms compose in a fixed order regardless of how they appear in the
URI: **filter → skip → top → select → count**. Multiple `?filter=` terms
compose with AND.

---

## The reference table

Examples below use this 6-row fixture committed at `src/data/features.md`:

```proof:include
md://src/data/features.md#:table:0
```

To run the examples in your own corpus, point them at any markdown file with
a `#:table:N` element you can address.

---

## ?select — project columns

Drop columns you don't care about; keep ordering of the requested list. Use
this when a chart or element only needs two columns from a wide table.

```proof:table
md://src/data/features.md#:table:0?select=name,status
```

If a column you reference doesn't exist, compile fails fast with a clear
error naming the bad column — no silent column-mismatch.

---

## ?filter — keep rows that match

Equality, inequality, and numeric comparison are supported. The form is
`col=val`, `col!=val`, `col>val`, `col<val`. Numeric operators coerce both
sides to f64; equality is plain string compare.

Single filter — keep only stable items:

```proof:table
md://src/data/features.md#:table:0?filter=status=stable&select=name,category
```

Multiple filters compose with AND — repeat the `?filter=` key:

```proof:table
md://src/data/features.md#:table:0?filter=status=stable&filter=category=elements&select=name,directive
```

Numeric comparison — useful when the value column carries a count or score:

```
md://stats.md#:table:0?filter=goals>50
```

---

## ?top and ?skip — slice rows

`?top=N` keeps the first N rows. `?skip=N` drops the first N. They compose
into SQL-style paging when used together (skip first, then top):

```proof:table
md://src/data/features.md#:table:0?skip=2&top=3&select=name
```

Skip past the first two rows, then keep the next three.

---

## ?count — replace with a single-cell row count

`?count` replaces the entire result with a one-cell synthetic table holding
the row count. Useful when feeding `proof:element kind=value` from a count:

```proof:table
md://src/data/features.md#:table:0?filter=category=math&count
```

The synthetic table looks like `| count |\n|-------|\n| 2 |`.

---

## Composition example

A chart that shows only the top three stable elements by category — assuming
your data table has a numeric `score` or `count` column to chart:

```text
proof:chart kind=bar width=60 label-field=name value-field=score
            source=md://stats.md#:table:0?filter=status=stable&filter=category=elements&top=3
```

The transform pipeline filters the table to stable elements, takes the first
three matching rows, then hands those rows to the chart renderer with full
columns intact. `?select` would also work but the chart only consumes the
two named fields anyway.

---

## Where the transforms apply

The query string runs at the URI-resolution layer, so it works for **every**
md:// consumer in proof, not just one directive:

| Directive | URI path | Notes |
|-----------|----------|-------|
| `proof:chart` | `source=md://...?...` | filter rows before charting |
| `proof:tree` | `source=md://...?...` | drop rows from org/taxonomy/dependency tables |
| `proof:element` | `source=md://...?...` | with `?count` to feed a numeric value |
| `proof:row` | `source=md://...?...` | filter rows before per-row layout |
| `proof:table` | body URI `?...` | filter the embedded table itself |
| `proof:include` | inline `pin=md://...?...` | rare; mostly applies to data files |

The Tier-2 resolve cache keys on the *clean* URI (without the query string),
so multiple queries against the same source share a single cache entry —
filter doesn't re-read the file.

---

## Error handling

Common errors and what triggers them:

| Message | Cause |
|---------|-------|
| `?select references unknown column "foo"` | Column name typo or missing column in source |
| `?filter references unknown column "foo"` | Same as above for filter terms |
| `invalid ?filter term "foo" — expected col=val, col!=val, col>val, or col<val` | Filter term has no operator |
| `?top value must be a non-negative integer` | `?top=foo` — value didn't parse as usize |
| `?skip value must be a non-negative integer` | Same as above for skip |

All of these surface as `COMPILE-002` errors at the directive's source line.
