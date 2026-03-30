# converge-governance-hackathon

## What This Is

A hackathon starter kit for building AI governance infrastructure using Converge. One team, 5-8 people, working on vendor selection as a multi-agent governance problem.

Converge owns the governance model: proposals become facts through promotion gates, agents converge on decisions, every fact has authority and traceability. This repo owns the business domain, the agents, and the UX.

## Build & Test

```bash
just test      # cargo test --workspace
just server    # cargo run -p governance-server (http://localhost:8080)
just fmt       # cargo fmt --all
just lint      # cargo clippy --workspace
```

## Architecture

```
governance-server (HTTP API)
  └── truth_runtime/
        └── evaluate_vendor.rs    ← THE REFERENCE — study this first
              ├── ComplianceScreenerAgent
              ├── CostAnalysisAgent
              └── DecisionSynthesisAgent

governance-app (shared layer for desktop)
  └── GovernanceApp — view models, truth execution, queries

governance-kernel (domain model + in-memory store)
  └── Vendor, PolicyRule, ComplianceCheck, RiskScore, DecisionRecord, AuditEntry

governance-truths (truth catalog + converge bindings)
  └── TruthDefinition, build_intent(), EvaluateVendorEvaluator
```

## How to Add a New Truth

1. Define it in `governance-truths/src/lib.rs` (key, packs, summary)
2. Create `governance-server/src/truth_runtime/your_truth.rs`
3. Write 2-4 agents implementing `converge_core::Agent`
4. Write a criterion evaluator implementing `CriterionEvaluator`
5. Wire it in `truth_runtime/mod.rs` dispatcher
6. Add domain types to `governance-kernel` if needed
7. `just test` green

## Rules

- No unsafe code
- Use typed enums, not strings with semantics
- Agents emit proposals, not direct facts — converge promotes them
- Every mutation needs an Actor
- `just lint` clean before considering work done
