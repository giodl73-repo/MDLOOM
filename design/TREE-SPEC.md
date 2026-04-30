# proof tree — ASCII Tree Composer and Validator

> **Status**: ✅ Implemented — `src/tree/`. Kinds: dirtree, org, taxonomy, dependency, outline, decision. Schema-driven from md:// table or JSON sources. Inline body support: numbered bullets for `outline` (auto-indented by dot depth); dash bullets for `org`/`taxonomy`/`dependency` (auto-detected indent unit, proper `│` continuation). `proof tree generate` and `proof tree verify` CLI commands live.

---

## What it is

`proof tree` handles ASCII art trees across all kinds: directory structures,
org charts, taxonomies, dependency graphs, and more. It does three things:

1. **Validate** — check connector grammar, indentation consistency, and kind-specific rules
2. **Fix** — auto-correct broken connectors and misaligned continuation lines
3. **Generate** — produce a tree from source data addressed via `md://` URI

---

## The tree grammar (shared across all kinds)

All ASCII trees use the same connector set:

```
Root
├── Child A          ← ├── means "has more siblings after me"
│   ├── Grandchild 1
│   └── Grandchild 2 ← └── means "I am the last child"
├── Child B
└── Child C          ← └── at this level ends the parent's children
    └── Grandchild 3
```

### Connector characters

| Character | Meaning |
|-----------|---------|
| `├──` | Non-terminal child (siblings follow at same level) |
| `└──` | Terminal child (last sibling at this level) |
| `│` | Continuation line (parent has more children below) |
| `─` | Horizontal connector fill |

