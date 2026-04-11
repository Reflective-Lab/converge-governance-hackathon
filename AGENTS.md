# AGENTS

This repository supports both Claude Code and Codex. `kb/` is the canonical documentation. The root files are entrypoints for different agents, not the long-form docs.

Start here:

1. Read this file.
2. Read the agent-specific root file:
   - Claude: `CLAUDE.md`
   - Codex: `CODEX.md`
3. Read `kb/Home.md` as the index. Follow only the links you need. Do not bulk-read `kb/`.
4. Run the session opener:
   - Claude: `/focus`
   - Codex or terminal-first workflows: `just focus`
5. Update `kb/` when you learn something the next human or agent will need.

## Quick Rules

- This is an opinionated project. Rust, Svelte, Tauri. No React. No VMs.
- `kb/` is the knowledgebase. Obsidian vault. Update it when you learn something.
- Converge is the execution model. Agents propose facts. Facts are promoted through governance gates.
- All external access goes through Kong. No direct API calls.
- No `unsafe` code. Typed enums, not strings. `just lint` clean.
- Read `kb/Home.md` first, then follow only relevant pages.
- Run the session focus workflow at the start of a session. Read the kb. Check team activity.

## Workflow

- Claude skills live in `.claude/skills/`.
- Shared deterministic workflow helpers live in `scripts/workflow/` and are exposed as `just focus`, `just sync`, and `just status`.
- Codex users should read `CODEX.md`, then `kb/Workflow/Working with Codex.md`.
- If a user refers to `/focus`, `/sync`, `/checkpoint`, `/fix`, `/ticket`, `/parallel`, `/review`, `/pr`, `/merge`, `/status`, `/quality`, `/audit`, `/wip`, or `/feedback`, treat that as the workflow name even if your client does not support slash commands.

See `CODEX.md`, `CLAUDE.md`, `kb/Workflow/Working with Codex.md`, `kb/Workflow/Working with Claude.md`, `kb/Workflow/Daily Journey.md`, and `kb/Workflow/Skills Reference.md`.
