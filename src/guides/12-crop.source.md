# PROOF + CROP Corpus Intelligence

PROOF owns compilation, rendering, artifact manifests, and proof-specific
semantics. CROP owns reusable corpus intelligence: status pages, link graphs,
backlinks, frontmatter inventories, heading inventories, and named corpus views.

The `proof crop` command is a thin adapter over the CROP CLI. It keeps CROP
optional while making the recommended integration workflow discoverable from
PROOF.

For author-facing corpus pages, prefer the first-class PROOF commands:

```text
proof index --root docs/guides --output docs/INDEX.md
proof toc --root docs/guides --output docs/TOC.md
proof catalog --view .crop/views/ready-guides.json --output docs/CATALOG.md
```

These commands are backed by CROP's `index` and `catalog` engines, but they are
PROOF authoring surfaces. Use `proof crop ...` when you need a lower-level CROP
report directly.

---

## Generate a corpus status page

Use CROP status when you want a generated Markdown overview for a PROOF guide
set, docs folder, or view recipe:

```text
proof crop status --root docs/guides --output docs/STATUS.md
```

For CI, strict mode writes the Markdown artifact first and then relays CROP's
non-zero exit code when the corpus has broken links, orphan pages, or duplicate
anchors:

```text
proof crop status --root docs/guides --strict --output docs/STATUS.md
```

You can pass the same generic filters CROP exposes:

```text
proof crop status --root docs --extension md --exclude-dir target
```

---

## Use named CROP views

CROP views let PROOF reuse named slices of a larger corpus without baking those
selection rules into PROOF. A view file is a `crop.view.v1` JSON recipe:

```json
{
  "schema_version": "crop.view.v1",
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
proof crop status --view .crop/views/ready-guides.json --output READY_GUIDES.md
```

Generate a reusable view recipe from PROOF's config and source-frontmatter
selection flags:

```text
proof crop view --root src/guides --output .crop/views/ready-guides.json --name ready-guides --tag guide --op compile
```

`proof crop view` maps `proof.toml` `[files].include` to CROP
`include_extensions`, maps simple `[files].exclude` directory globs to
`exclude_dirs`, and maps `--tag`, `--op`, and `--content-tag` to CROP
`frontmatter_query` clauses. The generated `root` is written relative to the
view file, so the recipe can move with `.crop/views`. The resulting view can be
reused anywhere a CROP command accepts `--view`.

Generate first-class authoring pages from the same view:

```text
proof index --view .crop/views/ready-guides.json --output INDEX.md
proof catalog --view .crop/views/ready-guides.json --output CATALOG.md
```

Preflight every recipe in a view store:

```text
proof crop inspect-views --dir .crop/views --strict
```

---

## Generate side-info reports

For machine-readable corpus side-info, use the report wrappers. They default to
JSON and also support Markdown when a human-readable table is useful:

```text
proof crop links --view .crop/views/ready-guides.json --output .proof/side-info/links.json
proof crop backlinks --view .crop/views/ready-guides.json --output .proof/side-info/backlinks.json
proof crop frontmatter --view .crop/views/ready-guides.json --output .proof/side-info/frontmatter.json
proof crop headings --view .crop/views/ready-guides.json --output .proof/side-info/headings.json
```

Each report also accepts `--root`, `--extension`, `--exclude-dir`, `--format
json`, and `--format markdown`. PROOF relays CROP's exit code so CI can fail on
CROP-side report errors without PROOF reimplementing those checks.

When PROOF source files need corpus side-info during compilation, sync all JSON
reports into the compiler's default side-info store:

```text
proof crop prepare --view .crop/views/ready-guides.json
proof crop sync --view .crop/views/ready-guides.json
```

This writes `.proof/side-info/links.json`, `.proof/side-info/backlinks.json`,
`.proof/side-info/frontmatter.json`, and `.proof/side-info/headings.json`.
Use `proof crop prepare` when you want the repeatable docs preflight: it first
strictly inspects `.crop/views`, then runs the same side-info sync.
When a source uses `proof:backlinks`, `proof:headings`, or
`proof:frontmatter`, `proof compile` records the matching CROP JSON as a
resolved input in `.proof/artifacts.json` and includes it in the compile cache
key, so rerun `proof crop sync` before compiling when the corpus graph has
changed.

