# ADR-042 — Adopt proof as Documentation Compiler

**Status:** Accepted
**Date:** 2026-04-28
**Deciders:** Platform team

## Table of Contents

```proof:toc max-depth=2 style=numbered
```

## Context

Our markdown documentation corpus has grown to 2,700+ files across 13 sections.
Manual ASCII art alignment is error-prone. Cross-references break silently when
files are reorganized. We need a documentation quality system.

## Decision

Adopt `proof` as the official documentation compiler and linter for all
markdown-heavy projects in the org.

## Rationale

proof provides:
- Structural validation (ASCII art, tables, links, headings)
- Compilation from `.source.md` → `.md` with resolved directives
- LaTeX math rendering without external tooling
- Stable `md://` URI addressing for cross-references
- Watch mode for continuous compilation during authoring

## Consequences

### Positive

proof catches errors before they reach readers. Source documents are
first-class artifacts, not just raw text. Math renders in any terminal.

### Negative

Authors must learn the `.source.md` / `proof:` directive syntax. Initial
setup per repository requires creating `proof.toml` and `scripts/build-guides.sh`.

### Neutral

Existing `.md` files continue to work as-is. Migration to `.source.md` is
opt-in per document.

## Implementation Plan

```proof:ol
- Add proof.toml to each repository root
- Set up [[compile]] targets for guides and presentations
- Run proof check . to find existing errors
- Fix errors using proof fix --min-confidence high
- Train team on source document authoring
- Add proof check to CI pipeline
```

## Status History

| Date | Status | Author |
|------|--------|--------|
| 2026-04-28 | Proposed | Platform team |
| 2026-04-28 | Accepted | Architecture board |
