---
tags: [development, truths]
---
# Writing Truths

Step-by-step guide to adding a new truth.

## Steps

### 1. Define the truth

In `governance-truths/src/lib.rs`:

```rust
TruthDef {
    key: "assess-risk",
    display_name: "Assess Vendor Risk",
    summary: "Multi-agent risk assessment across operational and strategic dimensions",
    packs: &["risk-pack"],
    criteria: &[
        ("risk-scores-complete", "All vendors have risk scores"),
    ],
}
```

### 2. Create the executor

At `governance-server/src/truth_runtime/assess_risk.rs`.

### 3. Write agents

Implement the `Suggestor` trait from `converge-pack`. Use `converge-kernel` as the student-facing runtime import for `Engine`, `Context`, and budgets. See [[Development/Writing Suggestors]].

Not every truth needs an LLM or a large agent chain. The
`authorize-vendor-commitment` truth is intentionally narrow: one policy
suggestor calls `converge-policy` and emits a governed decision fact. That is a
good pattern for hard business gates.

When you do need a more dynamic loop, study
`governance-server/src/truth_runtime/dynamic_due_diligence.rs`. It shows a
student-safe pattern for:

- seeding typed research strategies from `organism-pack`
- refining those strategies through follow-up gaps
- making contradictions explicit instead of hiding them in a final summary
- returning a structured projection payload that the UI or API can render

### 4. Write criterion evaluator

```rust
impl CriterionEvaluator for AssessRiskEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "risk-scores-complete" => {
                // Check if all vendors have risk:score:* facts
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}
```

### 5. Wire it in the dispatcher

In `truth_runtime/mod.rs`:

```rust
"assess-risk" => assess_risk::execute(store, inputs, persist),
```

### 6. Add domain types

To `governance-kernel` if needed.

### 7. Test

```bash
just test
```

See also: [[Domain/Truths]], [[Domain/Key Types]]
