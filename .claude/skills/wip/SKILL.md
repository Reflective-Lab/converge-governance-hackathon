---
name: wip
description: Save work-in-progress and push — use before switching devices or ending a session
disable-model-invocation: true
allowed-tools: Bash
---

# Save WIP and Push

Quick save before switching devices or handing off to a teammate.

## Steps

1. **Show current state**
```bash
git status
```

2. **Save current work**
```bash
git add -A
git stash || true
git checkout -b wip/$(date +%Y%m%d-%H%M%S) 2>/dev/null || true
git stash pop 2>/dev/null || true
git add -A
git commit -m "WIP: work in progress — $(date +%Y-%m-%d)"
git push -u origin HEAD
```

3. **Push all local branches with unpushed changes**
```bash
git push --all
```

4. **Summary** — tell the user:
   - What was saved and pushed
   - Any branches in flight
   - How to resume: `git checkout <branch> && git pull`
