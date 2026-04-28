# System Architecture — Inline Pin Example

This document demonstrates the `pin=id` attribute on `proof:include`.

## What the pin attribute does

Adding `pin=goroutine-scheduler` to a `proof:include` directive declares
that this figure should be protected by a DaVinci invariant with that ID.

When no matching `[[davinci]]` entry exists in proof.toml, COMPILE-007 is
emitted as a warning, prompting you to run `proof pin`.

## Workflow

1. Add `pin=id` to the `proof:include` in your source document
2. First compile emits COMPILE-007 warning with the exact `proof pin` command to run
3. Run `proof pin <uri> --id <id>` — this adds `[[davinci]]` to proof.toml automatically
4. Subsequent compiles: invariants are validated silently

## Benefits

- The expected pin is declared **where the figure is used**, not just in proof.toml
- If someone removes the `[[davinci]]` entry by accident, the next compile warns immediately
- Works alongside the normal DaVinci validation — no double-counting
