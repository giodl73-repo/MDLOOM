# HTML Publish Target Closeout

## Mission

Prove that proof's compiler/typesetter model can publish beyond markdown without
forking the source workflow.

## Changes

- Added `proof compile --target md|html`, with `md` as the default.
- Added `proof_lib::publish` with a deterministic markdown-to-HTML document
  renderer for headings, paragraphs, and fenced code blocks.
- HTML compilation resolves `.source.md` through the existing markdown compiler
  first, so source frontmatter stripping and directive expansion stay shared.
- HTML outputs derive from source names when using `--output-dir` and accept
  explicit `-o` for single-file compiles.
- Guarded `--watch` to `--target md` until watch-mode target tracking is modeled
  in the artifact manifest.
- Preserved the existing markdown target behavior and fixed cached compile hits
  so an unchanged cached output is still counted as successfully compiled instead
  of falling through to source copying.
- Added integration coverage for HTML output escaping and fenced code rendering.
- Updated README and SPEC to frame HTML as the first publish target and PPTX as a
  future backend behind the compile graph.

## Validation

- `cargo test binary_compile_target_html_writes_html_document`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

The artifact manifest should record the target (`md`, `html`, future `pptx`),
source path, output path, config root, and stale status before broadening watch
mode or adding PPTX generation.
