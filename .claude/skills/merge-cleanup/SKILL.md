---
name: merge-cleanup
model: haiku
description: Retire a Vendor Selection release branch after release.
user-invocable: true
argument-hint: release/<version>
allowed-tools: Bash
---
# Merge Cleanup

Retire a release branch after the release has landed.

## Steps

1. Use `$ARGUMENTS` as the release branch name, or ask if missing.
2. If the branch is not `release/<version>`, stop.
3. Switch to `main` with `git switch main`.
4. Pull latest with `git pull --ff-only origin main`.
5. Delete the local release branch with `git branch -d <branch>`.
6. Delete the remote release branch with `git push origin --delete <branch>` only if it still exists and the user confirms.
7. Show final `git status --short --branch`.

## Rules

- Never force-delete a branch without explicit user confirmation.
- Do not clean up unreleased work.
- Do not use worktrees.
