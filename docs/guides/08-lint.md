# proof Lint — Markdown and ASCII Art Validation

`proof check` is the linter — it scans markdown files and reports structural
errors before they accumulate into an unmaintainable corpus. The core value is
catching problems that are invisible to human readers but break the document's
structural guarantees: a box that's one character too wide, a required section
that's missing, a link that rotted.

The fix pipeline converts lint errors into structured fix plans that can be
applied automatically or reviewed manually. The workflow is: check → draft →
fix → verify. Most common errors (box widths, separator positions) are
auto-fixable with high confidence.

---

## Running the linter

```bash
# Lint current directory
proof check .

# Lint a specific directory
proof check docs/

# Only show errors (suppress warnings)
proof check . --errors-only

# Exit with error code if any findings (useful for CI)
proof check . --fail-on-error
```

---

## What proof checks

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
proof checks
├── ASCII art
├── ascii_box: Box border width and column separator alignment
├── ascii_flow: Flow diagram node labels and edge connector continuity
├── ascii_tree: Tree indentation consistency and root node count
├── ascii_barchart: Bar scale consistency
├── Markdown structure
├── markdown_h1: Required H1 heading
├── markdown_h2: Required H2 sections (configured per-path in proof.toml)
├── markdown_link: Broken internal links
├── Tables
├── markdown_table: Column count consistency across rows
├── markdown_table_header: Required header rows
├── Source documents
└── source_links: Broken md:// references in .source.md files
```
<!-- /proof:compiled -->

---

## proof.toml configuration

All check rules are configured in `proof.toml`. Rules cascade: a root
`proof.toml` sets defaults, per-directory files inherit and add. The most
powerful configuration is section schemas — they let you require specific H2
headings in matching files, which enforces consistent structure across a
document corpus.

```toml
[files]
root = true

[ascii_box]
enabled = true
tolerance = 0          # extra columns allowed at row end
error_on_wide = false  # treat wide Unicode as 2 columns
check_col_separators = true

[ascii_flow]
enabled = true

[markdown]
enabled = true
max_h1 = 1

# Require specific H2 sections in all guides
[[section_schemas]]
paths = ["docs/guides/*.md"]
required_h2_all = ["Usage", "Examples"]
paths_exclude = ["00-OVERVIEW.md"]
```

---

## All diagnostic codes

The full set of codes proof can emit during `proof check`:

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="md://src/data/diagnostic-codes.md" -->
```taxonomy
ascii_box
├── ascii_box_width
├── ascii_box_col
└── ascii_box_open
ascii_flow
├── ascii_flow_node
└── ascii_flow_edge
ascii_tree
├── ascii_tree_indent
└── ascii_tree_root
ascii_barchart
└── ascii_barchart_scale
markdown
├── markdown_h1
├── markdown_h2
└── markdown_link
math
├── MATH-001
├── MATH-002
├── MATH-003
├── MATH-004
├── MATH-005
└── MATH-006
symbol
└── SYMBOL-001
compile
├── COMPILE-001
├── COMPILE-002
└── COMPILE-003
dashboard
├── DASHBOARD-001
├── DASHBOARD-002
└── DASHBOARD-003
```
<!-- /proof:compiled -->

---

## ASCII box rules

ASCII boxes are the most common source of lint errors in technical documentation.
The rules ensure that boxes are geometrically correct — that a box reader can
parse as a grid actually IS a grid.

A valid box has equal-width top and bottom borders, and column separators that
align vertically across every row:

```
┌────────────────────┐   ← top border: 22 chars
│ content line       │
│ another line       │
└────────────────────┘   ← bottom border must also be 22 chars
```

The most common violation is an off-by-one in the top or bottom border — proof
reports the expected and actual widths with the exact line number.

---

## Auto-fix with proof fix

`proof fix` applies deterministic corrections to lint errors. It generates a
structured fix plan (JSON) that lists every proposed edit with the old and new
string, a confidence level, and a reasoning note.

```bash
# Preview what would change
proof fix . --dry-run

# Apply only high-confidence fixes (safe for CI)
proof fix . --min-confidence high

# Apply everything including medium-confidence
proof fix . --min-confidence medium
```

Confidence levels reflect how certain proof is about each fix:
- **high**: Deterministic — the correct fix follows directly from the rule. Box
  widths, trailing characters, exact column positions.
- **medium**: Heuristic — usually correct but occasionally wrong. Column separator
  positions in complex tables, indentation adjustments.
- **low**: Requires human judgment — proof has a suggestion but isn't confident.

---

## Markdown section schemas

Section schemas are the most powerful linting feature for documentation
corpora. They enforce consistent structure by requiring specific H2 headings
in all files matching a glob pattern.

```toml
[[section_schemas]]
paths = ["docs/**/*.md"]
required_h2_all = ["Summary", "Examples", "See also"]
```

Every file matching `docs/**/*.md` must have exactly those H2 headings.
Missing sections produce `md_missing_h2` warnings with the exact heading name
so an author or AI knows what to add.

Use `paths_exclude` to carve out exceptions:

```toml
[[section_schemas]]
paths = ["guides/*.md"]
paths_exclude = ["00-OVERVIEW.md", "CHANGELOG.md"]
required_h2_all = ["Usage", "Examples"]
```

---

## CI integration

proof exits with code 0 on success and 1 when `--fail-on-error` is set and
any errors are found. Warnings don't trigger a non-zero exit by default.

```yaml
# GitHub Actions
- name: proof lint
  run: C:/src/target/release/proof check . --fail-on-error

- name: proof compile check
  run: C:/src/target/release/proof compile --check src/guides/
```

The compile check validates that all directives can be resolved without actually
writing output — useful for catching broken `md://` references and malformed
directives in CI without requiring the full compile step.
