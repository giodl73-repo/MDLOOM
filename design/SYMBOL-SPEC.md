# proof symbol — Named Symbol and Shape Library

> **Status**: Design — not yet implemented.

---

## What it is

proof provides a named symbol system for decorative and semantic characters
that appear throughout slides, dashboards, documents, and element directives.
Symbols are the ASCII/Unicode equivalent of presentation clipart.

Three tiers:

| Tier | What | Width | Example |
|------|------|-------|---------|
| **Unicode symbol** | Single named character | 1 or 2 cols | `★` `✓` `⚠` `🏆` |
| **ASCII shape** | Multi-line ASCII art block | N cols × M rows | banner, badge, star, cloud |
| **Emoji** | Unicode emoji (width-aware) | 2 cols | `🏒` `🥅` `📊` |

---

## Inline symbol syntax

Symbols embed in any prose, bullet, or label using `{{name}}`:

```markdown
• {{checkmark}} Passed validation
• {{x}} Failed review  
• {{star}} Top performer — {{points}} pts/82
```

Rendered:
```
• ✓ Passed validation
• ✗ Failed review
• ★ Top performer — 138.0 pts/82
```

`{{name}}` always expands to the symbol's canonical Unicode character(s).
Width-2 symbols (emoji, fullwidth chars) are flagged to layout systems so
column budgets stay correct.

---

## `proof:symbol` directive

Block form — renders a symbol at a declared size:

```
proof:symbol name=star size=3 align=center
```

Sizes:
- `size=1` — single Unicode char (`★`)
- `size=2` — 3×3 block ASCII art
- `size=3` — 5×5 block
- `size=5` — 9×9 block (for slide section headers, large callouts)

---

## `proof:shape` directive

Multi-line ASCII art block. Shapes are named templates with optional text slots.

```
proof:shape name=banner title="Section 2 — Defense" style=double
proof:shape name=badge label="MVP" style=star
proof:shape name=ribbon text="WINNER" direction=diagonal
proof:shape name=callout-cloud text="Did you know?"
proof:shape name=arrow direction=right size=3
```

---

## Built-in symbol library

### Status / KPI

| Name | Char | Width | Use |
|------|------|-------|-----|
| `checkmark` | `✓` | 1 | Passed, complete |
| `x` / `cross` | `✗` | 1 | Failed, blocked |
| `check-box` | `☑` | 1 | Checked checkbox |
| `box` | `☐` | 1 | Unchecked checkbox |
| `circle-green` | `🟢` | 2 | Go / healthy |
| `circle-yellow` | `🟡` | 2 | Warning / caution |
| `circle-red` | `🔴` | 2 | Stop / critical |
| `circle-blue` | `🔵` | 2 | Info |
| `dot` | `●` | 1 | Filled bullet |
| `dot-open` | `○` | 1 | Open bullet |
| `diamond` | `◆` | 1 | Emphasis bullet |
| `triangle-right` | `▶` | 1 | Play / next |
| `triangle-up` | `▲` | 1 | Increase |
| `triangle-down` | `▼` | 1 | Decrease |

### Stars / rating

| Name | Char | Width | Use |
|------|------|-------|-----|
| `star` | `★` | 1 | Filled star |
| `star-open` | `☆` | 1 | Empty star |
| `star-4` | `✦` | 1 | 4-point star |
| `sparkle` | `✧` | 1 | Sparkle |
| `trophy` | `🏆` | 2 | Championship |

### Arrows

| Name | Char | Width | Use |
|------|------|-------|-----|
| `arrow-right` | `→` | 1 | Forward, next |
| `arrow-left` | `←` | 1 | Back, previous |
| `arrow-up` | `↑` | 1 | Up, increase |
| `arrow-down` | `↓` | 1 | Down, decrease |
| `arrow-right-double` | `⇒` | 1 | Strong implication |
| `arrow-right-long` | `⟹` | 1 | Conclusion |
| `arrow-both` | `↔` | 1 | Bidirectional |
| `arrow-curved-right` | `↪` | 1 | Redirect |

### Productivity

| Name | Char | Width | Use |
|------|------|-------|-----|
| `warning` | `⚠` | 1 | Caution |
| `info` | `ℹ` | 1 | Information |
| `flag` | `⚑` | 1 | Flag, mark |
| `pin` | `📌` | 2 | Pinned, important |
| `key` | `🔑` | 2 | Key, unlock |
| `lock` | `🔒` | 2 | Locked |
| `calendar` | `📅` | 2 | Date |
| `clock` | `🕐` | 2 | Time |
| `fire` | `🔥` | 2 | Hot, trending |
| `lightning` | `⚡` | 1 | Fast, power |
| `target` | `🎯` | 2 | Goal, KPI |
| `chart-up` | `📈` | 2 | Growth |
| `chart-down` | `📉` | 2 | Decline |
| `bar-chart` | `📊` | 2 | Analytics |
| `rocket` | `🚀` | 2 | Launch, fast |
| `hourglass` | `⌛` | 1 | Time remaining |

### Math / logic

| Name | Char | Width | Use |
|------|------|-------|-----|
| `plus` | `+` | 1 | Add |
| `minus` | `−` | 1 | Subtract (proper minus) |
| `times` | `×` | 1 | Multiply |
| `divide` | `÷` | 1 | Divide |
| `approx` | `≈` | 1 | Approximately |
| `not-equal` | `≠` | 1 | Not equal |
| `less-equal` | `≤` | 1 | Less than or equal |
| `greater-equal` | `≥` | 1 | Greater than or equal |
| `infinity` | `∞` | 1 | Infinity |
| `therefore` | `∴` | 1 | Therefore |
| `sum` | `∑` | 1 | Sum |
| `delta` | `Δ` | 1 | Change, delta |
| `percent` | `%` | 1 | Percent |
| `degree` | `°` | 1 | Degree |

