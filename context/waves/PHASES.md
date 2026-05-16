# proof Wave Phases

> Find the first row with `status: active`; that is the work rail for
> `/proof-wave next` and `/proof-pulse`.

proof keeps public release meaning in `CHANGELOG.md`. Waves are the execution
record beneath that history: each wave has a mission, pulses, gates, reviews,
close notes, and the artifacts that changed the system.

## Waves

| Date | Wave | Mission | Status |
|---|---|---|---|
| 2026-04-25 | ASCII Linter Foundation | Bootstrap proof as a schema-driven markdown and ASCII-art checker with box, flow, tree, and markdown rules. | archived |
| 2026-04-25 | Addressing and Pinning | Rename glint to proof, introduce `md://`, compile, resolve, layout, and DaVinci pinning. | archived |
| 2026-04-26 | Figure and Canvas Toolchain | Add figures, symbols, elements, slides, dashboards, mapping, and renderer specs. | archived |
| 2026-04-27 | Rendering and Compile Expansion | Ship math, source-link checks, tree/chart/slide/dashboard directives, guides, and multi-target watch. | archived |
| 2026-04-28 | Author Experience Release | Add xref, chart, reveal, AI CLI config, status, depends, unused checks, and richer diagnostics. | archived |
| 2026-05-14 | Architecture and Quality Review Rail | Backfill wave/pulse planning, reconcile spec/docs, add missing coverage, and clean local warnings from the quality review. | done |
| 2026-05-14 | Schema and Corpus Signal | Resolve schema compatibility choices, reduce MAXIM warning flood, and decide sibling mdpath warning cleanup. | done |
| 2026-05-14 | Open H2 Schema Policy | Make required H2 sections enforce presence without closing broad corpus schemas; keep `optional_h2` as explicit allowlist policy. | done |
| 2026-05-14 | ASCII Corpus Signal | Reduce confirmed proof-side ASCII false positives while preserving the no-error MAXIM gate. | done |
| 2026-05-14 | Wide Character Policy | Honor `ascii_char.error_on_wide=false` as intentional wide-content suppression and remove MAXIM `ascii_char_range` noise. | done |
| 2026-05-14 | Formula Padding Signal | Stop treating absolute-value and proof-notation pipes outside bordered boxes as ASCII table cells. | done |
| 2026-05-14 | Connector Drift Signal | Restrict connector drift to connector-only lines so timelines, formulas, and labeled drawings are not over-linted. | done |
| 2026-05-14 | Box Column Signal | Treat row separators and embedded borders as non-actionable bottom-column diffs while preserving width and missing-column errors. | done |
| 2026-05-14 | Compact Cell Padding | Suppress padding warnings for full-width cells that cannot add spaces without widening the box. | done |
| 2026-05-14 | Markdown Table Padding | Align markdown table padding with ignored extra-column rows and no-room cell policy. | done |
| 2026-05-15 | Arrow and Barchart Signal | Narrow arrow-gap and barchart detectors to proof-owned diagram cases while preserving real chart/arrow diagnostics. | done |
| 2026-05-14 | Residual ASCII Signal | Finish residual barchart, box-column, and arrow false positives from stacked bars, connector ports, spanning rows, tree branches, and layout gaps. | done |
| 2026-05-14 | Low-volume Markdown Signal | Remove residual link/table false positives from math notation and comparison-matrix corner headers. | done |
| 2026-05-14 | Remaining Signal Classification | Classify the remaining MAXIM warning families and separate proof-owned detector work from MAXIM content/schema carry-forward. | done |
| 2026-05-15 | Proof Architecture Config Boundary | Repair CLI/runner config ownership so explicit configs and file-selection cascade semantics match the spec. | done |
| 2026-05-15 | Spec Registry Boundary | Centralize diagnostic code ownership and update the design contract for proof's expanded command/config surface. | done |
| 2026-05-15 | Explicit Config Failure | Make explicit `--config` authoritative by failing loudly on missing or invalid config files. | done |
| 2026-05-15 | Extends Cascade Semantics | Make `extends` stop automatic ancestor cascade and inherit only the explicit parent plus child config. | done |
| 2026-05-15 | Effective Config Command | Make `proof config [PATH]` print the resolved effective config instead of a placeholder. | done |
| 2026-05-15 | Progress Surface Contract | Align CLI/spec progress behavior by documenting progress as a compile-only option and locking help output. | done |
| 2026-05-15 | Actual File Counts | Make check/stats summaries count the runner's actual include/exclude-selected files. | done |
| 2026-05-15 | Runner Summary API | Return diagnostics and selected file count from one runner pass to avoid duplicate directory walks. | done |
| 2026-05-15 | Loaded Config Explicitness | Move parser-only config explicitness out of effective runtime config while preserving cascade semantics. | done |
| 2026-05-15 | Named Run Summary | Promote runner directory results from tuple return values to a named summary type. | done |
| 2026-05-15 | Runner Path Summary | Let runner own file-vs-directory summary behavior instead of duplicating it in commands. | done |
| 2026-05-15 | CLI Lint Summary Helper | Centralize config loading, runner construction, and lint summary aggregation for CLI commands. | done |
| 2026-05-15 | Library Lint Orchestration | Extract shared lint orchestration from the CLI into a reusable proof_lib module. | done |
| 2026-05-15 | Stats Command Module | Extract `proof stats` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Draft Command Module | Extract `proof draft` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Config Command Module | Extract `proof config` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Status Command Module | Extract `proof status` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Init Command Module | Extract `proof init` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Pin List Command Module | Extract `proof pin-list` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Pin Command Module | Extract `proof pin` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Check Command Module | Extract `proof check` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Fix Command Module | Extract `proof fix` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Resolve Command Module | Extract `proof resolve` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Depends Command Module | Extract `proof depends` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Tree Command Module | Extract `proof tree` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Spec Generate Command Module | Extract `proof spec-generate` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Layout Command Module | Extract `proof layout` from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | Compile Command Module | Extract `proof compile` and watch mode from the monolithic CLI into a focused command module. | done |
| 2026-05-15 | CLI Command Shell Cleanup | Simplify `main.rs` after command extraction by removing dead dispatch scaffolding and centralizing path defaults. | done |
| 2026-05-15 | Check Command Args | Move `proof check` clap argument definitions into the check command module. | done |
| 2026-05-15 | Compile Command Args | Move `proof compile` clap argument definitions into the compile command module. | done |
| 2026-05-15 | Layout Command Args | Move `proof layout` clap argument definitions into the layout command module. | done |
| 2026-05-15 | Spec Generate Command Args | Move `proof spec-generate` clap argument definitions into the spec-generate command module. | done |
| 2026-05-15 | Addressing Lookup Command Args | Move `proof resolve` and `proof depends` clap argument definitions into their command modules. | done |
| 2026-05-15 | Pin Command Args | Move `proof pin` clap argument definitions into the pin command module. | done |
| 2026-05-15 | Draft Command Args | Move `proof draft` clap argument definitions into the draft command module. | done |
| 2026-05-15 | Reporting Command Args | Move `proof config`, `proof status`, and `proof stats` clap argument definitions into their command modules. | done |
| 2026-05-15 | Fix Command Args | Move `proof fix` clap argument definitions into the fix command module. | done |
| 2026-05-15 | Tree Command Args | Move the `proof tree` subcommand wrapper into the tree command module. | done |
| 2026-05-15 | Check Dispatch Flags | Move check-specific dispatch flag aggregation into the check command module. | done |
| 2026-05-15 | CLI Parser Module | Move the root clap parser and command enum into a dedicated CLI module. | done |
| 2026-05-15 | CLI Dispatch Routing | Separate explicit command dispatch from the default check route in `main.rs`. | done |
| 2026-05-15 | CLI Global Options | Group root-level global CLI options into one dispatch context. | done |
| 2026-05-15 | Addressing Dispatch Adapters | Let `proof resolve` and `proof depends` command modules consume their own args. | done |
| 2026-05-15 | Reporting Dispatch Adapters | Let `proof config` and `proof status` command modules consume their own args. | done |
| 2026-05-15 | Action Dispatch Adapters | Let `proof fix`, `proof pin`, and `proof tree` command modules consume their own args. | done |
| 2026-05-15 | Generation Dispatch Adapters | Let `proof draft`, `proof stats`, and `proof spec-generate` command modules consume their own args. | done |
| 2026-05-15 | Render Dispatch Adapters | Let `proof compile` and `proof layout` command modules consume their own args. | done |
| 2026-05-15 | CLI Global Context | Move parsed global CLI context ownership into the CLI module. | done |
| 2026-05-15 | CLI Path Defaults | Move shared CLI path default helpers into the CLI module. | done |
| 2026-05-15 | Check Global Adapter | Move default-check global option adaptation into the check command module. | done |
| 2026-05-15 | CLI Dispatch Module | Move command routing out of `main.rs` into a dedicated dispatch module. | done |
| 2026-05-15 | CLI Path Extraction | Centralize command path extraction helpers in the CLI module. | done |
| 2026-05-15 | Dispatch Expression Routing | Normalize command routing as a single match expression. | done |
| 2026-05-15 | Command Args Visibility | Tighten remaining command argument field visibility after dispatch extraction. | done |
| 2026-05-15 | Command Path Accessors | Keep command path extraction behind module-owned accessors instead of public fields. | done |
| 2026-05-15 | Source Frontmatter Tags | Add source-only frontmatter tags, op tags, content tags, stats summaries, and compile stripping. | done |
| 2026-05-15 | Command Args Privacy | Make command argument fields private now that dispatch routes through module-owned adapters. | done |
| 2026-05-15 | CLI Global Accessors | Keep root CLI parser and global option fields private behind explicit accessors. | done |
| 2026-05-15 | Command Path Default Adapters | Move command path fallback behavior out of dispatch and into command modules. | done |
| 2026-05-15 | Command Config Global Adapters | Move global config override plumbing into config-aware command adapters. | done |
| 2026-05-15 | CLI Dispatch Input | Replace positional CLI parse tuples with a named dispatch input boundary. | done |
| 2026-05-15 | Dispatch Context | Bundle globals and default-check paths into a named dispatch routing context. | done |
| 2026-05-15 | Dispatch Input Privacy | Keep dispatch input fields private behind named accessors and command extraction. | done |
| 2026-05-15 | Dispatch Context Runner | Move command/default routing behind `DispatchContext::run`. | done |
| 2026-05-15 | Command Path Helper Module | Move shared command cwd path defaulting out of the CLI parser module. | done |
| 2026-05-15 | Command Global Options Module | Move global command execution options out of the CLI parser module. | done |
| 2026-05-15 | Check Path Helper | Move check/default-check path fallback into the shared command path helper module. | done |
| 2026-05-15 | Owned Dispatch Context | Consume dispatch input into an owned routing context instead of borrowing parser state. | done |
| 2026-05-15 | Command-Owned Dispatch Context | Store selected command in the owned dispatch routing context. | done |
| 2026-05-15 | Status Global Config Adapter | Make `proof status` honor explicit global config overrides in its config summary. | done |
| 2026-05-15 | Compiler Typesetter Spec | Reframe proof's core spec as a compiler/LaTeX-style typesetter contract. | done |
| 2026-05-15 | Reverse Backfill Spec | Specify `proof backfill` as the quick adoption bridge from existing markdown to generated source. | done |
| 2026-05-15 | Backfill Review Role | Add a reverse-adoption role for migration fidelity, extraction confidence, and cutover safety. | done |
| 2026-05-15 | Spec Alignment Implementation Plan | Review spec drift and define the remaining compiler/typesetter implementation waves. | done |
| 2026-05-15 | Backfill MVP | Add literal-first `proof backfill` source mirroring, provenance, reports, and round-trip checks. | done |
| 2026-05-15 | Backfill Classifiers | Add advisory block inventory and evidence to backfill reports while preserving literal source generation. | done |
| 2026-05-15 | Backfill Table Extraction | Add opt-in markdown table sidecar extraction while preserving literal source generation. | done |
| 2026-05-15 | HTML Publish Target | Add `proof compile --target html` as the first non-markdown publish backend and document PPTX as a future target. | done |
| 2026-05-15 | Artifact Manifest | Write target-aware compile provenance to `.proof/artifacts.json` for publish graph and stale-check follow-up work. | done |
| 2026-05-15 | Tag-Driven Operations | Let source frontmatter tags filter check, compile, and stats without changing inclusive defaults. | done |
| 2026-05-15 | Fix Pipeline Consistency | Make `proof fix` honor global config during verification and write a structured application log. | done |
| 2026-05-15 | Directive Parser Boundary | Extract proof directive kind classification, fence/header parsing, directive collection, typed directive ownership, directive payload parsing, prose directive renderers, shared source resolution, chart data helpers, inline tree helpers, and TOC generation from the compile facade. | done |
| 2026-05-16 | Publish Backends | Plan JSON report bundle, static site, PDF, DOCX, and native PPTX publish targets while deferring LaTeX. | done |
| 2026-05-16 | Publication AST and Themes | Introduce a shared publication AST and theme token system for consistent professional publish output. | active |

## Operating Model

Each active wave keeps:

- `WAVE.md`: mission, scope, pulse table, gates, and carry-forwards.
- `pulses/NN+slug.md`: one executable slice with frontmatter, deliverables,
  validation, and non-goals.
- `forks/`: materialized pulse context for agent execution when needed.
- `panels/`: role reviews and consolidated findings.
- `CLOSE.md`: written when the wave is complete.

## Pulse Rules

- A pulse must be independently reviewable and testable.
- Every pulse names governing roles and validation commands.
- Quality claims must be explicit: mechanical proof, Rust tests, docs contract
  review, or deferred carry-forward.
- Completed pulses keep their checkboxes checked; future pulses remain open.
- Wave status only advances after docs, tests, and review findings agree.

## History Bridge

- `CHANGELOG.md` remains the release-history source of truth.
- Backfilled history cites existing release notes and artifacts rather than
  pretending a wave plan existed before the work.
- Closeout may recommend changelog entries, but release notes should stay
  semver-oriented.
