---
tags: [workflow, gemini]
---
# Working with Gemini

If you are using Claude Code, read [[Workflow/Working with Claude]] instead. If you are using Codex, read [[Workflow/Working with Codex]]. This page is Gemini-specific.

This project uses **Gemini CLI** as a first-class collaborator. To ensure precision and consistency with other agents, Gemini uses a combination of its own specialized tools and the shared project workflows.

## How Gemini Works Here

Gemini follows the same [[Workflow/Daily Journey|Daily Journey]] as other agents. It is instructed by `GEMINI.md` and `AGENTS.md` to treat slash commands as workflow intents.

### Native Tools vs Shared Workflows

| I want to... | Tool | Why |
|---|---|---|
| Run a session /focus | `gemini "read MILESTONES.md, show current milestone"` | Orient yourself at session start |
| Sync with the team (/sync) | `gemini "pull, show PRs and issues"` or `just sync` | Shared script for GitHub and git status |
| Pick next task (/next) | `gemini "pick next task from MILESTONES.md"` | Reads milestone, picks highest-priority task |
| Run quality checks (/check) | `gemini "run lint and check"` | Lint, compile check, tests |
| Deep architecture research | `codebase_investigator` | Gemini's specialized tool for complex analysis |
| Batch refactoring | `generalist` | Efficient multi-file operations |
| Fix a bug or implement a feature | `replace`, `write_file`, `run_shell_command` | Surgical code modifications |
| Finalize a session (/done) | `gemini "update MILESTONES.md and CHANGELOG.md"` | Capture what moved and what the next teammate needs |
| Security/compliance audit (/audit) | `gemini "run security and dependency audit"` | Monday ritual |

## Workflow Patterns

Gemini should be prompted using plain-language intents that mirror the project's slash commands:

```text
Read MILESTONES.md, show current milestone and recent activity.
```

```text
Fix issue #42. Read the issue, find the relevant code, make the change, run `just check && just test && just lint`, and prepare the PR.
```

```text
Update MILESTONES.md and CHANGELOG.md with what moved this session. Note what the next teammate needs to know.
```

## Sub-Agent Delegation

One of Gemini's key strengths is its ability to delegate complex tasks to sub-agents.

- **`codebase_investigator`**: Use this when a task is ambiguous or requires understanding deep dependencies across crates. It will return a structured report that Gemini can then act on.
- **`generalist`**: Use this for high-volume tasks that would otherwise clutter the main session history (e.g., adding license headers, updating imports across 10 files, or fixing multiple lint errors).

## Knowledgebase Discipline

Gemini is mandated to use the `kb/` directory as the primary source of truth and the place to store new architectural decisions. While it can use `save_memory` for personal preferences, any project-level knowledge MUST go in `kb/`.

See also: [[Workflow/Daily Journey]], [[Workflow/Working with Claude]], [[Workflow/Working with Codex]]
