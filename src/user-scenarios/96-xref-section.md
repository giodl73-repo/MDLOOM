# US-96 — Cross-reference into another file

Document with a `proof:xref` pointing at a heading inside the testing
guide. The directive resolves the heading text from the linked file at
compile time.

See ```proof:xref uri=md://docs/guides/08-lint.md``` for the full lint reference.

Or with a section-anchor:

<!-- proof:compiled from="proof:xref" -->
*See: [ascii art checks](docs/guides/08-lint.md#ascii-art-checks)*
<!-- /proof:compiled -->