### Sports (IceLines domain)

| Name | Char | Width | Use |
|------|------|-------|-----|
| `puck` | `🏒` | 2 | Hockey |
| `goal` | `🥅` | 2 | Net, goal |
| `ice` | `🧊` | 2 | Ice surface |
| `medal-gold` | `🥇` | 2 | 1st place |
| `medal-silver` | `🥈` | 2 | 2nd place |
| `medal-bronze` | `🥉` | 2 | 3rd place |
| `trophy-cup` | `🏆` | 2 | Championship |
| `skate` | `⛸` | 1 | Figure skating |

### Lines / dividers (decorative)

| Name | Pattern | Use |
|------|---------|-----|
| `rule-thin` | `─────────────` | Separator |
| `rule-double` | `═════════════` | Strong separator |
| `rule-dotted` | `·············` | Soft separator |
| `rule-dashed` | `- - - - - - -` | Dashed separator |
| `rule-wave` | `~~~~~~~~~~~~~` | Decorative |
| `rule-stars` | `* * * * * * *` | Decorative |

---

## Built-in ASCII shapes

### `banner`

```
╔══════════════════════════════╗
║        SECTION TITLE         ║
╚══════════════════════════════╝
```

Styles: `single` (`┌┐└┘`), `double` (shown), `rounded` (`╭╮╰╯`), `heavy` (`┏┓┗┛`), `ascii` (`+-+`)

### `badge`

```
 ╭──────╮
 │  MVP  │
 ╰──────╯
```

Styles: `rounded` (shown), `square`, `sharp`

### `star-shape` (size=3)

```
  ★
 ★★★
  ★
```

Size=5:
```
   ★
  ★★★
 ★★★★★
  ★★★
   ★
```

### `ribbon`

```
   ╱‾‾‾‾‾‾‾‾‾‾‾‾‾╲
  ╱    WINNER      ╲
 ╱_________________╲
```

### `callout-cloud`

```
  .-"""""-.
 /  Did     \
| you know?  |
 \_________./
      |
      |
```

### `arrow-block` (direction=right, size=3)

```
██▶
██▶
██▶
```

### `checkmark-large` (size=3)

```
    ✓
   ✓
  ✓
 ✓ ✓
  ✓✓
```

---

## Custom symbols

Define domain-specific symbols in `proof.toml`:

```toml
[[symbol]]
name = "oilers-logo"
char = "🛢"      # single Unicode char form
width = 2

[[symbol]]
name = "ufa"
char = "UFA"     # multi-char label treated as a unit
width = 3
style = "badge"  # rendered in badge frame when size > 1

[[symbol]]
name = "overtime-loss"
char = "OTL"
width = 3

# Multi-line ASCII shape
[[symbol]]
name = "crossed-sticks"
width = 5
height = 3
art = """
\\ //
 X
/ \\
"""
```

Custom symbols are then usable everywhere: `{{oilers-logo}}`, `{{ufa}}`, `proof:symbol name=crossed-sticks size=1`.

---

## Integration with other directives

### Bullets with symbols

```
proof:bullets bullet-1="★" bullet-2="◦" bullet-3="▸"
- McDavid leads all forwards in points
  - 138.0 pts/82 — highest in NHL history
    - Previous record was Gretzky in 1985-86
```

### Callouts with symbol

```
proof:callout style=key symbol=trophy
McDavid is the frontrunner for the Hart Trophy.
```

### Element badges

```
proof:element kind=label style=badge symbol=circle-green field=status width=12
```

Rendered: `🟢 Active    `

### Slide stats with symbol

```
proof:stat value=138.0 label="Pts/82" symbol=chart-up sublabel="#1 all-time"
```

Rendered:
```
  📈  138.0
  Pts/82
  #1 all-time
```

---

## Emoji handling

Emoji are width-2 by default (East Asian Width = W or emoji presentation).
proof measures them correctly via `visual_width()` (already implemented in
`layout.rs`). In tight spaces, emoji fall back to their text description:

```toml
[ascii_char]
emoji_fallback = true   # when width budget < 2, use text fallback
```

Fallbacks defined per symbol. Default: `{{trophy}}` → `(tph)` at width < 2.

---

## Symbol resolution order

1. Check `proof.toml` custom `[[symbol]]` entries (name match)
2. Check built-in library (exact name match)
3. Check built-in library (alias match — e.g. `cross` = `x`)
4. Emit `SYMBOL-001` warning if not found

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `SYMBOL-001` | warning | Symbol name not found in library or custom definitions |
| `SYMBOL-002` | warning | Emoji `{{name}}` in width-1 budget — using text fallback |
| `SYMBOL-003` | error | `proof:shape name=X` shape not found |
| `SYMBOL-004` | warning | `proof:shape` content exceeds declared `width × height` |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/symbol/mod.rs` | Symbol resolution, library lookup |
| `src/symbol/library.rs` | Built-in symbol definitions |
| `src/symbol/shape.rs` | Multi-line ASCII shape renderer |
| `src/symbol/emoji.rs` | Emoji width handling and fallback |
| `src/compile.rs` | `{{name}}` inline expansion, `proof:symbol` / `proof:shape` directives |
| `src/config.rs` | `[[symbol]]` custom definition parsing |

---

## See also

- [Element Spec](./element-spec.md) — `kind=label style=badge` uses symbols
- [Slide Spec](./slide-spec.md) — `proof:callout`, `proof:bullets` use symbols
- [Dashboard Spec](./dashboard-spec.md) — status indicators use symbols
- [Mapping Spec](./mapping-spec.md) — field values can resolve to symbol names
