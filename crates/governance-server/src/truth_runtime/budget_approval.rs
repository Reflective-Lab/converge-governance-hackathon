//! Truth executor: budget-approval
//!
//! Demonstrates threshold-based approval workflow with HITL (Human-in-the-Loop) gates.
//! 4-tier approval system:
//! - Tier 1: < $5,000 — auto-approve (no HITL)
//! - Tier 2: $5k–$25k — supervisory authority required + human approval
//! - Tier 3: $25k–$100k — director/sovereign authority required + human approval
//! - Tier 4: > $100k — forbid unless sovereign authority present

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use converge_kernel::{
    AuthorityLevel, Context, ContextKey, ContextState, Criterion, CriterionEvaluator,
    CriterionResult, Engine, FlowAction, FlowPhase, TypesRunHooks,
};
use converge_pack::{
    AgentEffect, Context as ContextView, DomainId, GateId, PrincipalId, ProposedFact, ResourceId,
    ResourceKind, Suggestor,
};
use converge_policy::{
    ContextIn, DecideRequest, PolicyEngine, PolicyOutcome, PrincipalIn, ResourceIn,
};
use governance_kernel::{Actor, ActorKind, DecisionRecord, InMemoryStore};
use governance_truths::{build_intent, find_truth};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TruthExecutionResult, TruthProjection};

const POLICY_PACK: &str = "policy-pack";
const REQUEST_PROVENANCE: &str = "governance-server:budget-approval";
const POLICY_PROVENANCE: &str = "policy:budget-approval";
const POLICY_TEXT: &str =
    include_str!("../../../../examples/budget-approval/budget-approval-policy.cedar");

// ---------------------------------------------------------------------------
// Suggestors for budget approval workflow
// ---------------------------------------------------------------------------

struct BudgetRequestValidatorSuggestor {
    request: BudgetApprovalRequest,
}

#[async_trait]
impl Suggestor for BudgetRequestValidatorSuggestor {
    fn name(&self) -> &str {
        "budget-request-validator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Seeds)
            .iter()
            .any(|fact| fact.id().as_str() == validation_fact_id(&self.request.request_id))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let errors = self.validate();
        let content = if errors.is_empty() {
            serde_json::json!({
                "request_id": self.request.request_id,
                "requester_id": self.request.requester_id,
                "amount_minor": self.request.amount_minor,
                "currency_code": self.request.currency_code,
                "authority": self.request.authority,
                "status": "valid",
                "errors": []
            })
        } else {
            serde_json::json!({
                "request_id": self.request.request_id,
                "requester_id": self.request.requester_id,
                "amount_minor": self.request.amount_minor,
                "currency_code": self.request.currency_code,
                "authority": self.request.authority,
                "status": "invalid",
                "errors": errors
            })
        }
        .to_string();

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Seeds,
                validation_fact_id(&self.request.request_id),
                content,
                REQUEST_PROVENANCE,
            )
            .with_confidence(1.0),
        )
    }
}

impl BudgetRequestValidatorSuggestor {
    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.request.request_id.trim().is_empty() {
            errors.push("request_id is required".to_string());
        }
        if self.request.requester_id.trim().is_empty() {
            errors.push("requester_id is required".to_string());
        }
        if self.request.amount_minor < 0 {
            errors.push("amount_minor must be non-negative".to_string());
        }
        if self.request.currency_code.trim().is_empty() {
            errors.push("currency_code is required".to_string());
        }
        if !matches!(
            self.request.authority.as_str(),
            "advisory" | "supervisory" | "participatory" | "sovereign"
        ) {
            errors.push(format!(
                "authority must be advisory, supervisory, participatory, or sovereign, got: {}",
                self.request.authority
            ));
        }

        errors
    }
}

struct BudgetPolicySuggestor {
    request: BudgetApprovalRequest,
    engine: Arc<PolicyEngine>,
}

