## Spec Clarifications (from scenario findings)

- **F71** (delta text representation): `delta` kind in text output: positive values render with `+` prefix and `▲` arrow; negative with `-` prefix and `▼` arrow; zero with `→`. No color. Example: `+12.3 ▲`, `-5.1 ▼`, `0.0 →`.
- **F73** (row two-pass): `proof:row` uses a two-pass render: pass 1 collects all cell values and computes max visual_width per column; pass 2 renders each row with pinned column widths (R-1 invariant).
- **F74** (separator width): Separator visual width (e.g., `" │ "` = 3 columns) is included in total row width for R-1. Total = sum(cell widths) + separator_visual_width × (n_cells - 1).
- **F75** (sparkline short series): When series length < display width, values are repeated to fill the width. When series length > display width, values are sampled evenly. ELEMENT-003 warning is emitted for short series.
- **F76** (constant sparkline): If all series values are equal (min = max), the normalized value is 0.5 for all points. All blocks render as `▄` (mid-height). No division-by-zero error.
- **F79** (value cleaning): For `kind=value`, cleaning before parse: remove commas, strip trailing `%`. If cleaned value parses as f64, use as Scalar. Otherwise use original string as Text (display preserved).
- **F80** (display string): When a value is treated as Text (e.g., `"1,024"`), the original unmodified string is displayed — not the cleaned form.
- **F82** (badge alignment): Badge content is centered within `width`. `[content]` — square brackets added, content centered between them. If `[content]` is wider than `width`, the badge is clipped.
