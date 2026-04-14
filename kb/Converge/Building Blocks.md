---
tags: [converge, reference]
---
# Building Blocks

Types and traits from `converge-core` that you use, not implement.

| Type | What it does |
|------|-------------|
| `Engine` | Runs the [[Architecture/Convergence Loop|convergence loop]]. Registers agents, detects fixed point, enforces budgets. |
| `Context` | Shared state all agents read. Partitioned by [[Converge/Context Keys|ContextKey]]. |
| `ContextKey` | Partition labels: Seeds, Hypotheses, Evaluations, Corrections, Metadata. |
| `Fact` | Immutable evidence in the context. Has `id`, `key`, `content`. |
| `ProposedFact` | What suggestors emit. Includes `confidence` and `provenance`. Promoted to Fact by the engine. |
| `AgentEffect` | Return value of `execute()`. Contains proposed facts. |
| `TypesRootIntent` | Declares objective, active packs, success criteria, budgets. |
| `Criterion` | A success condition. Engine checks these after convergence. |
| `CriterionResult` | `Met`, `Unmet`, `Blocked`, or `Indeterminate`. |
| `StreamingCallback` | Real-time notifications: `on_cycle_start`, `on_fact`, `on_cycle_end`. |
| `Budget` | Max cycles and max facts. Guarantees termination. |
| `HitlPolicy` | [[Converge/HITL Gates|Human-in-the-loop]] gate configuration. |

## Suggestor Trait

```rust
trait Suggestor: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[ContextKey];
    fn accepts(&self, ctx: &dyn ContextView) -> bool;
    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect;
}
```

See also: [[Domain/Key Types]], [[Development/Writing Suggestors]]
