---
tags: [development, api]
---
# Programming API Surfaces

This page defines the participant-facing Rust surface for this repo. The goal is familiarity: a developer should feel the same shape when moving between Converge, Organism, and hackathon code.

## Canonical Split

Use these crates by role:

| Role | Crate | What participants should import first |
|---|---|---|
| Converge authoring | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Converge runtime | `converge-kernel` | `Engine`, `Context`, `Budget`, criteria, run hooks |
| Converge capability contracts | `converge-provider-api` | `ChatBackend`, `DynChatBackend`, `ChatRequest`, `ChatResponse`, `SelectionCriteria` |
| Converge ready-made adapters | `converge-provider` | capability adapters that satisfy those contracts, search, tools |
| Axiom truth contract | `axiom-truth` | truth validation, Gherkin parsing, policy lens |
| Organism authoring | `organism-pack` | `IntentPacket`, `Plan`, `PlanStep`, reasoning primitives |
| Organism runtime | `organism-runtime` | `Registry`, readiness, built-in packs |

`converge-core` still matters, but treat it as the constitutional layer and internal re-export surface, not the first import path participants learn from examples.

## Current Foundation Baseline

The participant-facing repo is pinned to these foundation tags:

- Converge `v3.7.3` for runtime, formation substrate, provider contracts, policy, and domain packs.
- Organism `v1.4.0` for intent, planning, collaboration, simulation, and learning examples.
- Axiom `v0.7.0` for truth validation, Gherkin parsing, and policy lens work.

Do not switch this repo to sibling path dependencies for participant work. The hackathon template must remain cloneable without `../converge`, `../organism`, or `../axiom` checkouts.

Foundation dependency chain: Organism `v1.4.0` and Axiom `v0.7.0` are aligned to Converge `v3.7.3`, so participant code should see one Converge governance/provider contract across the user-side stack.

## Axiom Contract

Axiom is the executable specification layer for governed decisions. Participant-facing Axiom code should define and validate truth contracts: business intent, policies, examples, invariants, acceptance criteria, policy lens, and admissibility checks.

Axiom code should not:

- run convergence
- compile formations
- promote facts
- learn priors
- store the authoritative business decision

The handoff is: Axiom defines what must be true, Organism decides how to form the team to satisfy it, Converge governs execution and promotion, and the hackathon app owns product writeback.

## Converge Authoring Contract

New suggestor examples should start like this:

```rust
use converge_pack::{AgentEffect, Context, ContextKey, ProposedFact, Suggestor};
```

The authoring mental model is:

1. Read context
2. Propose facts
3. Let the engine promote them

Do not present students with hidden side channels, direct mutation, or agent-to-agent calls.

## Converge Runtime Contract

Embedded applications should present the runtime through `converge-kernel`:

```rust
use converge_kernel::{Budget, Context, Engine};
```

That keeps the execution story stable:

- packs author behavior
- kernel runs it
- domain crates provide reusable suggestor packs

## LLM Contract

The stable programming boundary for model access is `ChatBackend` plus `ChatRequest` / `ChatResponse`.

Application code should look like this shape:

```rust
use converge_provider_api::{ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat};
```

Kong is one useful operational remote path for the hackathon, but it is not required right now. Participants should not have to learn one API for Converge and another API for Kong-routed Converge. Long term, a `KongProvider` or more general `RouterProvider` should sit under the same capability contract.

Do not present these as the default participant-facing surface in new docs or examples:

- `KongGateway`
- `KongRoute`
- `LlmProvider`
- `LlmRequest`

If infrastructure routing changes, the application-facing contract should stay on `converge-provider-api` chat types, `WebSearchBackend`, and the same MCP client surface.

## Organism Contract

Organism should feel adjacent, not alien:

```rust
use organism_pack::{IntentPacket, Plan, PlanStep, ReasoningSystem};
use organism_runtime::Registry;
```

The intended handoff is:

1. Organism structures intent and planning proposals
2. The application loads built-in packs from `Registry::with_standard_packs()` when it needs standard Organism behavior
3. Converge governs promotion, evaluation, and commitment

That means Organism examples should stay typed and proposal-oriented, just like Converge examples do.

## Current Gaps

There are still places in this repo family that do not match the target surface yet:

- This hackathon repo still imports `converge-core` directly in some runtime code. That is acceptable internally, but participant-facing examples should prefer `converge-pack`, `converge-kernel`, and `converge-provider-api`.
- `apps/desktop/src-tauri` now uses `ChatBackend` selection plus offline fallback behavior; keep future participant examples on that same contract instead of reintroducing gateway-specific APIs.
- `../monterro/crates/monterro-core/src/due_diligence.rs` is useful as a prototype, but its direct provider HTTP calls are not the template we want participants copying.
- `../monterro/crates/monterro-core/src/convergent_dd.rs` is closer to the target shape because it uses curated Converge and Organism surfaces, but the hackathon repo should keep its participant-facing examples even smaller and more explicit.

When in doubt, optimize for the developer who will move between `converge`, `organism`, and this repo in the same day.

See also: [[Development/Getting Started]], [[Development/Writing Suggestors]], [[Integrations/Kong Gateway]]
