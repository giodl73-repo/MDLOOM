# MDLOOM + MDCROP Corpus Intelligence

MDLOOM owns compilation, rendering, artifact manifests, and mdloom-specific
semantics. MDCROP owns reusable corpus intelligence: status pages, link graphs,
backlinks, frontmatter inventories, heading inventories, and named corpus views.

The `mdloom mdcrop` command is a thin adapter over the MDCROP CLI. It keeps MDCROP
optional while making the recommended integration workflow discoverable from
MDLOOM.

For author-facing corpus pages, prefer the first-class MDLOOM commands:

```text
mdloom index --root docs/guides --output docs/INDEX.md
mdloom toc --root docs/guides --output docs/TOC.md
mdloom catalog --view .mdcrop/views/ready-guides.json --output docs/CATALOG.md
```

These commands are backed by MDCROP's `index` and `catalog` engines, but they are
MDLOOM authoring surfaces. Use `mdloom mdcrop ...` when you need a lower-level MDCROP
report directly.

---

## Generate a corpus status page

Use MDCROP status when you want a generated Markdown overview for a MDLOOM guide
set, docs folder, or view recipe:

```text
mdloom mdcrop status --root docs/guides --output docs/STATUS.md
```

For CI, strict mode writes the Markdown artifact first and then relays MDCROP's
non-zero exit code when the corpus has broken links, orphan pages, or duplicate
anchors:

```text
mdloom mdcrop status --root docs/guides --strict --output docs/STATUS.md
```

Use repeatable `--strict-on` selectors when a gate should start with a narrower
policy while still using MDCROP's strict status semantics. `--strict-on` requires
`--strict` and accepts `broken-links`, `orphan-pages`, or `duplicate-anchors`,
so unknown policy selectors fail at argument parsing instead of being passed as
ignored advisory flags:

```text
mdloom mdcrop status --root docs/guides --strict --strict-on broken-links --output docs/STATUS.md
```

Use `--format json` when an agent, registry, or CI job should consume MDCROP's
`mdcrop.corpus-status.v1` contract instead of Markdown:

```text
mdloom mdcrop status --view .mdcrop/views/ready-guides.json --format json --strict --strict-on broken-links --output READY_GUIDES.status.json
```

For the same MDCROP-backed health surface from the top-level status command, use
`mdloom status --mdcrop`. The local `mdloom status` summary remains the default:

```text
mdloom status --mdcrop --view .mdcrop/views/ready-guides.json --mdcrop-format json --strict --strict-on broken-links -o READY_GUIDES.status.json
```

When using `mdloom status --mdcrop`, pass either a positional directory for root
mode or `--view` for named-view mode; MDLOOM rejects combining both so a local
status directory is not silently ignored.

`mdloom mdcrop status` also honors MDLOOM's global `-o/--output` and `-f/--format`
when command-local `--output` or `--format` values are not supplied. Command-local
`mdloom mdcrop status --format` and `mdloom status --mdcrop --mdcrop-format` values
follow MDCROP's contract exactly: use `markdown` or `json`; `text` is rejected
instead of being treated as an alias.

You can pass the same generic filters MDCROP exposes:

```text
mdloom mdcrop status --root docs --extension md --exclude-dir target
```

In `--view` mode, omit those filters to preserve the recipe exactly. Supplying
`--extension` overrides the recipe extension allow-list for that run, while
`--exclude-dir` extends the recipe's excluded directory basenames.

---

## Use named MDCROP views

MDCROP views let MDLOOM reuse named slices of a larger corpus without baking those
selection rules into MDLOOM. A view file is a `mdcrop.view.v1` JSON recipe:

```json
{
  "schema_version": "mdcrop.view.v1",
  "name": "ready-guides",
  "root": "docs/guides",
  "task": "ready guide corpus",
  "token_budget": 12000,
  "seed": 0,
  "include_extensions": ["md"],
  "frontmatter_query": "tags has 'guide' and status eq 'ready'"
}
```

Generate a status page from the view:

