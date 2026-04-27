# Symbol System Implementation Plan

> **Spec**: `design/SYMBOL-SPEC.md`
> **Target**: v1.0 MVP — core tier only, `[sym:name]` inline expansion, `proof:symbol` and `proof:shape` directives, `[[symbol]]` config parsing.

---

## Wave 1 — Symbol Resolution Engine

**Files**: `src/symbol/mod.rs`, `src/symbol/library.rs`, `src/symbol/emoji.rs`
**Estimated LOC**: ~300
**Exit criterion**: `cargo test symbol` passes 20+ tests; `[sym:star]` expands to `★` in a prose string; emoji in a width-1 budget emits a single `?` fallback.

### Structs

```rust
// src/symbol/mod.rs
pub struct Symbol {
    pub canonical: &'static str,  // canonical name (first in slash-list)
    pub character: &'static str,  // Unicode character(s) to emit
    pub width: usize,             // visual columns: 1 or 2
    pub tier: Tier,
}

pub enum Tier { Core, Extended, Domain }

pub struct CustomSymbol {
    pub name: String,
    pub character: String,
    pub width: usize,
    pub art: Option<String>,      // raw multi-line art from TOML
    pub height: Option<usize>,
}

pub struct SymbolLibrary {
    pub custom: Vec<CustomSymbol>,
}
```

### Functions

```rust
// src/symbol/mod.rs
pub fn resolve<'a>(name: &str, lib: &'a SymbolLibrary) -> Option<ResolvedSymbol<'a>>
// Resolution order per spec:
//   1. custom [[symbol]] by name
//   2. built-in exact name
//   3. built-in alias
// Emits SYMBOL-001 via return None (caller warns)

pub fn expand_symbols(text: &str, lib: &SymbolLibrary) -> (String, Vec<SymbolWarning>)
// Scan text for [sym:name] patterns using hand-rolled parsing (NOT the regex crate —
// regex is not in Cargo.toml; use str::find("[sym:") + scan forward for ']').
// Skips expansion inside: fenced code blocks (``` delimited), inline code (`backtick`),
// URL paths. Returns expanded string + any SYMBOL-001/SYMBOL-002 warnings.
// visual_width() is called on the final expanded string, not on the template.

// src/symbol/emoji.rs
pub fn emoji_fallback(sym: &ResolvedSymbol, width_budget: usize, config: &AsciiCharConfig) -> String
// If width_budget < sym.width: emit config.emoji_fallback_char repeated to width_budget.
// Otherwise: emit sym.character padded/truncated to exactly width_budget cols.
```

### Built-in library (`src/symbol/library.rs`)

Core tier only for v1.0 MVP (~14 symbols + 2 rule symbols = 16 entries):

| Name(s) | Char | Width |
|---------|------|-------|
| `checkmark` | `✓` | 1 |
| `x` / `cross` | `✗` | 1 |
| `dot` | `●` | 1 |
| `triangle-up` | `▲` | 1 |
| `triangle-down` | `▼` | 1 |
| `star` | `★` | 1 |
| `star-open` | `☆` | 1 |
| `arrow-right` | `→` | 1 |
| `arrow-up` | `↑` | 1 |
| `arrow-down` | `↓` | 1 |
| `warning` | `⚠` | 1 |
| `info` | `ℹ` | 1 |
| `rule-thin` | `─` (repeated) | 1/col |
| `rule-double` | `═` (repeated) | 1/col |

Alias rule: slash-separated names in the library table — `x / cross` — first is canonical. All aliases resolve to the same character. `resolve("cross")` returns the same `Symbol` as `resolve("x")`.

Rule symbols (`rule-thin`, `rule-double`) emit a single character; repetition to fill width is handled at the call site, not in resolution.

### `visual_width` integration

`expand_symbols` does not call `visual_width` per symbol during scanning. It expands the full string first, then the caller calls `layout::visual_width()` on the result. The `emoji.rs` fallback function uses `visual_width` to verify the emitted string is exactly `width_budget` columns.

### `[[symbol]]` config parsing

Add to `src/config.rs`:

```rust
#[derive(Debug, Deserialize, Clone, Default)]
pub struct CustomSymbolConfig {
    pub name: String,
    pub char: Option<String>,    // single Unicode char or short label
    pub width: Option<usize>,    // default: 1
    pub height: Option<usize>,   // multi-line art only
    pub style: Option<String>,   // "badge" — badge frame when size > 1
    pub art: Option<String>,     // single-quoted multiline TOML (''')
}
```

Add `#[serde(default)] pub symbol: Vec<CustomSymbolConfig>` to `GlintConfig`.

