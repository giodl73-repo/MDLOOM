# proof symbol — Named Symbol and Shape Library

> **Status**: ✅ Implemented — `src/symbol/`. `[sym:name]` inline expansion, `proof:symbol` and `proof:shape` directives, built-in library (Core + Extended + Domain tiers), custom symbols via proof.toml. Did-you-mean suggestions for unknown names.

---

## What it is

proof provides a named symbol system for decorative and semantic characters
that appear throughout slides, dashboards, documents, and element directives.
Symbols are the ASCII/Unicode equivalent of presentation clipart.

Three tiers:

| Tier | What | Width | Example |
|------|------|-------|---------|
| **Unicode symbol** | Single named character | 1 or 2 cols | `★` `✓` `⚠` `🏆` |
| **ASCII shape** | Multi-line ASCII art block | N cols × M rows | banner, badge, ribbon |
| **Emoji** | Unicode emoji (width-aware) | 2 cols | `🏒` `🥅` `📊` |

---

## Inline symbol syntax

Symbols embed in any prose, bullet, or label using `[sym:name]`:

```markdown
• [sym:checkmark] Passed validation
• [sym:x] Failed review  
• [sym:star] Top performer — [sym:points] pts/82
```

Rendered:
```
• ✓ Passed validation
• ✗ Failed review
• ★ Top performer — 138.0 pts/82
```

`[sym:name]` always expands to the symbol's canonical Unicode character(s).
Width-2 symbols (emoji, fullwidth chars) are flagged to layout systems so
column budgets stay correct.

The `[sym:]` prefix makes the reference unambiguous — it cannot conflict with
Jinja/Handlebars templates, shell variables, or URL bracket syntax.

