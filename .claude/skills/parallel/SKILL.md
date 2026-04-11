---
name: parallel
description: Run multiple tasks in parallel using isolated git worktrees — each agent pushes and creates a PR for team review
disable-model-invocation: true
argument-hint: [task descriptions separated by |]
---

# Parallel Task Execution

Run multiple independent tasks simultaneously, each in its own git worktree.

## Input format

Tasks separated by `|` in $ARGUMENTS:
```
/parallel fix the compliance agent | add risk scoring | update kb with new patterns
```

## Execution

For each task, launch an Agent with `isolation: "worktree"` and `run_in_background: true`.

Each agent MUST:

1. **Read `kb/Home.md`** for project context
2. **Work in isolation** — full copy of the repo, no conflicts
3. **Implement the task** — minimum changes needed
4. **Verify** — `just check && just lint`
5. **Commit** with a clear message
6. **Push the branch**
   ```bash
   git push -u origin HEAD
   ```
7. **Create a PR** for team review:
   ```bash
   gh pr create --title "<short description>" --body "$(cat <<'EOF'
   ## Summary
   <what changed and why>

   ## Review notes
   <anything a reviewer should pay attention to>

   ## Launched by
   `/parallel` skill — autonomous agent work. Needs human review before merge.

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   EOF
   )"
   ```

Launch ALL agents in a **single message** with multiple tool calls (truly parallel).

## After completion

Report for each agent:
- PR URL (or error if it failed)
- Summary of changes
- Files modified

**Do not self-merge.** These PRs need team review via `/review`.
