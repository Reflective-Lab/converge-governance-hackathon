---
name: quality
description: Capture code quality metrics and track trends
disable-model-invocation: true
user-invocable: true
argument-hint: [baseline|check|trend]
allowed-tools: Bash, Read, Write, Grep, Glob
---

# Code Quality Index

Capture quality metrics for the governance hackathon codebase.

## Metrics collected

### Rust
1. **Clippy warnings** — `cargo clippy --workspace 2>&1 | grep "warning:" | wc -l`
2. **Clippy complexity** — `cargo clippy --workspace -- -W clippy::cognitive_complexity 2>&1 | grep "cognitive_complexity" | wc -l`
3. **Test count** — `cargo test --workspace 2>&1 | grep "test result"` (extract pass/fail/ignore)
4. **Unsafe blocks** — `grep -r "unsafe" --include="*.rs" -l` (excluding target/)
5. **Dependency vulnerabilities** — `cargo audit 2>&1 | grep -c "Vulnerability found"`

### Svelte/TypeScript (desktop)
6. **Svelte errors/warnings** — `bunx svelte-check 2>&1 | grep "COMPLETED"` (from apps/desktop/)
7. **TODO/FIXME count** — grep across all source files

### Codebase size
8. **Lines of code** — `tokei` if available, otherwise wc -l by language
9. **File count** — by type (*.rs, *.svelte, *.ts)

## Commands

### `check` (default)
Run all metrics, output summary table, append to `quality-log.csv`.

### `baseline`
First row to `quality-log.csv`. Run once to establish starting point.

### `trend`
Show trend over last 5 entries. Flag any metric worsened for 3+ consecutive entries.

## Rules

- Always run from repo root.
- Use `bunx` not `npx` for Svelte checks.
- Compare each metric to previous entry and show +/- delta.
- Do not fail on bad metrics — report honestly.
