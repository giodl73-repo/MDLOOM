# proof Elements — Numeric Displays and Badges

Elements are fixed-width data cells — the building block for terminal dashboards,
status boards, and data-rich documentation. The key constraint: every element
has an exact `width` in character columns. This makes them composable: put them
side by side and they align perfectly regardless of content.

The six kinds cover the most common data display patterns. `value` and `delta`
show scalar numbers with optional sign and direction. `sparkline` and `mini-bar`
show distributions visually. `label` and `badge` show text. Combine them with
`proof:row` to render a whole table as aligned column blocks.

---

## Element kinds

### value — scalar number with optional label

Use `value` for any single metric: a count, a rate, a percentage. The `label`
attribute adds a caption below the value. proof accepts formatted numbers
(`"1,024"`, `"99.9%"`) as display strings — use these when the formatting
matters more than the numeric precision.

```proof:element kind=value value="42.7" width=8
```

```proof:element kind=value value="1,024" label="requests/sec" width=16
```

```proof:element kind=value value="99.9%" label="uptime" width=14
```

### delta — change with direction arrow

Use `delta` to show movement: how a metric changed since last period. proof
formats the value with an explicit sign and renders an arrow indicating direction.
Use `+` or `-` prefix in the value string to control sign display.

```proof:element kind=delta value="+12.3" width=10
```

```proof:element kind=delta value="-5.1" width=10
```

```proof:element kind=delta value="0.0" width=10
```

### sparkline — trend line from data points

A sparkline turns a comma-separated list of values into a miniature ASCII line
chart using block characters (▁▂▃▄▅▆▇█). The values are normalized to the
local min/max — sparklines show the shape of a trend, not absolute values. Use
them for time series, performance histories, or any sequence where you want to
see the pattern at a glance.

```proof:element kind=sparkline value="1,3,2,5,4,7,6,9,8,10" width=14
```

```proof:element kind=sparkline value="10,8,6,4,2,1" width=12
```

```proof:element kind=sparkline value="5,5,5,5,5,5" width=12
```

### mini-bar — horizontal progress bar

A mini-bar shows a single value as a filled bar within a range. Unlike a
sparkline (which shows a series), a mini-bar shows one number's position in a
scale — useful for progress, capacity, or comparison against a maximum.
The `width` controls the total width including the bar characters.

```proof:element kind=mini-bar value="75" width=20
```

```proof:element kind=mini-bar value="30" width=20
```

```proof:element kind=mini-bar value="100" width=20
```

### label — plain text cell

Label is the simplest element: text padded or truncated to exactly `width`
columns. Use it for status strings, category names, or any text that needs to
align with other elements in a column layout.

```proof:element kind=label value="PASSING" width=12
```

```proof:element kind=label value="FAILING" width=12
```

```proof:element kind=label value="PENDING" width=12
```

### badge — bracketed tag

Badge wraps text in brackets, making it visually distinct from plain labels.
Good for version numbers, tags, status codes, and any short string that needs
visual separation from surrounding content.

```proof:element kind=badge value="v0.5.0" width=10
```

```proof:element kind=badge value="stable" width=10
```

```proof:element kind=badge value="beta" width=10
```

---

## proof:row — column-aligned rows from data

`proof:row` is where elements become truly powerful. Instead of repeating
individual `proof:element` blocks, you declare a template with one element per
column, then proof iterates over every row in a data table and renders a
column-pinned line for each one.

This is how you build status dashboards, feature comparison tables, or any
view where the same structure repeats across data. The `separator` attribute
controls what goes between columns — `" │ "` gives you a visual column divider.

```proof:row source=md://src/data/features.md foreach=row separator=" │ "
proof:element kind=label field=name width=32
proof:element kind=badge field=status width=10
proof:element kind=label field=category width=12
```

---

## Width budgeting

Every element has an exact visual width. When composing a row, the total width
is the sum of all element widths plus separators. Plan your column widths to
fit within the available terminal or slide width.

A typical 80-column row with three elements and `" │ "` separators:

```proof:element kind=value value="128" label="LOC" width=12
```
```proof:element kind=value value="640" label="tests" width=12
```
```proof:element kind=value value="100%" label="passing" width=12
```

Three 12-wide elements + two 3-wide separators = 42 columns. Leaves room for
a label column.

---

## Element attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `kind` | yes | `value`, `delta`, `sparkline`, `mini-bar`, `label`, `badge` |
| `value` | one of | Literal value — number, string, or comma-separated series |
| `field` | one of | Column name from a `source` data table |
| `source` | if field | `md://` URI of the data table |
| `width` | yes | Output column width in visual characters |
| `label` | no | Caption below value (value/delta/mini-bar only) |
| `align` | no | `left`, `center`, `right` (default: right for numbers) |
