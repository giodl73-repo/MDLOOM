# Extends Cascade Semantics Close

## Outcome

Aligned `extends` behavior with the config contract:

- A config that declares `extends = ".../base.toml"` now loads that explicit
  parent and stops automatic ancestor discovery.
- The effective merge order is explicit parent first, then extending child.
- Ordinary directory cascade without `extends` remains additive from root to
  nearest child.

## Tests Added

- `config_extends_stops_automatic_ancestor_cascade`

Existing `config_cascade_additive_required_sections` continues to cover normal
ancestor + child cascade.

## Validation

- `cargo test config_extends_stops_automatic_ancestor_cascade`
- `cargo test config_cascade_additive_required_sections`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The raw/effective config split remains the larger cleanup target. This wave
focuses only on correcting the explicit-parent stop condition.
