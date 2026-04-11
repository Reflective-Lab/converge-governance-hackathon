---
tags: [workflow, codex]
---
# Working with Codex

This project works well with Codex, but the workflow looks slightly different than Claude Code. Start from the root `CODEX.md` entrypoint, then use this page for the long-form workflow guidance. Codex does not use the repo's Claude slash commands directly. Instead, it uses `AGENTS.md`, the knowledgebase, the shared `just` recipes, and plain-language workflow requests.

`AGENTS.md` also tells Codex to treat names like `/focus`, `/fix`, or `/checkpoint` as workflow intents when a user uses Claude-style shorthand.

## What to Read First

1. `AGENTS.md`
2. `CLAUDE.md` for the shared project rules and architecture
3. `kb/Home.md`
4. The specific `kb/` page your task needs

Do not bulk-read the whole knowledgebase. Start with `kb/Home.md` and follow one relevant link at a time.

## Shared Automation vs Agent Work

This project now has three shared repo-state commands:

```bash
just focus
just sync
just status
```

Use those when you want deterministic output from the repo itself.

Use Codex when the task needs reading, synthesis, code changes, or GitHub workflows:

- understanding a bug
- planning a change
- editing code
- reviewing a PR
- writing an issue
- updating `kb/`

## Workflow Equivalents

| Claude workflow | Use with Codex |
|---|---|
| `/focus` | Ask Codex to "run the focus workflow for this repo" or run `just focus` and discuss the result |
| `/sync` | Ask Codex to "run a team sync for this repo" or run `just sync` |
| `/status` | Ask Codex to "run the project status workflow" or run `just status` |
| `/fix 42` | Ask Codex to "fix issue 42 end to end: read the issue, make the smallest safe change, run `just check && just test && just lint`, and prepare the PR" |
| `/ticket add risk agent` | Ask Codex to "create an agent-ready GitHub issue for adding the risk agent" |
| `/review 17` | Ask Codex to "review PR 17; findings first, with blockers, suggestions, and questions" |
| `/checkpoint` | Ask Codex to "write a session checkpoint: what moved, what kb pages changed, and what the next teammate needs to know" |
| `/parallel a \| b \| c` | Ask Codex explicitly to split the work into parallel subtasks if your client supports that; otherwise use separate sessions or worktrees |

## Prompt Patterns That Work Well

```text
Run the focus workflow for this repository. Read AGENTS.md, CLAUDE.md, and kb/Home.md, then summarize build health, recent activity, and what I should read next.
```

```text
Fix issue 42 end to end. Read the issue, inspect the relevant kb pages, make the minimum safe change, run just check/test/lint, and give me the PR summary.
```

```text
Review PR 17. Findings first, ordered by severity, with file references.
```

```text
Write a checkpoint for this session. Include what moved, whether kb/ needs updates, and what the next teammate should know.
```

## GitHub and Network Notes

The repo workflows assume `gh` is available for issue and PR work. In restricted environments, `just focus` and `just sync` will still run, but GitHub sections may report as unavailable instead of failing the whole workflow.

## Knowledgebase Discipline

When Codex learns something that should outlive the session:

- code changes go in code
- architecture and process knowledge go in `kb/`
- new workflow expectations should be documented where the next student will actually read them

See also: [[Workflow/Daily Journey]], [[Workflow/Working with Claude]], [[Workflow/Skills Reference]]