**Expansion scope:** `[sym:name]` is expanded in: prose paragraphs, bullet
labels, callout text, slide titles/subtitles, element label values. It is NOT
expanded in: fenced code blocks (` ``` ` delimited), inline code spans
(`` `backtick` ``), URL paths, or YAML/TOML values. When a file contains
Jinja/Handlebars templates, proof will not conflict because `[sym:]` is not a
template delimiter in any common system.

**Width measurement:** `[sym:name]` is expanded to its Unicode character(s)
first, then `visual_width()` is called on the fully-expanded string. The
layout budget is always measured on the rendered result, not the template.

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

Currently supported: `banner`, `badge`, `ribbon`. The geometric image-import
shapes (circle, heart, star, octagon, hexagon, etc.) listed in FIGURE-SPEC
are exposed via `proof figure import --shape <name>`, not via `proof:shape`
in source documents.

```
proof:shape name=banner title="Section 2 — Defense" style=double
proof:shape name=badge label="MVP" style=star
proof:shape name=ribbon text="WINNER" direction=diagonal
```

---

## Built-in symbol library

Symbols are tiered: **core** (shipped in v1.0), **extended** (shipped in
v1.1+), **domain** (user-defined or shipped in extension packages). Only core
symbols are guaranteed in the built-in library.

### Status / KPI

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `checkmark` | `✓` | 1 | Passed, complete | core |
| `x` / `cross` | `✗` | 1 | Failed, blocked | core |
| `check-box` | `☑` | 1 | Checked checkbox | extended |
| `box` | `☐` | 1 | Unchecked checkbox | extended |
| `circle-green` | `🟢` | 2 | Go / healthy | extended |
| `circle-yellow` | `🟡` | 2 | Warning / caution | extended |
| `circle-red` | `🔴` | 2 | Stop / critical | extended |
| `circle-blue` | `🔵` | 2 | Info | extended |
| `dot` | `●` | 1 | Filled bullet | core |
| `dot-open` | `○` | 1 | Open bullet | extended |
| `diamond` | `◆` | 1 | Emphasis bullet | extended |
| `triangle-right` | `▶` | 1 | Play / next | extended |
| `triangle-up` | `▲` | 1 | Increase | core |
| `triangle-down` | `▼` | 1 | Decrease | core |

### Stars / rating

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `star` | `★` | 1 | Filled star | core |
| `star-open` | `☆` | 1 | Empty star | core |
| `star-4` | `✦` | 1 | 4-point star | extended |
| `sparkle` | `✧` | 1 | Sparkle | extended |
| `trophy` | `🏆` | 2 | Championship | extended |

### Arrows

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `arrow-right` | `→` | 1 | Forward, next | core |
| `arrow-left` | `←` | 1 | Back, previous | extended |
| `arrow-up` | `↑` | 1 | Up, increase | core |
| `arrow-down` | `↓` | 1 | Down, decrease | core |
| `arrow-right-double` | `⇒` | 1 | Strong implication | extended |
| `arrow-right-long` | `⟹` | 1 | Conclusion | extended |
| `arrow-both` | `↔` | 1 | Bidirectional | extended |
| `arrow-curved-right` | `↪` | 1 | Redirect | extended |

### Productivity

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `warning` | `⚠` | 1 | Caution | core |
| `info` | `ℹ` | 1 | Information | core |
| `flag` | `⚑` | 1 | Flag, mark | extended |
| `pin` | `📌` | 2 | Pinned, important | extended |
| `key` | `🔑` | 2 | Key, unlock | extended |
| `lock` | `🔒` | 2 | Locked | extended |
| `calendar` | `📅` | 2 | Date | extended |
| `clock` | `🕐` | 2 | Time | extended |
| `fire` | `🔥` | 2 | Hot, trending | extended |
| `lightning` | `⚡` | 1 | Fast, power | extended |
| `target` | `🎯` | 2 | Goal, KPI | extended |
| `chart-up` | `📈` | 2 | Growth | extended |
| `chart-down` | `📉` | 2 | Decline | extended |
| `bar-chart` | `📊` | 2 | Analytics | extended |
| `rocket` | `🚀` | 2 | Launch, fast | extended |
| `hourglass` | `⌛` | 1 | Time remaining | extended |

### Math / logic

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `plus` | `+` | 1 | Add | extended |
| `minus` | `−` | 1 | Subtract (proper minus) | extended |
| `times` | `×` | 1 | Multiply | extended |
| `divide` | `÷` | 1 | Divide | extended |
| `approx` | `≈` | 1 | Approximately | extended |
| `not-equal` | `≠` | 1 | Not equal | extended |
| `less-equal` | `≤` | 1 | Less than or equal | extended |
| `greater-equal` | `≥` | 1 | Greater than or equal | extended |
| `infinity` | `∞` | 1 | Infinity | extended |
| `therefore` | `∴` | 1 | Therefore | extended |
| `sum` | `∑` | 1 | Sum | extended |
| `delta` | `Δ` | 1 | Change, delta | extended |
| `percent` | `%` | 1 | Percent | extended |
| `degree` | `°` | 1 | Degree | extended |

### Sports (IceLines domain)

These symbols are in the **domain** tier — they are not part of proof's core
built-in library. IceLines ships them as a `[[symbol]]` extension package. To
use them in non-IceLines projects, copy the definitions into your `proof.toml`.

| Name | Char | Width | Use | Tier |
|------|------|-------|-----|------|
| `puck` | `🏒` | 2 | Hockey | domain |
| `goal` | `🥅` | 2 | Net, goal | domain |
| `ice` | `🧊` | 2 | Ice surface | domain |
| `medal-gold` | `🥇` | 2 | 1st place | domain |
| `medal-silver` | `🥈` | 2 | 2nd place | domain |
| `medal-bronze` | `🥉` | 2 | 3rd place | domain |
| `skate` | `⛸` | 1 | Figure skating | domain |

### Lines / dividers (decorative)

| Name | Pattern | Use | Tier |
|------|---------|-----|------|
| `rule-thin` | `─────────────` | Separator | core |
| `rule-double` | `═════════════` | Strong separator | core |
| `rule-dotted` | `·············` | Soft separator | extended |
| `rule-dashed` | `- - - - - - -` | Dashed separator | extended |
| `rule-wave` | `~~~~~~~~~~~~~` | Decorative | extended |
| `rule-stars` | `* * * * * * *` | Decorative | extended |

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

**Font note:** curved ASCII (`.-""-./\`) aligns correctly only in strictly
monospace fonts (Cascadia Mono, JetBrains Mono, Courier). For portability, use
`style=box` which uses box-drawing characters (`╭ ╰`) that render consistently.
`style=curved` (the cloud) is labeled experimental.

### `arrow-block` (direction=right, size=3)

```
██▶
██▶
██▶
```

### `checkmark-large` (size=3)

A large checkmark is better rendered as `proof:symbol name=checkmark size=3`
which scales the character, not as a multi-line block shape. The block-art
form is removed — multi-line `✓` tiling does not produce a recognizable
checkmark glyph at any size.

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
                 # NOTE: style="badge" uses the same badge renderer as
                 # proof:element kind=label style=badge (see ELEMENT-SPEC.md)
                 # — same frame, same width budget, same right-padding behavior.
                 # This is intentional coupling, not coincidence.

[[symbol]]
name = "overtime-loss"
char = "OTL"
width = 3

# Multi-line ASCII shape
[[symbol]]
name = "crossed-sticks"
width = 5
height = 3
art = '''
\ //
 X
/ \
'''
# Use single-quoted multiline (''') not double-quoted ("""). TOML
# single-quoted multilines preserve all whitespace literally — no escape
# sequences. Leading whitespace on each line is literal and part of the art.
# Trailing spaces are preserved. Width × height is computed after TOML parsing.
```

Custom symbols are then usable everywhere: `[sym:oilers-logo]`, `[sym:ufa]`, `proof:symbol name=crossed-sticks size=1`.

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
emoji_fallback = true        # when width budget < 2, use text fallback
emoji_fallback_char = '?'    # override default replacement character
```

Fallback strings are always padded or truncated to exactly `width` columns so
E-1 (output character count = width) is never violated. The default fallback
for width=1 contexts is a single replacement character `?`. For width=2+, the
fallback abbreviation is padded/truncated to `width`. Authors may override:
`emoji_fallback_char = '?'` in `[ascii_char]` config.

---

## Symbol resolution order

1. Check `proof.toml` custom `[[symbol]]` entries (name match)
2. Check built-in library (exact name match)
3. Check built-in library (alias match — e.g. `cross` = `x`)
4. Emit `SYMBOL-001` warning if not found

**Alias rule:** In the built-in library table, all `/`-separated names in the
Name column are aliases for the same symbol (e.g. `x / cross`). The first name
is canonical. All aliases resolve to the same Unicode character. Custom
`[[symbol]]` entries do not support aliases — define a separate entry per name
if needed.

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `SYMBOL-001` | warning | Symbol name not found in library or custom definitions |
| `SYMBOL-002` | warning | Emoji `[sym:name]` in width-1 budget — using text fallback |
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
| `src/compile.rs` | `[sym:name]` inline expansion, `proof:symbol` / `proof:shape` directives |
| `src/config.rs` | `[[symbol]]` custom definition parsing |

---

## See also

- [Element Spec](./element-spec.md) — `kind=label style=badge` uses symbols
- [Slide Spec](./slide-spec.md) — `proof:callout`, `proof:bullets` use symbols
- [Dashboard Spec](./dashboard-spec.md) — status indicators use symbols
- [Mapping Spec](./mapping-spec.md) — field values can resolve to symbol names

---

## Spec Clarifications (from scenario findings)

- **F95** (sym in YAML): `[sym:name]` inside YAML string values must be quoted: `title: "[sym:star] proof"`. Unquoted values starting with `[` may be parsed as YAML arrays by some parsers.
- **F96** (case rule): Symbol names are case-insensitive. `[sym:CheckMark]` resolves the same as `[sym:checkmark]`. Lookup normalizes to lowercase before matching.
- **F97** (size units): The `size` attribute in `proof:symbol` is in glyph-count units, not display columns. A size-3 symbol renders as a 3×N character block where N is the glyph height.
- **F101** (sym inside math): `[sym:name]` tags inside `$...$` math spans are NOT expanded. The math tokenizer treats `[` as a literal character. Symbol expansion runs before math expansion, but only in prose contexts — not inside already-tokenized math spans.

