//! Truth executor: vendor-selection
//!
//! Full-stack governed vendor selection demonstrating the Reflective Labs stack:
//! - Organism intent packet with admission control
//! - Formation assembly with role assignments
//! - Multi-agent evaluation (compliance, cost, risk, shortlist, synthesis)
//! - Cedar policy gate for commitment authorization
//! - HITL gate for high-value decisions

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use converge_kernel::{AuthorityLevel, ContextState, Engine, FlowAction, FlowPhase, TypesRunHooks};
use converge_model::{
    FormationPlan, FormationRequest, ProfileSnapshot, RoleAssignment, SuggestorCapability,
    SuggestorRole,
};
use converge_pack::{
    AgentEffect, Context as ContextView, ContextKey, DomainId, GateId, PolicyVersionId,
    PrincipalId, ProposedFact, ResourceId, ResourceKind, Suggestor,
};
use converge_policy::{ContextIn, DecideRequest, PolicyEngine, PrincipalIn, ResourceIn};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{VendorSelectionEvaluator, build_intent, find_truth};
use organism_pack::{IntentPacket, Plan, PlanStep, ReasoningSystem};
use organism_runtime::Registry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TruthExecutionResult;

const POLICY_TEXT: &str =
    include_str!("../../../../examples/vendor-selection/vendor-selection-policy.cedar");
const HITL_THRESHOLD_MAJOR: i64 = 50_000;

// ---------------------------------------------------------------------------
// Vendor input data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VendorInput {
    name: String,
    score: f64,
    risk_score: f64,
    compliance_status: String,
    certifications: Vec<String>,
    monthly_cost_minor: i64,
    currency_code: String,
}

fn parse_vendors(inputs: &HashMap<String, String>) -> Result<Vec<VendorInput>, String> {
    if let Some(json) = inputs.get("vendors_json") {
        serde_json::from_str(json).map_err(|e| format!("invalid vendors_json: {e}"))
    } else {
        let raw = inputs
            .get("vendors")
            .or_else(|| inputs.get("vendor_names"))
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "missing required input: vendors".to_string())?;
        let names: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if names.is_empty() {
            return Err("at least one vendor name is required".into());
        }
        Ok(names
            .into_iter()
            .map(|name| VendorInput {
                name,
                score: 80.0,
                risk_score: 20.0,
                compliance_status: "compliant".into(),
                certifications: vec!["SOC2".into()],
                monthly_cost_minor: 50_000,
                currency_code: "USD".into(),
            })
            .collect())
    }
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn amount_major_from_minor(amount_minor: i64) -> i64 {
    let amount_minor = amount_minor.max(0);
    (amount_minor + 99) / 100
}

fn parse_optional_bool(
    inputs: &HashMap<String, String>,
    key: &str,
) -> Result<Option<bool>, String> {
    let Some(raw) = super::common::optional_input(inputs, key) else {
        return Ok(None);
    };
    match raw.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(Some(true)),
        "false" | "0" | "no" => Ok(Some(false)),
        _ => Err(format!("invalid {key}: expected true/false")),
    }
}

fn parse_authority(inputs: &HashMap<String, String>) -> AuthorityLevel {
    match super::common::optional_input(inputs, "principal_authority")
        .unwrap_or_else(|| "supervisory".into())
        .as_str()
    {
        "advisory" => AuthorityLevel::Advisory,
        "supervisory" => AuthorityLevel::Supervisory,
        "participatory" => AuthorityLevel::Participatory,
        "sovereign" => AuthorityLevel::Sovereign,
        _ => AuthorityLevel::Advisory,
    }
}

// ---------------------------------------------------------------------------
// Organism planning seed
// ---------------------------------------------------------------------------