```text
mdloom mdcrop status --view .mdcrop/views/ready-guides.json --output READY_GUIDES.md
```

Generate a reusable view recipe from MDLOOM's config and source-frontmatter
selection flags:

```text
mdloom mdcrop view --root src/guides --output .mdcrop/views/ready-guides.json --name ready-guides --frontmatter-query "status eq 'ready'" --tag guide --op compile
```

`mdloom mdcrop view` maps `mdloom.toml` `[files].include` to MDCROP
`include_extensions`, maps simple `[files].exclude` directory globs to
`exclude_dirs`, accepts a raw `--frontmatter-query`, and maps `--tag`, `--op`,
and `--content-tag` to additional MDCROP `frontmatter_query` clauses. The
generated `root` is written relative to the view file, so the recipe can move
with `.mdcrop/views`. The command also honors global `-o/--output` when local
`--output` is omitted. Because the recipe is JSON, non-JSON global formats such
as `-f markdown` are rejected. The resulting view can be reused anywhere a MDCROP
command accepts `--view`.

List the recipes in a view store when a CI job, registry, or review workflow
needs the machine-readable inventory:

```text
mdloom mdcrop list-views --dir .mdcrop/views -o MDCROP_VIEWS.json
```

`mdloom mdcrop list-views` delegates to MDCROP's `view --list` surface, honors global
`-o/--output` when local `--output` is omitted, and rejects non-JSON global
formats before invoking MDCROP.

Run the named view as a JSON context pack when an agent or review workflow needs
the corpus slice itself:

```text
mdloom mdcrop run-view --file .mdcrop/views/ready-guides.json --query "refresh guide index" -o READY_GUIDES.pack.json
```

`mdloom mdcrop run-view` delegates to MDCROP's `view --file` surface and forwards
one-off `--query`, `--extension`, `--exclude-dir`, and `--prefix-cache` values.
`--prefix-cache` currently accepts MDCROP's `generic` profile and rejects unknown
profiles at argument parsing. The command also honors global `-o/--output` when
local `--output` is omitted. View packs are JSON-only, so non-JSON global
formats such as `-f markdown` are rejected before MDCROP is invoked.

Generate first-class authoring pages from the same view:

```text
mdloom index --view .mdcrop/views/ready-guides.json --output INDEX.md
mdloom catalog --view .mdcrop/views/ready-guides.json --output CATALOG.md
```

These top-level page commands also honor MDLOOM's global `-o/--output`, so
`mdloom index -o INDEX.md --view .mdcrop/views/ready-guides.json` is equivalent to
passing the command-local `--output`. They are Markdown-only page generators, so
non-Markdown global formats such as `-f json` are rejected instead of ignored.

Preflight every recipe in a view store:

```text
mdloom mdcrop inspect-views --dir .mdcrop/views --strict
```

Preflight one recipe while authoring it:

```text
mdloom mdcrop inspect-views --file .mdcrop/views/ready-guides.json
```

When previewing a one-off context-pack run, add the same `--query`,
`--extension`, and `--exclude-dir` overrides that `mdloom mdcrop run-view` would
forward:

```text
mdloom mdcrop inspect-views --file .mdcrop/views/ready-guides.json --query "refresh guide index" --extension md
```

Add `--output inspect.json` or global `-o inspect.json` to save MDCROP's JSON
inspection report as a CI or review artifact; MDLOOM writes the captured report
even when strict inspection returns a non-zero exit code. Inspection reports are
JSON-only, so non-JSON global formats such as `-f markdown` are rejected before
MDCROP is invoked. Strict mode applies to store inspection (`--dir`) and is
rejected with single-file inspection (`--file`) because a single invalid recipe
already fails during inspection. `--file` and a custom `--dir` are mutually
exclusive so a store path is not silently ignored during single-file inspection.

---

## Generate side-info reports

For machine-readable corpus side-info, use the report wrappers. They default to
JSON and also support Markdown when a human-readable table is useful:

