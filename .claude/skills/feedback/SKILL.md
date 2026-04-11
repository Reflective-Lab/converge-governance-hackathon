---
name: feedback
description: Capture unstructured testing feedback — turn observations into GitHub issues
disable-model-invocation: true
user-invocable: true
argument-hint: [paste your observations]
allowed-tools: Bash, Read
---

# Testing Feedback

Take the user's unstructured feedback from $ARGUMENTS and turn it into well-formed GitHub issues.

## Steps

1. **Parse the feedback** — identify distinct issues. Each observation becomes one issue.

2. **Classify each issue:**
   - `bug` — something broken or wrong
   - `ux` — friction, confusing flow
   - `feature` — missing capability

3. **Create GitHub issues** for each:
   - Title: `[AREA]: <concise description>` (area: kernel, server, desktop, app, truths, kb)
   - Label: the classification above
   - Body: what the user observed, expected behavior, which flow it's in

4. **Output a summary:**

```
── Feedback filed ─────────────────────────────────

1. #<num> [ux] <title>
2. #<num> [bug] <title>
3. ...

────────────────────────────────────────────────────
```

## Rules

- One issue per observation. Don't merge unrelated feedback.
- Keep issue titles short and scannable.
- Don't editorialize. Use the user's words.
- Ask the user to confirm before creating if anything is ambiguous.
- Reference relevant kb/ pages in the issue body when applicable.
