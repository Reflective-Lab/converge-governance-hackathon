---
tags: [architecture]
---
# Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Helm                                                       │
│ apps/desktop (Svelte + Tauri)                              │
│ Operator workflow, file picker, validation UI              │
├─────────────────────────────────────────────────────────────┤
│ Axiom                                                      │
│ governance-app + governance-truths + governance-kernel     │
│ Truth definitions, projections, validation, view models    │
├─────────────────────────────────────────────────────────────┤
│ Organism                                                   │
│ Intent → Huddle → Debate → Suggestors                      │
│ The reasoning path that produces proposals                 │
├─────────────────────────────────────────────────────────────┤
│ Converge                                                   │
│ Engine, context, promotion gate, criteria, audit           │
│ The governance path that decides what becomes fact         │
├─────────────────────────────────────────────────────────────┤
│ Providers                                                  │
│ ChatBackend, WebSearchBackend, MCP, DdLlm, DdSearch        │
│ External models, search, and service adapters              │
└─────────────────────────────────────────────────────────────┘
```

## Crate Responsibilities

| Crate | Role |
|---|---|
| `apps/desktop` | **Helm** — the operator-facing control surface |
| `governance-app` | **Axiom** — shared app layer for view models, truth execution, and queries |
| `governance-truths` | **Axiom** — truth catalog + Converge bindings: `TruthDefinition`, `build_intent()`, evaluator wiring |
| `governance-kernel` | **Axiom** — domain model + in-memory store: Vendor, PolicyRule, ComplianceCheck, RiskScore, DecisionRecord, AuditEntry |
| `governance-server` | A local harness for exercising Axiom + Converge during development |
| `converge-*` crates | **Converge** — engine, context, promotion, policy, criteria, audit |
| provider and search adapters | **Providers** — external LLM, search, and tool connectivity |

## App Boundary

The Svelte frontend sits in Helm and calls local Rust commands in Axiom through Tauri — not HTTP/REST/gRPC. The existing server is a *dev harness*, not the product surface.

`Engine.run()` starts from Axiom, activates Organism reasoning, and commits only through Converge. Organism never bypasses the promotion gate, and providers never leak into Helm directly.

See also: [[Architecture/Overview]], [[Domain/Key Types]]
