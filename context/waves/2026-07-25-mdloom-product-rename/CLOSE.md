# MDLOOM product rename closeout

MDLOOM is now the repository, Cargo package, binary, library, directive prefix,
configuration/state namespace, schema namespace, skill family, and release
identity. GitHub moved from `giodl73-repo/PROOF` to
`giodl73-repo/MDLOOM`.

The rename also simplified the publication boundary:

- `mdloom-math` is the single math implementation.
- `mdloom-canvas` and `mdloom-math` package independently.
- MDLOOM no longer has direct Git dependencies on PEBBLE or SLICE.
- MDPATH remains the only portfolio Git dependency until `mdpath` is published.

Validation evidence:

- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo package --allow-dirty --no-verify` for `mdloom-math`
- `cargo package --allow-dirty --no-verify` for `mdloom-canvas`
- `mdloom --version` reports `mdloom 0.8.0`
- live legacy-name audit is clean outside explicit naming history

The root `mdloom` package must be published after `mdloom-canvas` and
`mdloom-math`; its dry run correctly stops until those package names exist in
the crates.io index.
