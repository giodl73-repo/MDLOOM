# mdloom md:// Query Parameters — Filter, Slice, Project Tables

Every mdloom directive that pulls data from a markdown table — `mdloom:chart`,
`mdloom:tree`, `mdloom:element`, `mdloom:row`, `mdloom:table`, `mdloom:include` —
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

<!-- mdloom:compiled from="md://src/data/features.md#:table:0" -->
```
name | category | status | directive | output
------ | ---------- | -------- | ----------- | --------
LaTeX math inline | math | stable | $...$ | inline Unicode
LaTeX math display | math | stable | mdloom:math | multi-line ASCII art
Symbol expansion | symbols | stable | [sym:name] | Unicode glyph
Symbol block | symbols | stable | mdloom:symbol | ASCII art block
Shape renderer | symbols | stable | mdloom:shape | ASCII art shape
Element value | elements | stable | mdloom:element kind=value | numeric cell
Element delta | elements | stable | mdloom:element kind=delta | delta with arrow
Element sparkline | elements | stable | mdloom:element kind=sparkline | ASCII sparkline
Element mini-bar | elements | stable | mdloom:element kind=mini-bar | ASCII bar chart
Element label | elements | stable | mdloom:element kind=label | text label
Element badge | elements | stable | mdloom:element kind=badge | bracketed badge
Row compositor | elements | stable | mdloom:row | column-pinned row
Slide title | slides | stable | mdloom:slide layout=title | title card
Slide title-content | slides | stable | mdloom:slide layout=title-content | two-zone slide
Slide two-column | slides | stable | mdloom:slide layout=two-column | split layout
Slide section | slides | stable | mdloom:slide layout=section | section divider
Slide stats | slides | stable | mdloom:slide layout=stats | stat row
Slide blank | slides | stable | mdloom:slide layout=blank | empty canvas
Slide bullets | slides | stable | mdloom:bullets | bullet list
Slide callout | slides | stable | mdloom:callout | callout box
Slide divider | slides | stable | mdloom:divider | horizontal rule
Slide quote | slides | stable | mdloom:quote | attributed quote
Slide centered | slides | stable | mdloom:centered | centered text
Dashboard canvas | dashboard | stable | mdloom:region | canvas grid
Tree dirtree | trees | stable | mdloom:tree kind=dirtree | filesystem tree
Tree org | trees | stable | mdloom:tree kind=org | org chart
Tree taxonomy | trees | stable | mdloom:tree kind=taxonomy | taxonomy tree
Tree dependency | trees | stable | mdloom:tree kind=dependency | dependency graph
Tree outline | trees | stable | mdloom:tree kind=outline | numbered outline
Figure import | figures | beta | mdloom:include kind=figure | ASCII image
DaVinci pin | figures | beta | mdloom pin | invariant storage
Lint check | linting | stable | mdloom check | diagnostic report
Auto-fix | linting | stable | mdloom fix | patched files
Compile pipeline | compile | stable | mdloom compile | resolved output
```
<!-- /mdloom:compiled -->

To run the examples in your own corpus, point them at any markdown file with
a `#:table:N` element you can address.

---

## ?select — project columns

Drop columns you don't care about; keep ordering of the requested list. Use
this when a chart or element only needs two columns from a wide table.

<!-- mdloom:compiled from="md://src/data/features.md#:table:0?select=name,status" -->
```
| name | status |
|---|---|
| LaTeX math inline | stable |
| LaTeX math display | stable |
| Symbol expansion | stable |
| Symbol block | stable |
| Shape renderer | stable |
| Element value | stable |
| Element delta | stable |
| Element sparkline | stable |
| Element mini-bar | stable |
| Element label | stable |
| Element badge | stable |
| Row compositor | stable |
| Slide title | stable |
| Slide title-content | stable |
| Slide two-column | stable |
| Slide section | stable |
| Slide stats | stable |
| Slide blank | stable |
| Slide bullets | stable |
| Slide callout | stable |
| Slide divider | stable |
| Slide quote | stable |
| Slide centered | stable |
| Dashboard canvas | stable |
| Tree dirtree | stable |
| Tree org | stable |
| Tree taxonomy | stable |
| Tree dependency | stable |
| Tree outline | stable |
| Figure import | beta |
| DaVinci pin | beta |
| Lint check | stable |
| Auto-fix | stable |
| Compile pipeline | stable |
```
<!-- /mdloom:compiled -->

If a column you reference doesn't exist, compile fails fast with a clear
error naming the bad column — no silent column-mismatch.