#[async_trait]
impl Suggestor for BudgetPolicySuggestor {
    fn name(&self) -> &str {
        "budget-policy"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|fact| fact.id().as_str() == policy_fact_id(&self.request.request_id))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let content = match self.engine.evaluate(&self.request.to_decide_request()) {
            Ok(decision) => match serde_json::to_string(&PolicyDecisionFact::from_parts(
                &self.request,
                decision,
            )) {
                Ok(content) => content,
                Err(err) => serde_json::json!({
                    "request_id": self.request.request_id,
                    "requester_id": self.request.requester_id,
                    "amount_minor": self.request.amount_minor,
                    "currency_code": self.request.currency_code,
                    "authority": self.request.authority,
                    "outcome": "reject",
                    "reason": format!("serialization failed: {err}"),
                })
                .to_string(),
            },
            Err(err) => serde_json::json!({
                "request_id": self.request.request_id,
                "requester_id": self.request.requester_id,
                "amount_minor": self.request.amount_minor,
                "currency_code": self.request.currency_code,
                "authority": self.request.authority,
                "outcome": "reject",
                "reason": format!("policy evaluation failed: {err}"),
            })
            .to_string(),
        };

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                policy_fact_id(&self.request.request_id),
                content,
                POLICY_PROVENANCE,
            )
            .with_confidence(1.0),
        )
    }
}

struct BudgetApprovalGateSuggestor {
    request: BudgetApprovalRequest,
}

#[async_trait]
impl Suggestor for BudgetApprovalGateSuggestor {
    fn name(&self) -> &str {
        "budget-approval-gate"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|fact| fact.id().as_str() == gate_fact_id(&self.request.request_id))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let policy_decision = find_policy_decision(ctx);

        let (gate_status, gate_reason) = match policy_decision {
            Some(decision) => match decision.outcome {
                PolicyOutcome::Promote => ("passed", "policy permits auto-approval".to_string()),
                PolicyOutcome::Escalate => {
                    let tier = determine_tier(self.request.amount_minor);
                    (
                        "escalated",
                        format!("Tier {} approval requires human review", tier),
                    )
                }
                PolicyOutcome::Reject => (
                    "blocked",
                    decision
                        .reason
                        .clone()
                        .unwrap_or_else(|| "amount exceeds authorization limits".to_string()),
                ),
            },
            None => ("unmet", "no policy decision available".to_string()),
        };

        let content = serde_json::json!({
            "request_id": self.request.request_id,
            "requester_id": self.request.requester_id,
            "amount_minor": self.request.amount_minor,
            "tier": determine_tier(self.request.amount_minor),
            "gate_status": gate_status,
            "gate_reason": gate_reason,
        })
        .to_string();

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                gate_fact_id(&self.request.request_id),
                content,
                REQUEST_PROVENANCE,
            )
            .with_confidence(1.0),
        )
    }
}

struct AuditLoggerSuggestor {
    request: BudgetApprovalRequest,
}

#[async_trait]
impl Suggestor for AuditLoggerSuggestor {
    fn name(&self) -> &str {
        "audit-logger"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Proposals)
            .iter()
            .any(|fact| fact.id().as_str() == audit_fact_id(&self.request.request_id))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let policy_decision = find_policy_decision(ctx);
        let recommendation = match policy_decision.as_ref().map(|d| d.outcome) {
            Some(PolicyOutcome::Promote) => format!(
                "Auto-approve budget request {} for {}",
                self.request.request_id, self.request.currency_code
            ),
            Some(PolicyOutcome::Escalate) => format!(
                "Escalate budget request {} for human approval",
                self.request.request_id
            ),
            Some(PolicyOutcome::Reject) | None => {
                format!("Reject budget request {}", self.request.request_id)
            }
        };

        let rationale = policy_decision
            .as_ref()
            .and_then(|d| d.reason.clone())
            .unwrap_or_else(|| {
                format!(
                    "{} {} {}",
                    self.request.authority,
                    determine_tier(self.request.amount_minor),
                    self.request.amount_minor
                )
            });

        let content = serde_json::json!({
            "request_id": self.request.request_id,
            "requester_id": self.request.requester_id,
            "amount_minor": self.request.amount_minor,
            "currency_code": self.request.currency_code,
            "authority": self.request.authority,
            "tier": determine_tier(self.request.amount_minor),
            "recommendation": recommendation,
            "rationale": rationale,
            "timestamp": Utc::now().to_rfc3339(),
        })
        .to_string();

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Proposals,
                audit_fact_id(&self.request.request_id),
                content,
                REQUEST_PROVENANCE,
            )
            .with_confidence(1.0),
        )
    }
}

