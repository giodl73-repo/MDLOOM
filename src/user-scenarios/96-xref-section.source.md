# US-96 — Cross-reference into another file

Document with a `mdloom:xref` pointing at a heading inside the testing
guide. The directive resolves the heading text from the linked file at
compile time.

See ```mdloom:xref uri=md://docs/guides/08-lint.md``` for the full lint reference.

Or with a section-anchor:

```mdloom:xref uri=md://docs/guides/08-lint.md#ascii-art-checks
```
