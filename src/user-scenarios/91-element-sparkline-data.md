# US-91 — Sparkline element with inline value series

A sparkline rendered from a comma-separated number list passed via
`value="..."`. The element's parser splits on commas and renders one glyph
per number.

<!-- proof:compiled from="proof:element" uri="inline" -->
```
▂▃▁▆▇██▂▃▁▆▇██▂▃▁▆▇█
```
<!-- /proof:compiled -->

For sparklines driven from a data table, wrap `proof:element kind=sparkline
field=col` inside a `proof:row source=md://...` (see US-95).
