# Architecture

This repo is an application starter built on top of Converge. The architecture should be read in two layers:

- **Converge layer:** the shared runtime model for governed multi-agent execution
- **Hackathon layer:** the opinionated local-first desktop application that teams extend during the event

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

## How This Repo Depends On Converge

The repo is intentionally thin above Converge. It depends on Converge for the core execution semantics and adds challenge-specific domain modeling around it.

Converge owns:

- agent execution cycles
- shared context and context partitions
- fact proposal and promotion
- criteria evaluation
- convergence budgets and stop reasons

This repo owns:

- governance domain records such as vendors, decisions, and audit entries
- truth definitions for hackathon use cases
- projection from converged facts into domain records
- the shared application layer
- a lightweight local harness for developer testing

That split matters because hackathon teams should build with Converge, not around it. The value of the demo is governed convergence, not just "multiple calls to an LLM."

## Layers

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

Today the repo includes the Rust layers and a simple server harness. The Svelte/Tauri desktop layer is the intended product surface, and the app should call the Rust core locally rather than talking to a remote governance backend.

## Opinionated Implementation Guidance

The intended build style is:

- **Rust-first** for orchestration, domain logic, policy enforcement, integrations, and mocks
- **Svelte** for the user interface
- **Tauri** for desktop packaging and native shell integration
- **Kong** as the only intended remote integration layer for LLM traffic and business-service access

This keeps the interesting logic in one typed runtime while still letting teams build a polished demo surface.

## Local Input Model

The desktop app should be able to take one of two local input formats for vendor selection:

- Gherkin `.feature`
- truth-spec `.truths.json`

Those files are normalized in the Rust app layer before execution. That gives the frontend a simple job:

1. Let the operator open a local file.
2. Send its name and contents to the Rust core.
3. Show the parsed preview.
4. Execute `evaluate-vendor`.

The initial parsing support is already exposed from the shared app layer.

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

In this hackathon repo, an agent should usually follow one of these patterns:

- rule-based agent in Rust
- analytics or scoring agent in Rust
- LLM-backed agent that calls the model through Kong
- service-backed agent that gets business context through the Kong-facing APIs or MCP tools defined by the platform team

If a real enterprise service is missing, prefer mocking that service behind the same interface rather than embedding all data directly into the agent.

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
