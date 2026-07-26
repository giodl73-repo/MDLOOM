---
wave: mdport-adoption
date_open: 2026-07-26
status: done
source_request: "Rename PEBBLE and pebble.v1 to MDPORT and mdport.v1."
---

# Wave: MDPORT adoption

## Mission

Rename MDLOOM's compact portable compile target and schema from PEBBLE to
MDPORT while preserving the same source-resolution and artifact-manifest
pipeline.

## Changes

- Compile target: `mdport`.
- Schema: `mdport.v1`.
- Internal serializer: `mdport_output`.
- Documentation, tests, and publication contracts use MDPORT terminology.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `git diff --check`
