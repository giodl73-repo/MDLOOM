# US-98 — Include with inline pin reference

```proof:include pin=test-fig
md://src/data/diagnostic-codes.md#:table:0
```

The `pin=test-fig` attribute references a `[[davinci]]` entry in proof.toml
that may or may not exist. If it doesn't, proof emits a warning at compile
time alerting the author to register a pin if they want invariant
protection on this figure.
