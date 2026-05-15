# Artifact Manifest Closeout

## Mission

Make compile output provenance durable so markdown, HTML, and future PPTX/site
targets share one artifact graph.

## Changes

- Added `proof_lib::artifact` with serializable manifest, artifact, status, and
  diagnostic records.
- `proof compile` now writes `.proof/artifacts.json` for non-watch compile runs.
- Manifest entries record:
  - source path
  - output path
  - target (`md`, `html`, future publish backends)
  - status (`written`, `cached`, `up_to_date`, `error`)
  - resolved directive count
  - cache usage
  - diagnostics
- The manifest records config root and generation timestamp.
- Kept cache and manifest responsibilities separate: cache stores reusable
  content, manifest describes the latest compile run.
- Added integration coverage for target-aware HTML manifest entries.
- Updated README, SPEC, session plan, and wave history.

## Validation

- `cargo test binary_compile_writes_artifact_manifest`
- `cargo test binary_compile_target_html_writes_html_document`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Next manifest work should wire stale checks and `proof status` to
`.proof/artifacts.json`, then let watch mode and future PPTX/site backends record
target-specific provenance through the same structure.
