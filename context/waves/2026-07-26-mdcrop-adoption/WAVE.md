---
wave: mdcrop-adoption
date_open: 2026-07-26
status: done
source_request: "Rename CROP to MDCROP."
---

# Wave: MDCROP adoption

MDLOOM now delegates corpus indexing, catalogs, status, views, and side-info to
MDCROP through the `mdcrop` binary and `mdcrop.view.v1` contracts.

Validation:

- `cargo fmt --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `git diff --check`
