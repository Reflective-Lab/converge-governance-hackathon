---
name: test
model: opus
description: Add or run tests for Vendor Selection Rust, web, or desktop work.
user-invocable: true
argument-hint: [crate-or-area] [category]
allowed-tools: Read, Edit, Write, Bash, Grep, Glob
---
# Test

Add or run tests for `$ARGUMENTS`.

## Steps

1. Identify the target crate, app, or module.
2. Read nearby tests before adding new ones.
3. Add the smallest test that proves the behavior.
4. Prefer unit tests for pure domain logic, integration tests for API flows, property tests for invariants, and regression tests for fixed bugs.
5. Run the narrow test first.
6. Run `just test` before finishing.
7. Run `just lint` if test code changed.

## Rules

- Do not duplicate existing tests.
- Every bug fix should get a named regression test when practical.
- Keep tests deterministic and offline unless the test is explicitly marked live.
