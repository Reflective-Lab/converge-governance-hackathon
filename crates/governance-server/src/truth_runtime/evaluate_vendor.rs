//! Reference truth executor: evaluate-vendor
//!
//! Study this file, then build your own truth executor.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use converge_kernel::{Context, Engine, TypesRunHooks};
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{EvaluateVendorEvaluator, build_intent, find_truth};
use organism_pack::{IntentPacket, Plan, PlanStep, ReasoningSystem};
use organism_runtime::Registry;
use serde::Deserialize;
use uuid::Uuid;

use super::TruthExecutionResult;

#[derive(Debug, Clone)]
struct PlannedStrategy {
    id: &'static str,
    plan: Plan,
}

#[derive(Debug, Clone)]
struct VendorEvaluationPlanningSeed {
    _intent: IntentPacket,
    strategies: Vec<PlannedStrategy>,
    registry_pack_count: usize,
}

impl VendorEvaluationPlanningSeed {
    fn strategy_facts(&self) -> Vec<(&str, String)> {
        self.strategies
            .iter()
            .map(|strategy| {
                let content = strategy
                    .plan
                    .steps
                    .iter()
                    .map(|step| step.action.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                (strategy.id, content)
            })
            .collect()
    }
}

fn build_vendor_evaluation_planning_seed(vendor_names: &[String]) -> VendorEvaluationPlanningSeed {
    let registry = Registry::with_standard_packs();
    let vendor_list = vendor_names.join(", ");
    let intent = IntentPacket::new(
        format!("Evaluate AI vendors: {vendor_list}"),
        Utc::now() + Duration::hours(1),
    )
    .with_context(serde_json::json!({
        "vendors": vendor_names,
        "goal": "screen, compare, and recommend a vendor with explicit evidence",
    }))
    .with_authority(vec!["vendor_evaluation".to_string()]);

    let mut compliance = Plan::new(
        &intent,
        "Plan the compliance screening needed before any recommendation",
    );
    compliance.contributor = ReasoningSystem::DomainModel;
    compliance.steps = vec![PlanStep {
        action:
            "[planning] [compliance] screen vendors for regulation, residency, and certification"
                .into(),
        expected_effect: "Compliance evidence exists for every candidate vendor".into(),
    }];

    let mut cost = Plan::new(
        &intent,
        "Plan the operating-cost evaluation needed for budget fit",
    );
    cost.contributor = ReasoningSystem::CostEstimation;
    cost.steps = vec![PlanStep {
        action: "[planning] [cost] estimate budget fit and monthly operating cost for each vendor"
            .into(),
        expected_effect: "Comparable cost evidence exists for every vendor".into(),
    }];

    let mut risk = Plan::new(
        &intent,
        "Plan the risk comparison needed before a recommendation",
    );
    risk.contributor = ReasoningSystem::CausalAnalysis;
    risk.steps = vec![PlanStep {
        action: "[planning] [risk] compare lock-in, operational, and compliance risk".into(),
        expected_effect: "A risk profile exists for every vendor".into(),
    }];

    let mut synthesis = Plan::new(
        &intent,
        "Plan the final recommendation as an evidence-backed decision",
    );
    synthesis.contributor = ReasoningSystem::LlmReasoning;
    synthesis.steps = vec![PlanStep {
        action: "[planning] [decision] synthesize a recommendation from the governed evidence"
            .into(),
        expected_effect: "A final recommendation is proposed with explicit evidence".into(),
    }];

    VendorEvaluationPlanningSeed {
        _intent: intent,
        strategies: vec![
            PlannedStrategy {
                id: "strategy:vendor-eval:compliance",
                plan: compliance,
            },
            PlannedStrategy {
                id: "strategy:vendor-eval:cost",
                plan: cost,
            },
            PlannedStrategy {
                id: "strategy:vendor-eval:risk",
                plan: risk,
            },
            PlannedStrategy {
                id: "strategy:vendor-eval:decision",
                plan: synthesis,
            },
        ],
        registry_pack_count: registry.packs().len(),
    }
}

struct PlanningSeedSuggestor {
    planning: VendorEvaluationPlanningSeed,
}

#[async_trait]
impl Suggestor for PlanningSeedSuggestor {
    fn name(&self) -> &str {
        "planning-seed"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id.starts_with("strategy:vendor-eval:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let proposals = self
            .planning
            .strategy_facts()
            .into_iter()
            .map(|(id, content)| {
                ProposedFact::new(
                    ContextKey::Strategies,
                    id,
                    content,
                    "organism-planning:vendor-evaluation",
                )
                .with_confidence(1.0)
            })
            .collect();
        AgentEffect::with_proposals(proposals)
    }
}

// ---------------------------------------------------------------------------
// Agents
// ---------------------------------------------------------------------------

struct ComplianceScreenerAgent {
    vendor_names: Vec<String>,
}

#[async_trait]
impl Suggestor for ComplianceScreenerAgent {
    fn name(&self) -> &str {
        "compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "strategy:vendor-eval:compliance")
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
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
        AgentEffect { proposals }
    }
}

struct CostAnalysisAgent;

#[async_trait]
impl Suggestor for CostAnalysisAgent {
    fn name(&self) -> &str {
        "cost-analysis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds, ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        let has_plan = ctx
            .get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "strategy:vendor-eval:cost");
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id.starts_with("compliance:screen:"))
            && has_plan
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for fact in ctx.get(ContextKey::Seeds).iter() {
            if !fact.id.starts_with("compliance:screen:") {
                continue;
            }
            let vendor_slug = fact
                .id
                .strip_prefix("compliance:screen:")
                .unwrap_or("unknown");
            let cost_id = format!("cost:estimate:{vendor_slug}");
            if ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id == cost_id)
            {
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
        AgentEffect { proposals }
    }
}

struct VendorRiskAgent;

#[async_trait]
impl Suggestor for VendorRiskAgent {
    fn name(&self) -> &str {
        "vendor-risk"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Seeds,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        let has_plan = ctx
            .get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "strategy:vendor-eval:risk");
        let has_compliance = ctx
            .get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id.starts_with("compliance:screen:"));
        let has_costs = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id.starts_with("cost:estimate:"));
        let has_risks = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id.starts_with("risk:score:"));
        has_plan && has_compliance && has_costs && !has_risks
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for fact in ctx.get(ContextKey::Seeds).iter() {
            if !fact.id.starts_with("compliance:screen:") {
                continue;
            }
            let vendor_slug = fact
                .id
                .strip_prefix("compliance:screen:")
                .unwrap_or("unknown");
            let risk_id = format!("risk:score:{vendor_slug}");
            if ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id == risk_id)
            {
                continue;
            }
            // Placeholder: replace with real risk scoring via Kong/LLM
            proposals.push(ProposedFact {
                key: ContextKey::Evaluations,
                id: risk_id,
                content: serde_json::json!({
                    "vendor_slug": vendor_slug,
                    "lock_in_risk": "medium",
                    "compliance_risk": "low",
                    "operational_risk": "low",
                    "overall_risk": "low",
                })
                .to_string(),
                confidence: 0.70,
                provenance: "agent:vendor-risk".into(),
            });
        }
        AgentEffect { proposals }
    }
}

