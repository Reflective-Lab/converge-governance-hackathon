---
tags: [workflow, codex]
---
# Working with Codex

This project works well with Codex. Start from the root `CODEX.md` entrypoint, then use this page for the long-form workflow guidance. Keep the same workflow names used in Claude docs. In Codex, name the workflow directly in plain text: `focus`, `run focus`, `check`, `done`, `audit`, `fix issue 42`, `review PR 17`.

## What to Read First

1. `AGENTS.md` — shared project rules, architecture, and Converge guidance
2. `kb/Home.md`
3. The specific `kb/` page your task needs

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

## Canonical Workflows

| Workflow | Use with Codex |
|---|---|
| `/focus` | `focus`, `run focus`, or `just focus` |
| `/sync` | `sync`, `run sync`, or `just sync` |
| `/next` | `next` or `pick next task from MILESTONES.md` |
| `/fix 42` | `fix 42` or `fix issue #42` |
| `/check` | `check` or `run lint and check` |
| `/done` | `done` or `update MILESTONES.md and CHANGELOG.md` |
| `/ticket <desc>` | `ticket` or `create a GitHub issue for <desc>` |
| `/review 17` | `review 17` or `review PR 17; findings first, with blockers, suggestions, and questions` |
| `/pr` | `pr` or `create a PR from the current branch` |
| `/audit` | `audit` or `run security and dependency audit` |

## Prompt Patterns That Work Well

```text
Read MILESTONES.md, show current milestone, and summarize build health and recent activity.
```

```text
Fix issue 42 end to end. Read the issue, inspect the relevant kb pages, make the minimum safe change, run just check/test/lint, and give me the PR summary.
```

```text
Review PR 17. Findings first, ordered by severity, with file references.
```

```text
Update MILESTONES.md and CHANGELOG.md with what moved this session. Include what the next teammate should know.
```

## GitHub and Network Notes

The repo workflows assume `gh` is available for issue and PR work. In restricted environments, `just focus` and `just sync` will still run, but GitHub sections may report as unavailable instead of failing the whole workflow.

## Knowledgebase Discipline

When Codex learns something that should outlive the session:

- code changes go in code
- architecture and process knowledge go in `kb/`
- new workflow expectations should be documented where the next student will actually read them

Older repos may still refer to `checkpoint` and `quality`. In this repo, keep `/done` and `/check` as the public workflow names in docs and daily use.

See also: [[Workflow/Daily Journey]], [[Workflow/Working with Claude]], [[Workflow/Skills Reference]]
