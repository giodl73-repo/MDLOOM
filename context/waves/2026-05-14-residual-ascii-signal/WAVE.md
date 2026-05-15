# Residual ASCII Signal

## Mission

Close the remaining proof-owned ASCII false positives after the arrow/barchart
wave, focusing on residual barchart shape detection and bottom-border column
warnings.

## Scope

- Treat multi-run texture rows, equation operators, axis-attached bars, and
  connector strokes as non-barchart rows.
- Allow stacked bars to mix default fill characters such as `█` and `░`.
- Treat top-border incoming connectors, bottom connector ports, spanning rows,
  and ASCII tree branches as non-column structures for `ascii_box_col`.
- Preserve real content-row separator drift and true equal-structure bottom
  border drift.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| Classify residual ASCII | done | Sampled `ascii_barchart_*`, `ascii_box_col`, `ascii_cell_padding`, arrows, and unclosed fences. |
| Residual barchart parser | done | Skipped multi-run textures, equation operators, axes, connector labels, and stacked-bar char noise. |
| Box-column connector policy | done | Removed incoming top connectors and bottom ports from required column structures. |
| Box-column spanning policy | done | Suppressed bottom-border warnings when the bottom closes a spanning row or spatial layout. |
| Residual arrow layout policy | done | Stopped arrow scanning at multi-space layout gaps and ignored bidirectional scale rulers. |
| Validate corpus impact | done | MAXIM warning total dropped from 1142 to 1053 with zero errors. |

## Gates

- Focused barchart tests pass.
- Focused ASCII box tests pass.
- MAXIM corpus stays at zero errors.

