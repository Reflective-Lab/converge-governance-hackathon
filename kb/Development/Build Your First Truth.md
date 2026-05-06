---
tags: [development, truths, tutorial]
---
# Build Your First Truth

A hands-on walkthrough of the truth lifecycle: define, implement, test, wire, and execute. By the end, you'll have a working truth that proposes and evaluates facts through the Converge governance loop.

## Why Build a Truth?

A **truth** is a named governance decision process with typed agents (suggestors) and typed guardrails (criterion evaluators). When you define a truth, you're specifying:

- What problem it solves (the intent)
- How agents collaborate to solve it (suggestors in packs)
- How we know it's solved (criteria)
- How we store the decision (projection)

Truths let you *compose* governance: a vendor recommendation truth calls policy truths, which call compliance truths, all in a single convergent loop. Each truth is independently testable, reusable, and auditable.

## The Example: Client Happiness Check

We'll build a minimal truth to demonstrate the full lifecycle. It has:

- **One planning suggestor** that seeds a client check strategy
- **One evaluation suggestor** that proposes a happiness score
- **One criterion evaluator** that checks if the score exists
- **One projected decision** written to the kernel

This is simpler than `evaluate-vendor` but shows every required piece.

---

# Step 1: Define the Truth

Open `crates/governance-truths/src/lib.rs` and find the `TRUTHS` array. Add your truth definition **before the closing bracket**:

```rust
TruthDef {
    key: "client-happiness-check",
    display_name: "Client Happiness Check",
    summary: "Simple happiness survey: seed a plan, gather feedback, produce a happiness score",
    packs: &["planning-pack", "feedback-pack"],
    criteria: &[
        ("happiness-score-produced", "A happiness score fact exists"),
    ],
},
```

**What each field means:**

