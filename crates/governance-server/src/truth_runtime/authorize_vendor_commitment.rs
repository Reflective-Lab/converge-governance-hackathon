//! Truth executor: authorize-vendor-commitment
//!
//! Demonstrates `converge-policy` as a pure library inside a governed business
//! flow. This is a procurement-style commitment check, not a toy auth demo.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use converge_core::{
    AgentEffect, Context, ContextKey, Criterion, CriterionEvaluator, CriterionResult, Engine,
    ProposedFact, Suggestor, TypesRunHooks,
};
use converge_policy::{ContextIn, DecideRequest, PolicyEngine, PolicyOutcome, PrincipalIn, ResourceIn};
use governance_kernel::{Actor, ActorKind, DecisionRecord, InMemoryStore};
use governance_truths::{build_intent, find_truth};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TruthExecutionResult, TruthProjection};

const POLICY_PACK: &str = "policy-pack";
const REQUEST_PROVENANCE: &str = "governance-server:authorize-vendor-commitment";
const POLICY_PROVENANCE: &str = "policy:vendor-commitment";
const POLICY_TEXT: &str =
    include_str!("../../../../examples/policy-vendor-commitment/vendor-commitment-policy.cedar");

struct CommitmentPolicySuggestor {
    request: VendorCommitmentRequest,
    engine: Arc<PolicyEngine>,
}

impl Suggestor for CommitmentPolicySuggestor {
    fn name(&self) -> &str {
        "commitment-policy"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        !ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|fact| fact.id == policy_fact_id(&self.request.commitment_id))
    }

    fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let content = match self.engine.evaluate(&self.request.to_decide_request()) {
            Ok(decision) => match serde_json::to_string(&PolicyDecisionFact::from_parts(
                &self.request,
                decision,
            )) {
                Ok(content) => content,
                Err(err) => serde_json::json!({
                    "principal_id": self.request.principal_id,
                    "principal_authority": self.request.principal_authority,
                    "vendor_name": self.request.vendor_name,
                    "commitment_id": self.request.commitment_id,
                    "action": self.request.action,
                    "commitment_type": self.request.commitment_type,
                    "amount_minor": self.request.amount_minor,
                    "currency_code": self.request.currency_code,
                    "human_approval_present": self.request.human_approval_present,
                    "required_gates_met": self.request.required_gates_met,
                    "outcome": "reject",
                    "reason": format!("policy decision serialization failed: {err}"),
                })
                .to_string(),
            },
            Err(err) => serde_json::json!({
                "principal_id": self.request.principal_id,
                "principal_authority": self.request.principal_authority,
                "vendor_name": self.request.vendor_name,
                "commitment_id": self.request.commitment_id,
                "action": self.request.action,
                "commitment_type": self.request.commitment_type,
                "amount_minor": self.request.amount_minor,
                "currency_code": self.request.currency_code,
                "human_approval_present": self.request.human_approval_present,
                "required_gates_met": self.request.required_gates_met,
                "outcome": "reject",
                "reason": format!("policy evaluation failed: {err}"),
            })
            .to_string(),
        };

        AgentEffect::with_proposal(
            ProposedFact::new(
                ContextKey::Evaluations,
                policy_fact_id(&self.request.commitment_id),
                content,
                POLICY_PROVENANCE,
            )
            .with_confidence(1.0),
        )
    }
}

struct VendorCommitmentEvaluator;

impl CriterionEvaluator for VendorCommitmentEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "policy-decision-produced" => {
                if find_policy_decision(context).is_some() {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no policy decision fact has been produced".into(),
                    }
                }
            }
            "commitment-authorized" => match find_policy_decision(context) {
                Some(decision) => match decision.outcome {
                    PolicyOutcome::Promote => CriterionResult::Met { evidence: vec![] },
                    PolicyOutcome::Escalate => CriterionResult::Blocked {
                        reason: decision.reason.unwrap_or_else(|| {
                            "human approval is required before commitment can proceed".into()
                        }),
                        approval_ref: Some(format!("approval:{}", decision.commitment_id)),
                    },
                    PolicyOutcome::Reject => CriterionResult::Unmet {
                        reason: decision.reason.unwrap_or_else(|| {
                            "policy rejected the commitment request".into()
                        }),
                    },
                },
                None => CriterionResult::Unmet {
                    reason: "policy decision not available".into(),
                },
            },
            _ => CriterionResult::Indeterminate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VendorCommitmentRequest {
    principal_id: String,
    principal_authority: String,
    domains: Vec<String>,
    policy_version: Option<String>,
    commitment_id: String,
    vendor_name: String,
    commitment_type: String,
    phase: String,
    action: String,
    amount_minor: i64,
    currency_code: String,
    human_approval_present: bool,
    required_gates_met: bool,
}

impl VendorCommitmentRequest {
    fn from_inputs(inputs: &HashMap<String, String>) -> Result<Self, String> {
        let action = inputs
            .get("action")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "commit".to_string());

