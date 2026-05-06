---
name: next
model: sonnet
description: Pick the next useful task from the current Vendor Selection milestone.
user-invocable: true
allowed-tools: Bash, Read, Grep
---
# Next

Find the next milestone task.

## Steps

1. Read `MILESTONES.md`.
2. Identify unchecked items in the current milestone.
3. Check `git status --short` so recommendations do not ignore in-progress work.
4. Recommend one small, high-leverage next task.

## Rules

- Do not invent commitments outside the current milestone.
- Prefer tasks that reduce uncertainty or unblock product architecture.
- Keep the answer short.
