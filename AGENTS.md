# Converge Governance Hackathon

This is the canonical agent entrypoint — all agents (Claude, Codex, Gemini, or otherwise) start here. Long-form documentation lives in `kb/`.

## Philosophy

Strongly typed languages that compile to native code. Rust for the system. Svelte for the UI. Tauri for the desktop shell. No React. No virtual machines. No garbage collectors in the hot path.

Converge is the execution model. Agents propose facts. Facts are promoted through governance gates. Decisions converge or honestly stop. Every fact has provenance. Every decision has evidence.

## The Knowledgebase

`kb/` is an Obsidian vault. It is THE documentation for this project.

**Do NOT read the entire kb on startup.** Lazy-load:

1. Read `kb/Home.md` only when you need to find something (it's the index).
2. Follow ONE wikilink to the specific page you need.
3. Never bulk-read `kb/`.

## Stack

| Layer | Technology |
|---|---|
| System logic | Rust (edition 2024, 1.94+) |
| Agent runtime | Converge v3.4.0 (`converge-pack`, `converge-kernel`, `converge-provider`) |
| Intelligence | Organism v1.2.0 (`organism-pack`, `organism-runtime`) |
| Truth validation | Axiom v0.6.0 |
| Desktop shell | Tauri 2 + SvelteKit 5 |
| Package manager | Bun |
| Task runner | just |

## Architecture

```
governance-server (HTTP API — dev harness)
  └── truth_runtime/
        └── evaluate_vendor.rs    <- THE REFERENCE — study this first

governance-app (shared layer for desktop)
governance-kernel (domain model + in-memory store)
governance-truths (truth catalog + converge bindings)
```

## Build

```bash
just test              # cargo test --workspace
just test-coverage     # tests with coverage report
just server            # cargo run -p governance-server (localhost:8080)
just lint              # cargo clippy --workspace
just check             # cargo check --workspace (fast)
```

## Rules

- No `unsafe` code. Ever.
- Use typed enums, not strings with semantics.
- Suggestors emit proposals, not direct facts — Converge promotes them.
- Every mutation needs an Actor.
- `just lint` clean before considering work done.
- No feature flags. No backwards-compat shims.
- If a real service is unavailable, mock it behind the same trait boundary.

## Converge

| Crate | What |
|---|---|
| `converge-pack` | Authoring: `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| `converge-kernel` | Runtime: `Engine`, `Context`, `Budget`, criteria, run hooks |
| `converge-provider-api` | Chat contracts and capability-routing |
| `converge-provider` | Chat backends, search adapters, tool clients |
| `converge-domain` | Pre-built suggestor packs |

## Organism

| Crate | What |
|---|---|
| `organism-pack` | Intent, planning, adversarial, simulation, learning types |
| `organism-runtime` | Registry, readiness, collaboration runner |
| `organism-planning` | Reasoners, charters, topology transitions |

See `kb/Converge/Organism Patterns.md` for the full pattern catalog: six-stage pipeline, four collaboration topologies, five skepticism kinds, five simulation dimensions, 15 domain packs.

## How to Add a New Truth

1. Define in `governance-truths/src/lib.rs` (key, packs, criteria)
2. Create `governance-server/src/truth_runtime/your_truth.rs`
3. Write suggestors implementing `converge_pack::Suggestor`
4. Write a criterion evaluator implementing `CriterionEvaluator`
5. Wire in `truth_runtime/mod.rs` dispatcher
6. Add domain types to `governance-kernel` if needed
7. `just test` green, `just lint` clean

## Testing

Tests are part of the development process, not an afterthought.

```bash
just test              # all tests
just test-coverage     # with coverage report
```

The test suite includes:
- **Unit tests** — domain model, serialization, truth catalog
- **Integration tests** — HTTP endpoint → engine → projection
- **Negative tests** — invalid inputs, missing fields, failed writes
- **Property tests** — domain invariants hold for arbitrary inputs
- **Soak tests** — repeated execution stability

## Milestones

Read `MILESTONES.md` at the start of every session. Scope all work to the current milestone.