        if !matches!(
            action.as_str(),
            "propose" | "validate" | "promote" | "commit" | "advance_phase"
        ) {
            return Err(format!(
                "unsupported action {action}; expected propose, validate, promote, commit, or advance_phase"
            ));
        }

        let authority = super::common::required_input(inputs, "principal_authority")?;
        if !matches!(
            authority,
            "advisory" | "supervisory" | "participatory" | "sovereign"
        ) {
            return Err(format!(
                "unsupported principal_authority {authority}; expected advisory, supervisory, participatory, or sovereign"
            ));
        }

        Ok(Self {
            principal_id: super::common::required_input(inputs, "principal_id")?.to_string(),
            principal_authority: authority.to_string(),
            domains: split_csv(inputs.get("domains").map_or("procurement,vendor-selection", String::as_str)),
            policy_version: inputs
                .get("policy_version")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            commitment_id: super::common::required_input(inputs, "commitment_id")?.to_string(),
            vendor_name: super::common::required_input(inputs, "vendor_name")?.to_string(),
            commitment_type: inputs
                .get("commitment_type")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "contract".to_string()),
            phase: inputs
                .get("phase")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "commitment".to_string()),
            action,
            amount_minor: parse_i64(inputs, "amount_minor")?,
            currency_code: inputs
                .get("currency_code")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "USD".to_string()),
            human_approval_present: parse_bool(inputs, "human_approval_present")?,
            required_gates_met: parse_bool(inputs, "required_gates_met")?,
        })
    }

    fn to_decide_request(&self) -> DecideRequest {
        DecideRequest {
            principal: PrincipalIn {
                id: self.principal_id.clone(),
                authority: self.principal_authority.clone(),
                domains: self.domains.clone(),
                policy_version: self.policy_version.clone(),
            },
            resource: ResourceIn {
                id: self.commitment_id.clone(),
                resource_type: Some(self.commitment_type.clone()),
                phase: Some(self.phase.clone()),
                gates_passed: Some(if self.required_gates_met {
                    vec!["evidence".into(), "risk".into(), "compliance".into()]
                } else {
                    Vec::new()
                }),
            },
            action: self.action.clone(),
            context: Some(ContextIn {
                commitment_type: Some(self.commitment_type.clone()),
                amount: Some(self.amount_minor),
                human_approval_present: Some(self.human_approval_present),
                required_gates_met: Some(self.required_gates_met),
            }),
            delegation_b64: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolicyDecisionFact {
    principal_id: String,
    principal_authority: String,
    vendor_name: String,
    commitment_id: String,
    action: String,
    commitment_type: String,
    amount_minor: i64,
    currency_code: String,
    human_approval_present: bool,
    required_gates_met: bool,
    outcome: PolicyOutcome,
    reason: Option<String>,
}

impl PolicyDecisionFact {
    fn from_parts(
        request: &VendorCommitmentRequest,
        decision: converge_policy::PolicyDecision,
    ) -> Self {
        Self {
            principal_id: request.principal_id.clone(),
            principal_authority: request.principal_authority.clone(),
            vendor_name: request.vendor_name.clone(),
            commitment_id: request.commitment_id.clone(),
            action: request.action.clone(),
            commitment_type: request.commitment_type.clone(),
            amount_minor: request.amount_minor,
            currency_code: request.currency_code.clone(),
            human_approval_present: request.human_approval_present,
            required_gates_met: request.required_gates_met,
            outcome: decision.outcome,
            reason: decision.reason,
        }
    }
}

pub fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("authorize-vendor-commitment").ok_or("truth not found")?;
    let intent = build_intent(truth);
    let request = VendorCommitmentRequest::from_inputs(inputs)?;
    let policy_engine = Arc::new(
        PolicyEngine::from_policy_str(POLICY_TEXT)
            .map_err(|err| format!("policy setup failed: {err}"))?,
    );

    let mut initial_context = Context::new();
    initial_context
        .add_input_with_provenance(
            ContextKey::Seeds,
            request_fact_id(&request.commitment_id),
            serde_json::to_string(&request)
                .map_err(|err| format!("failed to serialize policy request: {err}"))?,
            REQUEST_PROVENANCE,
        )
        .map_err(|err| format!("failed to seed context: {err}"))?;

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        POLICY_PACK,
        CommitmentPolicySuggestor {
            request: request.clone(),
            engine: Arc::clone(&policy_engine),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            initial_context,
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(VendorCommitmentEvaluator)),
                event_observer: None,
            },
        )
        .map_err(|err| format!("convergence failed: {err}"))?;

    let projection = if persist {
        let decision = find_policy_decision(&result.context)
            .ok_or("policy decision fact missing from context")?;
        let actor = actor_from_principal(&request.principal_id);
        let recommendation = match decision.outcome {
            PolicyOutcome::Promote => format!(
                "Authorize commitment {} for {}",
                decision.commitment_id, decision.vendor_name
            ),
            PolicyOutcome::Escalate => format!(
                "Escalate commitment {} for human review",
                decision.commitment_id
            ),
            PolicyOutcome::Reject => format!(
                "Reject commitment {} for {}",
                decision.commitment_id, decision.vendor_name
            ),
        };
        let rationale = decision.reason.clone().unwrap_or_else(|| {
            format!(
                "{} {} {} {}",
                decision.principal_authority,
                decision.action,
                decision.commitment_type,
                decision.amount_minor
            )
        });

        let write_result = store
            .write_with_events(|kernel| {
                kernel.record_decision(
                    DecisionRecord {
                        id: Uuid::new_v4(),
                        truth_key: "authorize-vendor-commitment".into(),
                        recommendation,
                        confidence_bps: 10_000,
                        rationale: format!(
                            "{} {} {} {} {} {}",
                            decision.vendor_name,
                            decision.currency_code,
                            decision.amount_minor,
                            decision.commitment_type,
                            decision.action,
                            rationale
                        ),
                        vendor_id: None,
                        needs_human_review: matches!(decision.outcome, PolicyOutcome::Escalate),
                        decided_by: Actor::system(),
                        decided_at: Utc::now(),
                    },
                    &actor,
                );
                Ok(())
            })
            .map_err(|err| format!("projection failed: {err}"))?;

        Some(TruthProjection {
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
            .map(|outcome| super::CriterionOutcomeView {
                criterion: outcome.criterion.description.clone(),
                result: format!("{:?}", outcome.result),
            })
            .collect(),
        projection,
    })
}

