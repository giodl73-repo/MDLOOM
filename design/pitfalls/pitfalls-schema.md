# Schema & Config Pitfalls (SC-01..SC-03)

Failure modes in the schema loading, config composition, and rule interpretation layer.

---

## SC-01: Schema silently ignored when config file not found

**Pattern:** If `load_or_default()` falls through all candidate config paths and returns a default
config, the user gets no feedback. They believe their `glint.toml` is being applied when it isn't
(wrong file name, wrong directory, typo). All checks run with default settings, not schema settings.

**Domain:** CI pipelines and pre-commit hooks where the config path is assumed correct.

**Structural solution:** Add a `--config` flag that is required-or-explicit. When `--config` is
given but the file doesn't exist, fail immediately with a clear error. When auto-detection is used
and no config is found, emit a `note:` line on stderr: `"no glint.toml found — using defaults"`.
This makes silent fallback visible.

**Status:** PARTIAL — `--config` flag exists and errors on missing file; auto-detection is silent.
**Test:** `tests/integration_tests.rs::default_config_loads_without_panic`

---

## SC-02: Glob patterns applied from wrong base directory

**Pattern:** `include = ["**/*.md"]` is matched against paths relative to the `root` argument
passed to `Runner::run()`. If the caller passes an absolute path as root but the glob is written
expecting a relative path from the project directory, no files match and `run()` returns zero
diagnostics — silently.

**Domain:** Integration scenarios where `glint` is invoked from a different working directory
than the project root.

**Structural solution:** Always strip the root prefix before globbing: `path.strip_prefix(root)`.
Add a `--verbose` flag that logs `"checked N files"` — if N=0, the user knows something is wrong
with their include patterns.

**Status:** SOLVED — `matches()` uses `path.strip_prefix(root)` for glob matching.
**Test:** `tests/integration_tests.rs::runner_scans_fixture_dir`

---

## SC-03: Custom rules with `negate = true` cause confusion

**Pattern:** A custom rule with `negate = true` warns when the pattern IS found. A user reads
`negate = true` as "negate the rule" (i.e., "don't apply this rule") rather than "invert the
match sense." When they toggle `negate = true` thinking they're disabling the rule, they instead
flip from "warn when absent" to "warn when present."

**Domain:** Schema authors unfamiliar with lint-rule inversion terminology.

**Structural solution:** Rename the field to make the semantics explicit. Options:
- `match_mode = "present" | "absent"` — warn when pattern is present vs. absent
- `warn_when = "found" | "missing"` — direct statement of when to warn

Until the rename, the schema file comment must be very explicit with an example showing
both states.

**Status:** OPEN — `negate` field kept for now; schema comment documents the behavior.
**Test:** Not yet written — pending field rename decision.
