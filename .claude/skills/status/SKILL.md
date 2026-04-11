---
name: status
description: Check build health, test results, and project state
disable-model-invocation: true
user-invocable: true
allowed-tools: Bash, Read
---

# Project Status

Quick health check of the entire project.

## Steps

1. **Rust workspace**
   ```bash
   just check
   just test
   just lint
   ```

2. **Desktop frontend** (if scaffolded)
   ```bash
   cd apps/desktop && bun run build 2>&1
   ```

3. **Desktop Tauri** (if scaffolded)
   ```bash
   cd apps/desktop && bun run tauri build 2>&1
   ```

4. **Output:**

```
── Status ─────────────────────────────────────────

Rust check:     <pass | fail>
Rust tests:     <N passed, M failed>
Clippy:         <clean | N warnings>
Desktop build:  <pass | fail | not scaffolded>
Tauri build:    <pass | fail | not scaffolded>

────────────────────────────────────────────────────
```
