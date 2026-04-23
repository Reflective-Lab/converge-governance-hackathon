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
│ axiom-truth + governance-truths boundary                  │
│ Truth contracts, validation, simulation, policy lens       │
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
| `governance-app` | **Hackathon app** — shared app layer for view models, truth execution, and queries |
| `governance-truths` | **Axiom-facing app boundary** — truth catalog, acceptance criteria, and Converge intent/evaluator wiring |
| `governance-kernel` | **Hackathon app** — local product model and writeback store: Vendor, PolicyRule, ComplianceCheck, RiskScore, DecisionRecord, AuditEntry |
| `governance-server` | A local harness for exercising Axiom contracts + Organism planning + Converge execution during development |
| `converge-*` crates | **Converge** — engine, context, promotion, policy, criteria, audit |
| provider and search adapters | **Providers** — external LLM, search, and tool connectivity |

## App Boundary

The Svelte frontend sits in Helm and calls local Rust commands through Tauri — not HTTP/REST/gRPC. The existing server is a *dev harness*, not the product surface.

Axiom is the truth-and-policy specification layer: it defines and validates what the decision must satisfy. Organism owns the strategy for forming the team to satisfy that contract. Converge owns the governed execution and promotion boundary. The hackathon app owns imported artifacts, demo data, and local writeback after Converge has promoted evidence.

`Engine.run()` belongs to Converge execution, not Axiom. Organism never bypasses the promotion gate, Axiom never promotes facts, and providers never leak into Helm directly.

See also: [[Architecture/Overview]], [[Architecture/Axiom Truth Contract]], [[Domain/Key Types]]
