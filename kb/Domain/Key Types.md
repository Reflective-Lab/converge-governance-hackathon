---
tags: [domain, reference]
---
# Key Types

Types from the curated Converge surfaces and the governance crates.

## Converge Types

| Type | Purpose |
|---|---|
| `Engine` | Runs the [[Architecture/Convergence Loop|convergence loop]]. Registers agents, detects fixed point, enforces budgets. |
| `Context` | Shared state that all agents read. Partitioned by [[Converge/Context Keys|ContextKey]]. |
| `ContextKey` | Partition labels: Seeds, Hypotheses, Evaluations, Corrections, Metadata |
| `Fact` | Immutable piece of evidence in the context. Has id, key, content. |
| `ProposedFact` | What agents emit. Includes confidence and provenance. Gets promoted to Fact by the engine. |
| `AgentEffect` | Return value of `execute()`. Contains proposed facts. |
| `TypesRootIntent` | Declares objective, active packs, success criteria, budgets for a run. |
| `Budget` | Max cycles and max facts. |
| `Criterion` | A success condition checked after convergence. |
| `CriterionResult` | Met, Unmet, Blocked, or Indeterminate. |
| `StreamingCallback` | Real-time notifications: `on_cycle_start`, `on_fact`, `on_cycle_end`. |

## Governance Kernel Types

| Type | Purpose |
|---|---|
| `Vendor` | A vendor being evaluated |
| `PolicyRule` | A compliance policy to check against |
| `ComplianceCheck` | Result of screening a vendor against a policy |
| `RiskScore` | Weighted risk assessment for a vendor |
| `DecisionRecord` | The final recommendation |
| `AuditEntry` | Traceable record of who decided what, when, why |

See also: [[Converge/Building Blocks]], [[Architecture/Layers]]