```text
mdloom mdcrop links --view .mdcrop/views/ready-guides.json --output .mdloom/side-info/links.json
mdloom mdcrop backlinks --view .mdcrop/views/ready-guides.json --output .mdloom/side-info/backlinks.json
mdloom mdcrop frontmatter --view .mdcrop/views/ready-guides.json --output .mdloom/side-info/frontmatter.json
mdloom mdcrop headings --view .mdcrop/views/ready-guides.json --output .mdloom/side-info/headings.json
```

Each report also accepts `--root`, `--extension`, `--exclude-dir`, `--format
json`, and `--format markdown`. MDLOOM relays MDCROP's exit code so CI can fail on
MDCROP-side report errors without MDLOOM reimplementing those checks.
The report wrappers and snippet list commands honor global `-o/--output` when
their command-local `--output` is omitted.

When MDLOOM source files need corpus side-info during compilation, sync all JSON
reports into the compiler's default side-info store:

```text
mdloom mdcrop prepare --view .mdcrop/views/ready-guides.json
mdloom mdcrop sync --view .mdcrop/views/ready-guides.json
```

This writes `.mdloom/side-info/links.json`, `.mdloom/side-info/backlinks.json`,
`.mdloom/side-info/frontmatter.json`, and `.mdloom/side-info/headings.json`.
`prepare` and `sync` produce JSON artifacts, so non-JSON global formats such as
`-f markdown` are rejected. Because they write a set of side-info files, global
`-o/--output` is rejected; use `--output-dir` to choose the destination
directory.
Use `mdloom mdcrop prepare` when you want the repeatable docs preflight: it first
strictly inspects `.mdcrop/views`, inspects the exact `--view` recipe, then runs
the same side-info sync.
When a source uses `mdloom:links`, `mdloom:backlinks`, `mdloom:headings`, or
`mdloom:frontmatter`, `mdloom compile` records the matching MDCROP JSON as a
resolved input in `.mdloom/artifacts.json` and includes it in the compile cache
key, so rerun `mdloom mdcrop sync` before compiling when the corpus graph has
changed.

For README or non-compiled Markdown authoring, render a target-specific
backlink snippet directly from the synced side-info:

```text
mdloom mdcrop link-list --source README.md --status broken --format table --output LINKS.md
mdloom mdcrop backlink-list --target README.md
mdloom mdcrop backlink-list --target README.md --format table --output BACKLINKS.md
mdloom mdcrop heading-list --source README.md
mdloom mdcrop heading-list --source README.md --format table --output OUTLINE.md
mdloom mdcrop frontmatter-list --field tags --value guide --format table --output GUIDES.md
```

These snippet commands render Markdown snippets with command-local
`--format list|table|count`; non-Markdown global formats such as `-f json` are
rejected instead of ignored. `mdloom mdcrop link-list --status` is limited to
`all|ok|broken`, and `mdloom mdcrop frontmatter-list --op` is limited to `has|eq`,
so invalid filter values fail before side-info files are read.

MDLOOM dogfoods this workflow with `.mdcrop/views/mdloom-guides.json`, a
`mdcrop.view.v1` recipe generated from the mdloom-authored guide sources:

```text
mdloom mdcrop view --root src/guides --output .mdcrop/views/mdloom-guides.json --name mdloom-guides --extension md
mdloom mdcrop prepare --view .mdcrop/views/mdloom-guides.json
mdloom mdcrop backlink-list --target 12-mdcrop.source.md --format table
mdloom mdcrop heading-list --source 12-mdcrop.source.md --format count
mdloom compile src/guides/12-mdcrop.source.md --output docs/guides/12-mdcrop.md
```

The blocks below are generated by this guide from MDCROP side-info. The backlink
block renders the same empty state authors see when a target has no inbound
links in the current view; the link and heading counts come from MDCROP link and
heading inventories for this same source.

```mdloom:backlinks target="12-mdcrop.source.md" format=table
```

```mdloom:links source="12-mdcrop.source.md" format=count
```

