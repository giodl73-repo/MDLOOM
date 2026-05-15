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
