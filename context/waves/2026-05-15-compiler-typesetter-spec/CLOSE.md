# Compiler Typesetter Spec Closeout

## Mission

Update the project bible so future work is judged against mdloom as a staged
document compiler and LaTeX-style markdown-native typesetter, not only a linter.

## Changes

- Bumped `design/SPEC.md` to v0.3.
- Rewrote the purpose around source trees, reference-aware compilation,
  rendered artifacts, stable diagnostics, and deterministic repair.
- Added a compiler/typesetter model covering source, resolve, compile/typeset,
  check, and plan/fix layers.
- Made the CLI architecture boundary part of the spec:
  `main.rs -> cli parser -> dispatch context -> command adapters -> mdloom_lib`.
- Updated backlog, skills, documentation, tests, and non-goals toward corpus
  compile graphs, tag-driven operations, artifact manifests, and golden
  source-to-artifact tests.

## Validation

- `git --no-pager diff --check`

## Carry-forward

Future features should map to compiler phases and artifact contracts before
implementation. Source frontmatter tags should become selection and policy
inputs for compile/check/report slices rather than remaining only summaries.
