---
tags: [workflow, cheat-sheet]
---
# Daily Journey

Your day, start to finish. 14 skills, one habit.

Claude users can run the slash commands below directly. Codex and Gemini users should ask for the same workflow by name (see equivalents in their respective pages).

## The Daily Habit

```
Morning:    /focus → /sync → /next
Work:       /fix, /check, /pr
Evening:    /done
Monday:     /audit
Anytime:    /help
```

## Morning

```
/focus              Orient yourself — kb, build health, team activity
/sync               What did the team do? PRs waiting? Unclaimed issues?
/next               Pick the next task from the current milestone
```

If it's your first session, `/focus` will point you to the key kb pages. Read them.

## Working

```
/dev server         Start the local harness (localhost:8080)
/dev desktop        Start the Tauri desktop app
/dev all            Both

/ticket <desc>      Create an issue any teammate can pick up
/fix <issue#>       Pick up an issue, branch, fix, PR
/check              Run lint, compile check, tests
```

### Build loop
```bash
just check          Fast compile check (no tests)
just test           Full test suite
just lint           Clippy — must be clean before you stop
just fmt            Format everything
```

## Reviewing

```
/review <pr#>       Security, correctness, style review
/pr [title]         Create a PR from current branch
```

## End of Day

```
/done               What moved? What's open? KB updated?
/wip                Save and push everything
```

The "left for the team" section of `/done` is the most important part. What does the next person need to know?

When you learn something that isn't in the code:
1. Find the right page in `kb/`
2. Update it
3. If no page fits, create one and link it from `kb/Home.md`

The kb is shared. Keep it current. Your teammates and their agents read it too.

## Quick Reference Card

| I want to... | Do this |
|---|---|
| Start my session | `/focus` then `/dev all` |
| See what the team did | `/sync` |
| Pick next task | `/next` |
| Fix a bug | `/fix 42` |
| Create a task anyone can grab | `/ticket add risk scoring agent` |
| Run quality checks | `/check` |
| Save and go | `/wip` |
| Review a teammate's PR | `/review 17` |
| Create a PR | `/pr` |
| Deploy | `/deploy` |
| Security/compliance audit | `/audit` |
| End the day | `/done` |
| Get help | `/help` |

See also: [[Workflow/Skills Reference]]
