# Spec Alignment Implementation Plan Closeout

## Mission

Review the refreshed compiler/typesetter/backfill spec for implementation drift,
align small documented mismatches, and plan the remaining implementation waves.

## Review Findings

The spec, roles, and wave history are directionally aligned after the
compiler/typesetter and BACKFILL updates. The review found small fixable drift:

- `backfill` was documented like a live command even though it is planned.
- `proof fix -o/--output` was documented but is not implemented.
- `proof compile --root` is implemented but was missing from the CLI reference.
- The sample fix plan included `generated_at`, which is not in `FixPlan`.
- `md_broken_link` is registered but was missing from markdown-table diagnostics.

## Changes

- Marked `backfill` as planned in the CLI command list.
- Removed undocumented `proof fix -o/--output` from the spec.
- Added `proof compile --root` to the compile options.
- Removed `generated_at` from the sample fix plan.
- Added `md_broken_link` to the markdown-table diagnostics table.
- Created the session implementation plan at:
  `C:\Users\giodl\.copilot\session-state\a620d177-e5f9-4b77-8433-f3c0137c3ec8\plan.md`
- Reflected remaining implementation waves into SQL todos.

## Implementation Waves Planned

1. Backfill MVP, literal-first.
2. Backfill classifiers.
3. Structured extraction.
4. Artifact manifest and compile graph.
5. Tag-driven operations.
6. Fix pipeline consistency.
7. Directive module split.
8. Docs and review skills.

## Validation

- `git --no-pager diff --check`

## Carry-forward

Start with the backfill MVP. Do not implement semantic extraction before
literal round-trip gates and report formats are stable.
