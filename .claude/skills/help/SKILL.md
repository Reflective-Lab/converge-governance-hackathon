---
name: help
model: haiku
description: Show available Vendor Selection skills and daily workflow.
user-invocable: true
allowed-tools: Read
---
# Skills

```text
Morning:    /focus -> /sync -> /next
Work:       /fix, /test, /check, /pr
Evening:    /done or /wip
Weekly:     /audit
As needed:  /review, /ticket, /experiment, /deploy, /branch release/<version>
```

## Developer

| Skill | Purpose |
|---|---|
| `/dev` | Start local backend, desktop, web, or all targets |
| `/check` | Run quality gate |
| `/test` | Add or run focused tests |
| `/fix <issue>` | Fix an issue |
| `/pr` | Prepare a pull request |
| `/wip` | Save resumable work |

## Product And Planning

| Skill | Purpose |
|---|---|
| `/focus` | Session opener |
| `/next` | Pick the next milestone task |
| `/ticket` | Draft or create an issue |
| `/done` | End-session handoff |
| `/experiment` | Run and record a falsifiable experiment |

## Review And Operations

| Skill | Purpose |
|---|---|
| `/audit` | Security, compliance, drift, and milestone review |
| `/review <pr>` | Review a pull request |
| `/sync` | Pull, PRs, issues, milestone progress, health |
| `/deploy` | Deploy or package with confirmation |
| `/branch release/<version>` | Start or switch to a release branch |
| `/merge-cleanup release/<version>` | Retire a release branch after release |

## Git Rule

Use one train: `main` plus release branches only. Do not use worktrees or feature branches.

For the workspace reference, read `/Users/kpernyer/dev/work/kb/Workflow/Cheat Sheet.md`.