```mdloom:headings source="12-mdcrop.source.md" format=count
```

---

## Insert backlink lists in source documents

After `mdloom mdcrop sync`, authors can render inbound links directly from MDCROP's
backlink graph:

````markdown
\`\`\`mdloom:backlinks target="reference.source.md"
\`\`\`
````

By default the directive reads `.mdloom/side-info/backlinks.json` and renders a
Markdown list. Use `format=count` for a numeric count or `format=table` for a
source/target table:

````markdown
\`\`\`mdloom:backlinks target="reference.source.md" format=table
\`\`\`
````

Use `side-info="path/to/backlinks.json"` when a source should consume a
non-default MDCROP report.

---

## Insert link audit summaries in source documents

After `mdloom mdcrop sync`, authors can render outbound links or broken-link
summaries directly from MDCROP's link audit:

````markdown
\`\`\`mdloom:links source="reference.source.md" status=broken
\`\`\`
````

By default the directive reads `.mdloom/side-info/links.json` and renders a
Markdown list. Omit `source` to summarize all audited links, use `status=ok`,
`status=broken`, or `status=all`, and use `format=count` or `format=table` for
compact dashboards:

````markdown
\`\`\`mdloom:links status=broken format=table
\`\`\`
````

Use `side-info="path/to/links.json"` when a source should consume a non-default
MDCROP report.

---

## Insert source outlines in source documents

After `mdloom mdcrop sync`, authors can render a source outline directly from
MDCROP's heading inventory:

````markdown
\`\`\`mdloom:headings source="reference.source.md"
\`\`\`
````

By default the directive reads `.mdloom/side-info/headings.json` and renders a
Markdown outline. Use `format=count` for a numeric count or `format=table` for a
level/heading/URI table:

````markdown
\`\`\`mdloom:headings source="reference.source.md" format=table
\`\`\`
````

Use `side-info="path/to/headings.json"` when a source should consume a
non-default MDCROP report.

---

## Insert frontmatter-driven source lists

After `mdloom mdcrop sync`, authors can render metadata-driven source lists from
MDCROP's frontmatter inventory:

````markdown
\`\`\`mdloom:frontmatter field=tags value=guide
\`\`\`
````

By default the directive reads `.mdloom/side-info/frontmatter.json` and renders a
Markdown list using each page's `title` field when present. Use `format=count`
for a numeric count or `format=table` for a source/field table:

````markdown
\`\`\`mdloom:frontmatter field=status value=ready op=eq format=table
\`\`\`
````

`op=has` is the default and is useful for array-like fields such as
`tags: [mdloom, guide]`; use `op=eq` for exact scalar values such as
`status: ready`. Use `side-info="path/to/frontmatter.json"` when a source should
consume a non-default MDCROP report.

---

## Check generated artifact health

After `mdloom compile` writes `.mdloom/artifacts.json`, MDCROP can report generated
artifact health through its MDLOOM manifest adapter:

```text
mdloom mdcrop artifacts --manifest .mdloom/artifacts.json --format markdown --output ARTIFACTS.md
```

Use this for missing, stale, cached, or diagnostic artifact rows. Pass either
`--manifest` or `--root`; MDLOOM rejects missing or combined selectors before
invoking MDCROP. Artifact reports use MDCROP's `json`/`markdown` format contract, so
unsupported global formats such as `-f rich` are rejected before invoking MDCROP.
Generic corpus status pages should still use `mdloom mdcrop status`; artifact
health is a MDLOOM-manifest adapter over generated outputs.

---

## Choosing MDLOOM vs. MDCROP

Use MDLOOM when the task is about compiling `.source.md`, rendering charts,
slides, dashboards, math, symbols, HTML, or artifact manifests.

Use MDCROP when the task is about corpus inventory, links, backlinks,
frontmatter, headings, named corpus slices, or generated status pages.

If MDLOOM needs additional MDCROP behavior, file the request against MDCROP as a
generic corpus capability with a small fixture, input contract, output contract,
and acceptance tests.
