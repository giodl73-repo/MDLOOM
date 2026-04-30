# US-104 — DaVinci regex invariant in action

This scenario assumes a `[[davinci]]` entry in proof.toml with a regex
invariant on a figure file. The figure must match the pattern; otherwise
`fig_invariant_violated` fires.

```proof:include pin=status-pinned
md://src/data/diagnostic-codes.md#:table:0
```

If the pin's regex invariant is `^\| Code \|` (header row check) and the
figure starts with that pattern, lint passes. If someone reorders or
removes the header row, lint catches it before merge.
