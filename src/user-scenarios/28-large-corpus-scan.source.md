# Large Corpus Scan — maxim Reference Library

This document demonstrates `proof check` on the maxim reference library:
2,703 markdown files across 13 sections and 217 directories. The goal is
to verify that the baseline is clean — zero errors — before any new
authoring session.

## Running the scan

```bash
# From the maxim repo root:
proof check . --errors-only

# Expected output when clean:
#   Checked 2703 files — 0 errors, 0 warnings

# If errors exist, scope to the failing section:
proof check computing/ --errors-only
proof check mathematics/ --errors-only
```

## Interpreting results

| Output | Meaning |
|--------|---------|
| `0 errors, 0 warnings` | Baseline clean — safe to author |
| `N errors` | Broken boxes or structural issues — run `proof fix` before authoring |
| `ascii_box_width` errors | Box rows differ in visual width — most common in Unicode-heavy files |
| `md_missing_section` | A required H2 section is absent from a guide |
| `md_table_col_count` | A table has fewer columns in a body row than the header declares |

## Fix workflow when errors appear

```bash
# Preview auto-fixes
proof fix . --min-confidence high --dry-run

# Apply high-confidence fixes only
proof fix . --min-confidence high

# Verify clean
proof check . --errors-only
```

## Baseline status

| Section | Files | Errors |
|---------|-------|--------|
| Computing & Software | 270 | 0 |
| Mathematics & Physics | 540 | 0 |
| Mechanics | 378 | 0 |
| Technology | 243 | 0 |
| Life Sciences | 486 | 0 |
| Earth & Space | 378 | 0 |
| History & Ideas | 405 | 0 |
| Social Sciences | 432 | 0 |
| Language & Communication | 324 | 0 |
| Arts & Culture | 459 | 0 |
| Material Culture | 297 | 0 |
| Natural World | 324 | 0 |
| People | 324 | 0 |
| **Total** | **2703** | **0** |
