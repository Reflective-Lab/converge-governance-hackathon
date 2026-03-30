# Architecture

## The Converge Model

Converge is a correctness-first multi-agent runtime. Instead of letting agents do whatever they want, every action goes through a governance model:

```
Agent proposes a fact
  → Promotion gate validates it (authority, schema, confidence)
  → Fact is promoted into the shared context
  → Other agents read the updated context
  → Engine runs cycles until criteria are met or budget is exhausted
```

This means:
- **No agent can bypass governance.** Facts have private constructors — you can't create one without going through the promotion gate.
- **Every fact is traceable.** Who proposed it, when, with what confidence, and what authority.
- **Convergence is observable.** The criterion evaluator tells you exactly which success conditions were met or unmet.
- **Stopping is honest.** If the system can't converge, it tells you why (budget exhausted, criteria blocked, human intervention required).

## Layers

```
┌─────────────────────────────────────────┐
│           governance-server             │  HTTP API + truth executors
│                                         │  Routes truth keys to executors
├─────────────────────────────────────────┤
│           governance-app                │  Shared app layer (desktop + server)
│                                         │  View models, truth execution
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

## Truth Execution Pattern

Every truth executor follows the same pattern:

```rust
fn execute(store, inputs, persist) -> Result<TruthExecutionResult> {
    // 1. Load truth definition and build converge intent
    let truth = find_truth("your-truth-key")?;
    let intent = build_intent(truth);

    // 2. Create engine and register agents in packs
    let mut engine = Engine::new();
    engine.register_in_pack("your-pack", YourAgent { ... });

    // 3. Run convergence with criteria
    let result = engine.run_with_types_intent_and_hooks(
        context, &intent, hooks
    )?;

    // 4. Project converge facts into governance kernel
    if persist {
        store.write_with_events(|kernel| {
            // Read facts from result.context, write to kernel
        })?;
    }

    Ok(result)
}
```

## Writing an Agent

```rust
struct YourAgent;

impl Agent for YourAgent {
    fn name(&self) -> &str { "your-agent" }

    fn dependencies(&self) -> Vec<&str> {
        vec![]  // or vec!["other-agent"] if you need its facts first
    }

    fn accepts(&self, context: &dyn ContextView) -> bool {
        true  // return false to skip this cycle
    }

    fn execute(&self, context: &dyn ContextView) -> AgentEffect {
        // Read existing facts from context
        // Do your analysis (LLM call, computation, rule check)
        // Propose new facts

        AgentEffect::Propose(vec![
            ProposedFact::new(
                "your:fact:id",           // unique ID
                ContextKey::Derived,      // Seed or Derived
                payload_json,             // JSON string
                0.85,                     // confidence 0.0-1.0
                "agent:your-agent".into() // provenance
            )
        ])
    }
}
```

## Key Types

| Type | Purpose |
|---|---|
| `Engine` | Runs the convergence loop |
| `Context` | Shared state — agents read from it, propose to it |
| `ContextKey` | Partition: `Seed` (input), `Derived` (agent output), `Meta` |
| `Fact` | Promoted, immutable piece of evidence |
| `ProposedFact` | Agent's proposal — not yet a fact |
| `TypesRootIntent` | Configuration for a run: packs, budget, criteria |
| `Budget` | Max cycles and max facts |
| `CriterionEvaluator` | Checks if success conditions are met |
| `ConvergeResult` | Output: context, cycles, converged, stop_reason, criteria |
