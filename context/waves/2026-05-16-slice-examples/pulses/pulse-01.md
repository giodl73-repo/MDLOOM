# Pulse 01: Artifact selector examples

## Goal

Prove PROOF can use SLICE selectors over prepared artifact rows without adding
selector semantics to PROOF's compile or render pipeline.

## Changes

- Add a dev-only `slice-core` dependency.
- Add artifact-row selector tests for target/status/diagnostic filters.
- Document the adapter boundary in `docs/specs/slice-selectors.md`.

## Validation

- `cargo test --test slice_artifact_selector`
- `git diff --check`

## Status

Done.
