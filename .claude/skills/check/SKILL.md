---
name: check
model: sonnet
description: Run the Vendor Selection quality gate.
user-invocable: true
allowed-tools: Bash, Read
---
# Check

Run the repo quality gate.

## Steps

1. Run `just check`.
2. Run `just test` when code changed or behavior is being verified.
3. Run `just lint` before calling work complete.
4. For desktop packaging changes, also run `just package-desktop`.

## Output

- Report pass/fail for each command.
- Include the first actionable error with file path and line when available.
- If everything passes, say `Clean.`
