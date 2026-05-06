---
name: done
model: sonnet
description: End a Vendor Selection session with a short handoff.
user-invocable: true
allowed-tools: Bash, Read, Edit
---
# Done

End-session handoff.

## Steps

1. Read `MILESTONES.md`.
2. Run `git status --short`.
3. Summarize changed files by area.
4. List verification commands run.
5. Update `CHANGELOG.md` under `[Unreleased]` for notable shipped changes.
6. List remaining work from the current milestone.

Keep it concise and useful for the next session.
