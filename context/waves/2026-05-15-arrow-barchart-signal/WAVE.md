# Arrow and Barchart Signal

## Mission

Reduce remaining MAXIM corpus noise from overbroad ASCII arrow and barchart
heuristics without weakening real diagnostics.

## Scope

- Treat `->` prose/source arrows and intentionally spaced visual axis arrows as
  non-actionable for `ascii_arrow_gap`.
- Keep warnings for isolated gaps in actual Unicode horizontal arrow bodies.
- Avoid applying barchart validation to boxed multi-panel drawings, adjacent
  texture/pattern runs, and typed programming fences.
- Keep real plain-text bar chart validation and numeric duration values.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| Classify arrow samples | done | MAXIM samples included chemistry reactions, word-wrapped prose arrows, and spaced speed axes. |
| Fix arrow heuristic | done | Requires enough Unicode line body and treats repeated spaced rulers differently from isolated breaks. |
| Classify barchart samples | done | MAXIM samples included boxed population pyramids, map panels, code operators, texture fills, and semantic charts. |
| Fix barchart scope | done | Plain-text diagram fences only; boxed rows and adjacent pattern runs are skipped. |
| Validate corpus impact | done | MAXIM warning total dropped from 1354 to 1142 with zero errors. |

## Gates

- Focused arrow regression tests pass.
- Focused barchart regression tests pass.
- MAXIM corpus stays at zero errors.