// ---------------------------------------------------------------------------
// Evaluator for budget approval criteria
// ---------------------------------------------------------------------------

struct BudgetApprovalEvaluator;

impl CriterionEvaluator for BudgetApprovalEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &dyn Context) -> CriterionResult {
        match criterion.id.as_str() {
            "policy-decision-produced" => {
                if find_policy_decision_from_context(context).is_some() {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no policy decision fact has been produced".into(),
                    }
                }
            }
            "request-approved-or-blocked" => match find_policy_decision_from_context(context) {
                Some(decision) => match decision.outcome {
                    PolicyOutcome::Promote => CriterionResult::Met { evidence: vec![] },
                    PolicyOutcome::Escalate => {
                        let approval_ref = format!("approval:budget:{}", decision.request_id);
                        CriterionResult::Blocked {
                            reason: decision
                                .reason
                                .clone()
                                .unwrap_or_else(|| "human approval required".into()),
                            approval_ref: Some(approval_ref.into()),
                        }
                    }
                    PolicyOutcome::Reject => CriterionResult::Unmet {
                        reason: decision
                            .reason
                            .clone()
                            .unwrap_or_else(|| "budget request was rejected".into()),
                    },
                },
                None => CriterionResult::Unmet {
                    reason: "policy decision not available".into(),
                },
            },
            "audit-entry-recorded" => {
                if find_audit_entry(context).is_some() {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no audit entry has been recorded".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}

// ---------------------------------------------------------------------------
// Budget approval request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetApprovalRequest {
    pub request_id: String,
    pub requester_id: String,
    pub amount_minor: i64,
    pub currency_code: String,
    pub authority: String,
    pub description: String,
    pub human_approval_present: bool,
}

impl BudgetApprovalRequest {
    fn from_inputs(inputs: &HashMap<String, String>) -> Result<Self, String> {
        let authority = super::common::required_input(inputs, "authority")?;
        if !matches!(
            authority,
            "advisory" | "supervisory" | "participatory" | "sovereign"
        ) {
            return Err(format!(
                "unsupported authority {authority}; expected advisory, supervisory, participatory, or sovereign"
            ));
        }

        Ok(Self {
            request_id: super::common::required_input(inputs, "request_id")?.to_string(),
            requester_id: super::common::required_input(inputs, "requester_id")?.to_string(),
            amount_minor: parse_i64(inputs, "amount_minor")?,
            currency_code: inputs
                .get("currency_code")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "USD".to_string()),
            authority: authority.to_string(),
            description: inputs
                .get("description")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "Budget request".to_string()),
            human_approval_present: parse_bool(inputs, "human_approval_present")?,
        })
    }

