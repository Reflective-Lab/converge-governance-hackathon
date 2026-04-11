---
tags: [architecture, converge]
---
# Convergence Loop

The [[Converge/Building Blocks|Engine]] runs a loop until a fixed point or budget exhaustion.

## Step-by-Step Example (Vendor Evaluation)

```
Cycle 1:
  ComplianceScreenerAgent has no dependencies → runs
  Proposes: compliance:screen:acme-ai, compliance:screen:beta-ml, compliance:screen:gamma-llm
  Engine promotes proposals to facts. Seeds key is now dirty.

Cycle 2:
  Seeds changed → CostAnalysisAgent wakes up
  Proposes: cost:estimate:acme-ai, cost:estimate:beta-ml, cost:estimate:gamma-llm
  Engine promotes. Evaluations key is now dirty.

Cycle 3:
  Evaluations changed → DecisionSynthesisAgent wakes up
  Proposes: decision:recommendation
  Engine promotes.

Cycle 4:
  No keys changed. Fixed point detected → convergence.

Post-convergence:
  Engine evaluates criteria:
    "all-vendors-screened" → Met
    "recommendation-produced" → Met

Result: converged=true, cycles=4
```

## Termination Guarantee

The engine guarantees termination through budgets. `TypesBudgets::with_cycles(10)` stops the run after 10 cycles if agents keep producing facts without converging.

## Fan-Out

Register multiple agents in the same pack to run them in parallel on the same cycle:

```rust
engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
engine.register_in_pack("compliance-pack", DataResidencyAgent { vendor_names });
engine.register_in_pack("compliance-pack", CertificationAgent { vendor_names });
```

All three share dependencies. In a single cycle, all run and propose facts into Seeds. The engine merges all effects deterministically (sorted by agent name).

## The Full Picture

```
vendor-selection.feature        (human-readable spec)
        │
        ▼
  Source parser                  (Gherkin → inputs map)
        │
        ▼
  TruthDef + build_intent()     (declare packs, criteria, budgets)
        │
        ▼
  Engine                         (register agents in packs)
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │  Convergence Loop                            │
  │                                              │
  │  Cycle 1: Screening agents run               │
  │    → propose facts into Seeds                │
  │                                              │
  │  Cycle 2: Analysis agents wake up            │
  │    → propose facts into Evaluations          │
  │                                              │
  │  Cycle 3: Synthesis agent wakes up           │
  │    → proposes recommendation                 │
  │                                              │
  │  Cycle 4: No changes → fixed point           │
  └─────────────────────────────────────────────┘
        │
        ▼
  Criteria evaluation            (Met / Unmet / Blocked)
        │
        ▼
  Domain projection              (persist decision to kernel)
        │
        ▼
  Audit trail                    (who decided what, when, why)
```

Every fact has provenance. Every decision has evidence. Every run has a stop reason. That is the governance story.

## How the Engine Decides What Runs

Each cycle, the engine:

1. Checks all registered agents
2. For each agent, calls `dependencies()` — which [[Converge/Context Keys|ContextKey]] partitions does it watch?
3. If any watched partition changed since last cycle, calls `accepts()` — pure predicate, no I/O
4. If `accepts()` returns true, calls `execute()` — agent reads context, returns proposals
5. Engine promotes all proposals through the governance gate
6. If no new facts were promoted, fixed point detected → convergence

Agents that have no dependencies run on cycle 1. Agents that depend on Seeds run when Seeds changes. This is how the cascade works without agents calling each other.

See also: [[Domain/Agents]], [[Converge/Context Keys]], [[Converge/Building Blocks]]
