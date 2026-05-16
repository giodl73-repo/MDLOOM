# Wave: SLICE Examples

## Goal

Show how PROOF can apply SLICE selectors to prepared artifact/report rows without
moving compilation or rendering policy into SLICE.

## Pulse table

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Artifact selector examples | done | Added dev-only SLICE tests over `.proof/artifacts.json`-shaped rows. |

## Success criteria

- SLICE is dev/example-only.
- PROOF keeps source fidelity, directives, compile graph, artifacts, and
  rendering.
- Examples cover artifact status and diagnostic filters.
- `cargo test --test slice_artifact_selector` passes.