- `key`: The unique identifier for your truth (used in APIs)
- `display_name`: Human-readable name (shown in UIs)
- `summary`: One-line description of the governance flow
- `packs`: Agent packs that will be registered in the executor (you'll register suggestors in these)
- `criteria`: Ranked conditions that must be met for convergence (criterion_id, description)

Now add the criterion evaluator **after the `TRUTHS` array** (search for `impl CriterionEvaluator`):

```rust
pub struct ClientHappinessCheckEvaluator;

impl CriterionEvaluator for ClientHappinessCheckEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &dyn Context) -> CriterionResult {
        match criterion.id.as_str() {
            "happiness-score-produced" => {
                // Check if a happiness:score:* fact exists in the context
                if context
                    .get(ContextKey::Evaluations)
                    .iter()
                    .any(|f| f.id == "happiness:score:final")
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no happiness score produced yet".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}
```

**Why this pattern:**

- The evaluator checks if a *fact already exists* in the context under a specific key (`ContextKey::Evaluations`)
- `CriterionResult::Met` means the criterion is satisfied (the truth can converge)
- `CriterionResult::Unmet` with a reason tells the engine to keep trying
- `Indeterminate` means this evaluator doesn't handle this criterion (another evaluator might)

---

# Step 2: Create the Executor Module

Create a new file: `crates/governance-server/src/truth_runtime/client_happiness_check.rs`

Start with this template:

```rust
//! Truth executor: client-happiness-check
//!
//! Demonstrates a minimal end-to-end truth:
//! - One planning suggestor seeds a client survey
//! - One feedback suggestor evaluates happiness
//! - Criterion checks for the happiness score fact
//! - Result is projected to the kernel as a decision

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use converge_kernel::{ContextState, Engine, TypesRunHooks};
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{ClientHappinessCheckEvaluator, build_intent, find_truth};
use serde::Deserialize;
use uuid::Uuid;

use super::TruthExecutionResult;

// Marker types for organization (optional, but helpful for large executors)
struct PlanningPhase;
struct EvaluationPhase;

// --- STEP 2A: Planning Suggestor ---
// This suggestor seeds the convergence loop with a plan fact.
// It runs first (no dependencies) and proposes a single strategy.

struct PlanningStrategySeeder {
    client_name: String,
}

#[async_trait]
impl Suggestor for PlanningStrategySeeder {
    // Name: used in logs and telemetry
    fn name(&self) -> &str {
        "happiness-planning-seed"
    }

    // Dependencies: which context keys does this suggestor watch?
    // Empty means it runs immediately (no prerequisites).
    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    // Accepts: pure predicate (no I/O, no mutations)
    // Return true if this suggestor should fire this cycle.
    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Only run if no strategy facts exist yet (ensures idempotency)
        !ctx.get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "happiness:strategy:survey")
    }

    // Execute: read the context and return proposals
    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        // Propose a single strategy fact
        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Strategies,
                "happiness:strategy:survey",
                format!(
                    "Run a happiness survey for {} across product experience, support, and pricing",
                    self.client_name
                ),
                "happiness-planning-seed",
            )
            .with_confidence(1.0), // We're confident in the plan
        )
    }
}

// --- STEP 2B: Evaluation Suggestor ---
// This suggestor waits for the strategy to exist, then proposes a happiness score.
// In a real system, this would call an LLM or integrate with a survey service.

struct HappinessScorerAgent {
    client_name: String,
}

#[async_trait]
impl Suggestor for HappinessScorerAgent {
    fn name(&self) -> &str {
        "happiness-scorer"
    }

    // This suggestor depends on Strategies: it waits for the planning phase to complete
    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Fire only if:
        // 1. The survey strategy exists
        // 2. No happiness score has been produced yet (idempotency)
        let has_strategy = ctx
            .get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "happiness:strategy:survey");
        let score_missing = !ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|fact| fact.id == "happiness:score:final");
        has_strategy && score_missing
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        // In a real system, parse survey results or call an LLM here.
        // For this tutorial, we return a mock score.
        let happiness_data = serde_json::json!({
            "client_name": &self.client_name,
            "nps_score": 42,
            "sentiment": "neutral",
            "top_feedback": ["good support", "pricing could be better"],
            "confidence": 0.78,
        });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "happiness:score:final",
                happiness_data.to_string(),
                "happiness-scorer",
            )
            .with_confidence(0.78),
        )
    }
}

// --- STEP 2C: Main Executor Function ---
// This is the entry point called by the dispatcher.
// It sets up the engine, registers suggestors, and runs convergence.

pub async fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    // 1. Look up the truth definition (defined in Step 1)
    let truth = find_truth("client-happiness-check").ok_or("truth not found")?;

    // 2. Build the intent from the truth definition
    // (This creates a TypesRootIntent with success criteria, budgets, etc.)
    let intent = build_intent(truth);

    // 3. Extract input: client name (required)
    let client_name = super::common::required_input(inputs, "client_name")?
        .to_string();

    // 4. Create and configure the convergence engine
    let mut engine = Engine::new();

    // 5. Register suggestors in their packs
    // Pack names must match the `packs` array in your truth definition
    engine.register_suggestor_in_pack(
        "planning-pack",
        PlanningStrategySeeder {
            client_name: client_name.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "feedback-pack",
        HappinessScorerAgent {
            client_name: client_name.clone(),
        },
    );

    // 6. Run convergence with the evaluator
    // TypesRunHooks wires in the criterion evaluator (defined in Step 1)
    let result = engine
        .run_with_types_intent_and_hooks(
            ContextState::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(ClientHappinessCheckEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|e| format!("convergence failed: {e}"))?;

    // 7. Optionally, persist the decision to the kernel
    let projection = if persist {
        let write_result = store
            .write_with_events(|kernel| {
                // Extract the happiness score from the converged context
                if let Some(fact) = result
                    .context
                    .get(ContextKey::Evaluations)
                    .iter()
                    .find(|f| f.id == "happiness:score:final")
                {
                    #[derive(Deserialize)]
                    struct HappinessPayload {
                        nps_score: i32,
                        sentiment: String,
                        confidence: f64,
                    }

                    if let Ok(payload) = serde_json::from_str::<HappinessPayload>(&fact.content) {
                        let actor = Actor::agent("happiness-scorer");
                        // Write a decision record to the kernel
                        // (This is audit-logged and queryable later)
                        kernel.record_decision(
                            DecisionRecord {
                                id: Uuid::new_v4(),
                                truth_key: "client-happiness-check".into(),
                                recommendation: format!(
                                    "Client reported NPS {} ({})",
                                    payload.nps_score, payload.sentiment
                                ),
                                confidence_bps: super::common::converge_confidence_to_bps(
                                    payload.confidence,
                                ),
                                rationale: format!(
                                    "Happiness check converged in {} cycles",
                                    result.cycles
                                ),
                                vendor_id: None,
                                needs_human_review: false,
                                decided_by: actor,
                                decided_at: chrono::Utc::now(),
                            },
                            &Actor::agent("happiness-scorer"),
                        );
                    }
                }
                Ok(())
            })
            .map_err(|e| format!("projection failed: {e}"))?;
        Some(super::TruthProjection {
            events_emitted: write_result.events.len(),
            details: None,
        })
    } else {
        None
    };

    // 8. Return the execution result
    // (converged, cycles, criteria outcomes, projection)
    Ok(TruthExecutionResult {
        converged: result.converged,
        cycles: result.cycles,
        stop_reason: format!("{:?}", result.stop_reason),
        criteria_outcomes: result
            .criteria_outcomes
            .iter()
            .map(|o| super::CriterionOutcomeView {
                criterion: o.criterion.description.clone(),
                result: format!("{:?}", o.result),
            })
            .collect(),
        projection,
        llm_calls: None,
    })
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn happiness_check_converges() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("client_name".into(), "Acme Corp".into())]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn missing_client_name_returns_error() {
        let store = InMemoryStore::new();
        let result = execute(&store, &HashMap::new(), false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn happiness_score_projected_to_kernel() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("client_name".into(), "Test Client".into())]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        let decision_count = store.read(|k| k.decisions.len()).unwrap();
        assert_eq!(decision_count, 1);
    }
}
```

**Key concepts in this code:**

1. **`Suggestor` trait:** Implement `name()`, `dependencies()`, `accepts()`, and `async execute()`
2. **`accepts()` predicate:** Check the context to see if the suggestor should run this cycle
3. **Idempotency:** Always check if a fact already exists before proposing it (prevents infinite loops)
4. **`ContextKey` partitions:** Seeds, Strategies, Evaluations, Proposals, Hypotheses—each is a typed namespace
5. **`ProposedFact`:** The data structure suggestors use to propose new facts
6. **`TypesRunHooks`:** Wires in your criterion evaluator so the engine can check convergence
7. **Projection:** The optional write to `InMemoryStore`, which persists decisions as audit-logged records

---

# Step 3: Wire the Executor in the Dispatcher

Open `crates/governance-server/src/truth_runtime/mod.rs`.

Add a public module declaration at the top:

```rust
pub mod client_happiness_check;
```

Find the `execute_truth` function and add your truth **inside the match statement**:

```rust
pub async fn execute_truth(
    store: &InMemoryStore,
    truth_key: &str,
    inputs: HashMap<String, String>,
    persist: bool,
    experience: &crate::experience::ExperienceRegistry,
) -> Result<TruthExecutionResult, String> {
    match truth_key {
        // ... existing truths ...
        "client-happiness-check" => client_happiness_check::execute(store, &inputs, persist).await,
        // ... existing fallthrough ...
        _ => Err(format!("no executor for truth: {truth_key}")),
    }
}
```

This registration tells the API dispatcher: "when someone calls `execute_truth(..., "client-happiness-check", ...)`, use the `client_happiness_check::execute` function."

---

# Step 4: Run Tests

```bash
just test
```

You should see output like:

```
running 3 tests
test truth_runtime::client_happiness_check::tests::happiness_check_converges ... ok
test truth_runtime::client_happiness_check::tests::missing_client_name_returns_error ... ok
test truth_runtime::client_happiness_check::tests::happiness_score_projected_to_kernel ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

If tests fail:

- **"truth not found"**: Check that your truth definition in `governance-truths/src/lib.rs` uses the correct `key`
- **"criterion_evaluator: Some(...)"**: Did you export `ClientHappinessCheckEvaluator` from `governance-truths/src/lib.rs`?
- **Compilation errors**: Make sure imports match the crates being used in other executors (especially `converge_pack`, `governance_kernel`, `async_trait`)

---

# Step 5: Call the API Endpoint

Once tests pass, you can execute your truth via the HTTP API.

```bash
curl -X POST http://localhost:3001/truth/execute \
  -H "Content-Type: application/json" \
  -d '{
    "truth_key": "client-happiness-check",
    "inputs": {
      "client_name": "Acme Corp"
    },
    "persist": true
  }'
