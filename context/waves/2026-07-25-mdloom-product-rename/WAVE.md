---
wave: mdloom-product-rename
date_open: 2026-07-25
status: done
source_request: "Rename the PROOF repository and tool to MDLOOM."
---

# Wave: MDLOOM product rename

## Mission

Establish MDLOOM as the clean public identity for the Markdown quality,
compilation, and multi-format publishing toolchain before crates.io release.

## Scope

- Rename the package, binary, library, repository metadata, supporting crates,
  directives, configuration file, state directory, schemas, skills, docs, and
  tests.
- Remove the obsolete duplicate root math implementation.
- Preserve MDPATH as the stable addressing dependency.
- Prepare the child repository for a GitHub repository rename and subsequent
  TRACKER submodule-path update.

## Trace links

- Decision: MDLOOM is the public product and Cargo package name.
- Inputs: `Cargo.toml`, `README.md`, `CHANGELOG.md`, command tests, workflows,
  and package metadata.
- Evidence: clean repository search for legacy product identifiers, Rust
  validation, package dry runs, and command smoke tests.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo package --allow-dirty --no-verify`
- `mdloom --version`
- `git diff --check`

## Non-goals

- Do not publish crates in this wave.
- Do not change the `md://` addressing scheme.
- Do not update unrelated child repositories before the MDLOOM child commit is
  pushed.
