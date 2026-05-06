# Capabilities

What this vendor-selection starter provides and what is planned next.

## Current

### Governed Decision Loop

- Agents emit proposals, not direct facts.
- Promotion gates validate proposals before they become facts.
- Cedar policies govern consequential actions.
- The engine converges to a fixed point or stops honestly.
- Decision records preserve provenance and evidence.

### Product Truth

`vendor-selection` is the one product truth. It owns vendor intake, evidence, scoring, policy gates, audit output, and the final recommendation.

Other runtimes may exist during migration as examples, fixtures, or tests. They should not be exposed as product workflows.

### Operator Surfaces

- Local backend harness on `localhost:8080`.
- Tauri/Svelte desktop app scaffold.
- Seed data, policies, and vendor-selection examples.
- Workflow skills under `.claude/skills`.
- Knowledge base under `kb/`.
- One git train: `main` plus release branches only. No worktrees and no feature branches.

## Planned

### Web App

- Svelte/SvelteKit app deployable to Firebase Hosting.
- Firefox-supported browser workflow.
- Shared UI patterns with the desktop operator shell.

### Backend

- Rust gRPC service boundary.
- Durable database-backed projections.
- Typed contracts for vendors, criteria, evidence, decisions, and audit trails.

### Cloud

- Google Cloud runtime and monitored database.
- Terraform-managed infrastructure.
- Firebase Hosting for web.
- Observability for backend, database, and governance runs.

### Desktop Releases

- Tauri bundles for macOS Apple silicon, macOS Intel, and Windows.
- Signing, notarization, checksums, and downloadable releases.

## Commands

```bash
just check
just test
just lint
just server
just desktop
just package-desktop
```