For README or non-compiled Markdown authoring, render a target-specific
backlink snippet directly from the synced side-info:

```text
proof crop backlink-list --target README.md
proof crop backlink-list --target README.md --format table --output BACKLINKS.md
proof crop heading-list --source README.md
proof crop heading-list --source README.md --format table --output OUTLINE.md
proof crop frontmatter-list --field tags --value guide --format table --output GUIDES.md
```

PROOF dogfoods this workflow with `.crop/views/proof-guides.json`, a
`crop.view.v1` recipe generated from the proof-authored guide sources:

```text
proof crop view --root src/guides --output .crop/views/proof-guides.json --name proof-guides --extension md
proof crop prepare --view .crop/views/proof-guides.json
proof crop backlink-list --target 12-crop.source.md --format table
proof crop heading-list --source 12-crop.source.md --format count
proof compile src/guides/12-crop.source.md --output docs/guides/12-crop.md
```

The block below is generated by this guide from CROP backlink side-info. It
renders the same empty state authors see when a target has no inbound links in
the current view. The following heading count is generated from CROP heading
side-info for this same source.

```proof:backlinks target="12-crop.source.md" format=table
```

```proof:headings source="12-crop.source.md" format=count
```

---

## Insert backlink lists in source documents

After `proof crop sync`, authors can render inbound links directly from CROP's
backlink graph:

````markdown
\`\`\`proof:backlinks target="reference.source.md"
\`\`\`
````

By default the directive reads `.proof/side-info/backlinks.json` and renders a
Markdown list. Use `format=count` for a numeric count or `format=table` for a
source/target table:

````markdown
\`\`\`proof:backlinks target="reference.source.md" format=table
\`\`\`
````

Use `side-info="path/to/backlinks.json"` when a source should consume a
non-default CROP report.

---

## Insert source outlines in source documents

After `proof crop sync`, authors can render a source outline directly from
CROP's heading inventory:

````markdown
\`\`\`proof:headings source="reference.source.md"
\`\`\`
````

By default the directive reads `.proof/side-info/headings.json` and renders a
Markdown outline. Use `format=count` for a numeric count or `format=table` for a
level/heading/URI table:

````markdown
\`\`\`proof:headings source="reference.source.md" format=table
\`\`\`
````

Use `side-info="path/to/headings.json"` when a source should consume a
non-default CROP report.

---

## Insert frontmatter-driven source lists

After `proof crop sync`, authors can render metadata-driven source lists from
CROP's frontmatter inventory:

````markdown
\`\`\`proof:frontmatter field=tags value=guide
\`\`\`
````

By default the directive reads `.proof/side-info/frontmatter.json` and renders a
Markdown list using each page's `title` field when present. Use `format=count`
for a numeric count or `format=table` for a source/field table:

````markdown
\`\`\`proof:frontmatter field=status value=ready op=eq format=table
\`\`\`
````

`op=has` is the default and is useful for array-like fields such as
`tags: [proof, guide]`; use `op=eq` for exact scalar values such as
`status: ready`. Use `side-info="path/to/frontmatter.json"` when a source should
consume a non-default CROP report.

---

## Check generated artifact health

After `proof compile` writes `.proof/artifacts.json`, CROP can report generated
artifact health through its PROOF manifest adapter:

```text
proof crop artifacts --manifest .proof/artifacts.json --format markdown --output ARTIFACTS.md
```

Use this for missing, stale, cached, or diagnostic artifact rows. Generic corpus
status pages should still use `proof crop status`; artifact health is a
PROOF-manifest adapter over generated outputs.

---

## Choosing PROOF vs. CROP

Use PROOF when the task is about compiling `.source.md`, rendering charts,
slides, dashboards, math, symbols, HTML, or artifact manifests.

Use CROP when the task is about corpus inventory, links, backlinks,
frontmatter, headings, named corpus slices, or generated status pages.

If PROOF needs additional CROP behavior, file the request against CROP as a
generic corpus capability with a small fixture, input contract, output contract,
and acceptance tests.