fn build_planning_seed(vendors: &[VendorInput]) -> Vec<(&'static str, String)> {
    let vendor_list: Vec<&str> = vendors.iter().map(|v| v.name.as_str()).collect();
    let intent = IntentPacket::new(
        format!("Governed vendor selection: {}", vendor_list.join(", ")),
        Utc::now() + Duration::hours(1),
    )
    .with_context(serde_json::json!({
        "vendors": vendor_list,
        "goal": "formation-assembled, evidence-backed vendor selection with policy gate",
    }))
    .with_authority(vec!["vendor_selection".to_string()]);

    let registry = Registry::with_standard_packs();
    let _pack_count = registry.packs().len();

    let mut compliance = Plan::new(&intent, "Screen vendors for regulatory compliance");
    compliance.contributor = ReasoningSystem::DomainModel;
    compliance.steps = vec![PlanStep {
        action: "[screening] check GDPR, AI Act, data residency, certifications".into(),
        expected_effect: "Compliance evidence for every vendor".into(),
    }];

    let mut cost = Plan::new(&intent, "Evaluate cost and budget fit");
    cost.contributor = ReasoningSystem::CostEstimation;
    cost.steps = vec![PlanStep {
        action: "[evaluation] estimate monthly cost and budget alignment".into(),
        expected_effect: "Comparable cost evidence for every vendor".into(),
    }];

    let mut risk = Plan::new(&intent, "Assess multi-dimensional risk");
    risk.contributor = ReasoningSystem::CausalAnalysis;
    risk.steps = vec![PlanStep {
        action: "[evaluation] score lock-in, operational, and compliance risk".into(),
        expected_effect: "Risk profile for every vendor".into(),
    }];

    let mut shortlist = Plan::new(&intent, "Rank and shortlist qualifying vendors");
    shortlist.contributor = ReasoningSystem::ConstraintSolver;
    shortlist.steps = vec![PlanStep {
        action: "[shortlist] filter by thresholds, rank by composite score".into(),
        expected_effect: "Ranked shortlist of qualifying vendors".into(),
    }];

    let mut synthesis = Plan::new(&intent, "Synthesize final recommendation");
    synthesis.contributor = ReasoningSystem::LlmReasoning;
    synthesis.steps = vec![PlanStep {
        action: "[decision] produce evidence-backed recommendation".into(),
        expected_effect: "Final recommendation with confidence and rationale".into(),
    }];

    vec![
        (
            "strategy:vendor-sel:compliance",
            compliance.steps[0].action.clone(),
        ),
        ("strategy:vendor-sel:cost", cost.steps[0].action.clone()),
        ("strategy:vendor-sel:risk", risk.steps[0].action.clone()),
        (
            "strategy:vendor-sel:shortlist",
            shortlist.steps[0].action.clone(),
        ),
        (
            "strategy:vendor-sel:decision",
            synthesis.steps[0].action.clone(),
        ),
    ]
}

// ---------------------------------------------------------------------------
// Formation assembly
// ---------------------------------------------------------------------------

fn build_formation_catalog() -> Vec<ProfileSnapshot> {
    vec![
        ProfileSnapshot {
            name: "compliance-screener".into(),
            role: SuggestorRole::Analysis,
            output_keys: vec![ContextKey::Seeds],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Interactive,
            capabilities: vec![SuggestorCapability::PolicyEnforcement],
            confidence_min: 0.8,
            confidence_max: 1.0,
        },
        ProfileSnapshot {
            name: "cost-analysis".into(),
            role: SuggestorRole::Evaluation,
            output_keys: vec![ContextKey::Evaluations],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Interactive,
            capabilities: vec![SuggestorCapability::Analytics],
            confidence_min: 0.7,
            confidence_max: 0.9,
        },
        ProfileSnapshot {
            name: "vendor-risk".into(),
            role: SuggestorRole::Evaluation,
            output_keys: vec![ContextKey::Evaluations],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Interactive,
            capabilities: vec![SuggestorCapability::Analytics],
            confidence_min: 0.6,
            confidence_max: 0.85,
        },
        ProfileSnapshot {
            name: "vendor-shortlist".into(),
            role: SuggestorRole::Synthesis,
            output_keys: vec![ContextKey::Proposals],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Interactive,
            capabilities: vec![SuggestorCapability::Optimization],
            confidence_min: 0.7,
            confidence_max: 1.0,
        },
        ProfileSnapshot {
            name: "decision-synthesis".into(),
            role: SuggestorRole::Synthesis,
            output_keys: vec![ContextKey::Evaluations],
            cost_hint: converge_provider_api::CostClass::Medium,
            latency_hint: converge_provider_api::LatencyClass::Background,
            capabilities: vec![SuggestorCapability::LlmReasoning],
            confidence_min: 0.75,
            confidence_max: 0.95,
        },
    ]
}