```

**Expected response:**

```json
{
  "converged": true,
  "cycles": 2,
  "stop_reason": "Converged",
  "criteria_outcomes": [
    {
      "criterion": "A happiness score fact exists",
      "result": "Met { evidence: [] }"
    }
  ],
  "projection": {
    "events_emitted": 1,
    "details": null
  }
}
```

**What this means:**

- `converged: true` — All criteria were met
- `cycles: 2` — It took 2 loops (planning → evaluation)
- `criteria_outcomes` — The criterion you defined in Step 1 was `Met`
- `events_emitted: 1` — One decision record was written to the kernel

---

# Troubleshooting

## 1. "truth not found" error

**Cause:** The `find_truth("client-happiness-check")` call in your executor can't find the truth definition.

**Fix:** Double-check the `key` field in your `TruthDef` in `governance-truths/src/lib.rs`. It must match exactly what you pass to `find_truth()`.

```rust
// In lib.rs
TruthDef {
    key: "client-happiness-check",  // <-- must match
    // ...
}

// In your executor
let truth = find_truth("client-happiness-check").ok_or("...")?;  // <-- must match
```

## 2. "no executor for truth" error

**Cause:** The dispatcher in `truth_runtime/mod.rs` doesn't have a match arm for your truth key.

**Fix:** Add the registration:

```rust
"client-happiness-check" => client_happiness_check::execute(store, &inputs, persist).await,
```

## 3. Suggestor runs forever / doesn't converge

**Cause:** The `accepts()` predicate is always returning `true`, or the proposed fact ID is different each cycle (breaking idempotency).

**Fix:** Check idempotency in `accepts()`:

```rust
// WRONG: always returns true
fn accepts(&self, _ctx: &dyn ContextView) -> bool {
    true
}

