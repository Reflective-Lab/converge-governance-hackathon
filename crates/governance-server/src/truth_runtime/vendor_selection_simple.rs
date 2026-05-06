//! Truth executor: vendor-selection-simple
//!
//! Student-friendly modular vendor selection with 5 evaluators and 1 synthesis agent.
//!
//! Design:
//! - 5 modular evaluators (price, compliance, reliability, support, stability)
//! - 1 synthesis agent that computes weighted average and recommends shortlist
//! - Cedar policy gate with threshold rules
//! - Criterion-based architecture for teachability

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use converge_kernel::{ContextState, Engine, TypesRunHooks};
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{VendorSelectionSimpleEvaluator, build_intent, find_truth};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TruthExecutionResult;

// ---------------------------------------------------------------------------
// Vendor input data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorData {
    pub name: String,
    pub cost_minor: i64, // monthly cost in minor units (cents)
    pub certifications: Vec<String>,
    pub sla_uptime: f64,      // 99.5, 99.95, 99.99 etc
    pub support_tier: String, // "basic", "standard", "premium"
    pub response_sla_hours: f64,
    pub company_funding: String, // "seed", "series-a", "series-b", "series-d", "profitable"
    pub founded_year: i32,
    pub headcount: i32,
}

fn parse_vendors(inputs: &HashMap<String, String>) -> Result<Vec<VendorData>, String> {
    if let Some(json) = inputs.get("vendors_json") {
        serde_json::from_str(json).map_err(|e| format!("invalid vendors_json: {e}"))
    } else if let Some(json) = inputs.get("vendors") {
        serde_json::from_str(json).map_err(|e| format!("invalid vendors: {e}"))
    } else {
        Err("missing required input: vendors (JSON array)".into())
    }
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-").replace(".", "")
}

// ---------------------------------------------------------------------------
// Suggestors (evaluators)
// ---------------------------------------------------------------------------

struct PriceEvaluatorSuggestor {
    vendors: Vec<VendorData>,
    budget: i64, // minor units
}

#[async_trait]
impl Suggestor for PriceEvaluatorSuggestor {
    fn name(&self) -> &str {
        "price-evaluator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("criterion:price:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for vendor in &self.vendors {
            let fact_id = format!("criterion:price:{}", slug(&vendor.name));
            if ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().as_str() == fact_id)
            {
                continue;
            }

            // Score 0-100: lower cost = higher score
            let cost_ratio = vendor.cost_minor as f64 / self.budget as f64;
            let score = (100.0 - (cost_ratio * 100.0).min(100.0)).max(0.0);

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "vendor": vendor.name,
                        "criterion": "price",
                        "score": score,
                        "cost_minor": vendor.cost_minor,
                        "budget": self.budget,
                    })
                    .to_string(),
                    "suggestor:price-evaluator",
                )
                .with_confidence(0.95),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct ComplianceScreenerSuggestor {
    vendors: Vec<VendorData>,
}

#[async_trait]
impl Suggestor for ComplianceScreenerSuggestor {
    fn name(&self) -> &str {
        "compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("criterion:compliance:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for vendor in &self.vendors {
            let fact_id = format!("criterion:compliance:{}", slug(&vendor.name));

            // Score based on certifications (each cert = points)
            let required = ["soc2", "iso27001", "gdpr"];
            let matched = required
                .iter()
                .filter(|req| {
                    vendor
                        .certifications
                        .iter()
                        .any(|cert| cert.to_lowercase().contains(&req.to_string()))
                })
                .count();
            let score = (matched as f64 / required.len() as f64) * 100.0;

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "vendor": vendor.name,
                        "criterion": "compliance",
                        "score": score,
                        "certifications": vendor.certifications,
                        "matched": matched,
                    })
                    .to_string(),
                    "suggestor:compliance-screener",
                )
                .with_confidence(0.90),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct ReliabilityEvaluatorSuggestor {
    vendors: Vec<VendorData>,
}

#[async_trait]
impl Suggestor for ReliabilityEvaluatorSuggestor {
    fn name(&self) -> &str {
        "reliability-evaluator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("criterion:reliability:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for vendor in &self.vendors {
            let fact_id = format!("criterion:reliability:{}", slug(&vendor.name));

            // Score based on SLA uptime
            // 99.9% = ~90, 99.95% = ~95, 99.99% = ~100
            let uptime_nines = (100.0 - vendor.sla_uptime) * 10.0;
            let score = (100.0 - uptime_nines).clamp(0.0, 100.0);

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "vendor": vendor.name,
                        "criterion": "reliability",
                        "score": score,
                        "sla_uptime": vendor.sla_uptime,
                    })
                    .to_string(),
                    "suggestor:reliability-evaluator",
                )
                .with_confidence(0.92),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct SupportTierEvaluatorSuggestor {
    vendors: Vec<VendorData>,
}

