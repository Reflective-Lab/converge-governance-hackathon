---
name: checkpoint
description: End-of-session — capture what you moved, update kb, leave a trail for the team
disable-model-invocation: true
user-invocable: true
allowed-tools: Read, Edit, Write, Bash, Glob
---

# Session Checkpoint

End the session with accountability. Leave a trail so your teammates know what changed.

## Steps

1. **Review session work**
   ```bash
   git diff --stat HEAD 2>/dev/null
   git log --oneline -10 2>/dev/null
   ```

2. **Check if knowledgebase needs updating** — if architecture decisions, new patterns, or domain knowledge emerged this session, update the relevant `kb/` files.

3. **Write a team-visible update** — if there's an active issue or PR for the work, add a comment summarizing progress:
   ```bash
   gh issue comment <number> --body "Progress update: ..."
   # or
   gh pr comment <number> --body "Progress update: ..."
   ```
   If there's no issue or PR, just report to the user.

4. **Output the checkpoint:**

```
── Checkpoint ─────────────────────────────────────

Who:    <git user.name>
Moved:
- <what was accomplished>

KB updated:
- <list of kb/ files updated, or "None">

Left for the team:
- <open threads, unfinished work, things the next person should know>

────────────────────────────────────────────────────
```

## Rules

- Be honest. If nothing meaningful moved, say so.
- Keep it to 10 lines max. Log entry, not a report.
- Update kb/ files when new knowledge was generated, not just code.
- The "left for the team" section is the most important part — what does the next person need to know?
