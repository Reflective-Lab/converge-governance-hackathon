---
tags: [workflow, skills]
---
# Skills Reference

Claude Code skills available in this project. Run them with `/skillname` in the prompt.

If you are using Codex, see [[Workflow/Working with Codex]]. The workflow names are the same even though Codex does not use project slash commands.

## Session Management

### /focus
Session opener. Orients you in the project — reads the knowledgebase, shows team activity, checks build health. First-time users get a reading list. Returning users get straight to state.

### /checkpoint
End-of-session. Captures what you moved, prompts kb updates, writes a "left for the team" summary so the next person knows what's in flight.

### /sync
Team sync. Recent commits by author, PRs awaiting review, unclaimed issues, build health, stale kb pages. Designed to avoid duplicate work.

### /status
Quick health check. Rust build, tests, clippy, desktop build status.

## Development

### /dev [server|desktop|all]
Start local development environment. Server runs the harness on :8080. Desktop runs Tauri+Svelte.

### /fix [issue-number]
End-to-end issue fix. Reads the issue, creates a branch, implements, verifies with `just check && just test && just lint`, commits, creates PR.

### /ticket [description]
Creates a well-defined GitHub issue any teammate can pick up. Includes context, requirements, "where to start" section with kb links and code patterns, key files, and test plan.

### /parallel [task | task | task]
Runs independent tasks in parallel git worktrees. Each agent commits, pushes, creates a PR. PRs are flagged for team review — not self-merged.

## Code Quality

### /quality [baseline|check|trend]
Captures quality metrics: clippy warnings, complexity, test counts, unsafe blocks, vulnerabilities, LOC. Tracks trends in `quality-log.csv`.

### /audit
Security and dependency audit. Checks for vulnerabilities, hardcoded secrets, unsafe code, exposed `.env` files. Zero tolerance.

## Git & GitHub

### /pr [title]
Creates a PR from current branch changes. Pushes to remote, drafts body from commit history.

### /review [pr-number]
Reviews a PR for security, correctness, style, and operational concerns. Reports findings — does not leave PR comments.

### /merge [pr-number]
Squash-merges a PR. Checks CI status, merges, syncs local main, deletes the branch.

### /wip
Saves work-in-progress and pushes. Use before switching devices or ending a session.

### /feedback [observations]
Turns unstructured testing feedback into well-formed GitHub issues. Classifies as bug, ux, or feature.

See also: [[Workflow/Daily Journey]]