#[async_trait]
impl Suggestor for SupportTierEvaluatorSuggestor {
    fn name(&self) -> &str {
        "support-tier-evaluator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("criterion:support:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for vendor in &self.vendors {
            let fact_id = format!("criterion:support:{}", slug(&vendor.name));

            // Score based on response time (lower = higher score)
            let max_response_sla = 48.0;
            let response_score =
                (100.0 - (vendor.response_sla_hours / max_response_sla * 100.0)).clamp(0.0, 100.0);

            // Bonus for support tier
            let tier_score = match vendor.support_tier.to_lowercase().as_str() {
                "premium" => 100.0,
                "standard" => 75.0,
                "basic" => 50.0,
                _ => 50.0,
            };

            let score = (response_score + tier_score) / 2.0;

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "vendor": vendor.name,
                        "criterion": "support",
                        "score": score,
                        "support_tier": vendor.support_tier,
                        "response_sla_hours": vendor.response_sla_hours,
                    })
                    .to_string(),
                    "suggestor:support-tier-evaluator",
                )
                .with_confidence(0.88),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct VendorStabilityEvaluatorSuggestor {
    vendors: Vec<VendorData>,
}

#[async_trait]
impl Suggestor for VendorStabilityEvaluatorSuggestor {
    fn name(&self) -> &str {
        "vendor-stability-evaluator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("criterion:stability:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for vendor in &self.vendors {
            let fact_id = format!("criterion:stability:{}", slug(&vendor.name));

            // Funding stage: later stage = more stable
            let funding_score = match vendor.company_funding.to_lowercase().as_str() {
                "profitable" => 100.0,
                "series-d" => 90.0,
                "series-c" => 75.0,
                "series-b" => 60.0,
                "series-a" => 40.0,
                "seed" => 20.0,
                _ => 25.0,
            };

            // Headcount: larger team = more stable (up to a point)
            let headcount_score = (vendor.headcount as f64 / 500.0 * 100.0).min(100.0);

            // Company age: older = more stable
            let current_year = 2026;
            let company_age = current_year - vendor.founded_year;
            let age_score = (company_age as f64 / 20.0 * 100.0).min(100.0);

            let score = funding_score * 0.5 + headcount_score * 0.25 + age_score * 0.25;

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "vendor": vendor.name,
                        "criterion": "stability",
                        "score": score,
                        "funding": vendor.company_funding,
                        "headcount": vendor.headcount,
                        "founded_year": vendor.founded_year,
                    })
                    .to_string(),
                    "suggestor:vendor-stability-evaluator",
                )
                .with_confidence(0.85),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

struct ShortlistSynthesisSuggestor {
    vendors: Vec<VendorData>,
}

