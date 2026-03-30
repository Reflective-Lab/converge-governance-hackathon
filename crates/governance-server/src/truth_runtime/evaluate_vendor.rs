//! Reference truth executor: evaluate-vendor
//!
//! Study this file, then build your own truth executor.

use std::collections::HashMap;
use std::sync::Arc;

use converge_core::{
    Agent, AgentEffect, Context, ContextKey, Engine, ProposedFact, TypesRunHooks,
};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{EvaluateVendorEvaluator, build_intent, find_truth};
use serde::Deserialize;
use uuid::Uuid;

use super::TruthExecutionResult;

// ---------------------------------------------------------------------------
// Agents
// ---------------------------------------------------------------------------

struct ComplianceScreenerAgent {
    vendor_names: Vec<String>,
}

impl Agent for ComplianceScreenerAgent {
    fn name(&self) -> &str {
        "compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, _ctx: &dyn converge_core::ContextView) -> bool {
        true
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for name in &self.vendor_names {
            let fact_id = format!("compliance:screen:{}", slug(name));
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
                    "data_residency": "EU",
                })
                .to_string(),
                confidence: 0.85,
                provenance: "agent:compliance-screener".into(),
            });
        }
        AgentEffect {
            proposals,
            ..Default::default()
        }
    }
}

struct CostAnalysisAgent;

impl Agent for CostAnalysisAgent {
    fn name(&self) -> &str {
        "cost-analysis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id.starts_with("compliance:screen:"))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for fact in ctx.get(ContextKey::Seeds).iter() {
            if !fact.id.starts_with("compliance:screen:") {
                continue;
            }
            let vendor_slug = fact.id.strip_prefix("compliance:screen:").unwrap_or("unknown");
            let cost_id = format!("cost:estimate:{vendor_slug}");
            if ctx.get(ContextKey::Evaluations).iter().any(|f| f.id == cost_id) {
                continue;
            }
            proposals.push(ProposedFact {
                key: ContextKey::Evaluations,
                id: cost_id,
                content: serde_json::json!({
                    "vendor_slug": vendor_slug,
                    "monthly_cost_minor": 50_000,
                    "currency_code": "USD",
                })
                .to_string(),
                confidence: 0.75,
                provenance: "agent:cost-analysis".into(),
            });
        }
        AgentEffect {
            proposals,
            ..Default::default()
        }
    }
}

struct DecisionSynthesisAgent;

impl Agent for DecisionSynthesisAgent {
    fn name(&self) -> &str {
        "decision-synthesis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds, ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id.starts_with("compliance:screen:"))
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("cost:estimate:"))
    }

    fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
        // Replace with an LLM call via Kong in the real implementation
        AgentEffect::with_proposal(ProposedFact {
            key: ContextKey::Evaluations,
            id: "decision:recommendation".into(),
            content: serde_json::json!({
                "recommendation": "Vendor A recommended based on compliance and pricing",
                "confidence": 0.82,
                "needs_human_review": false,
            })
            .to_string(),
            confidence: 0.82,
            provenance: "agent:decision-synthesis".into(),
        })
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

pub fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("evaluate-vendor").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let vendor_names: Vec<String> = super::common::required_input(inputs, "vendors")?
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if vendor_names.is_empty() {
        return Err("at least one vendor name is required".into());
    }

    let mut engine = Engine::new();
    engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
    engine.register_in_pack("cost-pack", CostAnalysisAgent);
    engine.register_in_pack("cost-pack", DecisionSynthesisAgent);

    let result = engine
        .run_with_types_intent_and_hooks(
            Context::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(EvaluateVendorEvaluator)),
                event_observer: None,
            },
        )
        .map_err(|e| format!("convergence failed: {e}"))?;

    let projection = if persist {
        let write_result = store
            .write_with_events(|kernel| {
                if let Some(fact) = result
                    .context
                    .get(ContextKey::Evaluations)
                    .iter()
                    .find(|f| f.id == "decision:recommendation")
                {
                    #[derive(Deserialize)]
                    struct Payload {
                        recommendation: String,
                        confidence: f64,
                        needs_human_review: bool,
                    }
                    if let Ok(p) = serde_json::from_str::<Payload>(&fact.content) {
                        let actor = Actor::agent("decision-synthesis");
                        kernel.record_decision(
                            DecisionRecord {
                                id: Uuid::new_v4(),
                                truth_key: "evaluate-vendor".into(),
                                recommendation: p.recommendation,
                                confidence_bps: super::common::converge_confidence_to_bps(
                                    p.confidence,
                                ),
                                rationale: "Multi-agent evaluation".into(),
                                vendor_id: None,
                                needs_human_review: p.needs_human_review,
                                decided_by: actor.clone(),
                                decided_at: chrono::Utc::now(),
                            },
                            &actor,
                        );
                    }
                }
                Ok(())
            })
            .map_err(|e| format!("projection failed: {e}"))?;
        Some(super::TruthProjection {
            events_emitted: write_result.events.len(),
        })
    } else {
        None
    };

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
    })
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_vendor_end_to_end() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);
        let result = execute(&store, &inputs, true).unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 1);
    }

    #[test]
    fn no_persist_leaves_kernel_empty() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI".into())]);
        let result = execute(&store, &inputs, false).unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 0);
    }

    #[test]
    fn missing_vendors_returns_error() {
        let store = InMemoryStore::new();
        assert!(execute(&store, &HashMap::new(), false).is_err());
    }
}
