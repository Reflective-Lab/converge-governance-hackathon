---
tags: [development, api]
---
# Programming API Surfaces

This page defines the student-facing Rust surface for this repo. The goal is familiarity: a developer should feel the same shape when moving between Converge, Organism, and hackathon code.

## Canonical Split

Use these crates by role:

| Role | Crate | What students should import first |
|---|---|---|
| Converge authoring | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Converge runtime | `converge-kernel` | `Engine`, `Context`, `Budget`, criteria, run hooks |
| Converge model access | `converge-provider` | capability adapters that satisfy `ChatBackend`, search, tools |
| Converge tooling | `converge-tool` | validators, truth parsing, `mock_llm::StaticChatBackend` |
| Organism authoring | `organism-pack` | `IntentPacket`, `Plan`, `PlanStep`, reasoning primitives |
| Organism runtime | `organism-runtime` | `Registry`, readiness, built-in packs |

`converge-core` still matters, but treat it as the constitutional layer and internal re-export surface, not the first import path students learn from examples.

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
use converge_core::traits::{ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat};
```

Kong is one useful operational remote path for the hackathon, but it is not required right now. Students should not have to learn one API for Converge and another API for Kong-routed Converge. Long term, a `KongProvider` or more general `RouterProvider` should sit under the same capability contract.

Do not present these as the default student-facing surface in new docs or examples:

- `KongGateway`
- `KongRoute`
- `LlmProvider`
- `LlmRequest`

If infrastructure routing changes, the application-facing contract should stay on `ChatBackend`, `WebSearchBackend`, and the same MCP client surface.

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

- This hackathon repo still imports `converge-core` directly in some runtime code. That is acceptable internally, but student-facing examples should prefer `converge-pack` and `converge-kernel`.
- `apps/desktop/src-tauri` now uses `ChatBackend` selection plus `StaticChatBackend`; keep future student examples on that same contract instead of reintroducing gateway-specific APIs.
- `../monterro/crates/monterro-core/src/due_diligence.rs` is useful as a prototype, but its direct provider HTTP calls are not the template we want students copying.
- `../monterro/crates/monterro-core/src/convergent_dd.rs` is closer to the target shape because it uses curated Converge and Organism surfaces, but the hackathon repo should keep its student-facing examples even smaller and more explicit.

When in doubt, optimize for the developer who will move between `converge`, `organism`, and this repo in the same day.

See also: [[Development/Getting Started]], [[Development/Writing Suggestors]], [[Integrations/Kong Gateway]]
