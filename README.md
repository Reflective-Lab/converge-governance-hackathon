# Scout — Sourcing

**Scout** is the governed sourcing & vendor-selection app: compare vendors, promote evidence through Converge gates, and produce decisions that can be audited.

This repository is being migrated from a hackathon starter kit into a product baseline:

- **Web app:** Svelte/SvelteKit front end, deployed through Firebase Hosting and supported in Firefox.
- **Backend:** Rust services, with the current HTTP harness evolving toward a gRPC API boundary.
- **Database:** monitored Google Cloud database for durable decisions, evidence, audit trails, and operational telemetry.
- **Desktop app:** Tauri 2 + Svelte + Rust, packaged for macOS Apple silicon, macOS Intel, and Windows.

The governance model remains the same: agents propose facts, Converge promotes valid proposals through policy gates, and every decision either converges with evidence or stops honestly.

## Product Rules

- **Scout** (this repo) is the one product truth.
- Supporting runtimes are examples or migration fixtures, not product workflows.
- The repo uses one train: `main` plus `release/<version>` branches only.
- Do not use git worktrees.
- Do not create feature branches for normal work.

## Stack

| Layer | Technology |
|---|---|
| System logic | Rust edition 2024, rust-version 1.94+ |
| Governance runtime | Converge 3.8.1 |
| Intelligence | Organism 1.5.0 |
| Truth validation | Axiom 0.7.x |
| Web UI | SvelteKit 5 + Bun |
| Desktop shell | Tauri 2 + Svelte |
| Cloud | Google Cloud, Firebase Hosting, monitored database |
| Task runner | just |

## Quick Start

```bash
just setup
just check
just test
just dev
```

Start only the backend harness:

```bash
just server
```

Run a governed vendor evaluation:

```bash
curl -X POST http://localhost:8080/v1/truths/vendor-selection/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs":{"vendors":"Acme AI, Beta ML, Gamma LLM"},"persist_projection":true}'
```

Launch the desktop app:

```bash
just desktop
```

Package the desktop app:

```bash
just package-desktop
```

## Project Layout

```text
crates/
  governance-kernel/       Domain model and in-memory store
  governance-truths/       Vendor-selection truth and Converge bindings
  governance-server/       Local backend harness and truth executors
  governance-app/          Shared Rust app layer
apps/
  desktop/                 Tauri 2 + Svelte desktop shell
examples/
  vendor-selection/        Seed data, policies, and example requests
kb/                        Obsidian knowledge base
```

## Key Docs

- [AGENTS.md](AGENTS.md) - canonical instructions for all agents
- [MILESTONES.md](MILESTONES.md) - current product roadmap
- [CAPABILITIES.md](CAPABILITIES.md) - what the starter supports
- [DEPLOYMENT.md](DEPLOYMENT.md) - Google Cloud and Firebase deployment plan
- [RELEASE.md](RELEASE.md) - desktop release packaging plan
- [CONTRIBUTING.md](CONTRIBUTING.md) - contributor workflow
- [SECURITY.md](SECURITY.md) - vulnerability reporting
- [SUPPORT.md](SUPPORT.md) - support channels

## Rules

- No `unsafe` code.
- Use typed enums for semantic state.
- Every mutation needs an Actor.
- Suggestors emit proposals, not direct facts.
- Providers stay behind trait boundaries.
- Run `just lint` before considering code complete.

## License

MIT. See [LICENSE](LICENSE).
