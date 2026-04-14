---
tags: [development]
---
# Getting Started

## Prerequisites

- Rust 1.93+ (stable)
- cargo
- just (task runner)
- tauri-cli
- Bun

## First Time Setup

```bash
git clone <repo>
cp .env.example .env
just hit-the-ground-running
```

## Quick Reference

| Command | What it does |
|---------|-------------|
| `just hit-the-ground-running` | First time setup: build, test, lint |
| `just test` | Run all tests |
| `just build` | Build the workspace |
| `just server` | Start local HTTP harness on port 8080 |
| `just lint` | Run clippy |
| `just fmt` | Format all code |
| `just clean` | Delete all build artifacts |
| `just install-desktop` | Install desktop frontend dependencies |
| `just dev-desktop` | Run desktop app in dev mode |
| `just package-desktop` | Build native desktop bundle |

## Local Harness API

| Endpoint | Purpose |
|---|---|
| `GET /health` | Health check |
| `GET /v1/truths` | List available truths |
| `POST /v1/truths/{key}/execute` | Execute a truth |
| `GET /v1/decisions` | Query decisions |
| `GET /v1/vendors` | Query vendors |
| `GET /v1/audit` | Query audit trail |

## Project Structure

```
kb/                    Obsidian knowledgebase — THE documentation
.claude/skills/        Claude Code slash commands
scripts/workflow/      Shared workflow helpers for Claude and Codex
apps/desktop/          Svelte + Tauri desktop app
crates/
  governance-kernel/   Domain model + in-memory store
  governance-truths/   Truth catalog + converge bindings
  governance-server/   HTTP API + truth runtime
  governance-app/      Shared app layer
examples/              Example input files
```

## Next Steps

- Read [[Development/Programming API Surfaces]] before copying patterns from `apps/desktop/` or external prototypes
- Read [[Workflow/Daily Journey]] for the daily cheat sheet
- Read [[Workflow/Working with Claude]] if you are using Claude Code
- Read [[Workflow/Working with Codex]] if you are using Codex
- Read [[Workflow/Skills Reference]] for the Claude `/slash` commands
- Read [[Domain/Vendor Selection]] to understand the challenge
- Study `crates/governance-server/src/truth_runtime/evaluate_vendor.rs` — the reference implementation

See also: [[Architecture/Layers]], [[Development/Writing Suggestors]]
