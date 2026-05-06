---
tags: [workflow, skills]
source: mixed
---
# Skills Reference

18 skills. Run them with `/skillname` in Claude Code.

Codex and Gemini users: see [[Workflow/Working with Codex]] and [[Workflow/Working with Gemini]] for equivalents.

## The Daily Habit

```
Morning:    /focus -> /sync -> /next
Work:       /fix, /test, /check, /pr
Evening:    /done or /wip
Monday:     /audit
Anytime:    /help, /experiment
```

## Session Management

### /focus
Session opener. Orients you in the project — reads MILESTONES.md, shows current milestone and days remaining, scopes the session.

### /sync
Team sync. Pulls latest, shows open PRs, issues, and milestone state.

### /next
Pick next task from the current milestone. Reads MILESTONES.md, selects highest-priority unclaimed work.

### /done
End-of-session. Updates MILESTONES.md and CHANGELOG.md, captures what moved, writes a "left for the team" summary.

### /experiment
Hypothesis-driven development with evidence recorded under `kb/Experiments/`.

## Development

### /dev [server|desktop|all]
Start local development environment. Server runs the harness on :8080. Desktop runs Tauri+Svelte.

### /fix [issue-number]
End-to-end issue fix on the current train or release branch. Reads the issue, implements, verifies with `just check && just test && just lint`, commits, creates PR.

### /check
Run the full quality suite — lint, compile check, tests.

### /test
Add or run focused tests for a crate, app, module, or regression.

### /ticket [description]
Creates a well-defined GitHub issue any teammate can pick up. Includes context, requirements, "where to start" section with kb links and code patterns, key files, and test plan.

## Git & GitHub

### /branch release/[version]
Starts or switches to a release branch. This repo does not use feature branches or worktrees.

### /pr [title]
Creates a PR from current branch changes. Pushes to remote, drafts body from commit history.

### /review [pr-number]
Reviews a PR for security, correctness, style, and operational concerns. Reports findings — does not leave PR comments.

### /wip
Saves work-in-progress and pushes. Use before switching devices or ending a session.

### /merge-cleanup release/[version]
Retires a release branch after release. This is not a feature-branch cleanup workflow.

## Operations

### /deploy
Deploy to target environment.

### /audit
Security, dependency, compliance, and drift audit. Checks for vulnerabilities, hardcoded secrets, unsafe code, exposed `.env` files, compliance gaps, and configuration drift. Zero tolerance.

### /help
Show available skills and usage.

See also: [[Workflow/Daily Journey]]
