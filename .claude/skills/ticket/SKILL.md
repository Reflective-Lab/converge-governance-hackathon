---
name: ticket
description: Create a well-defined GitHub issue that any teammate or agent can pick up and execute
disable-model-invocation: true
argument-hint: [description of what needs to be done]
allowed-tools: Bash, Read, Grep, Glob
---

# Create Agent-Ready Ticket

Create a GitHub issue from $ARGUMENTS detailed enough for a teammate who hasn't seen this code to execute without questions.

## Steps

1. **Understand the request** — explore the codebase if needed. Read relevant `kb/` pages for context.

2. **Determine area and size**
   - Area: kernel, truths, server, app, desktop, integrations, kb
   - Size: small (< 1hr), medium (1-4hr), large (4hr+)

3. **Find similar work** — look for existing patterns the implementer can follow:
   ```bash
   # Example: if adding a new agent, point to the reference executor
   grep -r "impl Agent" crates/ --include="*.rs" -l
   ```

4. **Create the issue** using `gh`:

```bash
gh issue create --title "[AREA]: short description" --label "task" --body "$(cat <<'EOF'
## Context
Why this needs to happen. What problem it solves.

## Requirements
- [ ] Concrete acceptance criterion 1
- [ ] Concrete acceptance criterion 2
- [ ] Concrete acceptance criterion 3

## Where to start
- Read `kb/<relevant-page>.md` for background
- Study `<path/to/similar/code.rs>` for the pattern to follow
- The new code goes in `<path/to/target/location>`

## Key files
- `path/to/file.rs` — what this file does and why it matters
- `path/to/other.rs` — related code to understand first

## Test plan
- [ ] `just test` passes
- [ ] `just lint` clean
- [ ] <specific verification step>

## Size
small | medium | large
EOF
)"
```

5. Return the issue URL.

## Rules
- Every requirement must be testable
- Key files must be actual paths in the repo
- "Where to start" is mandatory — a new teammate should know exactly what to read first
- If the task is large, break it into smaller tickets
- Reference the relevant kb/ page so context is one click away
