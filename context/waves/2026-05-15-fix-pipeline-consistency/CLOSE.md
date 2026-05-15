# Fix Pipeline Consistency Closeout

## Mission

Bring `proof fix` into the same command architecture contract as check, stats,
draft, and compile.

## Changes

- Routed `proof fix` through global command options.
- Verification now uses the explicit global `--config` override instead of
  loading a default config independently.
- Verification checks the files modified by the applied plan.
- Extended `FixResult` with `modified_files` for precise downstream reporting.
- Added `.proof/last-fix.json` for every successful fix command run.
- The fix log records:
  - schema version
  - plan path
  - dry-run flag
  - minimum confidence
  - applied/skipped counts
  - modified file count and paths
  - verification status, errors, warnings, config, and paths
- Added integration coverage proving explicit `--config` is honored during
  verification and the structured log is written.
- Updated README, SPEC, session plan, and wave history.

## Validation

- `cargo test binary_fix_uses_global_config_for_verification_and_writes_log`
- `cargo test binary_fix_dry_run_writes_nothing`
- `cargo test fix_plan_confidence_filtering`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Future fix work should connect `.proof/last-fix.json` to `proof status` and
extend the log with per-fix skip details if review tooling needs a richer audit
trail.
