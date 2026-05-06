---
name: review
model: opus
description: Review a pull request for correctness, regressions, security, and missing tests.
user-invocable: true
argument-hint: [pr-number]
allowed-tools: Bash, Read, Grep
---
# Review

Review a PR in findings-first style.

## Steps

1. Inspect the PR diff with `gh pr diff $ARGUMENTS` or local git diff.
2. Check changed tests and docs.
3. Look for behavior regressions, data handling risks, missing policy gates, and missing verification.
4. Report findings first, ordered by severity, with file and line references.

## Rules

- Do not summarize before findings.
- If there are no findings, say so clearly.
- Mention residual risk or tests not run.
