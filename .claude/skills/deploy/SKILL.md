---
name: deploy
model: sonnet
description: Deploy or package Vendor Selection targets with confirmation.
user-invocable: true
argument-hint: [desktop|web|backend|all]
allowed-tools: Bash, Read
---
# Deploy

Deploy or package a target.

## Steps

1. Run `/check` first. Stop if it fails.
2. Choose target from `$ARGUMENTS`, or ask.
3. For `desktop`, run `just package-desktop`.
4. For `web` or `backend`, confirm the deployment recipe exists before running it.
5. Verify health after deploy or package creation.
6. Report artifacts, URLs, or missing recipes.

## Rules

- Confirm before production-affecting commands.
- Cloud resources must be Terraform-managed.
- If no deploy recipe exists, document the missing recipe instead of improvising production changes.