// ---------------------------------------------------------------------------
// Suggestors
// ---------------------------------------------------------------------------

struct PlanningSeedSuggestor {
    strategies: Vec<(&'static str, String)>,
    catalog: Vec<ProfileSnapshot>,
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
            .any(|f| f.id.starts_with("strategy:vendor-sel:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals: Vec<ProposedFact> = self
            .strategies
            .iter()
            .map(|(id, content)| {
                ProposedFact::new(
                    ContextKey::Strategies,
                    *id,
                    content.clone(),
                    "organism-planning:vendor-selection",
                )
                .with_confidence(1.0)
            })
            .collect();

        // Seed a formation request
        let request = FormationRequest {
            id: "vendor-selection".into(),
            required_roles: vec![
                SuggestorRole::Analysis,
                SuggestorRole::Evaluation,
                SuggestorRole::Synthesis,
            ],
            required_capabilities: vec![
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::Analytics,
                SuggestorCapability::Optimization,
            ],
        };

        // Assemble formation plan deterministically
        let plan = assemble_formation(&request, &self.catalog);

        proposals.push(
            ProposedFact::new(
                ContextKey::Strategies,
                format!("formation:plan:{}", request.id),
                serde_json::to_string(&plan).unwrap_or_default(),
                "formation-assembly:vendor-selection",
            )
            .with_confidence(plan.coverage_ratio),
        );

        AgentEffect::with_proposals(proposals)
    }
}

fn assemble_formation(request: &FormationRequest, catalog: &[ProfileSnapshot]) -> FormationPlan {
    let mut assignments = Vec::new();
    let mut used = Vec::new();

    for role in &request.required_roles {
        if let Some(profile) = catalog
            .iter()
            .find(|p| &p.role == role && !used.contains(&p.name))
        {
            assignments.push(RoleAssignment {
                role: *role,
                suggestor: profile.name.clone(),
            });
            used.push(profile.name.clone());
        }
    }

    let coverage = if request.required_roles.is_empty() {
        1.0
    } else {
        assignments.len() as f64 / request.required_roles.len() as f64
    };

    FormationPlan {
        request_id: request.id.clone(),
        assignments,
        unmatched_roles: request
            .required_roles
            .iter()
            .filter(|r| {
                !used
                    .iter()
                    .any(|u| catalog.iter().any(|p| &p.role == *r && &p.name == u))
            })
            .cloned()
            .collect(),
        coverage_ratio: coverage,
    }
}

struct ComplianceScreenerAgent {
    vendors: Vec<VendorInput>,
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
            .any(|f| f.id == "strategy:vendor-sel:compliance")
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for vendor in &self.vendors {
            let fact_id = format!("compliance:screen:{}", slug(&vendor.name));
            if ctx.get(ContextKey::Seeds).iter().any(|f| f.id == fact_id) {
                continue;
            }
            let compliant = vendor.compliance_status == "compliant";
            proposals.push(
                ProposedFact::new(
                    ContextKey::Seeds,
                    fact_id,
                    serde_json::json!({
                        "vendor_name": vendor.name,
                        "compliance_status": vendor.compliance_status,
                        "compliant": compliant,
                        "certifications": vendor.certifications,
                        "gdpr_pass": compliant,
                        "ai_act_pass": compliant,
                        "data_residency": "EU",
                    })
                    .to_string(),
                    "agent:compliance-screener",
                )
                .with_confidence(if compliant { 0.9 } else { 0.5 }),
            );
        }
        AgentEffect { proposals }
    }
}

struct CostAnalysisAgent {
    vendors: Vec<VendorInput>,
}

#[async_trait]
impl Suggestor for CostAnalysisAgent {
    fn name(&self) -> &str {
        "cost-analysis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds, ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id == "strategy:vendor-sel:cost")
            && ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id.starts_with("compliance:screen:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for vendor in &self.vendors {
            let vendor_slug = slug(&vendor.name);
            let cost_id = format!("cost:estimate:{vendor_slug}");
            if ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id == cost_id)
            {
                continue;
            }
            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    cost_id,
                    serde_json::json!({
                        "vendor_name": vendor.name,
                        "vendor_slug": vendor_slug,
                        "monthly_cost_minor": vendor.monthly_cost_minor,
                        "currency_code": vendor.currency_code,
                    })
                    .to_string(),
                    "agent:cost-analysis",
                )
                .with_confidence(0.80),
            );
        }
        AgentEffect { proposals }
    }
}

