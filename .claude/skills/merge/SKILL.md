---
name: merge
description: Squash-merge a PR by number, sync main, delete the branch
disable-model-invocation: true
argument-hint: [pr-number]
allowed-tools: Bash
---

# Merge PR #$ARGUMENTS

Squash-merge, sync local main, clean up.

## Steps

1. **Show what's about to be merged**
```bash
gh pr view $ARGUMENTS
```

2. **Confirm CI is green.** If checks are failing, stop and report.
```bash
gh pr checks $ARGUMENTS
```

3. **Squash-merge and delete remote branch.**
```bash
gh pr merge $ARGUMENTS --squash --delete-branch
```

4. **Sync local main.**
```bash
git checkout main && git pull --ff-only
```

5. **Delete local branch** if it still exists.
```bash
BRANCH=$(gh pr view $ARGUMENTS --json headRefName -q .headRefName)
git branch -d "$BRANCH" 2>/dev/null || true
```

6. Report the merge commit SHA and confirm main is up to date.

## Notes

- Never merge PRs targeting a base other than main without confirming.
- Only use `--admin` if the user explicitly says so.
- If `git pull --ff-only` fails, stop and report.