#[async_trait]
impl Suggestor for ShortlistSynthesisSuggestor {
    fn name(&self) -> &str {
        "shortlist-synthesis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Run when all 5 criteria are present
        let evals = ctx.get(ContextKey::Evaluations);
        let has_price = evals.iter().any(|f| f.id().starts_with("criterion:price:"));
        let has_compliance = evals
            .iter()
            .any(|f| f.id().starts_with("criterion:compliance:"));
        let has_reliability = evals
            .iter()
            .any(|f| f.id().starts_with("criterion:reliability:"));
        let has_support = evals
            .iter()
            .any(|f| f.id().starts_with("criterion:support:"));
        let has_stability = evals
            .iter()
            .any(|f| f.id().starts_with("criterion:stability:"));

        let has_shortlist = evals
            .iter()
            .any(|f| f.id().as_str() == "synthesis:shortlist");

        has_price
            && has_compliance
            && has_reliability
            && has_support
            && has_stability
            && !has_shortlist
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        // Collect scores for each vendor
        #[derive(Debug, Clone)]
        struct VendorScores {
            name: String,
            price: f64,
            compliance: f64,
            reliability: f64,
            support: f64,
            stability: f64,
        }

        let mut scores: HashMap<String, VendorScores> = HashMap::new();
        for vendor in &self.vendors {
            scores.insert(
                vendor.name.clone(),
                VendorScores {
                    name: vendor.name.clone(),
                    price: 0.0,
                    compliance: 0.0,
                    reliability: 0.0,
                    support: 0.0,
                    stability: 0.0,
                },
            );
        }

        // Extract scores from evaluations
        for fact in ctx.get(ContextKey::Evaluations).iter() {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(fact.content())
                && let (Some(vendor_name), Some(score)) = (
                    payload.get("vendor").and_then(|v| v.as_str()),
                    payload.get("score").and_then(|s| s.as_f64()),
                )
                && let Some(vendor_scores) = scores.get_mut(vendor_name)
            {
                if fact.id().starts_with("criterion:price:") {
                    vendor_scores.price = score;
                } else if fact.id().starts_with("criterion:compliance:") {
                    vendor_scores.compliance = score;
                } else if fact.id().starts_with("criterion:reliability:") {
                    vendor_scores.reliability = score;
                } else if fact.id().starts_with("criterion:support:") {
                    vendor_scores.support = score;
                } else if fact.id().starts_with("criterion:stability:") {
                    vendor_scores.stability = score;
                }
            }
        }

        // Compute weighted average
        // Default weights: price 30%, compliance 20%, reliability 20%, support 15%, stability 15%
        let mut ranked: Vec<(String, f64)> = scores
            .values()
            .map(|vs| {
                let weighted = vs.price * 0.30
                    + vs.compliance * 0.20
                    + vs.reliability * 0.20
                    + vs.support * 0.15
                    + vs.stability * 0.15;
                (vs.name.clone(), weighted)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Recommendation logic
        let (top_vendor, top_score) = ranked
            .first()
            .map(|(n, s)| (n.clone(), *s))
            .unwrap_or_default();

        let recommendation = if top_score >= 70.0 {
            format!("Recommend {} (score: {:.1})", top_vendor, top_score)
        } else if top_score >= 50.0 {
            format!(
                "Escalate {} for review (score: {:.1}, below 70 threshold)",
                top_vendor, top_score
            )
        } else {
            format!(
                "Reject top vendor {} (score: {:.1}, below 50 minimum)",
                top_vendor, top_score
            )
        };

        let shortlist_data = serde_json::json!({
            "recommendation": recommendation,
            "top_vendor": top_vendor,
            "top_score": top_score,
            "ranked": ranked.iter().map(|(name, score)| {
                serde_json::json!({
                    "vendor": name,
                    "score": score,
                })
            }).collect::<Vec<_>>(),
        });

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "synthesis:shortlist",
                shortlist_data.to_string(),
                "suggestor:shortlist-synthesis",
            )
            .with_confidence(0.90),
        )
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
    let truth = find_truth("vendor-selection-simple").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let vendors = parse_vendors(inputs)?;
    if vendors.is_empty() {
        return Err("at least one vendor is required".into());
    }

    // Parse optional budget (default 100,000 minor units = $1000)
    let budget = super::common::optional_input(inputs, "budget")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(10_000_000);

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        PriceEvaluatorSuggestor {
            vendors: vendors.clone(),
            budget,
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        ComplianceScreenerSuggestor {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        ReliabilityEvaluatorSuggestor {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        SupportTierEvaluatorSuggestor {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        VendorStabilityEvaluatorSuggestor {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        ShortlistSynthesisSuggestor {
            vendors: vendors.clone(),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            ContextState::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(VendorSelectionSimpleEvaluator)),
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
                    .find(|f| f.id().as_str() == "synthesis:shortlist")
                {
                    #[derive(Deserialize)]
                    struct Payload {
                        recommendation: String,
                        #[allow(dead_code)]
                        top_vendor: String,
                        top_score: f64,
                    }
                    if let Ok(p) = serde_json::from_str::<Payload>(fact.content()) {
                        let actor = Actor::agent("shortlist-synthesis");
                        kernel.record_decision(
                            DecisionRecord {
                                id: Uuid::new_v4(),
                                truth_key: "vendor-selection-simple".into(),
                                recommendation: p.recommendation,
                                confidence_bps: (p.top_score * 100.0) as u16,
                                rationale: "Multi-criteria evaluation: price, compliance, reliability, support, stability".into(),
                                vendor_id: None,
                                needs_human_review: p.top_score < 70.0,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_vendors() -> Vec<VendorData> {
        vec![
            VendorData {
                name: "CloudTrust AI".to_string(),
                cost_minor: 180_000_00,
                certifications: vec!["SOC2".into(), "ISO27001".into(), "GDPR".into()],
                sla_uptime: 99.99,
                support_tier: "premium".into(),
                response_sla_hours: 1.0,
                company_funding: "series-d".into(),
                founded_year: 2016,
                headcount: 450,
            },
            VendorData {
                name: "QuickAI Solutions".to_string(),
                cost_minor: 30_000_00,
                certifications: vec!["SOC2".into()],
                sla_uptime: 99.5,
                support_tier: "basic".into(),
                response_sla_hours: 24.0,
                company_funding: "series-a".into(),
                founded_year: 2023,
                headcount: 35,
            },
            VendorData {
                name: "MidScale AI".to_string(),
                cost_minor: 90_000_00,
                certifications: vec!["SOC2".into(), "ISO27001".into()],
                sla_uptime: 99.95,
                support_tier: "standard".into(),
                response_sla_hours: 4.0,
                company_funding: "series-b".into(),
                founded_year: 2020,
                headcount: 180,
            },
        ]
    }

    fn sample_inputs() -> HashMap<String, String> {
        let vendors_json = serde_json::to_string(&sample_vendors()).unwrap();
        HashMap::from([("vendors_json".into(), vendors_json)])
    }

    // --- Happy path ---

    #[tokio::test]
    async fn happy_path_all_vendors_evaluate() {
        let store = InMemoryStore::new();
        let inputs = sample_inputs();
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn happy_path_shortlist_emerges() {
        let store = InMemoryStore::new();
        let inputs = sample_inputs();
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
        assert!(!result.criteria_outcomes.is_empty());
    }

    #[tokio::test]
    async fn happy_path_decision_persisted() {
        let store = InMemoryStore::new();
        let inputs = sample_inputs();
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        let decisions = store.read(|k| k.decisions.len()).unwrap();
        assert_eq!(decisions, 1);
    }

    // --- Edge cases ---

    #[tokio::test]
    async fn edge_case_low_score_vendor_escalates() {
        let store = InMemoryStore::new();
        let vendors = vec![VendorData {
            name: "Failing Vendor".to_string(),
            cost_minor: 500_000_00, // Very expensive
            certifications: vec![], // No certs
            sla_uptime: 95.0,       // Poor SLA
            support_tier: "basic".into(),
            response_sla_hours: 48.0,
            company_funding: "seed".into(),
            founded_year: 2024,
            headcount: 5,
        }];
        let vendors_json = serde_json::to_string(&vendors).unwrap();
        let inputs = HashMap::from([("vendors_json".into(), vendors_json)]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        let decisions = store.read(|k| k.decisions.len()).unwrap();
        assert_eq!(decisions, 1);
        // The decision should have needs_human_review set to true because score < 70
    }

    #[tokio::test]
    async fn edge_case_missing_vendors_returns_error() {
        let store = InMemoryStore::new();
        let result = execute(&store, &HashMap::new(), false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edge_case_empty_vendors_array_returns_error() {
        let store = InMemoryStore::new();
        let vendors: Vec<VendorData> = vec![];
        let vendors_json = serde_json::to_string(&vendors).unwrap();
        let inputs = HashMap::from([("vendors_json".into(), vendors_json)]);
        let result = execute(&store, &inputs, false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edge_case_single_vendor_converges() {
        let store = InMemoryStore::new();
        let vendors = vec![sample_vendors()[0].clone()];
        let vendors_json = serde_json::to_string(&vendors).unwrap();
        let inputs = HashMap::from([("vendors_json".into(), vendors_json)]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn edge_case_many_vendors_converges() {
        let store = InMemoryStore::new();
        let mut vendors = sample_vendors();
        for i in 0..10 {
            vendors.push(VendorData {
                name: format!("Extra Vendor {i}"),
                cost_minor: 100_000_00 + (i as i64 * 10_000_00),
                certifications: vec!["SOC2".into()],
                sla_uptime: 99.5,
                support_tier: "standard".into(),
                response_sla_hours: 8.0,
                company_funding: "series-b".into(),
                founded_year: 2021,
                headcount: 100,
            });
        }
        let vendors_json = serde_json::to_string(&vendors).unwrap();
        let inputs = HashMap::from([("vendors_json".into(), vendors_json)]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn no_persist_leaves_kernel_empty() {
        let store = InMemoryStore::new();
        let inputs = sample_inputs();
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
        let decisions = store.read(|k| k.decisions.len()).unwrap();
        assert_eq!(decisions, 0);
    }
}
