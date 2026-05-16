# SLICE selector examples

PROOF can use SLICE over prepared report rows after compilation or CROP side-info
loading. SLICE should not enter the compile graph or rendering pipeline.

## Boundary

- PROOF owns source fidelity, directives, compile targets, artifact manifests,
  diagnostics, Markdown/HTML/Pebble rendering, and CROP wrapper commands.
- SLICE owns selector parsing, typed field catalogs, requirements, diagnostics,
  and row predicate evaluation.
- An adapter projects PROOF-owned structs into rows. Selection happens after
  those rows already exist.

## Artifact rows

Example selector:

```text
target eq 'html' and status eq 'written' and diagnostics_count eq 0
```

The checked test `tests/slice_artifact_selector.rs` demonstrates selection over
`.proof/artifacts.json`-shaped rows without adding selector semantics to the
artifact manifest schema.
