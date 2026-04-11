---
name: dev
description: Start local development environment
disable-model-invocation: true
argument-hint: [server|desktop|all]
allowed-tools: Bash
---

# Start Development Environment

Start the specified service locally ($ARGUMENTS or "all").

## Server (local harness)
```bash
just server
```
Runs on http://localhost:8080. Development harness only — not the product surface.

## Desktop
```bash
just dev-desktop
```
Tauri + Svelte app. Calls Rust core locally. Outbound calls to Kong only.
Requires `bun install` first (`just install-desktop`).

## All
Start server in background, then desktop in foreground.

Remind the user to have `.env` set up with:
```
KONG_AI_GATEWAY_URL=https://...
KONG_API_KEY=...
```
