---
name: branch
model: haiku
description: Start or switch to a release branch for Vendor Selection work.
user-invocable: true
argument-hint: release/<version>
allowed-tools: Bash
---
# Branch

Start or switch to a release branch.

## Allowed Branches

| Prefix | Use |
|---|---|
| `main` | The single train |
| `release/<version>` | Release stabilization |

## Steps

1. Use `$ARGUMENTS` as the release branch name, or ask if missing.
2. Run `git status --short --branch`.
3. If the branch is not `release/<version>`, stop and explain that feature/topic branches are not used in this repo.
4. If the checkout is dirty, warn and stop unless the user explicitly wants to carry current changes.
5. Pull latest main with `git pull --ff-only origin main` when on `main`.
6. Create or switch to the release branch with `git switch -c <branch>` or `git switch <branch>`.

## Rules

- One train. No worktrees.
- No feature, topic, fix, docs, chore, or spike branches.
- Never discard local changes.
- Do not push without user confirmation.