struct VendorRiskAgent {
    vendors: Vec<VendorInput>,
}

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
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id == "strategy:vendor-sel:risk")
            && ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id.starts_with("compliance:screen:"))
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("cost:estimate:"))
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("risk:score:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];
        for vendor in &self.vendors {
            let vendor_slug = slug(&vendor.name);
            let risk_id = format!("risk:score:{vendor_slug}");
            let risk_level = if vendor.risk_score > 30.0 {
                "high"
            } else if vendor.risk_score > 15.0 {
                "medium"
            } else {
                "low"
            };
            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    risk_id,
                    serde_json::json!({
                        "vendor_name": vendor.name,
                        "vendor_slug": vendor_slug,
                        "risk_score": vendor.risk_score,
                        "risk_level": risk_level,
                        "lock_in_risk": risk_level,
                        "compliance_risk": if vendor.compliance_status == "compliant" { "low" } else { "high" },
                        "operational_risk": "low",
                    })
                    .to_string(),
                    "agent:vendor-risk",
                )
                .with_confidence(0.75),
            );
        }
        AgentEffect { proposals }
    }
}

struct VendorShortlistAgent {
    vendors: Vec<VendorInput>,
    min_score: f64,
    max_risk: f64,
    max_vendors: usize,
}

#[async_trait]
impl Suggestor for VendorShortlistAgent {
    fn name(&self) -> &str {
        "vendor-shortlist"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Seeds,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id == "strategy:vendor-sel:shortlist")
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("risk:score:"))
            && !ctx
                .get(ContextKey::Proposals)
                .iter()
                .any(|f| f.id == "vendor:shortlist")
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut qualifying: Vec<_> = self
            .vendors
            .iter()
            .filter(|v| {
                v.compliance_status == "compliant"
                    && v.score >= self.min_score
                    && v.risk_score <= self.max_risk
            })
            .collect();

        qualifying.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let shortlisted: Vec<_> = qualifying.iter().take(self.max_vendors).collect();

        let rejected: Vec<_> = self
            .vendors
            .iter()
            .filter(|v| {
                v.compliance_status != "compliant"
                    || v.score < self.min_score
                    || v.risk_score > self.max_risk
            })
            .map(|v| {
                let mut reasons = vec![];
                if v.compliance_status != "compliant" {
                    reasons.push(format!("non-compliant ({})", v.compliance_status));
                }
                if v.score < self.min_score {
                    reasons.push(format!(
                        "score {:.0} below minimum {:.0}",
                        v.score, self.min_score
                    ));
                }
                if v.risk_score > self.max_risk {
                    reasons.push(format!(
                        "risk {:.0} above maximum {:.0}",
                        v.risk_score, self.max_risk
                    ));
                }
                serde_json::json!({
                    "vendor_name": v.name,
                    "reasons": reasons,
                })
            })
            .collect();

        let shortlist_entries: Vec<_> = shortlisted
            .iter()
            .enumerate()
            .map(|(rank, v)| {
                serde_json::json!({
                    "rank": rank + 1,
                    "vendor_name": v.name,
                    "score": v.score,
                    "risk_score": v.risk_score,
                    "composite_score": v.score - v.risk_score,
                })
            })
            .collect();

        let confidence = if shortlisted.is_empty() { 0.5 } else { 0.9 };

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Proposals,
                "vendor:shortlist",
                serde_json::json!({
                    "shortlist": shortlist_entries,
                    "rejected": rejected,
                    "total_candidates": self.vendors.len(),
                    "qualifying_count": shortlisted.len(),
                })
                .to_string(),
                "agent:vendor-shortlist",
            )
            .with_confidence(confidence),
        )
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
            ContextKey::Proposals,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id == "strategy:vendor-sel:decision")
            && ctx
                .get(ContextKey::Proposals)
                .iter()
                .any(|f| f.id == "vendor:shortlist")
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id == "decision:recommendation")
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let shortlist_fact = ctx
            .get(ContextKey::Proposals)
            .iter()
            .find(|f| f.id == "vendor:shortlist");

        let (recommendation, confidence, needs_review) = if let Some(fact) = shortlist_fact {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&fact.content) {
                let shortlist = payload.get("shortlist").and_then(|v| v.as_array());
                if let Some(entries) = shortlist {
                    if let Some(top) = entries.first() {
                        let name = top
                            .get("vendor_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown");
                        let score = top.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        (
                            format!("{name} recommended (score: {score:.0})"),
                            0.85,
                            false,
                        )
                    } else {
                        (
                            "No qualifying vendors — manual review required".into(),
                            0.4,
                            true,
                        )
                    }
                } else {
                    (
                        "Shortlist parse error — manual review required".into(),
                        0.3,
                        true,
                    )
                }
            } else {
                (
                    "Shortlist malformed — manual review required".into(),
                    0.3,
                    true,
                )
            }
        } else {
            (
                "No shortlist available — cannot synthesize".into(),
                0.2,
                true,
            )
        };

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "decision:recommendation",
                serde_json::json!({
                    "recommendation": recommendation,
                    "confidence": confidence,
                    "needs_human_review": needs_review,
                })
                .to_string(),
                "agent:decision-synthesis",
            )
            .with_confidence(confidence),
        )
    }
}

