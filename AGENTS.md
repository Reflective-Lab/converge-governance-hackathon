# Vendor Selection

This is the canonical agent entrypoint. Claude, Codex, Gemini, and other agents start here. Long-form documentation lives in `kb/`.

## Philosophy

Strongly typed languages that compile to native code. Rust for the system. Svelte for the UI. Tauri for the desktop shell. No React. No virtual machines. No garbage collectors in the hot path.

Converge is the execution model. Agents propose facts. Facts are promoted through governance gates. Decisions converge or honestly stop. Every fact has provenance. Every decision has evidence.

## Operating Model

One product, one train, one product truth.

- The product truth is `vendor-selection`.
- Supporting runtimes may remain as examples, migration fixtures, or tests, but they are not product truths.
- Do not add new product truths by default. Extend `vendor-selection` unless the milestone explicitly says otherwise.
- Do not use git worktrees.
- Do not create feature/topic branches for normal work.
- Use `main` for the train and `release/<version>` branches only when preparing or stabilizing a release.

## Product Direction

This repo is now the starting point for a vendor-selection product, not a hackathon template.

- Web app: Svelte/SvelteKit front end, Firebase Hosting, Firefox support.
- Backend: Rust service boundary, evolving from the current HTTP harness toward gRPC.
- Data: monitored Google Cloud database for durable decisions, evidence, audit, and telemetry.
- Desktop: Tauri 2 + Svelte + Rust, packaged for macOS Apple silicon, macOS Intel, and Windows.
- Cloud resources: Terraform first. `gcloud` is for checks and operational fixes, not provisioning.

## Knowledge Base

`kb/` is an Obsidian vault. Lazy-load it.

1. Read `kb/Home.md` only when you need to find something.
2. Follow one wikilink to the specific page you need.
3. Never bulk-read `kb/`.

Also follow `/Users/kpernyer/dev/work/kb` standards when project rules are unclear, especially:

- `Standards/Project Scaffold.md`
- `Standards/Conventions.md`
- `Workflow/Cheat Sheet.md`

## Stack

| Layer | Technology |
|---|---|
| System logic | Rust edition 2024, rust-version 1.94+ |
| Agent runtime | Converge 3.8.1 |
| Intelligence | Organism 1.5.0 |
| Truth validation | Axiom 0.7.x |
| Desktop shell | Tauri 2 + Svelte |
| Web shell | SvelteKit 5 + Bun |
| Cloud | Google Cloud + Firebase |
| Task runner | just |

## Architecture

```text
governance-server
  truth_runtime/
    vendor_selection.rs
    vendor_selection_live.rs
    evaluate_vendor.rs

governance-app      shared app layer
governance-kernel   domain model and store
governance-truths   vendor-selection truth and Converge bindings
apps/desktop        Tauri/Svelte operator shell
```

Study the vendor-selection runtime before changing governed decision behavior.

## Build

```bash
just check             # cargo check --workspace
just test              # cargo test --workspace
just lint              # cargo clippy --workspace
just server            # cargo run -p governance-server
just desktop           # Tauri desktop app
just package-desktop   # desktop release bundle for current platform
```

## Rules

- No `unsafe` code.
- Use typed enums, not strings with semantics.
- Suggestors emit proposals, not direct facts.
- Every mutation needs an Actor.
- Providers stay behind trait boundaries.
- No feature flags or backwards-compat shims unless the milestone explicitly calls for them.
- If a real service is unavailable, mock it behind the same trait boundary.
- `just lint` must be clean before work is considered done.

## How To Change The Product Truth

1. Start from `vendor-selection` in `governance-truths/src/lib.rs`.
2. Change or add suggestors inside the vendor-selection runtime.
3. Extend criteria on the existing `vendor-selection` truth.
4. Add domain types to `governance-kernel` if needed.
5. Keep every promoted fact tied to provenance.
6. Add tests and documentation.
7. Run `just test` and `just lint`.

## Testing

Tests are part of the development process.

```bash
just test
just test-coverage
```

Expected coverage areas:

- Unit tests for domain model, serialization, and product-truth behavior.
- Integration tests for backend endpoint to engine to projection.
- Negative tests for invalid inputs, missing fields, and failed writes.
- Property tests for domain invariants.
- Soak tests for repeated execution stability where runtime behavior is risky.

## Milestones

Read `MILESTONES.md` at the start of every session and scope work to the current milestone.