fn request_fact_id(commitment_id: &str) -> String {
    format!("policy:request:{commitment_id}")
}

fn policy_fact_id(commitment_id: &str) -> String {
    format!("policy:decision:{commitment_id}")
}

fn find_policy_decision(context: &Context) -> Option<PolicyDecisionFact> {
    context
        .get(ContextKey::Evaluations)
        .iter()
        .find(|fact| fact.id.starts_with("policy:decision:"))
        .and_then(|fact| serde_json::from_str::<PolicyDecisionFact>(&fact.content).ok())
}

fn parse_i64(inputs: &HashMap<String, String>, key: &str) -> Result<i64, String> {
    let raw = super::common::required_input(inputs, key)?;
    raw.parse::<i64>()
        .map_err(|err| format!("invalid {key}: {err}"))
}

fn parse_bool(inputs: &HashMap<String, String>, key: &str) -> Result<bool, String> {
    let raw = super::common::required_input(inputs, key)?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(format!("invalid {key}: expected true/false")),
    }
}

fn split_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
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
        name,
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approval_inputs() -> HashMap<String, String> {
        HashMap::from([
            ("principal_id".into(), "user:procurement-lead".into()),
            ("principal_authority".into(), "supervisory".into()),
            ("commitment_id".into(), "commitment:vendor-a-2026-04".into()),
            ("vendor_name".into(), "Vendor A".into()),
            ("commitment_type".into(), "contract".into()),
            ("action".into(), "commit".into()),
            ("amount_minor".into(), "75000".into()),
            ("currency_code".into(), "USD".into()),
            ("human_approval_present".into(), "true".into()),
            ("required_gates_met".into(), "true".into()),
        ])
    }

    fn criterion_result<'a>(
        result: &'a TruthExecutionResult,
        needle: &str,
    ) -> Option<&'a str> {
        result
            .criteria_outcomes
            .iter()
            .find(|outcome| outcome.criterion.contains(needle))
            .map(|outcome| outcome.result.as_str())
    }

    #[test]
    fn authorizes_supervisory_commit_with_human_approval() {
        let store = InMemoryStore::new();
        let result = execute(&store, &approval_inputs(), true).unwrap();

        assert!(criterion_result(&result, "policy decision fact exists")
            .unwrap()
            .contains("Met"));
        assert!(criterion_result(&result, "authorized or blocked honestly")
            .unwrap()
            .contains("Met"));
        assert_eq!(result.projection.as_ref().unwrap().events_emitted, 1);

        let decisions = store.read(|kernel| kernel.recent_decisions(1).len()).unwrap();
        assert_eq!(decisions, 1);
    }

    #[test]
    fn blocks_commit_without_human_approval() {
        let store = InMemoryStore::new();
        let mut inputs = approval_inputs();
        inputs.insert("human_approval_present".into(), "false".into());

        let result = execute(&store, &inputs, false).unwrap();

        assert!(criterion_result(&result, "authorized or blocked honestly")
            .unwrap()
            .contains("Blocked"));
    }

    #[test]
    fn rejects_advisory_spend_above_cap() {
        let store = InMemoryStore::new();
        let mut inputs = approval_inputs();
        inputs.insert("principal_id".into(), "agent:cost-analyst".into());
        inputs.insert("principal_authority".into(), "advisory".into());
        inputs.insert("action".into(), "propose".into());
        inputs.insert("commitment_type".into(), "spend".into());
        inputs.insert("amount_minor".into(), "15000".into());
        inputs.insert("human_approval_present".into(), "false".into());

        let result = execute(&store, &inputs, false).unwrap();

        assert!(criterion_result(&result, "authorized or blocked honestly")
            .unwrap()
            .contains("Unmet"));
    }
}
