# Compiler Bible

proof treats documentation like source code: authored sources compile into
reviewable artifacts, and each compiler phase has a narrow owner. This guide is
the map for changing the compiler without turning `compile.rs` back into a
catch-all module.

---

## Waves and pulses

A **wave** is a mission with a clear outcome: for example, "split directive
renderers by artifact family." A **pulse** is one small, independently validated
change inside that wave.

Good pulses:

- move one boundary at a time
- preserve emitted markdown byte-for-byte unless the wave explicitly changes UX
- add or keep focused tests near the moved behavior
- run the smallest focused tests first, then the full validation gate
- end with a commit that names the boundary

```proof:tree kind=dependency
root: Wave 8 directive module split
- Pulse: directive parser facade
- Pulse: source resolution boundary
- Pulse: chart data resolver
- Pulse: tree renderer
- Pulse: element and row renderer
- Pulse: slide compiler
- Pulse: dashboard compiler
- Pulse: region no-chrome renderer
- Pulse: compile validation helpers
```

---

## Compiler phase map

`compile.rs` is the facade. It owns command-visible orchestration: reading the
source, dispatching special artifact routes, applying replacements, writing
outputs, cache manifest updates, and public result types.

Focused modules own behavior families:

| Module | Ownership |
|--------|-----------|
| `compile_directive` | Directive spans, typed `Directive`, attribute parsing, collection |
| `compile_source` | `md://` resolution, query transforms, include/table source helpers |
| `compile_format` | Compiled traceability wrappers |
| `compile_element` | `proof:element` and `proof:row` data resolution/rendering |
| `compile_chart` | Chart data resolution from inline bodies or tables |
| `compile_tree` | `proof:tree`, inline tree, and outline rendering |
| `compile_toc` | TOC heading collection, section scoping, and formatting |
| `compile_prose` | Prose-only xref and blockquote rendering |
| `compile_symbol` | Symbol and shape rendering |
| `compile_math` | Display and inline math rendering |
| `compile_crop` | CROP side-info rendering and dependency cache keys |
| `compile_slides` | `.slides.source.md` compile route |
| `compile_dashboard` | `.dashboard.source.md` compile route |
| `compile_region` | Dashboard region body and no-chrome embedded directive rendering |
| `compile_validation` | Figure linting and DaVinci invariant checks |

---

## Source-to-artifact examples

General sources use the standard route:

```text
src/guides/07-compile.source.md
  -> collect directives
  -> resolve/render replacements
  -> docs/guides/07-compile.md
```

Slides and dashboards dispatch before the general route:

```text
src/guides/04-slides.slides.source.md
  -> compile_slides
  -> docs/guides/04-slides.slides.md

src/user-scenarios/03-metrics-dashboard.dashboard.source.md
  -> compile_dashboard
  -> docs/user-scenarios/03-metrics-dashboard.dashboard.md
```

Embedded dashboard directives intentionally use no-chrome rendering:

```text
proof:region body
  -> compile_region::render_region_body
  -> compile_region::render_one_directive_no_chrome
  -> raw glyph rows pasted into the dashboard canvas
```

---

## Change checklist

Before changing a compiler phase:

1. Identify the artifact family: parser, source resolver, renderer, compositor,
   validation, or output formatting.
2. Reuse the existing module boundary if one exists.
3. Keep `CompileViolation` codes and messages stable unless the wave is a
   diagnostic UX change.
4. Preserve cache dependencies: if a directive reads side-info or an external
   `md://` source, the compile key must include that input.
5. Validate with focused tests and then the full suite.

The preferred end state is boring: `compile.rs` should read like the compiler
table of contents, not like the implementation of every directive.
