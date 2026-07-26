# Mdloom Architecture Config Boundary Close

## Outcome

Repaired two architecture drift points where config ownership crossed CLI and
runner boundaries incorrectly:

- Added an explicit-config runner path for `--config`, so check-like commands can
  apply the supplied config directly instead of re-resolving per-file config from
  disk.
- Wired `mdloom stats --config` through the same config path instead of ignoring
  the override.
- Preserved `files.include` cascade semantics by distinguishing omitted includes
  from explicitly set includes, including explicit values equal to the default.

## Tests Added

- `runner_explicit_config_skips_disk_cascade`
- `binary_stats_honors_config_override`
- `config_merge_default_child_include_preserves_parent_include`
- `config_merge_explicit_default_child_include_replaces_parent_include`

## Validation

- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The larger architecture finding remains: `src/main.rs` and `src/compile.rs` are
large orchestration modules. Future architecture waves should extract command
handlers and compile directive families behind smaller module boundaries after
the config boundary is stable.
