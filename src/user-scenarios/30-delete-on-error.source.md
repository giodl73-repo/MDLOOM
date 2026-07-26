# CI Workflow — mdloom compile --delete-on-error

This scenario demonstrates `mdloom compile --delete-on-error` in a GitHub
Actions pipeline. Stale compiled output is automatically removed when the
source cannot be compiled, preventing docs from serving outdated content.

## The problem without --delete-on-error

Without this flag, a compile failure leaves the previously compiled file
on disk. The next deploy picks up the old output instead of failing loudly.
The docs site then serves content that does not match the current source.

## Workflow

```yaml
# .github/workflows/docs.yml
name: Docs

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  compile-and-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mdloom
        run: cargo install mdloom --locked

      - name: Lint source files
        run: mdloom check src/ --fail-on-error

      - name: Compile docs
        run: mdloom compile --delete-on-error

      - name: Verify no stale output
        run: mdloom check docs/ --errors-only --fail-on-error

      - name: Deploy
        if: github.ref == 'refs/heads/main'
        run: ./scripts/deploy-docs.sh
```

## What --delete-on-error does

| Compile result | Without flag | With --delete-on-error |
|---------------|-------------|------------------------|
| Success | Output written | Output written |
| Failure (parse error) | Old output left on disk | Old output deleted |
| Failure (directive error) | Old output left on disk | Old output deleted |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All targets compiled successfully |
| 1 | One or more targets failed; stale output deleted |
| 2 | Configuration error (mdloom.toml unreadable, etc.) |

## mdloom.toml configuration

```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

Running `mdloom compile --delete-on-error` processes both targets. If either
fails, the corresponding output directory is cleaned before the process exits
with code 1, ensuring the deploy step never runs with partial output.
