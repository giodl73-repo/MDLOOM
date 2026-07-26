# Command Path Default Adapters Closeout

## Mission

Move command-specific path defaulting out of dispatch so routing stays close to
a pure command match.

## Changes

- Let compile, draft, and stats own their own cwd fallback for empty path lists.
- Removed the generic dispatch-side path extraction trait/helper that only served
  those commands.
- Moved check/default-check path fallback rules into `cmd_check`.
- Narrowed check internals further by making check flags, options, and low-level
  runner functions module-private.
- Kept the root `mdloom PATH` default-check behavior unchanged.

## Validation

- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_draft_command_writes_plan`
- `cargo test binary_stats_command_runs`
- `cargo test binary_stats_by_tag_reports_source_frontmatter`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test cli_mdloom_version_exits_zero`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Dispatch now routes commands without inspecting command flags or paths. Future
waves can reduce remaining config coupling by giving config-aware command modules
small global-context adapters.