TOML single-quoted multiline (`'''`) preserves all whitespace literally — no escapes. Width × height are computed after parsing if not declared. Merge semantics: additive (child symbols extend parent's symbol list, matching `davinci` pattern).

### Tests (20+ required)

| Test | Asserts |
|------|---------|
| `test_expand_checkmark` | `[sym:checkmark]` → `✓` |
| `test_expand_alias_cross` | `[sym:cross]` → `✗` (alias of `x`) |
| `test_expand_multiple` | two symbols in one string, both replaced |
| `test_no_expand_in_code_fence` | ` ```\n[sym:star]\n``` ` → unchanged |
| `test_no_expand_in_inline_code` | `` `[sym:star]` `` → unchanged |
| `test_no_expand_in_url` | `https://example.com/[sym:star]` → unchanged |
| `test_unknown_symbol_warning` | `[sym:nonexistent]` → SYMBOL-001 warning, literal passthrough |
| `test_custom_symbol_priority` | custom `[[symbol]]` name shadows built-in |
| `test_custom_symbol_char` | custom `char="🛢"` resolved correctly |
| `test_custom_art_symbol` | custom with `art='''...'''` stores raw art |
| `test_emoji_fallback_width1` | emoji sym in width-1 budget → `?` |
| `test_emoji_fallback_width2` | emoji sym in width-2 budget → character |
| `test_emoji_fallback_custom_char` | `emoji_fallback_char='!'` → `!` |
| `test_visual_width_after_expand` | expanded string visual_width correct |
| `test_resolve_order_custom_first` | custom overrides built-in of same name |
| `test_resolve_exact_then_alias` | exact match before alias scan |
| `test_rule_thin_char` | `rule-thin` resolves to `─` |
| `test_expand_no_sym_prefix` | `[notasym:foo]` untouched |
| `test_expand_empty_name` | `[sym:]` untouched (invalid) |
| `test_config_symbol_parse` | TOML `[[symbol]]` round-trips through serde |

---

## Wave 2 — proof:symbol and proof:shape Directives

**Files**: `src/symbol/shape.rs`, additions to `src/compile.rs`
**Estimated LOC**: ~250
**Exit criterion**: `proof:symbol name=star size=3` renders a 5×5 star block; `proof:shape name=banner style=double title="Test"` renders a double-border banner; `proof:shape name=badge label="MVP"` renders a rounded badge.

### `src/symbol/shape.rs`

```rust
pub struct ShapeAttrs {
    pub name: String,
    pub title: Option<String>,
    pub label: Option<String>,
    pub text: Option<String>,
    pub style: String,        // "single" | "double" | "rounded" | "heavy" | "ascii"
    pub direction: String,    // for arrow, ribbon
    pub size: usize,          // 1..=5
    pub width: Option<usize>, // explicit column budget
}

pub fn render_shape(attrs: &ShapeAttrs) -> Result<String, ShapeError>
// Dispatches to shape-specific renderer. Returns multi-line string (no fencing).
// Error → SYMBOL-003 if shape name not found.

fn render_banner(title: &str, style: &str, width: usize) -> String
fn render_badge(label: &str, style: &str) -> String
fn render_ribbon(text: &str, direction: &str) -> String

pub struct ShapeError { pub code: &'static str, pub message: String }
```

**Banner border characters by style:**

| Style | TL | TR | BL | BR | H | V |
|-------|----|----|----|----|----|---|
| `single` | `┌` | `┐` | `└` | `┘` | `─` | `│` |
| `double` | `╔` | `╗` | `╚` | `╝` | `═` | `║` |
| `rounded` | `╭` | `╮` | `╰` | `╯` | `─` | `│` |
| `heavy` | `┏` | `┓` | `┗` | `┛` | `━` | `┃` |
| `ascii` | `+` | `+` | `+` | `+` | `-` | `|` |

Default style: `double`.

**Badge styles:**
- `rounded`: `╭──╮` / `│ label │` / `╰──╯`
- `square`: `┌──┐` / `│ label │` / `└──┘`
- `sharp`: `+--+` / `| label |` / `+--+`

Default style: `rounded`.

### `proof:symbol` directive in `compile.rs`

Add to `proof_directive_kind`:
```rust
else if rest.starts_with("symbol") { Some("symbol") }
else if rest.starts_with("shape")  { Some("shape") }
```

Add variants to `Directive` enum:
```rust
Symbol {
    name: String,
    size: usize,    // 1..=5, default 1
    align: String,  // "left" | "center" | "right"
    line_start: usize,
    line_end: usize,
},
Shape {
    attrs: ShapeAttrs,
    line_start: usize,
    line_end: usize,
},
```

`proof:symbol size=1` emits a single Unicode character inline. `size=2..=5` emits a multi-line block: size maps to grid dimensions (1→1×1, 2→3×3, 3→5×5, 5→9×9). For symbols without an ASCII art form, the character is tiled in a diamond/cross pattern at the requested size (see SYMBOL-SPEC.md `star-shape` examples).

Output is wrapped in the standard `<!-- proof:compiled -->` block, matching the pattern in `format_include_block`.

### Tests

| Test | Asserts |
|------|---------|
| `test_symbol_size1_renders_char` | `proof:symbol name=star size=1` → `★` |
| `test_symbol_size3_renders_block` | `proof:symbol name=star size=3` → 5 lines |
| `test_banner_double_width40` | title centered, double borders |
| `test_banner_single_style` | single-line box characters |
| `test_banner_rounded_style` | `╭╮╰╯` corners |
| `test_banner_heavy_style` | `┏┓┗┛` corners |
| `test_banner_ascii_style` | `+--+` only ASCII |
| `test_badge_rounded` | `╭──╮` frame, label centered |
| `test_badge_square` | `┌──┐` frame |
| `test_shape_unknown_emits_symbol003` | SYMBOL-003 on bad shape name |
| `test_shape_content_exceeds_width_warns_symbol004` | SYMBOL-004 warning |
| `test_directive_symbol_parsed` | `proof:symbol` appears in `collect_directives` |
| `test_directive_shape_parsed` | `proof:shape` appears in `collect_directives` |

---

## Wave 3 — Integration and Config Wiring

**Files**: `src/compile.rs` (prose expansion pass), `src/config.rs` (merge), integration tests
**Estimated LOC**: ~150
**Exit criterion**: End-to-end `compile_file` with `[sym:checkmark]` in a bullet label expands correctly; `proof:bullets bullet-1="★"` uses the symbol; `proof:callout symbol=trophy` resolves the symbol; `proof:element kind=label style=badge` uses the badge renderer.

### Prose expansion pass in compile pipeline

After `apply_replacements` reconstructs the output text, run a second pass over all non-directive prose lines:

```rust
fn expand_prose_symbols(text: &str, lib: &SymbolLibrary) -> (String, Vec<SymbolWarning>)
```

This pass must skip:
- Lines inside fenced code blocks (track state with a `in_fence: bool` flag)
- `<!-- proof:compiled -->` blocks (already resolved)
- Inline code spans (`` `backtick` ``)
- URL tokens

Warnings from this pass are converted to `CompileViolation` entries at `ViolationSeverity::Warning`.

The `SymbolLibrary` is constructed once per `compile_file` call from `config.symbol`.

### `[[symbol]]` merge semantics in `config.rs`

Add to the `merge` function body, matching the `davinci` additive pattern:

```rust
symbol: {
    let mut v = parent.symbol;
    v.extend(child.symbol);
    v
},
```

### `proof:bullets` symbol config

`proof:bullets` directive already accepts `bullet-1=`, `bullet-2=`, `bullet-3=` attribute strings. In Wave 3, these values are passed through `expand_symbols` so that `bullet-1="[sym:star]"` is expanded before rendering. Literal characters like `★` continue to work unchanged — no behavioral regression.

### `proof:callout symbol=` attribute

When `symbol=name` is present on a `proof:callout`, resolve via `SymbolLibrary`. Emit the character prepended to the callout label line. If the symbol is width-2 (emoji), add one space of padding after to preserve alignment. SYMBOL-001 warning if not found.

### `proof:element kind=label style=badge`

`proof:element` with `kind=label style=badge` calls `render_badge(value, "rounded")` from `src/symbol/shape.rs`. This is the same renderer used by Wave 2 `proof:shape name=badge`. Coupling is intentional — documented in SYMBOL-SPEC.md custom symbol `style="badge"` note.

### Integration tests

| Test | Asserts |
|------|---------|
| `test_compile_sym_in_prose` | `[sym:checkmark] Passed` → `✓ Passed` in output |
| `test_compile_sym_skipped_in_code_fence` | code block left unchanged |
| `test_compile_sym_skipped_in_compiled_block` | `<!-- proof:compiled -->` blocks untouched |
| `test_compile_unknown_sym_warns` | SYMBOL-001 in `CompileResult.violations` |
| `test_compile_bullets_sym_expansion` | `bullet-1="[sym:star]"` → `★` prefix |
| `test_compile_callout_symbol` | `symbol=warning` → `⚠` prepended |
| `test_compile_element_badge` | `kind=label style=badge` uses badge renderer |
| `test_compile_custom_symbol_from_config` | `[[symbol]]` in config used in expansion |
| `test_symbol_library_from_config` | `SymbolLibrary::from_config` builds correctly |

---

## Dependency Graph

```
Wave 1 (symbol resolution, library, emoji)
    ↓
Wave 2 (shape renderer, proof:symbol + proof:shape directives)
    ↓
Wave 3 (prose expansion wired into compile pipeline, config merge)
```

Wave 2 depends on Wave 1 (`resolve`, `SymbolLibrary`). Wave 3 depends on both.

## Key invariants

- `[sym:name]` is never expanded inside fenced code blocks or inline code — verified by 2 tests in Wave 1, 1 integration test in Wave 3.
- `visual_width()` is called on the fully-expanded string, never on the `[sym:name]` template — width budget accounting is always on rendered output.
- Emoji in a width-1 budget always emits exactly 1 column — the fallback character. Width-2 budget emits exactly 2 columns. E-1 is never violated.
- Resolution order is strict: custom `[[symbol]]` → built-in exact → built-in alias. A custom symbol can shadow a built-in of the same name.
- `proof:shape` content exceeding declared `width × height` is SYMBOL-004 warning, not error — authors can declare a smaller frame and let content overflow visually.
- `shape.rs` `render_badge` is the single implementation used by both `proof:shape name=badge` and `proof:element kind=label style=badge` — no duplication.
