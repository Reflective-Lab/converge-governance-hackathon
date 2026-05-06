---
name: ticket
model: sonnet
description: Turn a rough Vendor Selection task into an agent-ready GitHub issue.
user-invocable: true
argument-hint: [description]
allowed-tools: Bash, Read, Grep
---
# Ticket

Draft or create a GitHub issue.

## Steps

1. Read `MILESTONES.md` for current scope.
2. Turn the request into a concise problem statement.
3. Add acceptance criteria.
4. Identify likely files or modules.
5. Create the issue with `gh issue create` only if the user asks.

## Output

- Title
- Body
- Labels suggestion
- Milestone fit
