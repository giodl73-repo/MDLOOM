# Command Args Privacy Closeout

## Mission

Complete the command argument encapsulation pass after dispatch stopped reading
command fields directly.

## Changes

- Made command argument fields private across check, compile, config, depends,
  draft, fix, layout, pin, resolve, spec-generate, stats, status, and tree.
- Kept the command argument structs themselves `pub(crate)` so the root CLI enum
  can still expose subcommand variants to dispatch.
- Made the tree subcommand action enum module-private.
- Preserved clap parsing and all command behavior; dispatch continues to route
  through module-owned `run` functions and path accessors.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_stats_by_tag_reports_source_frontmatter`
- `cargo test binary_layout_composes_file_sources`
- `cargo test binary_pin_appends_davinci_entry`
- `cargo test binary_tree_generate_prints_dirtree`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

The remaining root CLI fields and global option fields can be wrapped behind
accessors/adapters in a later shell-hardening wave.

