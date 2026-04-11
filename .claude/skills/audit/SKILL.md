---
name: audit
description: Security and dependency audit
disable-model-invocation: true
user-invocable: true
allowed-tools: Bash, Read, Grep, Glob
---

# Security Audit

## Steps

1. **Dependency vulnerabilities**
   ```bash
   cargo audit 2>&1
   ```

2. **Secrets scan** — grep for hardcoded keys, tokens, passwords in source files (excluding target/, node_modules/, .git/).

3. **Unsafe code** — scan for `unsafe` blocks in Rust code.

4. **.env check** — verify `.env` is in `.gitignore` and not committed.

5. **Output:**

```
── Audit ──────────────────────────────────────────

Dependencies:   <N vulnerabilities | clean>
Secrets:        <N findings | clean>
Unsafe code:    <N occurrences | clean>
.env:           <gitignored | EXPOSED>

────────────────────────────────────────────────────
```

## Rules

- Zero tolerance for committed secrets.
- Zero tolerance for unsafe code.
- Flag all findings — do not auto-fix.
