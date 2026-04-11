pub mod authorize_vendor_commitment;
pub mod audit_vendor_decision;
pub mod common;
pub mod evaluate_vendor;
pub mod source_import;

use std::collections::HashMap;

use governance_kernel::InMemoryStore;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared types for truth execution results
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthExecutionResult {
    pub converged: bool,
    pub cycles: u32,
    pub stop_reason: String,
    pub criteria_outcomes: Vec<CriterionOutcomeView>,
    pub projection: Option<TruthProjection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriterionOutcomeView {
    pub criterion: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthProjection {
    pub events_emitted: usize,
}

// ---------------------------------------------------------------------------
// Dispatcher — routes truth key to executor
// ---------------------------------------------------------------------------

pub fn execute_truth(
    store: &InMemoryStore,
    truth_key: &str,
    inputs: HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    match truth_key {
        "authorize-vendor-commitment" => {
            authorize_vendor_commitment::execute(store, &inputs, persist)
        }
        "evaluate-vendor" => evaluate_vendor::execute(store, &inputs, persist),
        "audit-vendor-decision" => audit_vendor_decision::execute(store, &inputs, persist),
        // ---------------------------------------------------------------
        // Add your truth executor here:
        // "your-truth-key" => your_module::execute(store, &inputs, persist),
        // ---------------------------------------------------------------
        _ => Err(format!("no executor for truth: {truth_key}")),
    }
}
