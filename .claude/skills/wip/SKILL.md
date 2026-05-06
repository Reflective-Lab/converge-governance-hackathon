---
name: wip
model: sonnet
description: Save work in progress so it can be resumed safely.
user-invocable: true
allowed-tools: Bash, Read
---
# WIP

Save current work state.

## Steps

1. Run `git status --short`.
2. Summarize modified files by area.
3. Run a fast check if appropriate, usually `just check`.
4. Commit or push only if the user asks.
5. Provide a short resume note.

## Rules

- Do not hide failing checks.
- Do not include secrets or generated artifacts.
- Never push to `main` without confirmation.