struct PolicyGateSuggestor {
    vendors: Vec<VendorInput>,
    principal_authority: AuthorityLevel,
    human_approval_present: Option<bool>,
    engine: Arc<PolicyEngine>,
}

#[async_trait]
impl Suggestor for PolicyGateSuggestor {
    fn name(&self) -> &str {
        "policy-gate"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id == "decision:recommendation")
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id.starts_with("policy:decision:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let rec_fact = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .find(|f| f.id == "decision:recommendation");

        let needs_review = rec_fact
            .and_then(|f| serde_json::from_str::<serde_json::Value>(&f.content).ok())
            .and_then(|p| p.get("needs_human_review").and_then(|v| v.as_bool()))
            .unwrap_or(true);

        let selected_vendor_name = selected_vendor_name(ctx);
        let selected_vendor = selected_vendor_name
            .as_deref()
            .and_then(|name| self.vendors.iter().find(|vendor| vendor.name == name));
        let selected_amount_minor = selected_vendor
            .map(|vendor| vendor.monthly_cost_minor)
            .unwrap_or(0);
        let selected_amount_major = amount_major_from_minor(selected_amount_minor);
        let gates_met = !needs_review && selected_vendor.is_some();
        let human_approval = self
            .human_approval_present
            .unwrap_or_else(|| gates_met && selected_amount_major <= HITL_THRESHOLD_MAJOR);

        let request = DecideRequest {
            principal: PrincipalIn {
                id: PrincipalId::new("user:procurement-lead".to_string()),
                authority: self.principal_authority,
                domains: vec![DomainId::new("procurement".to_string())],
                policy_version: Some(PolicyVersionId::new("v1".to_string())),
            },
            resource: ResourceIn {
                id: ResourceId::new("commitment:vendor-selection".to_string()),
                resource_type: Some(ResourceKind::new("contract".to_string())),
                phase: Some(FlowPhase::Commitment),
                gates_passed: Some(if gates_met {
                    vec![
                        GateId::new("evidence"),
                        GateId::new("risk"),
                        GateId::new("compliance"),
                    ]
                } else {
                    Vec::new()
                }),
            },
            action: FlowAction::Commit,
            context: Some(ContextIn {
                commitment_type: Some("contract".into()),
                amount: Some(selected_amount_major),
                human_approval_present: Some(human_approval),
                required_gates_met: Some(gates_met),
            }),
            delegation_b64: None,
        };

        let (outcome, reason) = match self.engine.evaluate(&request) {
            Ok(decision) => (format!("{:?}", decision.outcome), decision.reason),
            Err(e) => ("Reject".into(), Some(format!("policy error: {e}"))),
        };

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                "policy:decision:vendor-selection",
                serde_json::json!({
                    "outcome": outcome,
                    "reason": reason,
                    "selected_vendor": selected_vendor_name,
                    "selected_amount_minor": selected_amount_minor,
                    "selected_amount_major": selected_amount_major,
                    "hitl_threshold_major": HITL_THRESHOLD_MAJOR,
                    "principal_authority": self.principal_authority.as_str(),
                    "human_approval_present": human_approval,
                    "gates_met": gates_met,
                })
                .to_string(),
                "policy:vendor-selection",
            )
            .with_confidence(1.0),
        )
    }
}

