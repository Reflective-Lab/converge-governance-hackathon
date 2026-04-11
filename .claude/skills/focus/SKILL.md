---
name: focus
description: Session opener — orient yourself in the project, see what the team is doing
user-invocable: true
allowed-tools: Read, Grep, Bash, Glob
---

# Session Focus

Start every session here. Whether it's your first time or your tenth.

## Steps

1. **Orient** — read `kb/Home.md` (index only — do NOT follow all the links). If this is the user's first session, list the key pages they should read on their own:
   - `kb/Architecture/Overview.md` — what this project is
   - `kb/Domain/Vendor Selection.md` — the challenge
   - `kb/Development/Getting Started.md` — how to build and run
   - `kb/Workflow/Working with Claude.md` — skills vs Justfile
   Do NOT read these pages yourself during /focus — just tell the user to read them.

2. **Team activity** — what happened while you were away:
   ```bash
   git log --oneline --all --since="24 hours ago" --format="%h %an: %s"
   ```

3. **Open work** — what's in flight:
   ```bash
   gh pr list 2>/dev/null
   gh issue list --limit 10 2>/dev/null
   ```

4. **Build health**
   ```bash
   just check
   ```

5. **Output:**

```
── Session Focus ──────────────────────────────────

Project:     Converge Governance Hackathon
Build:       <clean | broken>

Recent team activity (last 24h):
  <commit list with authors, or "No activity">

In flight:
  PRs: <list or "None">
  Issues: <list or "None">

Start here:
  <if first session: list the 4 kb pages above>
  <if returning: "You're up to speed. Pick an issue or /ticket something new.">

────────────────────────────────────────────────────
```

## Rules

- If the user has no commits in the repo, treat it as a first session.
- Keep it short — 10 seconds to read.
- Do not suggest priorities. Show state. The user decides.
- If the build is broken, flag it prominently — fixing it is everyone's first priority.