struct DecisionSynthesisAgent;

#[async_trait]
impl Suggestor for DecisionSynthesisAgent {
    fn name(&self) -> &str {
        "decision-synthesis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Seeds,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        let has_plan = ctx
            .get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id == "strategy:vendor-eval:decision");
        has_plan
            && ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id.starts_with("compliance:screen:"))
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("cost:estimate:"))
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("risk:score:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
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

pub async fn execute(
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
    let planning_seed = build_vendor_evaluation_planning_seed(&vendor_names);

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "planning-pack",
        PlanningSeedSuggestor {
            planning: planning_seed.clone(),
        },
    );
    engine.register_suggestor_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
    engine.register_suggestor_in_pack("risk-pack", VendorRiskAgent);
    engine.register_suggestor_in_pack("cost-pack", CostAnalysisAgent);
    engine.register_suggestor_in_pack("cost-pack", DecisionSynthesisAgent);

    let result = engine
        .run_with_types_intent_and_hooks(
            Context::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(EvaluateVendorEvaluator)),
                event_observer: None,
            },
        )
        .await
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
                                rationale: format!(
                                    "Organism-planned, multi-agent evaluation ({} packs available)",
                                    planning_seed.registry_pack_count
                                ),
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
            details: None,
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
        llm_calls: None,
    })
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn evaluate_vendor_end_to_end() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 1);
    }

    #[tokio::test]
    async fn no_persist_leaves_kernel_empty() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI".into())]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 0);
    }

    #[tokio::test]
    async fn missing_vendors_returns_error() {
        let store = InMemoryStore::new();
        assert!(execute(&store, &HashMap::new(), false).await.is_err());
    }
}