fn selected_vendor_name(ctx: &dyn ContextView) -> Option<String> {
    let shortlist_fact = ctx
        .get(ContextKey::Proposals)
        .iter()
        .find(|f| f.id == "vendor:shortlist")?;
    let payload = serde_json::from_str::<serde_json::Value>(&shortlist_fact.content).ok()?;
    let shortlist = payload
        .get("shortlist")
        .and_then(|value| value.as_array())?;
    shortlist
        .first()
        .and_then(|entry| entry.get("vendor_name"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

pub async fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("vendor-selection").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let vendors = parse_vendors(inputs)?;
    let strategies = build_planning_seed(&vendors);
    let catalog = build_formation_catalog();
    let principal_authority = parse_authority(inputs);
    let human_approval_present = parse_optional_bool(inputs, "human_approval_present")?;

    let policy_engine = Arc::new(
        PolicyEngine::from_policy_str(POLICY_TEXT)
            .map_err(|e| format!("policy setup failed: {e}"))?,
    );

    let min_score = inputs
        .get("min_score")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(70.0);
    let max_risk = inputs
        .get("max_risk")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(40.0);
    let max_vendors = inputs
        .get("max_vendors")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "intent-pack",
        PlanningSeedSuggestor {
            strategies,
            catalog,
        },
    );
    engine.register_suggestor_in_pack(
        "screening-pack",
        ComplianceScreenerAgent {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        CostAnalysisAgent {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        VendorRiskAgent {
            vendors: vendors.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        VendorShortlistAgent {
            vendors: vendors.clone(),
            min_score,
            max_risk,
            max_vendors,
        },
    );
    engine.register_suggestor_in_pack("evaluation-pack", DecisionSynthesisAgent);
    engine.register_suggestor_in_pack(
        "policy-pack",
        PolicyGateSuggestor {
            vendors: vendors.clone(),
            principal_authority,
            human_approval_present,
            engine: Arc::clone(&policy_engine),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            ContextState::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(VendorSelectionEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|e| format!("convergence failed: {e}"))?;

    let projection_details = vendor_selection_projection_details(&result.context);

    let projection = if persist {
        let policy = policy_decision_payload(&result.context);
        let write_result = store
            .write_with_events(|kernel| {
                let actor = Actor::agent("vendor-selection");

                // Register vendors
                for vendor in &vendors {
                    kernel.register_vendor(
                        vendor.name.clone(),
                        format!("score={}, risk={}", vendor.score, vendor.risk_score),
                        &actor,
                    );
                }

                // Record the decision
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
                        let policy_outcome = policy.as_ref().map(|policy| policy.outcome.as_str());
                        let needs_human_review =
                            p.needs_human_review || matches!(policy_outcome, Some("Escalate"));
                        let recommendation = match policy_outcome {
                            Some("Escalate") => format!(
                                "Escalate vendor selection for approval: {}",
                                p.recommendation
                            ),
                            Some("Reject") => {
                                format!("Reject vendor selection commitment: {}", p.recommendation)
                            }
                            _ => p.recommendation,
                        };
                        let rationale = policy
                            .as_ref()
                            .map(|policy| {
                                format!(
                                    "Policy outcome={} authority={} amount={} reason={}",
                                    policy.outcome,
                                    policy.principal_authority.as_deref().unwrap_or("unknown"),
                                    policy.selected_amount_major.unwrap_or_default(),
                                    policy.reason.as_deref().unwrap_or("none")
                                )
                            })
                            .unwrap_or_else(|| {
                                "Formation-assembled, multi-agent vendor selection".into()
                            });
                        kernel.record_decision(
                            DecisionRecord {
                                id: Uuid::new_v4(),
                                truth_key: "vendor-selection".into(),
                                recommendation,
                                confidence_bps: super::common::converge_confidence_to_bps(
                                    p.confidence,
                                ),
                                rationale,
                                vendor_id: None,
                                needs_human_review,
                                decided_by: actor.clone(),
                                decided_at: Utc::now(),
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
            details: projection_details,
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

#[derive(Debug, Deserialize)]
struct PolicyDecisionPayload {
    outcome: String,
    reason: Option<String>,
    selected_amount_major: Option<i64>,
    principal_authority: Option<String>,
}

fn policy_decision_payload(ctx: &ContextState) -> Option<PolicyDecisionPayload> {
    ctx.get(ContextKey::Evaluations)
        .iter()
        .find(|f| f.id == "policy:decision:vendor-selection")
        .and_then(|fact| serde_json::from_str::<PolicyDecisionPayload>(&fact.content).ok())
}

fn vendor_selection_projection_details(ctx: &ContextState) -> Option<serde_json::Value> {
    let recommendation = ctx
        .get(ContextKey::Evaluations)
        .iter()
        .find(|f| f.id == "decision:recommendation")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());
    let shortlist = ctx
        .get(ContextKey::Proposals)
        .iter()
        .find(|f| f.id == "vendor:shortlist")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());
    let policy = ctx
        .get(ContextKey::Evaluations)
        .iter()
        .find(|f| f.id == "policy:decision:vendor-selection")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());

    if recommendation.is_none() && shortlist.is_none() && policy.is_none() {
        return None;
    }

    Some(serde_json::json!({
        "recommendation": recommendation,
        "shortlist": shortlist,
        "policy": policy,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn criterion_result<'a>(result: &'a TruthExecutionResult, needle: &str) -> &'a str {
        result
            .criteria_outcomes
            .iter()
            .find(|outcome| outcome.criterion.contains(needle))
            .map(|outcome| outcome.result.as_str())
            .unwrap_or("")
    }

    // --- Happy path ---

    #[tokio::test]
    async fn vendor_selection_end_to_end() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML, Gamma LLM".into())]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 1);
    }

    #[tokio::test]
    async fn vendor_selection_with_json_input() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Acme AI",
                "score": 85.0,
                "risk_score": 15.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2", "ISO27001"],
                "monthly_cost_minor": 42000,
                "currency_code": "USD"
            },
            {
                "name": "Beta ML",
                "score": 78.0,
                "risk_score": 25.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2"],
                "monthly_cost_minor": 28000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([("vendors_json".into(), vendors_json.to_string())]);
        let result = execute(&store, &inputs, true).await.unwrap();
        assert!(result.converged);
        let vendors = store.read(|k| k.vendors.len()).unwrap();
        assert_eq!(vendors, 2);
    }

    #[tokio::test]
    async fn vendor_selection_no_persist() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI".into())]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
        assert_eq!(store.read(|k| k.decisions.len()).unwrap(), 0);
    }

    #[tokio::test]
    async fn vendor_selection_criteria_all_populated() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(!result.criteria_outcomes.is_empty());
        assert_eq!(result.criteria_outcomes.len(), 5);
    }

    #[tokio::test]
    async fn vendor_selection_filters_non_compliant() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Good Vendor",
                "score": 85.0,
                "risk_score": 15.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2"],
                "monthly_cost_minor": 30000,
                "currency_code": "USD"
            },
            {
                "name": "Pending Vendor",
                "score": 92.0,
                "risk_score": 10.0,
                "compliance_status": "pending",
                "certifications": [],
                "monthly_cost_minor": 65000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([("vendors_json".into(), vendors_json.to_string())]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn vendor_selection_policy_uses_selected_vendor_amount() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Acme AI",
                "score": 85.0,
                "risk_score": 15.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2"],
                "monthly_cost_minor": 4200000,
                "currency_code": "USD"
            },
            {
                "name": "Rejected Expensive Vendor",
                "score": 95.0,
                "risk_score": 12.0,
                "compliance_status": "pending",
                "certifications": [],
                "monthly_cost_minor": 12000000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([
            ("vendors_json".into(), vendors_json.to_string()),
            ("min_score".into(), "75".into()),
            ("max_risk".into(), "30".into()),
        ]);

        let result = execute(&store, &inputs, true).await.unwrap();

        assert!(result.converged);
        assert!(
            criterion_result(&result, "Cedar policy").contains("Met"),
            "policy should authorize the selected $42k vendor, not the rejected $120k vendor"
        );
        let details = result.projection.and_then(|projection| projection.details);
        assert_eq!(
            details
                .as_ref()
                .and_then(|details| details.pointer("/policy/selected_vendor"))
                .and_then(|value| value.as_str()),
            Some("Acme AI")
        );
    }

    #[tokio::test]
    async fn vendor_selection_high_value_vendor_requires_human_approval() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Gamma LLM",
                "score": 92.0,
                "risk_score": 12.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2"],
                "monthly_cost_minor": 6500000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([
            ("vendors_json".into(), vendors_json.to_string()),
            ("min_score".into(), "75".into()),
            ("max_risk".into(), "30".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();

        assert!(
            criterion_result(&result, "Cedar policy").contains("Blocked"),
            "high-value selected vendor should stop at the HITL policy gate"
        );
    }

    #[tokio::test]
    async fn vendor_selection_human_approval_authorizes_high_value_vendor() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Gamma LLM",
                "score": 92.0,
                "risk_score": 12.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2"],
                "monthly_cost_minor": 6500000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([
            ("vendors_json".into(), vendors_json.to_string()),
            ("min_score".into(), "75".into()),
            ("max_risk".into(), "30".into()),
            ("human_approval_present".into(), "true".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();

        assert!(result.converged);
        assert!(criterion_result(&result, "Cedar policy").contains("Met"));
    }

    #[tokio::test]
    async fn vendor_selection_advisory_authority_cannot_commit() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([
            ("vendors".into(), "Acme AI".into()),
            ("principal_authority".into(), "advisory".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();

        assert!(
            criterion_result(&result, "Cedar policy").contains("Unmet"),
            "advisory authority should be rejected for commitment"
        );
    }

    #[tokio::test]
    async fn vendor_selection_accepts_vendor_names_alias() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendor_names".into(), "Acme AI, Beta ML".into())]);

        let result = execute(&store, &inputs, false).await.unwrap();

        assert!(result.converged);
    }

    // --- Negative tests ---

    #[tokio::test]
    async fn missing_vendors_returns_error() {
        let store = InMemoryStore::new();
        assert!(execute(&store, &HashMap::new(), false).await.is_err());
    }

    #[tokio::test]
    async fn empty_vendors_returns_error() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "".into())]);
        assert!(execute(&store, &inputs, false).await.is_err());
    }

    #[tokio::test]
    async fn invalid_json_vendors_returns_error() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors_json".into(), "{bad json".into())]);
        assert!(execute(&store, &inputs, false).await.is_err());
    }

    // --- Soak tests ---

    #[tokio::test]
    async fn soak_repeated_execution_same_store() {
        let store = InMemoryStore::new();
        for i in 0..20 {
            let inputs = HashMap::from([("vendors".into(), format!("Vendor-{i}"))]);
            let result = execute(&store, &inputs, true).await.unwrap();
            assert!(result.converged, "run {i} should converge");
        }
        let decisions = store.read(|k| k.decisions.len()).unwrap();
        assert_eq!(decisions, 20);
    }

    #[tokio::test]
    async fn soak_repeated_execution_fresh_store() {
        for i in 0..20 {
            let store = InMemoryStore::new();
            let inputs = HashMap::from([("vendors".into(), format!("V-{i}"))]);
            let result = execute(&store, &inputs, true).await.unwrap();
            assert!(result.converged, "run {i} should converge");
        }
    }
}
