//! Truth executor: audit-vendor-decision
//!
//! Demonstrates using the trust pack from `converge-domain` out of the box.
//! No custom agents — this truth uses pre-built agents from the domain crate.

use std::collections::HashMap;
use std::sync::Arc;

use converge_domain::packs::trust::{
    AuditWriterAgent, ComplianceScannerAgent, ProvenanceTrackerAgent, RbacEnforcerAgent,
    SessionValidatorAgent,
};
use converge_kernel::{
    Context, ContextKey, Criterion, CriterionEvaluator, CriterionResult, Engine, TypesRunHooks,
};
use governance_kernel::InMemoryStore;
use governance_truths::{build_intent, find_truth};

use super::TruthExecutionResult;

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

struct AuditVendorDecisionEvaluator;

impl CriterionEvaluator for AuditVendorDecisionEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "audit-entries-written" => {
                if context
                    .get(ContextKey::Proposals)
                    .iter()
                    .any(|f| f.id.starts_with("audit:"))
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no audit entries written yet".into(),
                    }
                }
            }
            "compliance-scanned" => {
                if context
                    .get(ContextKey::Evaluations)
                    .iter()
                    .any(|f| f.id.starts_with("compliance:"))
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "compliance scan not completed".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

pub async fn execute(
    _store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    _persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("audit-vendor-decision").ok_or("truth not found")?;
    let intent = build_intent(truth);

    let decision_id = super::common::required_input(inputs, "decision_id")?;

    // Seed the context with a session token to trigger the trust pack chain.
    let mut initial_context = Context::new();
    initial_context
        .add_input_with_provenance(
            ContextKey::Seeds,
            format!("vendor-decision:{decision_id}"),
            serde_json::json!({
                "session.token": true,
                "decision_id": decision_id,
                "action": "vendor_evaluation",
            })
            .to_string(),
            "audit-executor",
        )
        .map_err(|e| format!("failed to seed context: {e}"))?;

    // All agents come from converge-domain's trust pack. No custom agents needed.
    let mut engine = Engine::new();
    engine.register_suggestor_in_pack("trust-pack", SessionValidatorAgent);
    engine.register_suggestor_in_pack("trust-pack", RbacEnforcerAgent);
    engine.register_suggestor_in_pack("trust-pack", AuditWriterAgent);
    engine.register_suggestor_in_pack("trust-pack", ProvenanceTrackerAgent);
    engine.register_suggestor_in_pack("trust-pack", ComplianceScannerAgent);

    let result = engine
        .run_with_types_intent_and_hooks(
            initial_context,
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(AuditVendorDecisionEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|e| format!("convergence failed: {e}"))?;

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
        projection: None,
        llm_calls: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn audit_vendor_decision_converges() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([("decision_id".into(), "eval-001".into())]);
        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn missing_decision_id_returns_error() {
        let store = InMemoryStore::new();
        assert!(execute(&store, &HashMap::new(), false).await.is_err());
    }
}
