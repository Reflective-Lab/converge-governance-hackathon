use std::collections::HashMap;

use governance_kernel::{AuditEntry, DecisionRecord, InMemoryStore, Vendor};
use governance_server::truth_runtime;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// View models — what the UI sees
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Dashboard {
    pub recent_decisions: Vec<DecisionRecord>,
    pub vendor_count: usize,
    pub audit_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthListItem {
    pub key: String,
    pub display_name: String,
    pub summary: String,
    pub executable: bool,
}

// ---------------------------------------------------------------------------
// Operator app — the shared layer for desktop and server
// ---------------------------------------------------------------------------

pub struct GovernanceApp {
    store: InMemoryStore,
}

impl GovernanceApp {
    pub fn new(store: InMemoryStore) -> Self {
        Self { store }
    }

    pub fn dashboard(&self) -> Dashboard {
        let recent_decisions = self
            .store
            .read(|k| k.recent_decisions(5).into_iter().cloned().collect())
            .unwrap_or_default();
        let vendor_count = self.store.read(|k| k.vendors.len()).unwrap_or(0);
        let audit_count = self.store.read(|k| k.audit_trail.len()).unwrap_or(0);
        Dashboard {
            recent_decisions,
            vendor_count,
            audit_count,
        }
    }

    pub fn list_truths(&self) -> Vec<TruthListItem> {
        governance_truths::TRUTHS
            .iter()
            .map(|t| TruthListItem {
                key: t.key.into(),
                display_name: t.display_name.into(),
                summary: t.summary.into(),
                executable: t.key == "evaluate-vendor", // only the reference is executable
            })
            .collect()
    }

    pub fn execute_truth(
        &self,
        key: &str,
        inputs: HashMap<String, String>,
        persist: bool,
    ) -> Result<truth_runtime::TruthExecutionResult, String> {
        truth_runtime::execute_truth(&self.store, key, inputs, persist)
    }

    pub fn list_vendors(&self) -> Vec<Vendor> {
        self.store
            .read(|k| k.vendors_list().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_decisions(&self) -> Vec<DecisionRecord> {
        self.store
            .read(|k| k.recent_decisions(20).into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_audit(&self, limit: usize) -> Vec<AuditEntry> {
        self.store
            .read(|k| k.recent_audit(limit).into_iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_vendor_evaluation_and_query() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);
        let result = app.execute_truth("evaluate-vendor", inputs, true).unwrap();

        assert!(result.converged);
        assert_eq!(app.list_decisions().len(), 1);
        assert!(app.list_audit(100).len() > 0);
    }

    #[test]
    fn dashboard_reflects_execution() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let dashboard = app.dashboard();
        assert_eq!(dashboard.recent_decisions.len(), 0);

        let inputs = HashMap::from([("vendors".into(), "Acme AI".into())]);
        app.execute_truth("evaluate-vendor", inputs, true).unwrap();

        let dashboard = app.dashboard();
        assert_eq!(dashboard.recent_decisions.len(), 1);
    }
}
