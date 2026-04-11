---
tags: [development, suggestors]
---
# Writing Suggestors

Every suggestor implements the `Suggestor` trait from `converge-core`.

## Suggestor Trait

```rust
impl Suggestor for YourSuggestor {
    fn name(&self) -> &str { "your-suggestor" }

    fn dependencies(&self) -> &[ContextKey] {
        &[]  // or &[ContextKey::Seeds] to wait for seed facts
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        true  // pure predicate, no I/O, no side effects
    }

    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        // Read context, do analysis, return proposals
        AgentEffect::with_proposal(ProposedFact {
            key: ContextKey::Seeds,
            id: "your:fact:id".into(),
            content: serde_json::json!({ "result": "data" }).to_string(),
            confidence: 0.85,
            provenance: "suggestor:your-suggestor".into(),
        })
    }
}
```

## Rules

- **`dependencies()`** declares which [[Converge/Context Keys|ContextKey]] partitions the suggestor watches. The engine only wakes the suggestor when those keys change.
- **`accepts()`** is a pure predicate. No I/O, no side effects.
- **`execute()`** reads the context and returns proposals. Never mutates the context directly.
- **Suggestors never call each other.** All communication through the shared context.
- **Check before proposing.** If a fact already exists, skip it. This gives idempotency.

## Suggestor Patterns

| Pattern | When to use |
|---|---|
| Rule-based | Policy checks, screening, deterministic logic |
| Analytics/scoring | Cost analysis, risk scoring, computation |
| LLM-backed | Capability matching, synthesis, natural language reasoning |
| Service-backed | External data from [[Integrations/Kong Gateway|Kong]] APIs or [[Integrations/MCP Tools|MCP]] |

## Complete Example: ComplianceScreenerAgent

```rust
struct ComplianceScreenerAgent {
    vendor_names: Vec<String>,
}

impl Suggestor for ComplianceScreenerAgent {
    fn name(&self) -> &str {
        "compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]  // runs first, no prerequisites
    }

    fn accepts(&self, _ctx: &dyn ContextView) -> bool {
        true
    }

    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for name in &self.vendor_names {
            let fact_id = format!("compliance:screen:{}", slug(name));
            // Idempotency: skip if already screened
            if ctx.get(ContextKey::Seeds).iter().any(|f| f.id == fact_id) {
                continue;
            }
            proposals.push(ProposedFact {
                key: ContextKey::Seeds,
                id: fact_id,
                content: serde_json::json!({
                    "vendor_name": name,
                    "gdpr_pass": true,
                    "ai_act_pass": true,
                }).to_string(),
                confidence: 0.85,
                provenance: "suggestor:compliance-screener".into(),
            });
        }
        AgentEffect::with_proposals(proposals)
    }
}
```

## Service-Backed Suggestor Example

Inject a trait so the suggestor works with real services or mocks. See [[Integrations/External Services]] for the full pattern.

```rust
struct ComplianceScreenerAgent {
    vendor_names: Vec<String>,
    policies: Arc<dyn PolicyService>,
}
```

## LLM-Backed Suggestor Example

```rust
struct SmartComplianceAgent {
    provider: Arc<dyn LlmProvider>,
}

impl Suggestor for SmartComplianceAgent {
    fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let prompt = format!("Evaluate GDPR compliance for vendor: {}", vendor_name);
        let response = self.provider.complete(&prompt);
        // Parse response into ProposedFact
    }
}
```

See also: [[Domain/Agents]], [[Architecture/Convergence Loop]], [[Converge/Building Blocks]]
