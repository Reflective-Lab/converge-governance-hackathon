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
use super::vendor_selection_live::{
    LiveComplianceScreenerAgent, LiveCostAnalysisAgent, LiveDecisionSynthesisAgent,
    LiveVendorRiskAgent, MODEL_FAST, MODEL_FAST_FALLBACK, MODEL_MID, MODEL_MID_FALLBACK,
    MODEL_STRONG, MODEL_STRONG_FALLBACK,
};
use crate::experience::{ExperienceRegistry, RunSummaryInput};
use crate::llm_helpers::{SelectedLlm, load_env, select_llm, select_llm_for_model};
use governance_telemetry::{InMemoryLlmCallCollector, LlmCallSink, LlmCallTelemetry};

const POLICY_TEXT: &str =
    include_str!("../../../../examples/vendor-selection/vendor-selection-policy.cedar");
const HITL_THRESHOLD_MAJOR: i64 = 50_000;

// ---------------------------------------------------------------------------
// Vendor input data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VendorInput {
    pub(crate) name: String,
    pub(crate) score: f64,
    pub(crate) risk_score: f64,
    pub(crate) compliance_status: String,
    pub(crate) certifications: Vec<String>,
    pub(crate) monthly_cost_minor: i64,
    pub(crate) currency_code: String,
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

pub(crate) fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn amount_major_from_minor(amount_minor: i64) -> i64 {
    let amount_minor = amount_minor.max(0);
    (amount_minor + 99) / 100
}

fn certification_coverage_score(vendor: &VendorInput) -> f64 {
    let required = ["SOC2", "ISO27001", "GDPR"];
    let matched = required
        .iter()
        .filter(|required_cert| {
            vendor
                .certifications
                .iter()
                .any(|cert| cert.eq_ignore_ascii_case(required_cert))
        })
        .count();
    (matched as f64 / required.len() as f64) * 100.0
}

fn cost_efficiency_score(cost_major: i64, min_cost: i64, max_cost: i64) -> f64 {
    if max_cost <= min_cost {
        return 100.0;
    }
    let range = (max_cost - min_cost) as f64;
    let normalized = 1.0 - ((cost_major - min_cost) as f64 / range);
    (normalized.clamp(0.0, 1.0) * 100.0 * 10.0).round() / 10.0
}