    fn to_decide_request(&self) -> DecideRequest {
        DecideRequest {
            principal: PrincipalIn {
                id: PrincipalId::new(self.requester_id.clone()),
                authority: parse_authority(&self.authority),
                domains: vec![DomainId::new("budget".to_string())],
                policy_version: None,
            },
            resource: ResourceIn {
                id: ResourceId::new(self.request_id.clone()),
                resource_type: Some(ResourceKind::new("budget_request".to_string())),
                phase: Some(FlowPhase::Convergence),
                gates_passed: Some(if self.human_approval_present {
                    vec![GateId::new("human_approval")]
                } else {
                    Vec::new()
                }),
            },
            action: FlowAction::Promote,
            context: Some(ContextIn {
                amount: Some(self.amount_minor),
                human_approval_present: Some(self.human_approval_present),
                commitment_type: Some("budget_request".to_string()),
                ..Default::default()
            }),
            delegation_b64: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyDecisionFact {
    request_id: String,
    requester_id: String,
    amount_minor: i64,
    currency_code: String,
    authority: String,
    tier: u8,
    outcome: PolicyOutcome,
    reason: Option<String>,
}

impl PolicyDecisionFact {
    fn from_parts(
        request: &BudgetApprovalRequest,
        decision: converge_policy::PolicyDecision,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            requester_id: request.requester_id.clone(),
            amount_minor: request.amount_minor,
            currency_code: request.currency_code.clone(),
            authority: request.authority.clone(),
            tier: determine_tier(request.amount_minor),
            outcome: decision.outcome,
            reason: decision.reason,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn determine_tier(amount_minor: i64) -> u8 {
    if amount_minor < 500_000 {
        1 // < $5,000
    } else if amount_minor < 2_500_000 {
        2 // $5k–$25k
    } else if amount_minor < 10_000_000 {
        3 // $25k–$100k
    } else {
        4 // > $100k
    }
}

fn parse_authority(authority: &str) -> AuthorityLevel {
    match authority {
        "advisory" => AuthorityLevel::Advisory,
        "supervisory" => AuthorityLevel::Supervisory,
        "participatory" => AuthorityLevel::Participatory,
        "sovereign" => AuthorityLevel::Sovereign,
        _ => AuthorityLevel::Advisory,
    }
}

fn parse_i64(inputs: &HashMap<String, String>, key: &str) -> Result<i64, String> {
    inputs
        .get(key)
        .map(|value| value.trim())
        .ok_or_else(|| format!("missing required input: {key}"))?
        .parse::<i64>()
        .map_err(|_| format!("invalid integer for {key}"))
}

fn parse_bool(inputs: &HashMap<String, String>, key: &str) -> Result<bool, String> {
    inputs
        .get(key)
        .map(|value| value.trim().to_lowercase())
        .unwrap_or_else(|| "false".to_string())
        .parse::<bool>()
        .map_err(|_| format!("invalid boolean for {key}"))
}

fn validation_fact_id(request_id: &str) -> String {
    format!("validation:{}", request_id)
}

fn policy_fact_id(request_id: &str) -> String {
    format!("policy:decision:{}", request_id)
}

fn gate_fact_id(request_id: &str) -> String {
    format!("gate:{}", request_id)
}

fn audit_fact_id(request_id: &str) -> String {
    format!("audit:{}", request_id)
}

fn request_fact_id(request_id: &str) -> String {
    format!("request:{}", request_id)
}

fn find_policy_decision(context: &dyn ContextView) -> Option<PolicyDecisionFact> {
    context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|fact| fact.id().starts_with("policy:decision:"))
        .and_then(|fact| serde_json::from_str(fact.content()).ok())
}

fn find_policy_decision_from_context(context: &dyn Context) -> Option<PolicyDecisionFact> {
    context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|fact| fact.id().starts_with("policy:decision:"))
        .and_then(|fact| serde_json::from_str(fact.content()).ok())
}

fn find_audit_entry(context: &dyn Context) -> Option<serde_json::Value> {
    context
        .get(ContextKey::Proposals)
        .iter()
        .find(|fact| fact.id().starts_with("audit:"))
        .and_then(|fact| serde_json::from_str(fact.content()).ok())
}

fn actor_from_principal(principal_id: &str) -> Actor {
    let (name, kind) = if principal_id.starts_with("agent:") {
        (
            principal_id.trim_start_matches("agent:").to_string(),
            ActorKind::Agent,
        )
    } else if principal_id == "system" {
        ("System".to_string(), ActorKind::System)
    } else {
        (
            principal_id
                .trim_start_matches("user:")
                .trim_start_matches("human:")
                .to_string(),
            ActorKind::Human,
        )
    };
    Actor {
        id: principal_id.to_string(),
        kind,
        name,
    }
}

// ---------------------------------------------------------------------------
// Main executor
// ---------------------------------------------------------------------------

pub async fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("budget-approval").ok_or("truth not found")?;
    let intent = build_intent(truth);
    let request = BudgetApprovalRequest::from_inputs(inputs)?;
    let policy_engine = Arc::new(
        PolicyEngine::from_policy_str(POLICY_TEXT)
            .map_err(|err| format!("policy setup failed: {err}"))?,
    );

    let mut initial_context = ContextState::new();
    initial_context
        .add_input_with_provenance(
            ContextKey::Seeds,
            request_fact_id(&request.request_id),
            serde_json::to_string(&request)
                .map_err(|err| format!("failed to serialize request: {err}"))?,
            REQUEST_PROVENANCE,
        )
        .map_err(|err| format!("failed to seed context: {err}"))?;

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        POLICY_PACK,
        BudgetRequestValidatorSuggestor {
            request: request.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        POLICY_PACK,
        BudgetPolicySuggestor {
            request: request.clone(),
            engine: Arc::clone(&policy_engine),
        },
    );
    engine.register_suggestor_in_pack(
        POLICY_PACK,
        BudgetApprovalGateSuggestor {
            request: request.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        POLICY_PACK,
        AuditLoggerSuggestor {
            request: request.clone(),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            initial_context,
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(BudgetApprovalEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|err| format!("convergence failed: {err}"))?;

    let projection = if persist {
        let decision = find_policy_decision_from_context(&result.context)
            .ok_or("policy decision fact missing from context")?;
        let actor = actor_from_principal(&request.requester_id);
        let recommendation = match decision.outcome {
            PolicyOutcome::Promote => format!(
                "Auto-approve budget request {} for {}",
                decision.request_id, request.amount_minor
            ),
            PolicyOutcome::Escalate => format!(
                "Escalate budget request {} for human review (Tier {})",
                decision.request_id, decision.tier
            ),
            PolicyOutcome::Reject => format!("Reject budget request {}", decision.request_id),
        };
        let rationale = decision.reason.clone().unwrap_or_else(|| {
            format!(
                "authority: {}, tier: {}, amount: {}",
                decision.authority, decision.tier, decision.amount_minor
            )
        });

        let write_result = store
            .write_with_events(|kernel| {
                kernel.record_decision(
                    DecisionRecord {
                        id: Uuid::new_v4(),
                        truth_key: "budget-approval".into(),
                        recommendation: recommendation.clone(),
                        confidence_bps: 10_000,
                        rationale: rationale.clone(),
                        vendor_id: None,
                        needs_human_review: matches!(decision.outcome, PolicyOutcome::Escalate),
                        decided_by: Actor::system(),
                        decided_at: Utc::now(),
                    },
                    &actor,
                );
                Ok(())
            })
            .map_err(|err| format!("failed to persist decision: {err}"))?;

        Some(TruthProjection {
            events_emitted: write_result.events.len(),
            details: Some(serde_json::json!({
                "request_id": request.request_id,
                "amount_minor": request.amount_minor,
                "tier": determine_tier(request.amount_minor),
                "outcome": format!("{:?}", decision.outcome),
            })),
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
            .map(|outcome| crate::truth_runtime::CriterionOutcomeView {
                criterion: outcome.criterion.id.to_string(),
                result: format!("{:?}", outcome.result),
            })
            .collect(),
        projection,
        llm_calls: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_1_below_5000() {
        assert_eq!(determine_tier(0), 1);
        assert_eq!(determine_tier(499_999), 1);
    }

    #[test]
    fn tier_2_5k_to_25k() {
        assert_eq!(determine_tier(500_000), 2); // $5,000
        assert_eq!(determine_tier(1_200_000), 2); // $12,000
        assert_eq!(determine_tier(2_499_999), 2); // just under $25k
    }

    #[test]
    fn tier_3_25k_to_100k() {
        assert_eq!(determine_tier(2_500_000), 3); // $25,000
        assert_eq!(determine_tier(7_500_000), 3); // $75,000
        assert_eq!(determine_tier(9_999_999), 3); // just under $100k
    }

    #[test]
    fn tier_4_over_100k() {
        assert_eq!(determine_tier(10_000_000), 4); // $100,000
        assert_eq!(determine_tier(150_000_000), 4); // $1,500,000
    }

    #[test]
    fn fact_ids_are_unique() {
        let request_id = "req-123";
        assert_ne!(validation_fact_id(request_id), policy_fact_id(request_id));
        assert_ne!(gate_fact_id(request_id), audit_fact_id(request_id));
        assert_ne!(request_fact_id(request_id), policy_fact_id(request_id));
    }

    #[test]
    fn parse_authority_maps_correctly() {
        assert_eq!(parse_authority("advisory"), AuthorityLevel::Advisory);
        assert_eq!(parse_authority("supervisory"), AuthorityLevel::Supervisory);
        assert_eq!(
            parse_authority("participatory"),
            AuthorityLevel::Participatory
        );
        assert_eq!(parse_authority("sovereign"), AuthorityLevel::Sovereign);
    }
}
