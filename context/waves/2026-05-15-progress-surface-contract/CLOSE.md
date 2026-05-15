# Progress Surface Contract Close

## Outcome

Aligned the progress option contract with the actual CLI surface:

- `--progress` is a `proof compile` option, where it already shows a running
  compiled/total count.
- `proof check` does not expose `--progress`; the spec no longer advertises it
  as a check option.
- Added a CLI help regression to ensure `check --help` does not list
  `--progress` while `compile --help` does.

## Tests Added

- `binary_help_documents_progress_only_for_compile`

## Validation

- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

If check-time progress becomes desirable, it should be added deliberately with a
runner API that can report per-file progress without giving up parallelism.
