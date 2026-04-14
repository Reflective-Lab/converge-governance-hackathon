---
tags: [workflow, claude]
---
# Working with Claude

If you are using Codex, read [[Workflow/Working with Codex]] instead. This page is Claude-specific.

This project has two layers of automation: **Claude Code skills** (slash commands) and **Justfile recipes** (shell commands). They do different things. Use the right one.

## What is a Skill?

A skill is a Claude Code slash command like `/focus` or `/fix 42`. It runs *inside Claude* — Claude reads it, understands the instructions, and executes multi-step workflows using its tools (file reading, editing, shell commands, GitHub CLI).

Skills can:
- Read and reason about code
- Make decisions about what to do next
- Create branches, commits, and PRs
- Update the knowledgebase
- Combine multiple steps into one command

Skills live in `.claude/skills/`. Each has a `SKILL.md` that tells Claude what to do.

## What is a Justfile Recipe?

A Justfile recipe is a shell command like `just test` or `just lint`. It runs *in your terminal* — it's a thin wrapper around cargo, bun, or other CLI tools. No AI involved.

Recipes are deterministic, fast, and dumb. They do exactly one thing.

## When to Use Which

| I want to... | Use | Why |
|---|---|---|
| Build the project | `just build` | Deterministic shell command |
| Run tests | `just test` | Deterministic shell command |
| Run clippy | `just lint` | Deterministic shell command |
| Format code | `just fmt` | Deterministic shell command |
| Start the server | `just server` | Deterministic shell command |
| Start the desktop app | `just dev-desktop` | Deterministic shell command |
| Check what the team did | `/sync` | Needs to read, interpret, summarize |
| Orient myself at session start | `/focus` | Reads kb, checks build, shows team activity |
| Pick next task | `/next` | Reads milestone, picks highest-priority task |
| Fix a GitHub issue end-to-end | `/fix 42` | Multi-step: read issue, branch, code, test, PR |
| Create a well-defined ticket | `/ticket add risk agent` | Needs to explore code, write requirements |
| Review a PR | `/review 17` | Reads diff, reasons about security/correctness |
| Run quality checks | `/check` | Runs lint, compile check, tests |
| Save and push WIP | `/wip` | Multi-step git workflow |
| Capture end-of-session state | `/done` | Reads git state, updates kb, writes summary |
| Deploy | `/deploy` | Deploy to target environment |
| Security/compliance audit | `/audit` | Scans deps, secrets, unsafe code, compliance, drift |

**Rule of thumb:** if it's a single deterministic command, use `just`. If it requires reading, thinking, or multi-step orchestration, use a skill.

## Skills You Should Know (14 total)

### Session lifecycle
```
/focus          → start of session (reads kb, shows team state)
/sync           → what did the team do since you last looked?
/next           → pick next task from the current milestone
/done           → end of session (captures what moved, updates kb)
```

### Development workflow
```
/dev [target]   → start local dev environment
/fix <issue#>   → branch, implement, test, PR — all in one
/check          → run lint, compile check, tests
/ticket <desc>  → create an issue any teammate can pick up
/pr [title]     → create a PR from current branch
/review <pr#>   → review someone's PR
/wip            → save everything and push before switching devices
```

### Operations
```
/deploy         → deploy to target environment
/audit          → security, dependency, compliance, and drift scan
/help           → show available skills
```

### The daily habit
```
Morning:    /focus → /sync → /next
Work:       /fix, /check, /pr
Evening:    /done
Monday:     /audit
Anytime:    /help
```

## Justfile Recipes

```bash
just focus                    # session opener — repo health + recent activity
just sync                     # recent activity, PRs, issues
just status                   # build health, test results
just hit-the-ground-running   # first time: build + test + lint
just check                    # fast compile check (no tests)
just test                     # cargo test --workspace
just build                    # cargo build --workspace
just lint                     # cargo clippy --workspace
just fmt                      # cargo fmt --all
just server                   # start local harness on :8080
just clean                    # delete all build artifacts
just install-desktop          # bun install for desktop frontend
just dev-desktop              # tauri dev mode
just build-desktop            # vite build for desktop frontend
just package-desktop          # native desktop bundle
```

## How They Work Together

A typical flow:

1. `/focus` — Claude reads kb, checks build, shows team state
2. `/next` — pick the highest-priority task from the milestone
3. `/fix 42` — Claude branches, implements, runs `just check && just test && just lint`, commits, creates PR
4. Teammate runs `/review 17` on your PR
5. `/done` — Claude captures what moved, updates kb

The skills call `just` recipes internally. You don't need to run `just lint` yourself if you're using `/fix` — it does that as part of the workflow.

## The Knowledgebase and Claude

Claude reads `kb/` pages when it needs context. The `/focus` skill starts by reading `kb/Home.md`. The `/ticket` skill reads relevant kb pages to write better issues.

When Claude learns something during a session that should be preserved:
- Code changes go in code
- Everything else goes in `kb/`
- The `/done` skill prompts you to update kb if new knowledge emerged

The kb is for humans AND agents. Write it so both can understand it.

See also: [[Workflow/Daily Journey]], [[Workflow/Skills Reference]]