fn vendor_objective_score(vendor: &VendorInput, min_cost: i64, max_cost: i64) -> f64 {
    let cost_major = amount_major_from_minor(vendor.monthly_cost_minor);
    let risk_adjusted = (100.0 - vendor.risk_score).clamp(0.0, 100.0);
    let score = vendor.score.clamp(0.0, 100.0);
    let cost = cost_efficiency_score(cost_major, min_cost, max_cost);
    let certifications = certification_coverage_score(vendor);
    let composite = 0.35 * score + 0.25 * risk_adjusted + 0.20 * cost + 0.20 * certifications;
    (composite * 10.0).round() / 10.0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoMode {
    GovernedSelection,
    ParetoBreakout,
}

impl DemoMode {
    fn parse(inputs: &HashMap<String, String>) -> Self {
        match super::common::optional_input(inputs, "demo_mode")
            .unwrap_or_else(|| "governed".into())
            .as_str()
        {
            "pareto-breakout" | "creative" | "open" => Self::ParetoBreakout,
            _ => Self::GovernedSelection,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::GovernedSelection => "governed",
            Self::ParetoBreakout => "pareto-breakout",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::GovernedSelection => "Governed selection",
            Self::ParetoBreakout => "Creative Pareto breakout",
        }
    }

    fn thesis(self) -> &'static str {
        match self {
            Self::GovernedSelection => {
                "Replace human document exchange with AI-supported converging flows, while keeping HITL and Cedar policy gates explicit."
            }
            Self::ParetoBreakout => {
                "Challenge the single-vendor assumption and search for a better Pareto balance through a governed provider mix and routing layer."
            }
        }
    }

    fn selection_boundary(self) -> &'static str {
        match self {
            Self::GovernedSelection => {
                "Selection remains among the vendors entered through the RFI/RFP intake."
            }
            Self::ParetoBreakout => {
                "Formation may propose a router/provider-mix strategy, but policy, authority, and provenance gates still apply."
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Organism planning seed
// ---------------------------------------------------------------------------

fn build_planning_seed(
    vendors: &[VendorInput],
    demo_mode: DemoMode,
) -> Vec<(&'static str, String)> {
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
            "strategy:vendor-sel:tactic",
            problem_solving_tactic(vendors, demo_mode).to_string(),
        ),
        (
            "strategy:vendor-sel:router-hypothesis",
            router_hypothesis(vendors, demo_mode).to_string(),
        ),
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

fn router_hypothesis(vendors: &[VendorInput], demo_mode: DemoMode) -> serde_json::Value {
    let compliant_count = vendors
        .iter()
        .filter(|vendor| vendor.compliance_status == "compliant")
        .count();
    let high_capability_count = vendors.iter().filter(|vendor| vendor.score >= 85.0).count();
    let low_risk_count = vendors
        .iter()
        .filter(|vendor| vendor.risk_score <= 20.0)
        .count();
    let cost_range = {
        let min_cost = vendors
            .iter()
            .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
            .min()
            .unwrap_or(0);
        let max_cost = vendors
            .iter()
            .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
            .max()
            .unwrap_or(min_cost);
        max_cost.saturating_sub(min_cost)
    };

    let router_fit = demo_mode == DemoMode::ParetoBreakout
        && compliant_count >= 3
        && high_capability_count >= 2
        && low_risk_count >= 2
        && cost_range >= 20_000;

    serde_json::json!({
        "mode": demo_mode.as_str(),
        "name": if router_fit { "router-first-provider-strategy" } else { "single-primary-provider-strategy" },
        "router_fit": router_fit,
        "why": if router_fit {
            "The candidate set has multiple viable providers with different strengths and costs. A router can assign models/providers per workload instead of forcing one winner."
        } else if demo_mode == DemoMode::GovernedSelection {
            "This mode stays inside the original RFI/RFP sandbox: select from the provided vendors, replace document exchange with governed flow, and use learning to calibrate future policy delegation."
        } else {
            "The candidate set does not yet show enough differentiated viable providers to justify routing as the primary strategy."
        },
        "gateway_options": ["Kong", "OpenRouter"],
        "provider_mix": [
            {
                "need": "programming and agentic reasoning",
                "route": "strong reasoning/coding model when ambiguity or risk is high"
            },
            {
                "need": "routine structured synthesis",
                "route": "fast reliable structured-output model"
            },
            {
                "need": "broad web evidence",
                "route": "Brave-style wide search"
            },
            {
                "need": "deep canonical evidence",
                "route": "Tavily-style focused retrieval"
            },
            {
                "need": "governance controls",
                "route": "Kong-style gateway for policy, rate limits, audit, PII, and cost controls"
            }
        ],
        "demo_line": if router_fit {
            "We thought we were selecting one AI vendor. The formation found that the better answer is a governed provider mix behind a router."
        } else if demo_mode == DemoMode::GovernedSelection {
            "This run replaces human document exchange with AI-supported convergence, but still selects among the original RFI/RFP vendors."
        } else {
            "The formation still prefers a single primary provider, with escalation paths for special cases."
        }
    })
}

fn problem_solving_tactic(vendors: &[VendorInput], demo_mode: DemoMode) -> serde_json::Value {
    let has_pending_or_failed_compliance = vendors
        .iter()
        .any(|vendor| vendor.compliance_status != "compliant");
    let max_risk = vendors
        .iter()
        .map(|vendor| vendor.risk_score)
        .fold(0.0_f64, f64::max);
    let min_risk = vendors
        .iter()
        .map(|vendor| vendor.risk_score)
        .fold(f64::INFINITY, f64::min);
    let max_cost = vendors
        .iter()
        .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
        .max()
        .unwrap_or(0);
    let min_cost = vendors
        .iter()
        .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
        .min()
        .unwrap_or(0);
    let cost_spread = max_cost.saturating_sub(min_cost);
    let risk_spread = if min_risk.is_finite() {
        max_risk - min_risk
    } else {
        0.0
    };

    let (name, why, surprise_line) = if demo_mode == DemoMode::ParetoBreakout
        && vendors.len() >= 5
        && has_pending_or_failed_compliance
        && cost_spread >= 20_000
    {
        (
            "pareto-breakout-router-search",
            "The requested single-vendor decision looks like a local minimum. Explore whether a multi-provider router gives a better Pareto balance across capability, cost, risk, and governance.",
            "Oboy, it broke out of the single-vendor sandbox and selected a router-first Pareto strategy this time.",
        )
    } else if vendors.len() >= 5 && has_pending_or_failed_compliance && cost_spread >= 20_000 {
        (
            "dijkstra-shortest-governance-path",
            "Many candidates, mixed compliance, and wide cost spread: minimize the path through required evidence and policy gates before ranking.",
            "Oboy, it selected a Dijkstra-style shortest governance path this time.",
        )
    } else if risk_spread >= 20.0 {
        (
            "pareto-frontier-risk-pruning",
            "Risk spread is high: prune dominated candidates before weighted ranking.",
            "It switched to Pareto-frontier risk pruning for this run.",
        )
    } else {
        (
            "weighted-constraint-ranking",
            "The candidate set is already narrow: apply hard constraints, then rank by the objective function.",
            "It stayed with weighted constraint ranking for this run.",
        )
    };

    serde_json::json!({
        "name": name,
        "class": "formation-selected problem-solving tactic",
        "why": why,
        "surprise_line": surprise_line,
        "note": "The formation selects the tactic from problem shape; lower layers still choose concrete providers, models, tools, and algorithms."
    })
}

// ---------------------------------------------------------------------------
// Formation assembly
// ---------------------------------------------------------------------------

fn build_formation_catalog() -> Vec<ProfileSnapshot> {
    vec![
        ProfileSnapshot {
            name: "planning-seed".into(),
            role: SuggestorRole::Planning,
            output_keys: vec![ContextKey::Strategies],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Interactive,
            capabilities: vec![
                SuggestorCapability::KnowledgeRetrieval,
                SuggestorCapability::ExperienceLearning,
            ],
            confidence_min: 0.8,
            confidence_max: 1.0,
        },
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
        ProfileSnapshot {
            name: "policy-gate".into(),
            role: SuggestorRole::Constraint,
            output_keys: vec![ContextKey::Evaluations],
            cost_hint: converge_provider_api::CostClass::Low,
            latency_hint: converge_provider_api::LatencyClass::Realtime,
            capabilities: vec![
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::HumanInTheLoop,
            ],
            confidence_min: 1.0,
            confidence_max: 1.0,
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
                SuggestorRole::Planning,
                SuggestorRole::Analysis,
                SuggestorRole::Evaluation,
                SuggestorRole::Evaluation,
                SuggestorRole::Synthesis,
                SuggestorRole::Synthesis,
                SuggestorRole::Constraint,
            ],
            required_capabilities: vec![
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::Analytics,
                SuggestorCapability::Optimization,
                SuggestorCapability::LlmReasoning,
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
    let mut unmatched_roles = Vec::new();

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
        } else {
            unmatched_roles.push(*role);
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
        unmatched_roles,
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

        let min_cost = self
            .vendors
            .iter()
            .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
            .min()
            .unwrap_or(0);
        let max_cost = self
            .vendors
            .iter()
            .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
            .max()
            .unwrap_or(min_cost);

        qualifying.sort_by(|a, b| {
            vendor_objective_score(b, min_cost, max_cost)
                .partial_cmp(&vendor_objective_score(a, min_cost, max_cost))
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
                let cost_major = amount_major_from_minor(v.monthly_cost_minor);
                let cost_score = cost_efficiency_score(cost_major, min_cost, max_cost);
                let certification_score = certification_coverage_score(v);
                let composite_score = vendor_objective_score(v, min_cost, max_cost);
                serde_json::json!({
                    "rank": rank + 1,
                    "vendor_name": v.name,
                    "score": v.score,
                    "risk_score": v.risk_score,
                    "cost_major": cost_major,
                    "cost_score": cost_score,
                    "certification_score": certification_score,
                    "composite_score": composite_score,
                    "objective": "0.35*capability + 0.25*risk_adjusted + 0.20*cost_efficiency + 0.20*certification_coverage",
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

/// Per-role model overrides for competition/benchmarking.
#[derive(Debug, Clone, Default)]
pub struct ModelOverrides {
    pub compliance: Option<String>,
    pub cost: Option<String>,
    pub risk: Option<String>,
    pub synthesis: Option<String>,
}

impl ModelOverrides {
    pub fn is_empty(&self) -> bool {
        self.compliance.is_none()
            && self.cost.is_none()
            && self.risk.is_none()
            && self.synthesis.is_none()
    }
}

pub async fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    execute_with_experience(store, inputs, persist, None).await
}

pub async fn execute_with_model_overrides(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
    experience: Option<&ExperienceRegistry>,
    overrides: &ModelOverrides,
) -> Result<TruthExecutionResult, String> {
    execute_inner(store, inputs, persist, experience, Some(overrides)).await
}

pub async fn execute_with_experience(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
    experience: Option<&ExperienceRegistry>,
) -> Result<TruthExecutionResult, String> {
    execute_inner(store, inputs, persist, experience, None).await
}

async fn select_live_llm(
    role: &str,
    primary_model: &str,
    fallback_model: &str,
    direct_providers: &[&str],
) -> Result<SelectedLlm, String> {
    match select_llm_for_model(primary_model).await {
        Ok(llm) => Ok(llm),
        Err(primary_error) => match select_llm_for_model(fallback_model).await {
            Ok(llm) => Ok(llm),
            Err(fallback_error) => {
                let mut direct_errors = Vec::new();
                for provider in direct_providers {
                    match select_llm(Some(provider), None).await {
                        Ok(llm) => return Ok(llm),
                        Err(error) => direct_errors.push(format!("{provider}: {error}")),
                    }
                }
                Err(format!(
                    "failed to select {role} LLM ({primary_model}); fallback ({fallback_model}) also failed: primary={primary_error}; fallback={fallback_error}; direct providers failed: {}",
                    direct_errors.join("; ")
                ))
            }
        },
    }
}

fn record_llm_selection_fallback(
    collector: &InMemoryLlmCallCollector,
    role: &str,
    requested_model: &str,
    error: &str,
) {
    let mut metadata = HashMap::new();
    metadata.insert("fallback".to_string(), "true".to_string());
    metadata.insert("stage".to_string(), "model-selection".to_string());
    metadata.insert("requested_model".to_string(), requested_model.to_string());
    metadata.insert("error".to_string(), error.to_string());

    collector.record_llm_call(LlmCallTelemetry {
        context: format!("{role}:model-selection"),
        provider: "none".to_string(),
        model: "deterministic-fallback".to_string(),
        elapsed_ms: 0,
        finish_reason: Some("provider-unavailable".to_string()),
        usage: None,
        metadata,
    });
}

async fn execute_inner(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
    experience: Option<&ExperienceRegistry>,
    model_overrides: Option<&ModelOverrides>,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("vendor-selection").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let vendors = parse_vendors(inputs)?;
    let demo_mode = DemoMode::parse(inputs);
    let strategies = build_planning_seed(&vendors, demo_mode);
    let catalog = build_formation_catalog();
    let principal_authority = parse_authority(inputs);
    let principal_authority_label = principal_authority.as_str().to_string();
    let human_approval_present = parse_optional_bool(inputs, "human_approval_present")?;
    let live_mode = parse_optional_bool(inputs, "live_mode")?.unwrap_or(false);

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

    // Collect prior decisions from experience store
    let prior_context = experience
        .map(|reg| reg.prior_decisions_summary("vendor-selection"))
        .unwrap_or_default();

    // LLM telemetry collector (populated only in live mode)
    let llm_collector = InMemoryLlmCallCollector::default();

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "intent-pack",
        PlanningSeedSuggestor {
            strategies,
            catalog,
        },
    );

    if live_mode {
        load_env();
        let compliance_model = model_overrides
            .and_then(|o| o.compliance.as_deref())
            .unwrap_or(MODEL_FAST);
        let cost_model = model_overrides
            .and_then(|o| o.cost.as_deref())
            .unwrap_or(MODEL_MID);
        let risk_model = model_overrides
            .and_then(|o| o.risk.as_deref())
            .unwrap_or(MODEL_MID);
        let synthesis_model = model_overrides
            .and_then(|o| o.synthesis.as_deref())
            .unwrap_or(MODEL_STRONG);

        let fast_llm = select_live_llm(
            "compliance",
            compliance_model,
            MODEL_FAST_FALLBACK,
            &["gemini", "openai", "anthropic"],
        )
        .await;
        let cost_llm = select_live_llm(
            "cost",
            cost_model,
            MODEL_MID_FALLBACK,
            &["gemini", "openai", "anthropic"],
        )
        .await;
        let risk_llm = select_live_llm(
            "risk",
            risk_model,
            MODEL_MID_FALLBACK,
            &["gemini", "openai", "anthropic"],
        )
        .await;
        let strong_llm = select_live_llm(
            "synthesis",
            synthesis_model,
            MODEL_STRONG_FALLBACK,
            &["anthropic", "openai", "gemini"],
        )
        .await;

        match fast_llm {
            Ok(llm) => {
                engine.register_suggestor_in_pack(
                    "screening-pack",
                    LiveComplianceScreenerAgent {
                        vendors: vendors.clone(),
                        llm,
                        collector: llm_collector.clone(),
                    },
                );
            }
            Err(error) => {
                tracing::warn!(role = "compliance", error = %error, "live LLM unavailable, using deterministic suggestor");
                record_llm_selection_fallback(
                    &llm_collector,
                    "compliance",
                    compliance_model,
                    &error,
                );
                engine.register_suggestor_in_pack(
                    "screening-pack",
                    ComplianceScreenerAgent {
                        vendors: vendors.clone(),
                    },
                );
            }
        }
        match cost_llm {
            Ok(llm) => {
                engine.register_suggestor_in_pack(
                    "evaluation-pack",
                    LiveCostAnalysisAgent {
                        vendors: vendors.clone(),
                        llm,
                        collector: llm_collector.clone(),
                    },
                );
            }
            Err(error) => {
                tracing::warn!(role = "cost", error = %error, "live LLM unavailable, using deterministic suggestor");
                record_llm_selection_fallback(&llm_collector, "cost", cost_model, &error);
                engine.register_suggestor_in_pack(
                    "evaluation-pack",
                    CostAnalysisAgent {
                        vendors: vendors.clone(),
                    },
                );
            }
        }
        match risk_llm {
            Ok(llm) => {
                engine.register_suggestor_in_pack(
                    "evaluation-pack",
                    LiveVendorRiskAgent {
                        vendors: vendors.clone(),
                        llm,
                        collector: llm_collector.clone(),
                    },
                );
            }
            Err(error) => {
                tracing::warn!(role = "risk", error = %error, "live LLM unavailable, using deterministic suggestor");
                record_llm_selection_fallback(&llm_collector, "risk", risk_model, &error);
                engine.register_suggestor_in_pack(
                    "evaluation-pack",
                    VendorRiskAgent {
                        vendors: vendors.clone(),
                    },
                );
            }
        }
        match strong_llm {
            Ok(llm) => {
                engine.register_suggestor_in_pack(
                    "evaluation-pack",
                    LiveDecisionSynthesisAgent {
                        llm,
                        collector: llm_collector.clone(),
                        prior_context: prior_context.clone(),
                    },
                );
            }
            Err(error) => {
                tracing::warn!(role = "synthesis", error = %error, "live LLM unavailable, using deterministic suggestor");
                record_llm_selection_fallback(&llm_collector, "synthesis", synthesis_model, &error);
                engine.register_suggestor_in_pack("evaluation-pack", DecisionSynthesisAgent);
            }
        }
    } else {
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
        engine.register_suggestor_in_pack("evaluation-pack", DecisionSynthesisAgent);
    }

    // Shortlist agent is always deterministic (constraint solver)
    engine.register_suggestor_in_pack(
        "evaluation-pack",
        VendorShortlistAgent {
            vendors: vendors.clone(),
            min_score,
            max_risk,
            max_vendors,
        },
    );
    engine.register_suggestor_in_pack(
        "policy-pack",
        PolicyGateSuggestor {
            vendors: vendors.clone(),
            principal_authority,
            human_approval_present,
            engine: Arc::clone(&policy_engine),
        },
    );

    // Wire experience observer if available
    let experience_stream = experience.map(|reg| reg.get_or_create("vendor-selection"));

    let started_at = std::time::Instant::now();
    let result = engine
        .run_with_types_intent_and_hooks(
            ContextState::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(VendorSelectionEvaluator)),
                event_observer: experience_stream
                    .as_ref()
                    .map(|s| Arc::clone(s) as Arc<dyn converge_kernel::ExperienceEventObserver>),
            },
        )
        .await
        .map_err(|e| format!("convergence failed: {e}"))?;
    let elapsed_ms = started_at.elapsed().as_millis() as u64;

    let source_material = source_material_payload(inputs);
    let projection_details = vendor_selection_projection_details(
        &result.context,
        &vendors,
        ProjectionOptions {
            min_score,
            max_risk,
            max_vendors,
            principal_authority: &principal_authority_label,
            human_approval_present,
            demo_mode,
            source_material: source_material.clone(),
        },
    );

    // Record run summary in experience store for learning
    let confidence = result
        .context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|f| f.id == "decision:recommendation")
        .and_then(|f| serde_json::from_str::<serde_json::Value>(&f.content).ok())
        .and_then(|v| v.get("confidence").and_then(|c| c.as_f64()))
        .unwrap_or(0.5);

    let synthesized_recommendation = result
        .context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|f| f.id == "decision:recommendation")
        .and_then(|f| serde_json::from_str::<serde_json::Value>(&f.content).ok())
        .and_then(|v| {
            v.get("recommendation")
                .and_then(|r| r.as_str())
                .map(String::from)
        })
        .unwrap_or_default();
    let recommended_vendor =
        selected_vendor_name(&result.context).unwrap_or(synthesized_recommendation);

    if let Some(registry) = experience {
        registry.record_run_summary(
            "vendor-selection",
            RunSummaryInput {
                cycles: result.cycles,
                elapsed_ms,
                vendor_count: vendors.len(),
                converged: result.converged,
                confidence,
                recommended_vendor: &recommended_vendor,
                source_document_path: source_material
                    .pointer("/source_document/path")
                    .and_then(serde_json::Value::as_str),
                static_fact_count: source_material
                    .pointer("/static_facts/fact_count")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or_default() as usize,
                static_fact_paths: source_material
                    .pointer("/static_facts/paths")
                    .and_then(serde_json::Value::as_array)
                    .map(|paths| {
                        paths
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .map(ToString::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
            },
        );
    }

    // Build learning metrics from prior runs
    let learning_metrics =
        experience.map(|reg| reg.learning_metrics("vendor-selection", result.cycles, confidence));

    // Merge learning metrics into projection details
    let enriched_details = match (projection_details, learning_metrics) {
        (Some(mut details), Some(metrics)) => {
            if let Some(obj) = details.as_object_mut() {
                obj.insert("learning".to_string(), metrics);
            }
            Some(details)
        }
        (Some(details), None) => Some(details),
        (None, Some(metrics)) => Some(serde_json::json!({ "learning": metrics })),
        (None, None) => None,
    };

    let projection = if persist {
        let policy = policy_decision_payload(&result.context);
        let write_result = store
            .write_with_events(|kernel| {
                let actor = Actor::agent("vendor-selection");

                for vendor in &vendors {
                    kernel.register_vendor(
                        vendor.name.clone(),
                        format!("score={}, risk={}", vendor.score, vendor.risk_score),
                        &actor,
                    );
                }

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
            details: enriched_details,
        })
    } else {
        enriched_details.map(|details| super::TruthProjection {
            events_emitted: 0,
            details: Some(details),
        })
    };

    let llm_calls_snapshot = if live_mode {
        let calls = llm_collector.snapshot();
        if calls.is_empty() { None } else { Some(calls) }
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
        llm_calls: llm_calls_snapshot,
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

struct ProjectionOptions<'a> {
    min_score: f64,
    max_risk: f64,
    max_vendors: usize,
    principal_authority: &'a str,
    human_approval_present: Option<bool>,
    demo_mode: DemoMode,
    source_material: serde_json::Value,
}

fn vendor_selection_projection_details(
    ctx: &ContextState,
    vendors: &[VendorInput],
    options: ProjectionOptions<'_>,
) -> Option<serde_json::Value> {
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
        "formation": formation_plan_payload(ctx),
        "agents": vendor_selection_agent_roster(),
        "root_intent": root_intent_payload(
            vendors,
            options.min_score,
            options.max_risk,
            options.max_vendors,
            options.principal_authority,
            options.demo_mode,
        ),
        "resources": resource_payload(vendors, options.human_approval_present),
        "source_material": options.source_material.clone(),
        "invariants": invariant_payload(vendors, options.min_score, options.max_risk),
        "optimization": optimization_payload(
            vendors,
            options.min_score,
            options.max_risk,
            options.max_vendors,
        ),
        "fixed_point": fixed_point_payload(ctx),
        "stack_pressure": stack_pressure_payload(ctx, vendors, &options),
        "context": {
            "strategies": fact_views(ctx, ContextKey::Strategies),
            "seeds": fact_views(ctx, ContextKey::Seeds),
            "evaluations": fact_views(ctx, ContextKey::Evaluations),
            "proposals": fact_views(ctx, ContextKey::Proposals),
        },
    }))
}

fn root_intent_payload(
    vendors: &[VendorInput],
    min_score: f64,
    max_risk: f64,
    max_vendors: usize,
    principal_authority: &str,
    demo_mode: DemoMode,
) -> serde_json::Value {
    let vendor_names: Vec<_> = vendors.iter().map(|vendor| vendor.name.as_str()).collect();
    serde_json::json!({
        "statement": "Select a preferred AI vendor with auditable rationale, bounded authority, and evidence-backed policy compliance.",
        "outcome": "A ranked vendor recommendation or an honest escalation/rejection.",
        "demo_mode": {
            "id": demo_mode.as_str(),
            "label": demo_mode.label(),
            "thesis": demo_mode.thesis(),
            "selection_boundary": demo_mode.selection_boundary(),
        },
        "candidate_vendors": vendor_names,
        "authority": {
            "principal": "user:procurement-lead",
            "level": principal_authority,
            "domain": "procurement",
        },
        "objective": {
            "max_vendors": max_vendors,
            "min_score": min_score,
            "max_risk": max_risk,
        },
    })
}

fn resource_payload(
    vendors: &[VendorInput],
    human_approval_present: Option<bool>,
) -> serde_json::Value {
    let total_monthly_cost_major: i64 = vendors
        .iter()
        .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
        .sum();
    serde_json::json!({
        "candidate_count": vendors.len(),
        "evidence_channels": ["declared vendor response", "compliance screen", "cost model", "risk model", "Cedar policy"],
        "agent_roles": ["planning", "formation", "compliance", "cost", "risk", "optimization", "synthesis", "policy"],
        "compute_budget": {
            "max_cycles": 10,
            "live_llm_optional": true,
            "deterministic_fallback": true,
        },
        "financial_boundary": {
            "hitl_threshold_major": HITL_THRESHOLD_MAJOR,
            "candidate_monthly_cost_major": total_monthly_cost_major,
        },
        "human_approval_present": human_approval_present,
    })
}

fn source_material_payload(inputs: &HashMap<String, String>) -> serde_json::Value {
    let source_document_path = super::common::optional_input(inputs, "source_document_path");
    let source_document = super::common::optional_input(inputs, "source_document");
    let source_document_lines = source_document
        .as_deref()
        .map(|value| value.lines().count())
        .unwrap_or_default();
    let source_document_bytes = source_document
        .as_ref()
        .map(String::len)
        .unwrap_or_default();
    let static_fact_paths = super::common::optional_input(inputs, "static_facts_paths_json")
        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
        .unwrap_or_default();
    let static_facts = super::common::optional_input(inputs, "static_facts_json")
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or(serde_json::Value::Null);
    let static_fact_count = static_fact_count(&static_facts);

    serde_json::json!({
        "source_document": {
            "present": source_document_path.is_some() || source_document.is_some(),
            "path": source_document_path,
            "line_count": source_document_lines,
            "byte_count": source_document_bytes,
        },
        "static_facts": {
            "present": static_fact_count > 0,
            "paths": static_fact_paths,
            "fact_count": static_fact_count,
        },
    })
}

fn static_fact_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(files) => files
            .iter()
            .map(|file| {
                file.get("content")
                    .map(count_fact_value)
                    .unwrap_or_else(|| count_fact_value(file))
            })
            .sum(),
        other => count_fact_value(other),
    }
}

fn count_fact_value(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Null => 0,
        serde_json::Value::Array(items) => items.len(),
        serde_json::Value::Object(map) => map
            .get("facts")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or(1),
        _ => 1,
    }
}

fn invariant_payload(vendors: &[VendorInput], min_score: f64, max_risk: f64) -> serde_json::Value {
    let screened = vendors.len();
    serde_json::json!([
        {
            "id": "score-floor",
            "statement": format!("A shortlisted vendor must have score >= {min_score:.0}."),
            "owned_by": "constraint solver",
        },
        {
            "id": "risk-ceiling",
            "statement": format!("A shortlisted vendor must have risk <= {max_risk:.0}."),
            "owned_by": "risk model",
        },
        {
            "id": "compliance-required",
            "statement": "A shortlisted vendor must be compliant before commitment.",
            "owned_by": "compliance screener",
        },
        {
            "id": "policy-gated-commitment",
            "statement": format!("Commitments above ${HITL_THRESHOLD_MAJOR} require human approval."),
            "owned_by": "Cedar policy",
        },
        {
            "id": "provenance",
            "statement": format!("All {screened} candidates must leave promoted facts with provenance."),
            "owned_by": "Converge promotion gate",
        }
    ])
}

fn optimization_payload(
    vendors: &[VendorInput],
    min_score: f64,
    max_risk: f64,
    max_vendors: usize,
) -> serde_json::Value {
    let min_cost = vendors
        .iter()
        .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
        .min()
        .unwrap_or(0);
    let max_cost = vendors
        .iter()
        .map(|vendor| amount_major_from_minor(vendor.monthly_cost_minor))
        .max()
        .unwrap_or(min_cost);
    let mut rows: Vec<_> = vendors
        .iter()
        .map(|vendor| {
            let cost_major = amount_major_from_minor(vendor.monthly_cost_minor);
            let feasible = vendor.compliance_status == "compliant"
                && vendor.score >= min_score
                && vendor.risk_score <= max_risk;
            serde_json::json!({
                "vendor": vendor.name,
                "feasible": feasible,
                "score": vendor.score,
                "risk": vendor.risk_score,
                "cost_major": cost_major,
                "cost_score": cost_efficiency_score(cost_major, min_cost, max_cost),
                "certification_score": certification_coverage_score(vendor),
                "objective_score": vendor_objective_score(vendor, min_cost, max_cost),
                "pareto_frontier": is_pareto_frontier_vendor(vendor, vendors),
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        let a_score = a
            .get("objective_score")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        let b_score = b
            .get("objective_score")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    serde_json::json!({
        "solver": "deterministic constraint optimizer",
        "objective": "maximize 0.35*capability + 0.25*risk_adjusted + 0.20*cost_efficiency + 0.20*certification_coverage",
        "hard_constraints": [
            format!("score >= {min_score:.0}"),
            format!("risk <= {max_risk:.0}"),
            "compliance_status == compliant".to_string(),
            format!("selected_count <= {max_vendors}"),
        ],
        "rows": rows,
    })
}

fn is_pareto_frontier_vendor(vendor: &VendorInput, vendors: &[VendorInput]) -> bool {
    let cost = amount_major_from_minor(vendor.monthly_cost_minor);
    !vendors.iter().any(|other| {
        if other.name == vendor.name {
            return false;
        }
        let other_cost = amount_major_from_minor(other.monthly_cost_minor);
        let at_least_as_good = other.score >= vendor.score
            && other.risk_score <= vendor.risk_score
            && other_cost <= cost;
        let strictly_better =
            other.score > vendor.score || other.risk_score < vendor.risk_score || other_cost < cost;
        at_least_as_good && strictly_better
    })
}

fn fixed_point_payload(ctx: &ContextState) -> serde_json::Value {
    serde_json::json!({
        "definition": "No suggestor can propose a new promotable fact under the current context, budget, authority, and policy gates.",
        "fact_counts": {
            "strategies": ctx.get(ContextKey::Strategies).len(),
            "seeds": ctx.get(ContextKey::Seeds).len(),
            "evaluations": ctx.get(ContextKey::Evaluations).len(),
            "proposals": ctx.get(ContextKey::Proposals).len(),
        },
        "terminal_facts": [
            "vendor:shortlist",
            "decision:recommendation",
            "policy:decision:vendor-selection"
        ],
    })
}

fn stack_pressure_payload(
    ctx: &ContextState,
    vendors: &[VendorInput],
    options: &ProjectionOptions<'_>,
) -> serde_json::Value {
    let feasible_count = vendors
        .iter()
        .filter(|vendor| {
            vendor.compliance_status == "compliant"
                && vendor.score >= options.min_score
                && vendor.risk_score <= options.max_risk
        })
        .count();
    let frontier_count = vendors
        .iter()
        .filter(|vendor| is_pareto_frontier_vendor(vendor, vendors))
        .count();
    let promoted_fact_count = ctx.get(ContextKey::Strategies).len()
        + ctx.get(ContextKey::Seeds).len()
        + ctx.get(ContextKey::Evaluations).len()
        + ctx.get(ContextKey::Proposals).len();
    let policy_outcome = ctx
        .get(ContextKey::Evaluations)
        .iter()
        .find(|fact| fact.id == "policy:decision:vendor-selection")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok())
        .and_then(|payload| {
            payload
                .get("outcome")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "pending".to_string());

    serde_json::json!([
        {
            "layer": "Helm",
            "version": "0.1.0",
            "contract": "Operator workbench for source packs, truth execution, evidence inspection, and participant editing.",
            "demo_signal": format!("{} candidates, {} promoted facts, policy outcome {}.", vendors.len(), promoted_fact_count, policy_outcome),
            "pressure": "Make source-pack edits, timeline playback, and what-happened-why inspection first-class in the desktop lab.",
        },
        {
            "layer": "Axiom",
            "version": "0.7.0",
            "contract": "Normative truth contract: invariants, acceptance criteria, examples, and policy lens before execution.",
            "demo_signal": "Score floor, risk ceiling, compliance, HITL, and provenance invariants are projected for the run.",
            "pressure": "Compile the visible invariants and Cedar clauses from editable truth artifacts, then return structured diagnostics.",
        },
        {
            "layer": "Organism",
            "version": "1.4.0",
            "contract": "Intent, planning seed, formation assembly, topology choice, and strategy proposals.",
            "demo_signal": format!("{} formation roles assigned with mode {}.", formation_role_count(ctx), options.demo_mode.as_str()),
            "pressure": "Promote panel, huddle, adversarial, and self-organizing topologies from demo labels into typed plan bundles.",
        },
        {
            "layer": "Converge",
            "version": "3.7.4",
            "contract": "Engine cycles, context partitions, promoted facts, criteria, policy decisions, and fixed-point stop reasons.",
            "demo_signal": format!("{} strategies, {} seeds, {} evaluations, {} proposal fact(s).", ctx.get(ContextKey::Strategies).len(), ctx.get(ContextKey::Seeds).len(), ctx.get(ContextKey::Evaluations).len(), ctx.get(ContextKey::Proposals).len()),
            "pressure": "Expose richer criterion evidence, promotion traces, and stop reasons so participants can debug governance runs quickly.",
        },
        {
            "layer": "Ferrox",
            "version": "0.3.12",
            "contract": "Optimization substrate for feasible sets, Pareto frontier analysis, and MIP/CP-SAT style decisions.",
            "demo_signal": format!("{} feasible candidates, {} Pareto frontier candidates, shortlist cap {}.", feasible_count, frontier_count, options.max_vendors),
            "pressure": "Replace local weighted ranking with a Ferrox MIP/Pareto suggestor once the participant dependency stays fast to build.",
        },
    ])
}

fn formation_role_count(ctx: &ContextState) -> usize {
    formation_plan_payload(ctx)
        .and_then(|plan| {
            plan.get("assignments")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len)
        })
        .unwrap_or_default()
}

fn formation_plan_payload(ctx: &ContextState) -> Option<serde_json::Value> {
    ctx.get(ContextKey::Strategies)
        .iter()
        .find(|fact| fact.id == "formation:plan:vendor-selection")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok())
}

fn fact_views(ctx: &ContextState, key: ContextKey) -> Vec<serde_json::Value> {
    ctx.get(key)
        .iter()
        .map(|fact| {
            let content = serde_json::from_str::<serde_json::Value>(&fact.content)
                .unwrap_or_else(|_| serde_json::Value::String(fact.content.clone()));
            serde_json::json!({
                "key": format!("{:?}", key),
                "id": fact.id.as_str(),
                "content": content,
                "promotion": fact.promotion_record(),
            })
        })
        .collect()
}

fn vendor_selection_agent_roster() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "planning-seed",
            "pack": "intent-pack",
            "class": "Organism planning",
            "role": "Intent and formation",
            "model": "local deterministic",
            "output": "Strategies + formation plan"
        },
        {
            "id": "compliance-screener",
            "pack": "screening-pack",
            "class": "Policy analysis",
            "role": "Compliance screening",
            "model": "local deterministic",
            "output": "Compliance evidence"
        },
        {
            "id": "cost-analysis",
            "pack": "evaluation-pack",
            "class": "Analytics",
            "role": "Cost evaluation",
            "model": "local deterministic",
            "output": "Comparable cost facts"
        },
        {
            "id": "vendor-risk",
            "pack": "evaluation-pack",
            "class": "Risk model",
            "role": "Risk scoring",
            "model": "local deterministic",
            "output": "Risk evaluations"
        },
        {
            "id": "vendor-shortlist",
            "pack": "evaluation-pack",
            "class": "Optimization",
            "role": "Shortlist ranking",
            "model": "constraint solver",
            "output": "Ranked vendor proposal"
        },
        {
            "id": "decision-synthesis",
            "pack": "evaluation-pack",
            "class": "LLM reasoning",
            "role": "Decision synthesis",
            "model": "offline stub for hackathon baseline",
            "output": "Recommendation proposal"
        },
        {
            "id": "policy-gate",
            "pack": "policy-pack",
            "class": "Policy agent",
            "role": "Cedar authorization",
            "model": "Cedar policy engine",
            "output": "Promote / Escalate / Reject"
        }
    ])
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
    async fn vendor_selection_projection_exposes_foundation_pressure() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);

        let result = execute(&store, &inputs, false).await.unwrap();
        let details = result
            .projection
            .and_then(|projection| projection.details)
            .expect("projection details");
        let rows = details
            .get("stack_pressure")
            .and_then(serde_json::Value::as_array)
            .expect("stack pressure rows");
        let layers: Vec<_> = rows
            .iter()
            .filter_map(|row| row.get("layer").and_then(serde_json::Value::as_str))
            .collect();

        for expected in ["Helm", "Axiom", "Organism", "Converge", "Ferrox"] {
            assert!(
                layers.contains(&expected),
                "expected stack pressure for {expected}"
            );
        }
        assert!(
            rows.iter().any(|row| {
                row.get("pressure")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|pressure| pressure.contains("Ferrox MIP/Pareto suggestor"))
            }),
            "Ferrox pressure should name the missing optimizer handoff"
        );
    }

    #[tokio::test]
    async fn vendor_selection_projection_exposes_source_material() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([
            ("vendors".into(), "Acme AI, Beta ML".into()),
            (
                "source_document_path".into(),
                "examples/vendor-selection/buyer-brief.md".into(),
            ),
            (
                "source_document".into(),
                "Line one\nLine two\nLine three".into(),
            ),
            (
                "static_facts_paths_json".into(),
                serde_json::json!(["examples/vendor-selection/static-facts.json"]).to_string(),
            ),
            (
                "static_facts_json".into(),
                serde_json::json!([
                    {
                        "path": "examples/vendor-selection/static-facts.json",
                        "content": {
                            "facts": [
                                {"id": "fact:one", "statement": "One"},
                                {"id": "fact:two", "statement": "Two"}
                            ]
                        }
                    }
                ])
                .to_string(),
            ),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        let details = result
            .projection
            .and_then(|projection| projection.details)
            .expect("projection details");

        assert_eq!(
            details
                .pointer("/source_material/source_document/path")
                .and_then(serde_json::Value::as_str),
            Some("examples/vendor-selection/buyer-brief.md")
        );
        assert_eq!(
            details
                .pointer("/source_material/source_document/line_count")
                .and_then(serde_json::Value::as_u64),
            Some(3)
        );
        assert_eq!(
            details
                .pointer("/source_material/static_facts/fact_count")
                .and_then(serde_json::Value::as_u64),
            Some(2)
        );
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

    #[tokio::test]
    async fn pareto_breakout_projects_router_hypothesis() {
        let store = InMemoryStore::new();
        let vendors_json = serde_json::json!([
            {
                "name": "Acme AI",
                "score": 85.0,
                "risk_score": 15.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2", "ISO27001", "GDPR"],
                "monthly_cost_minor": 4200000,
                "currency_code": "USD"
            },
            {
                "name": "Beta ML",
                "score": 78.0,
                "risk_score": 25.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2", "HIPAA"],
                "monthly_cost_minor": 2800000,
                "currency_code": "USD"
            },
            {
                "name": "Gamma LLM",
                "score": 92.0,
                "risk_score": 35.0,
                "compliance_status": "pending",
                "certifications": ["ISO27001"],
                "monthly_cost_minor": 6500000,
                "currency_code": "USD"
            },
            {
                "name": "Delta Systems",
                "score": 70.0,
                "risk_score": 10.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2", "ISO27001", "GDPR", "FedRAMP"],
                "monthly_cost_minor": 5500000,
                "currency_code": "USD"
            },
            {
                "name": "Epsilon AI",
                "score": 88.0,
                "risk_score": 20.0,
                "compliance_status": "compliant",
                "certifications": ["SOC2", "GDPR"],
                "monthly_cost_minor": 3800000,
                "currency_code": "USD"
            }
        ]);
        let inputs = HashMap::from([
            ("vendors_json".into(), vendors_json.to_string()),
            ("demo_mode".into(), "pareto-breakout".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        let details = result
            .projection
            .and_then(|projection| projection.details)
            .expect("projection details");
        let router = details
            .pointer("/context/strategies")
            .and_then(|value| value.as_array())
            .and_then(|facts| {
                facts.iter().find(|fact| {
                    fact.get("id").and_then(serde_json::Value::as_str)
                        == Some("strategy:vendor-sel:router-hypothesis")
                })
            })
            .and_then(|fact| fact.get("content"))
            .expect("router hypothesis");

        assert_eq!(
            details
                .pointer("/root_intent/demo_mode/id")
                .and_then(serde_json::Value::as_str),
            Some("pareto-breakout")
        );
        assert_eq!(
            router
                .get("router_fit")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
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
