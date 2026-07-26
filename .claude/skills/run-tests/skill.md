---
name: run-tests
description: Run the mdloom test suite and report results. Flags any failures, identifies new coverage gaps, and verifies E2E behavior. Uses BENCH role.
user_invocable: true
---

# Run Tests

Runs the full test suite, checks results, and identifies gaps.

## Steps

### 1. Run unit tests

```bash
cd C:/src/mdloom && cargo test 2>&1
```

Expected: all unit tests in `src/checks/ascii_box.rs` pass (4 tests).

### 2. Run integration tests

```bash
cd C:/src/mdloom && cargo test --test integration_tests 2>&1
```

Expected: 18 tests pass, 0 fail.

Report any failures with:
- Test name
- Assertion that failed
- Likely cause (fixture wrong, detection logic wrong, API changed)

### 3. Build the binary

```bash
cd C:/src/mdloom && cargo build 2>&1
```

Check for warnings — none should remain. Any warning is a candidate for fixing.

### 4. E2E smoke test

Run mdloom against its own fixtures:

```bash
C:/src/mdloom/target/debug/mdloom tests/fixtures/perfect_box.md
# Expected: exit 0, zero diagnostics

C:/src/mdloom/target/debug/mdloom tests/fixtures/width_mismatch.md
# Expected: exit 1, at least one ascii_box_width error

C:/src/mdloom/target/debug/mdloom --format json --no-fail tests/fixtures/width_mismatch.md
# Expected: exit 0, valid JSON array
```

### 5. Run against maxim library (if available)

```bash
C:/src/mdloom/target/debug/mdloom --config C:/src/maxim/mdloom.toml C:/src/maxim/ 2>&1 | tail -20
```

Record:
- Wall-clock time
- Total files checked
- Error count / warning count
- Any unexpected diagnostic codes

### 6. Coverage assessment

After running tests, identify gaps:
- Is CRLF handling tested?
- Is a nested-box fixture present?
- Is the config cascade tested end-to-end (not just unit-tested)?
- Is there a test for `mdloom init` creating a mdloom.toml?

## Output

- Test results: N passed / M failed
- Build warnings (list)
- E2E results: exit codes, sample output
- Maxim library results: time, count, errors
- Coverage gaps: behaviors without tests
- Summary: GREEN / FAILING / GAPS