---

## ?filter — keep rows that match

Equality, inequality, and numeric comparison are supported. The form is
`col=val`, `col!=val`, `col>val`, `col<val`. Numeric operators coerce both
sides to f64; equality is plain string compare.

Single filter — keep only stable items:

<!-- mdloom:compiled from="md://src/data/features.md#:table:0?filter=status=stable&select=name,category" -->
```
| name | category |
|---|---|
| LaTeX math inline | math |
| LaTeX math display | math |
| Symbol expansion | symbols |
| Symbol block | symbols |
| Shape renderer | symbols |
| Element value | elements |
| Element delta | elements |
| Element sparkline | elements |
| Element mini-bar | elements |
| Element label | elements |
| Element badge | elements |
| Row compositor | elements |
| Slide title | slides |
| Slide title-content | slides |
| Slide two-column | slides |
| Slide section | slides |
| Slide stats | slides |
| Slide blank | slides |
| Slide bullets | slides |
| Slide callout | slides |
| Slide divider | slides |
| Slide quote | slides |
| Slide centered | slides |
| Dashboard canvas | dashboard |
| Tree dirtree | trees |
| Tree org | trees |
| Tree taxonomy | trees |
| Tree dependency | trees |
| Tree outline | trees |
| Lint check | linting |
| Auto-fix | linting |
| Compile pipeline | compile |
```
<!-- /mdloom:compiled -->

Multiple filters compose with AND — repeat the `?filter=` key:

<!-- mdloom:compiled from="md://src/data/features.md#:table:0?filter=status=stable&filter=category=elements&select=name,directive" -->
```
| name | directive |
|---|---|
| Element value | mdloom:element kind=value |
| Element delta | mdloom:element kind=delta |
| Element sparkline | mdloom:element kind=sparkline |
| Element mini-bar | mdloom:element kind=mini-bar |
| Element label | mdloom:element kind=label |
| Element badge | mdloom:element kind=badge |
| Row compositor | mdloom:row |
```
<!-- /mdloom:compiled -->

Numeric comparison — useful when the value column carries a count or score:

```
md://stats.md#:table:0?filter=goals>50
```

---

## ?top and ?skip — slice rows

`?top=N` keeps the first N rows. `?skip=N` drops the first N. They compose
into SQL-style paging when used together (skip first, then top):

<!-- mdloom:compiled from="md://src/data/features.md#:table:0?skip=2&top=3&select=name" -->
```
| name |
|---|
| Symbol expansion |
| Symbol block |
| Shape renderer |
```
<!-- /mdloom:compiled -->

Skip past the first two rows, then keep the next three.

---

## ?count — replace with a single-cell row count

`?count` replaces the entire result with a one-cell synthetic table holding
the row count. Useful when feeding `mdloom:element kind=value` from a count:

<!-- mdloom:compiled from="md://src/data/features.md#:table:0?filter=category=math&count" -->
```
| name | category | status | directive | output |
|---|---|---|---|---|
| LaTeX math inline | math | stable | $...$ | inline Unicode |
| LaTeX math display | math | stable | mdloom:math | multi-line ASCII art |
```
<!-- /mdloom:compiled -->

The synthetic table looks like `| count |\n|-------|\n| 2 |`.

---

## Composition example

A chart that shows only the top three stable elements by category — assuming
your data table has a numeric `score` or `count` column to chart:

```text
mdloom:chart kind=bar width=60 label-field=name value-field=score
            source=md://stats.md#:table:0?filter=status=stable&filter=category=elements&top=3
```

The transform pipeline filters the table to stable elements, takes the first
three matching rows, then hands those rows to the chart renderer with full
columns intact. `?select` would also work but the chart only consumes the
two named fields anyway.

---

## Where the transforms apply

The query string runs at the URI-resolution layer, so it works for **every**
md:// consumer in mdloom, not just one directive:

| Directive | URI path | Notes |
|-----------|----------|-------|
| `mdloom:chart` | `source=md://...?...` | filter rows before charting |
| `mdloom:tree` | `source=md://...?...` | drop rows from org/taxonomy/dependency tables |
| `mdloom:element` | `source=md://...?...` | with `?count` to feed a numeric value |
| `mdloom:row` | `source=md://...?...` | filter rows before per-row layout |
| `mdloom:table` | body URI `?...` | filter the embedded table itself |
| `mdloom:include` | inline `pin=md://...?...` | rare; mostly applies to data files |

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
