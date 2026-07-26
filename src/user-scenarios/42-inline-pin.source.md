# System Architecture — Inline Pin Example

This document demonstrates the `pin=id` attribute on `mdloom:include`.

## What the pin attribute does

Adding `pin=goroutine-scheduler` to a `mdloom:include` directive declares
that this figure should be protected by a DaVinci invariant with that ID.

When no matching `[[davinci]]` entry exists in mdloom.toml, COMPILE-007 is
emitted as a warning, prompting you to run `mdloom pin`.

## Workflow

1. Add `pin=id` to the `mdloom:include` in your source document
2. First compile emits COMPILE-007 warning with the exact `mdloom pin` command to run
3. Run `mdloom pin <uri> --id <id>` — this adds `[[davinci]]` to mdloom.toml automatically
4. Subsequent compiles: invariants are validated silently

## Benefits

- The expected pin is declared **where the figure is used**, not just in mdloom.toml
- If someone removes the `[[davinci]]` entry by accident, the next compile warns immediately
- Works alongside the normal DaVinci validation — no double-counting
