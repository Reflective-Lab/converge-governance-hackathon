---
name: pr
model: sonnet
description: Prepare a pull request from the current Vendor Selection branch.
user-invocable: true
argument-hint: [title]
allowed-tools: Bash, Read, Grep
---
# PR

Prepare a pull request.

## Steps

1. Run `git status --short`.
2. Run relevant verification, normally `just check`, `just test`, and `just lint`.
3. Summarize the diff.
4. Draft a PR title and body using `.github/pull_request_template.md`.
5. Push and create the PR only if the user asks.

## Rules

- Do not include generated build artifacts.
- Mention verification explicitly.
- Call out any checks that were not run.
