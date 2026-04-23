pub mod audit_vendor_decision;
pub mod authorize_vendor_commitment;
pub mod common;
pub mod dynamic_due_diligence;
pub mod evaluate_vendor;
pub mod source_import;
pub mod vendor_selection;

use std::collections::HashMap;

use governance_kernel::InMemoryStore;
use governance_telemetry::LlmCallTelemetry;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared types for truth execution results
// ---------------------------------------------------------------------------

/// Execution result returned by a truth run.
///
/// `TruthExecutionResult` is the public boundary contract for execution output.
/// It is intentionally conservative:
/// - `projection` and `criteria_outcomes` describe core governance output.
/// - `llm_calls` is an optional, runtime-only telemetry projection.
/// - runtime observability may be disabled per environment, so consumers should
///   treat `llm_calls` as best-effort data.
#[derive(Debug, Serialize, Deserialize)]
pub struct TruthExecutionResult {
    pub converged: bool,
    pub cycles: u32,
    pub stop_reason: String,
    pub criteria_outcomes: Vec<CriterionOutcomeView>,
    pub projection: Option<TruthProjection>,
    /// Best-effort projection of LLM call telemetry produced during execution.
    ///
    /// This field is optional by design and may be omitted (`None`) for
    /// compatibility with older API clients or when telemetry sinks are not
    /// wired in this environment.
    ///
    /// Consumers should never treat these entries as source-of-truth facts.
    /// They are non-authoritative operational diagnostics that must not affect
    /// governance decisions, audit semantics, or persisted outcomes.
    #[serde(default)]
    pub llm_calls: Option<Vec<LlmCallTelemetry>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriterionOutcomeView {
    pub criterion: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthProjection {
    pub events_emitted: usize,
    pub details: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Dispatcher — routes truth key to executor
// ---------------------------------------------------------------------------

pub async fn execute_truth(
    store: &InMemoryStore,
    truth_key: &str,
    inputs: HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    match truth_key {
        "authorize-vendor-commitment" => {
            authorize_vendor_commitment::execute(store, &inputs, persist).await
        }
        "dynamic-due-diligence" => dynamic_due_diligence::execute(store, &inputs, persist).await,
        "evaluate-vendor" => evaluate_vendor::execute(store, &inputs, persist).await,
        "audit-vendor-decision" => audit_vendor_decision::execute(store, &inputs, persist).await,
        "vendor-selection" => vendor_selection::execute(store, &inputs, persist).await,
        // ---------------------------------------------------------------
        // Add your truth executor here:
        // "your-truth-key" => your_module::execute(store, &inputs, persist).await,
        // ---------------------------------------------------------------
        _ => Err(format!("no executor for truth: {truth_key}")),
    }
}

#[cfg(test)]
mod tests {
    use super::TruthExecutionResult;

    #[test]
    fn truth_execution_result_deserializes_without_llm_calls() {
        let payload = serde_json::json!({
            "converged": true,
            "cycles": 1,
            "stop_reason": "StopReason::Converged",
            "criteria_outcomes": [
                {
                    "criterion": "criterion-a",
                    "result": "passed"
                }
            ],
            "projection": null
        });

        let parsed: TruthExecutionResult = serde_json::from_value(payload)
            .expect("old payload without llm_calls should deserialize");

        assert!(parsed.llm_calls.is_none());
    }

    #[test]
    fn truth_execution_result_deserializes_with_llm_calls() {
        let payload = serde_json::json!({
            "converged": true,
            "cycles": 2,
            "stop_reason": "StopReason::Converged",
            "criteria_outcomes": [],
            "projection": null,
            "llm_calls": [
                {
                    "context": "test:analysis",
                    "provider": "openrouter",
                    "model": "openrouter/gpt-4o",
                    "elapsed_ms": 1250,
                    "finish_reason": "Stop",
                    "usage": {
                        "prompt_tokens": 100,
                        "completion_tokens": 20,
                        "total_tokens": 120
                    },
                    "metadata": {
                        "attempt": "1"
                    }
                }
            ]
        });

        let parsed: TruthExecutionResult = serde_json::from_value(payload)
            .expect("extended payload with llm_calls should deserialize");

        let calls = parsed.llm_calls.expect("llm_calls should be populated");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context, "test:analysis");
    }
}
