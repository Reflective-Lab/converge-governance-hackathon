---
name: sync
description: Team sync — see what everyone did, what's open, avoid duplicate work
disable-model-invocation: true
user-invocable: true
allowed-tools: Bash, Read, Grep
---

# Team Sync

What happened since you last looked. Designed to avoid duplicate work on a team.

## Steps

1. **Recent commits by author** — who did what:
   ```bash
   git log --oneline --all --since="24 hours ago" --format="%an: %s" | sort
   ```

2. **Open PRs** — who has work waiting for review:
   ```bash
   gh pr list 2>/dev/null
   ```

3. **Open issues** — what's claimed vs unclaimed:
   ```bash
   gh issue list --limit 15 2>/dev/null
   ```

4. **Build health**
   ```bash
   just check && just test
   ```

5. **KB freshness** — any kb/ files older than 14 days:
   ```bash
   find kb/ -name "*.md" -mtime +14
   ```

## Output

```
── Team Sync ──────────────────────────────────────

Recent work (last 24h):
  <author>: <commit message>
  <author>: <commit message>
  ...

PRs awaiting review:
  #<num> <title> (by <author>)
  ...

Unclaimed issues:
  #<num> <title>
  ...

Build:       <clean | broken>
Tests:       <N passed>

Stale KB pages (>14 days):
  <list or "None">

────────────────────────────────────────────────────
```

## Rules

- Under 2 minutes to run.
- Group commits by author so you can see who's working on what.
- Highlight PRs that have been open >24h — they need review.
- Do not suggest priorities. Show state.