// RIGHT: returns false once the fact exists
fn accepts(&self, ctx: &dyn ContextView) -> bool {
    !ctx.get(ContextKey::Evaluations)
        .iter()
        .any(|f| f.id == "happiness:score:final")
}
```

And make sure fact IDs are deterministic:

```rust
// WRONG: fact ID changes each cycle
let fact_id = format!("score:{}", rand::random::<u32>());

// RIGHT: fact ID is stable
let fact_id = "happiness:score:final".to_string();
```

## 4. Compilation error: "expected module, found struct"

**Cause:** You declared `pub mod client_happiness_check;` but the file is named `client_happiness_check.rs` in the wrong directory.

**Fix:** Ensure the file exists at:
```
crates/governance-server/src/truth_runtime/client_happiness_check.rs
```

## 5. "missing required input: client_name"

**Cause:** The executor calls `super::common::required_input(inputs, "client_name")` but the API call didn't provide it.

**Fix:** When calling the endpoint, include the input:

```json
{
  "inputs": {
    "client_name": "Your Client Name"
  }
}
```

## 6. Criterion evaluator returns `Indeterminate` (truth never converges)

**Cause:** The `CriterionEvaluator` implementation doesn't match the criterion ID in your truth definition.

**Fix:** Make sure the `match criterion.id.as_str()` arms match your criteria:

```rust
// In lib.rs
criteria: &[
    ("happiness-score-produced", "..."),  // <-- this ID
]

// In lib.rs evaluator
match criterion.id.as_str() {
    "happiness-score-produced" => {  // <-- must match
        // ...
    }
}
```

---

# Next Steps

Now that you've built a minimal truth, you're ready for more advanced patterns:

- **[[Development/Modifying Evaluate Vendor]]** — Add a new agent to the existing vendor-evaluation truth
- **[[Development/Writing Suggestors]]** — Build LLM-backed agents and service-integrated agents
- **[[Development/Cedar Policies for Participants]]** — Add policy gates to your truth for hard business rules
- **[[Domain/Truths]]** — Understand the theory behind truth definitions and convergence
- **Study the reference implementations:**
  - `governance-server/src/truth_runtime/evaluate_vendor.rs` — Multi-agent vendor screening
  - `governance-server/src/truth_runtime/dynamic_due_diligence.rs` — Dynamic research loop with contradictions
  - `governance-server/src/truth_runtime/vendor_selection_simple.rs` — Five-evaluator synthesis

**Key takeaway:** A truth is three components working together:

1. **Definition** (in `governance-truths/lib.rs`): The problem statement, packs, and success criteria
2. **Executor** (in `truth_runtime/`): Suggestors that propose facts
3. **Evaluator** (in `governance-truths/lib.rs`): Logic that checks if criteria are met

Master this pattern, and you can build any governed decision process.
