# Source Frontmatter Tags Closeout

## Mission

Give ordinary `.source.md` files first-class source-only metadata for corpus,
operation, and content tagging.

## Changes

- Added a generic source frontmatter parser for top-of-file `---` blocks.
- Supported `tags`, `ops`, and `content_tags` / `content` metadata in inline
  list, scalar, and block-list forms.
- Stripped frontmatter from ordinary `.source.md` compile output while preserving
  the source body.
- Added `proof stats --by-tag` to summarize tag, op, and content-tag counts
  across the same files selected for stats.
- Added status surface for source frontmatter/tag coverage.
- Documented source frontmatter in README and the CLI spec.

## Validation

- `cargo test frontmatter`
- `cargo test source_frontmatter_is_stripped_from_compile_output`
- `cargo test binary_stats_by_tag_reports_source_frontmatter`
- `cargo test runner_path_summary_counts_file_and_directory_inputs`

## Carry-forward

Tag metadata is passive in this wave. Future waves can use it for config
selectors, check filters, compile routing, wave/pulse grouping, and content
policy rules.

