# PROOF + CROP Corpus Intelligence

PROOF owns compilation, rendering, artifact manifests, and proof-specific
semantics. CROP owns reusable corpus intelligence: status pages, link graphs,
backlinks, frontmatter inventories, heading inventories, and named corpus views.

The `proof crop` command is a thin adapter over the CROP CLI. It keeps CROP
optional while making the recommended integration workflow discoverable from
PROOF.

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

Preflight every recipe in a view store:

```text
proof crop inspect-views --dir .crop/views --strict
```

---

## Choosing PROOF vs. CROP

Use PROOF when the task is about compiling `.source.md`, rendering charts,
slides, dashboards, math, symbols, HTML, or artifact manifests.

Use CROP when the task is about corpus inventory, links, backlinks,
frontmatter, headings, named corpus slices, or generated status pages.

If PROOF needs additional CROP behavior, file the request against CROP as a
generic corpus capability with a small fixture, input contract, output contract,
and acceptance tests.