Unicode equivalents (`├`, `└`, `│`, `─`) are the canonical form.
ASCII fallbacks (`+`, `\`, `|`, `-`) are also accepted.

### Indentation

Each level adds a fixed number of spaces. The default is **4 spaces** per level
(the width of `│   ` or `    ` continuation prefix). Configurable via `indent_width`.

**Choosing indent_width:** The default `indent_width = 4` works well in code contexts. For wide documentation prose or MkDocs sidebars, `indent_width = 2` is recommended to prevent deep trees from exceeding 80 columns.

---

## Structural invariants (all kinds)

| Invariant | Rule |
|-----------|------|
| T-1 | `└──` must be the last child at its indentation level — no `├──` at the same level after a `└──`. **Note:** validating T-1 requires 1-token lookahead (the validator must buffer one line ahead to know whether a `└──` is truly the last sibling). This is not streaming-compatible without that one-line buffer. |
| T-2 | `│` continuation lines must align with the `├` or `│` of their parent |
| T-3 | Indentation must be consistent — each level adds exactly `indent_width` spaces |
| T-4 | Every non-leaf non-root node must have at least one child |
| T-5 | The root has no connector prefix (it is left-aligned or at indent=0) |
| T-6 | `├──` and `└──` must be followed by a space and then the node label |
| T-12 | A root with zero children is a valid leaf-root tree (emitted as a single line with no connectors) |

---

## dirtree — full spec

`dirtree` is the most common tree kind. It represents filesystem structure.

### Format

```
project/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   └── checks/
│       ├── ascii_box.rs
│       └── mod.rs
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

### dirtree-specific conventions

- **Directories** end with `/`
- **Files** do not end with `/`
- **Entries are sorted**: directories before files, then alphabetically (configurable)
- **Annotations**: a file entry may be followed by ` — description` on the same line

```
src/
├── main.rs        — binary entry point
└── lib.rs         — library root
```

### Validation (dirtree)

| Check | Code | Description |
|-------|------|-------------|
| Connector grammar | `tree_connector` | T-1 through T-6 |
| Trailing slash | `tree_dir_slash` | Directories must end with `/`; files must not |
| Annotation format | `tree_annotation` | Annotation must use ` — ` (em-dash with spaces) |
| Path existence | `tree_path_missing` | (opt-in) resolved path does not exist on disk |
| Duplicate entry | `tree_duplicate` | Same name appears twice under same parent |

**Note on false positives:** The tree validator only runs inside code blocks tagged with a `dirtree` info string (e.g. ` ```dirtree `) or identified as a tree by a `proof:tree` directive. It does not scan arbitrary code blocks, so `│` characters inside `ascii_box` diagrams or other fenced blocks do not trigger `tree_orphan`.

### Filesystem validation

With `--verify-paths` (or `verify_paths = true` in proof.toml), proof resolves
each entry against the filesystem root:

```bash
proof tree check --verify-paths --root /project figures/structure.md
```

For each directory or file in the tree, proof checks whether `{root}/{path}` exists.
Missing entries emit `tree_path_missing` errors.

### Auto-fixable

| Fix | Trigger |
|-----|---------|
| `├──` after `└──` at same level | Swap to `├──`/`└──` in correct order |
| Wrong connector (`├` vs `└`) | Recompute from structure — last child gets `└──` |
| Missing trailing `/` on directory | Add `/` if the path exists on disk and is a directory |
| Misaligned `│` continuation | Shift to correct column |
| Inconsistent indent width | Normalize to `indent_width` |

### Generation from filesystem

```bash
proof tree generate --kind dirtree --root /project/src --max-depth 3
```

Walks the filesystem from `--root`, applies `--max-depth`, `--exclude` globs,
and emits a formatted dirtree. Options:

| Option | Default | Description |
|--------|---------|-------------|
| `--root <dir>` | cwd | Filesystem root to walk |
| `--max-depth <n>` | unlimited | Max depth to recurse |
| `--exclude <globs>` | — | Patterns to skip (e.g. `target/**`, `*.log`) |
| `--dirs-first` | true | Directories before files at each level |
| `--sort` | name | Sort by: `name`, `ext`, `size`, `mtime` |
| `--annotate` | false | Add ` — description` from companion YAML |
| `--wrap-fence` | true | Wrap output in ` ``` ` fence |

---

## Other tree kinds — source schema and generation

For non-dirtree kinds, the source data lives at an `md://` address. Proof reads the
source, validates it against the kind's schema, and generates the tree.

### Field mapping — flexible, not rigid

Proof does **not** require specific column names. Field binding follows
**[MAPPING-SPEC.md](./mapping-spec.md)** — the shared system for all proof
directives. Summary below; the full spec covers query parameters, row selectors,
type coercion, and numeric formatting.

Proof uses a field mapping strategy:

1. **Explicit mapping** — you declare which field/column maps to which role via
   directive attributes or `proof tree generate` flags.
2. **Auto-detection** — if no mapping is provided, proof searches for common names
   (case-insensitive) in the source table's headers or JSON keys.

This means your existing data works as-is, with column names like `Employee`/`Manager`
or `taxon`/`parent_taxon` — no renaming required.

### Source formats

| Format | How to address |
|--------|----------------|
| GFM markdown table | `md://path.md#section:table:0` |
| JSON array of objects | `md://path.json` (future: `md://path.md#:json:0` for embedded JSON) |
| YAML sequence | `md://path.yaml` (future) |

For Wave 3, **markdown tables** and **inline JSON** are the primary formats.

### Directive syntax (compile mode)

```
```proof:tree kind=org name="Employee" parent="Manager" label="Title"
md://org/team.md#:table:0
```
```

```
```proof:tree kind=org format=json name="full_name" parent="reports_to" label="title"
md://org/team.json
```
```

### CLI syntax

```bash
proof tree generate --kind org \
    --name "Employee" --parent "Manager" --label "Title" \
    md://org/team.md#:table:0

proof tree generate --kind org --format json \
    --name "full_name" --parent "reports_to" \
    md://org/team.json
```

### Auto-detection defaults per kind

If no field mapping is provided, proof tries these names (case-insensitive, first match wins):

| Kind | name field | parent field | extra fields |
|------|-----------|--------------|-------------|
| `org` | `name`, `employee`, `person`, `member` | `parent`, `manager`, `reports_to`, `superior` | `label`, `title`, `role` |
| `taxonomy` | `label`, `name`, `taxon`, `term` | `parent`, `parent_taxon`, `belongs_to` | `level`, `rank` |
| `dependency` | `package`, `name`, `module`, `crate` | `depends_on`, `requires`, `dependency`, `uses` | `version` |
| `decision` | `node`, `condition`, `question`, `id` | — (special: `yes`, `no` are branch targets) | `label` |
| `outline` | uses heading text directly | uses heading level | — |

### Root marker

For tabular kinds (org, taxonomy, dependency), a node with no parent is the root.
Accepted root markers (auto-detected): `—`, `-`, `none`, `null`, `` (empty), `0`.
Override with `--root-marker "ROOT"`.

---

### `org` — organizational hierarchy

**Source — any table with name + parent columns:**

```markdown
| Employee | Manager | Title |
|----------|---------|-------|
| Gio | — | CTO |
| Alice | Gio | VP Engineering |
| Bob | Alice | Staff Engineer |
| Carol | Alice | Staff Engineer |
| Dave | Gio | VP Product |
```

Auto-detected: `Employee` → name, `Manager` → parent, `Title` → label.

Or explicit: `name="Employee" parent="Manager" label="Title"`

**JSON source:**

```json
[
  {"id": "gio", "full_name": "Gio", "reports_to": null, "title": "CTO"},
  {"id": "alice", "full_name": "Alice", "reports_to": "gio", "title": "VP Eng"}
]
```

Directive: `kind=org format=json name="full_name" parent="reports_to" label="title"`

**Generated tree:**
```
CTO: Gio
├── VP Engineering: Alice
│   ├── Staff Engineer: Bob
│   └── Staff Engineer: Carol
└── VP Product: Dave
```

**Validation rules:**
- Exactly one root (Parent = —); if no row has Parent = — emit `tree_no_root`
- No cycles
- No orphaned nodes (parent name must exist)

---

### `taxonomy` — hierarchical classification

**Source schema** (YAML front-matter + structured list):

```markdown
---
tree_kind: taxonomy
root: Animals
levels: [Kingdom, Phylum, Class, Order, Family, Genus, Species]
---

| Label | Parent | Level |
|-------|--------|-------|
| Animals | — | Kingdom |
| Chordata | Animals | Phylum |
| Mammalia | Chordata | Class |
| Carnivora | Mammalia | Order |
| Felidae | Carnivora | Family |
| Panthera | Felidae | Genus |
| Panthera leo | Panthera | Species |
```

**Generated tree:**
```
Kingdom: Animals
└── Phylum: Chordata
    └── Class: Mammalia
        └── Order: Carnivora
            └── Family: Felidae
                └── Genus: Panthera
                    └── Species: Panthera leo
```

**Validation rules:**
- Exactly one root (Parent = —); if no row has Parent = — emit `tree_no_root`
- Levels must follow declared order (can't skip)
- Each node's level must be exactly one below its parent's level

---

### `dependency` — package dependency graph

**Source schema** (markdown table):

```markdown
| Package | Depends On | Version |
|---------|-----------|---------|
| myapp | proof | 0.5.0 |
| myapp | mdpath | 0.5.0 |
| proof | mdpath | 0.5.0 |
| proof | clap | 4.x |
| proof | serde | 1.x |
| mdpath | thiserror | 2.x |
```

**Generated tree** (rooted at `myapp`):
```
myapp
├── proof 0.5.0
│   ├── mdpath 0.5.0
│   │   └── thiserror 2.x
│   ├── clap 4.x
│   └── serde 1.x
└── mdpath 0.5.0 (deduped ↑)
```

**Validation rules:**
- Cycles detected and reported as `tree_cycle`
- Deduplication: the tree is rendered depth-first. The first occurrence of a package (in DFS order) is rendered fully; subsequent occurrences show `(deduped ↑ N)` where N is the line number of the first occurrence (e.g. `└── mdpath 0.5.0 (deduped ↑ 3)`)
- Optional: cross-reference against `Cargo.toml` or `package.json` for version accuracy

---

### `outline` — document section hierarchy

**Source schema** (heading structure of a markdown file):

```bash
proof tree generate --kind outline md://docs/spec.md
```

Proof parses the heading structure of the target file and produces an outline tree.
No explicit source table needed — the headings ARE the source.

**Generated tree** (from a file with H1/H2/H3):
```
Spec Document
├── Overview
├── Grammar
│   ├── Connectors
│   └── Indentation
└── Validation
    ├── Structural
    └── Kind-specific
```

**Validation rules:**
- Heading levels must not skip (H1 → H3 without H2 is flagged)
- Consistent with standard `md_heading_hierarchy` check

---

### `decision` — decision tree

**Source schema** (markdown table with condition + branches):

```markdown
| Node | Condition | Yes → | No → |
|------|-----------|-------|------|
| root | Is the file .md? | parse | skip |
| parse | Has proof: directive? | compile | check-only |
| compile | DaVinci pin exists? | validate | embed |
```

**Generated tree:**
```
Is the file .md?
├── Yes → Has proof: directive?
│         ├── Yes → DaVinci pin exists?
│         │         ├── Yes → validate
│         │         └── No  → embed
│         └── No  → check-only
└── No  → skip
```

**Validation rules:**
- Every non-leaf node has exactly 2 branches (Yes/No)
- Every branch leads to either a child node or a leaf label
- No unreachable nodes
- A leaf is a node whose Node value does not appear in any `Yes →` or `No →` column. If a `Yes →` or `No →` references a node name that does not appear in the Node column, emit `tree_orphan` (schema form: referenced target does not exist as a declared node)

---

## CLI commands

```bash
# Validate a tree code block in a markdown file
proof tree check [--kind dirtree|org|taxonomy|...] [--verify-paths] [--root <dir>] <uri>

# Auto-fix connector and indentation errors
proof tree fix <uri>

# Generate a tree from filesystem
proof tree generate --kind dirtree --root <dir> [options]

# Generate a tree from source data at md:// URI
proof tree generate --kind org|taxonomy|dependency|outline|decision <source-uri>

# Output to file
proof tree generate --kind dirtree --root /project -o docs/structure.md
```

---

## The `proof:tree` directive (compile mode)

In source documents, use `proof:tree` to embed a live-generated tree:

````markdown
```proof:tree kind=dirtree root=src/ max-depth=3 exclude=target/**
```
````

````markdown
```proof:tree kind=org
md://docs/team.md#engineering-org:table:0
```
````

The compiler resolves the source, generates the tree, and embeds it with traceability:

```markdown
<!-- proof:compiled from="proof:tree kind=dirtree root=src/" -->
```
src/
├── main.rs
└── lib.rs
```
<!-- /proof:compiled -->
```

### Directive attributes

| Attribute | Kinds | Description |
|-----------|-------|-------------|
| `kind` | all | `dirtree`, `org`, `taxonomy`, `dependency`, `outline`, `decision` |
| `root` | dirtree | Filesystem root for generation |
| `max-depth` | dirtree | Max recursion depth |
| `exclude` | dirtree | Comma-separated glob patterns to exclude |
| `verify-paths` | dirtree | Validate each path exists on disk |
| `indent-width` | all | Spaces per level (default: 4) |
| `sort` | dirtree, org | Sort order: `name`, `alpha`, `mtime` |
| `dirs-first` | dirtree | Directories before files (default: true) |

**Attribute naming convention:** Directive attributes use hyphenated names (e.g. `max-depth`). The corresponding `proof.toml` keys use underscores (e.g. `max_depth`). CLI flags use `--max-depth`. This follows the same convention as all other proof directives.

**Canonical triple for boolean options** (e.g. path verification): directive attribute = `verify-paths`, CLI flag = `--verify-paths`, proof.toml key = `verify_paths`.

---

## Cache key for tree generation

When `proof compile` processes a `proof:tree` directive:

- **dirtree from filesystem**: cache key includes directory content hash (mtime-based)
- **generated from md:// source**: cache key = Tier 2 resolve_key of the source URI

A change to the source file or the target directory invalidates the tree and triggers
regeneration on the next compile.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| T-1 | `└──` is always the last child — no `├──` follows at the same indent level. Requires 1-line lookahead during parsing (not streaming-compatible without a one-line buffer). |
| T-2 | `│` continuation lines align exactly to their parent's indent |
| T-3 | Indentation per level is consistent (detected from the first two levels) |
| T-4 | Every non-leaf non-root node has at least one child |
| T-5 | Root has no connector prefix |
| T-6 | `├──` and `└──` are followed by exactly one space then the label |
| T-7 | (dirtree) Directories end with `/`; files do not |
| T-8 | (dirtree) No duplicate entry names under the same parent |
| T-9 | (org/taxonomy) Exactly one root node (Parent = —); zero roots emits `tree_no_root`, multiple roots emits `tree_connector` |
| T-10 | (org/taxonomy) No cycles |
| T-11 | (decision) Every non-leaf has exactly 2 children |
| T-12 | A root with zero children is a valid leaf-root tree (emitted as a single line with no connectors) |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `tree_connector` | error | Wrong connector for position (├ vs └) |
| `tree_indent` | error | Inconsistent indentation width |
| `tree_orphan` | error | Continuation `│` with no parent; or (decision) a `Yes →` / `No →` references a node name that is not declared in the Node column |
| `tree_dir_slash` | warning | Directory missing trailing `/` or file has one |
| `tree_duplicate` | error | Duplicate entry under same parent |
| `tree_path_missing` | error | (dirtree, verify-paths) Path does not exist on disk |
| `tree_cycle` | error | (dependency, org) Cycle detected |
| `tree_no_root` | error | (org, taxonomy) Source table has no row with Parent = — (or equivalent null/dash marker) — tree has no declared root |
| `tree_level_skip` | error | (taxonomy) Level skipped in hierarchy |
| `tree_annotation` | warning | Annotation not using ` — ` format |
| `tree_child_count` | error | (decision) Non-leaf node does not have exactly 2 children |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/checks/ascii_tree.rs` | Structural validation (T-1 through T-6) |
| `src/tree/dirtree.rs` | Filesystem walk, generation, path validation |
| `src/tree/generate.rs` | Tree generation from source schemas |
| `src/tree/schema.rs` | Source schema parsing (org, taxonomy, dependency, decision) |
| `src/commands/tree.rs` | CLI surface |

---

## See also

- [Compile Spec](./compile-spec.md) — `proof:tree` directive in compile mode
- [Layout Spec](./layout-spec.md) — composing multiple trees side by side
- [MDPATH](../../mdpath/design/SPEC.md) — `md://` URI scheme for source data

---

## Spec Clarifications (from scenario findings)

- **F83** (exclude pattern): `exclude=target,*.lock` — comma-separated patterns. Patterns are matched against the entry's BASE NAME (not full path). `target` skips any directory or file named exactly `target` at any depth.
- **F84** (depth origin): `max_depth=2` counts from the root: root is depth 0, root's children are depth 1, grandchildren are depth 2. At `max_depth=2`, grandchildren ARE included; great-grandchildren are NOT.
- **F85** (mixed indent): `indent_width` is fixed per-tree (default 4). Mixed indentation in the source is rounded to the nearest multiple of `indent_width`. If ambiguous, the first child's indent determines the unit.
- **F86** (root detection): In inline body, `root:` prefix declares the root node. If absent, the first non-indented, non-empty line is treated as the root. If all lines are indented, emit COMPILE-002.
- **F89** (org vs dependency): `org` and `dependency` produce identical ASCII output. The distinction is semantic intent: `org` for hierarchy (A manages B), `dependency` for dependency (A requires B). Future: may add `→` arrows for dependency kind.
- **F90** (cycle detection): Circular references (A parent=B, B parent=A) are detected during `build_dfs_tree`. Detected cycles skip the cyclic child and emit a COMPILE-002 warning with the cycle path. No infinite loop.
- **F91** (outline vs proof:ol): `proof:tree kind=outline` renders author-supplied numbers verbatim (pass-through). `proof:ol` auto-generates sequential decimal numbers. Use `outline` when the author controls numbering; use `proof:ol` when proof should generate it.

