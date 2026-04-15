# Converge Governance Hackathon — Capabilities

What this project provides for hackathon participants exploring governed multi-agent systems.

## Governance Loop

End-to-end demonstration of Converge's governance model:

- **Truth submission** — Declare what should be true, with typed evidence
- **Proposal protocol** — Agents emit proposals, never direct facts
- **Cedar policy evaluation** — Real Amazon Cedar authorizer decides who can do what
- **Promotion gates** — Proposals are validated before becoming facts
- **Convergence visibility** — Watch the engine iterate until fixed-point

## Cedar Policy Engine

Real Cedar-based authorization with hackathon-friendly examples:

| Policy type | What it governs |
|---|---|
| Agent authority levels | Advisory, participatory, supervisory, sovereign |
| Commitment actions | Propose, commit, promote |
| Amount thresholds | Spending limits requiring human approval |
| Phase gates | Development → testing → staging → production |
| Delegation tokens | Ed25519-signed, time-scoped, replay-protected |

Includes fixtures, test cases, and a policy validator you can modify.

## Truth Catalog

Pre-built truths students can explore and extend:

- **Vendor Selection** — Multi-criteria evaluation with swarm consensus
- **Dynamic Due Diligence** — Organism-seeded research loop with provenance, contradictions, and synthesis
- **Governance Truths** — Policy enforcement, access control, audit
- **Custom truths** — 7-step documented process for adding your own

## Desktop Visualization

Tauri + SvelteKit app showing governance in real time:

- Governance decision flow visualization
- Truth execution timeline
- Policy evaluation results
- Agent proposal/promotion lifecycle

## Server API

HTTP API for governance operations:

- Truth submission and execution
- Policy evaluation endpoints
- Fact query and timeline
- Agent registration

## Stack

| Layer | Technology |
|---|---|
| System | Rust (Edition 2024) |
| Governance | Cedar policy engine (from Converge) |
| LLM and search | `ChatBackend` plus provider adapters, with Kong optional for routing and MCP reuse |
| Desktop | Tauri 2 + SvelteKit 5 |
| Task runner | just |

## Canonical Student-Facing API Surface

These are the surfaces students should learn first:

| Role | Crate | Primary types |
|---|---|---|
| Converge authoring | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Converge runtime | `converge-kernel` | `Engine`, `Context`, `Budget`, criteria, run hooks |
| LLM and tool adapters | `converge-provider` | `ChatBackend`, chat backend selection, search, MCP/tool clients |
| Validation and offline tooling | `converge-axiom` | validators, truth parsing, `mock_llm::StaticChatBackend` |
| Organism authoring | `organism-pack` | `IntentPacket`, `Plan`, `PlanStep`, reasoning primitives |
| Organism runtime | `organism-runtime` | `Registry`, readiness, built-in packs |

Current note: the desktop scaffold is now on this contract. Keep new student examples on the same surfaces instead of introducing provider-specific APIs.

## What students can build

- Custom truths with domain-specific governance rules
- Cedar policies for new authority patterns
- Agents (Suggestors) that propose facts through the promotion gate
- Visualizations of convergence behavior
- Multi-agent scenarios with competing proposals

## What this project depends on

- **Converge** — engine, Cedar policy, capability adapters, validation tooling
- **Kong** — governed remote path for live external model and business access

## Getting started

```bash
just test          # Verify everything compiles
just server        # Start governance server
just dev-desktop   # Launch desktop app
just check         # Cargo check
just lint          # Format + clippy
```

See `kb/Development/Getting Started.md` for the full walkthrough.
