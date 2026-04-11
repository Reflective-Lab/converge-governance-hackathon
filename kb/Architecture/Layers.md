---
tags: [architecture]
---
# Architecture Layers

```
┌─────────────────────────────────────────┐
│      Svelte + Tauri desktop app         │  Primary product surface
│                                         │  Local file picker, operator UX
├─────────────────────────────────────────┤
│           governance-app                │  Shared app layer (desktop + server)
│                                         │  Source preview, truth execution
├─────────────────────────────────────────┤
│           governance-server             │  Local harness only
│                                         │  Exercising runtime during dev
├─────────────────────────────────────────┤
│           governance-truths             │  Truth catalog + converge bindings
│                                         │  What jobs exist, what criteria
├─────────────────────────────────────────┤
│           governance-kernel             │  Domain model + in-memory store
│                                         │  Vendor, Decision, AuditEntry
├─────────────────────────────────────────┤
│           converge-core                 │  Engine, Agent, Fact, Context
│                                         │  Promotion gate, convergence loop
└─────────────────────────────────────────┘
```

## Crate Responsibilities

| Crate | Role |
|---|---|
| `governance-kernel` | Domain model + in-memory store: Vendor, PolicyRule, ComplianceCheck, RiskScore, DecisionRecord, AuditEntry |
| `governance-truths` | Truth catalog + converge bindings: TruthDefinition, `build_intent()`, EvaluateVendorEvaluator |
| `governance-server` | HTTP API + truth runtime. The reference executor lives at `truth_runtime/evaluate_vendor.rs` |
| `governance-app` | Shared layer for desktop: view models, truth execution, queries |

## App Boundary

The Svelte frontend calls local Rust commands through Tauri — not HTTP/REST/gRPC. The existing server is a *dev harness*, not the product surface.

See also: [[Architecture/Overview]], [[Domain/Key Types]]
