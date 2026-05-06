---
name: focus
model: sonnet
description: Start a Vendor Selection work session with repo orientation.
user-invocable: true
allowed-tools: Bash, Read
---
# Focus

Start a work session.

## Steps

1. Read `MILESTONES.md`.
2. Show `git status --short`.
3. Show the current branch and latest commit.
4. Check open local work before suggesting a new task.
5. Run `just check` only when the user asks for a health check.

## Output

- Current milestone
- Branch and dirty state
- Important in-progress work
- Suggested next step
