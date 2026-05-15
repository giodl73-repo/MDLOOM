# Command Path Helper Module Closeout

## Mission

Keep the CLI parser module focused on parsing by moving shared command path
defaulting into a command helper module.

## Changes

- Added `cmd_paths::paths_or_cwd`.
- Removed the shared cwd path helper from `cli.rs`.
- Updated compile, draft, and stats to use the command helper module.
- Registered the helper module in the binary shell.
- Preserved cwd fallback behavior for empty command path lists.

## Validation

- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_draft_command_writes_plan`
- `cargo test binary_stats_command_runs`
- `cargo test binary_stats_by_tag_reports_source_frontmatter`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep parser-specific types in `cli.rs`; put shared command execution helpers in
command-owned modules instead of growing the parser module again.

