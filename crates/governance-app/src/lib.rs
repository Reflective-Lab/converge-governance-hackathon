use std::collections::HashMap;

use governance_kernel::{AuditEntry, DecisionRecord, InMemoryStore, Vendor};
use governance_server::experience::ExperienceRegistry;
use governance_server::truth_runtime;
use serde::{Deserialize, Serialize};

pub use governance_server::truth_runtime::source_import::{
    TruthSourceFile, TruthSourceFormat, VendorSelectionSourcePreview,
};

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
    experience: ExperienceRegistry,
}

impl GovernanceApp {
    pub fn new(store: InMemoryStore) -> Self {
        Self {
            store,
            experience: ExperienceRegistry::new(),
        }
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
                executable: matches!(t.key, "evaluate-vendor" | "dynamic-due-diligence"),
            })
            .collect()
    }

    pub async fn execute_truth(
        &self,
        key: &str,
        inputs: HashMap<String, String>,
        persist: bool,
    ) -> Result<truth_runtime::TruthExecutionResult, String> {
        truth_runtime::execute_truth(&self.store, key, inputs, persist, &self.experience).await
    }

    pub fn preview_vendor_selection_source(
        &self,
        source: TruthSourceFile,
    ) -> Result<VendorSelectionSourcePreview, String> {
        truth_runtime::source_import::preview_vendor_selection_source(source)
    }

    pub async fn execute_vendor_selection_source(
        &self,
        source: TruthSourceFile,
        persist: bool,
    ) -> Result<truth_runtime::TruthExecutionResult, String> {
        truth_runtime::source_import::execute_vendor_selection_source(&self.store, source, persist)
            .await
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

    #[tokio::test]
    async fn execute_vendor_evaluation_and_query() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let inputs = HashMap::from([("vendors".into(), "Acme AI, Beta ML".into())]);
        let result = app
            .execute_truth("evaluate-vendor", inputs, true)
            .await
            .unwrap();

        assert!(result.converged);
        assert_eq!(app.list_decisions().len(), 1);
        assert!(!app.list_audit(100).is_empty());
    }

    #[tokio::test]
    async fn dashboard_reflects_execution() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let dashboard = app.dashboard();
        assert_eq!(dashboard.recent_decisions.len(), 0);

        let inputs = HashMap::from([("vendors".into(), "Acme AI".into())]);
        app.execute_truth("evaluate-vendor", inputs, true)
            .await
            .unwrap();

        let dashboard = app.dashboard();
        assert_eq!(dashboard.recent_decisions.len(), 1);
    }

    #[test]
    fn previews_vendor_selection_from_gherkin_source() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let preview = app
            .preview_vendor_selection_source(TruthSourceFile {
                name: "vendor-selection.feature".into(),
                content: r#"
Feature: Evaluate AI vendors

  Scenario: shortlist
    Given vendors "Acme AI, Beta ML"
"#
                .into(),
            })
            .unwrap();

        assert_eq!(preview.format, TruthSourceFormat::Gherkin);
        assert_eq!(preview.vendors, vec!["Acme AI", "Beta ML"]);
    }

    #[tokio::test]
    async fn executes_vendor_selection_from_truth_spec_source() {
        let store = InMemoryStore::new();
        let app = GovernanceApp::new(store);

        let result = app
            .execute_vendor_selection_source(
                TruthSourceFile {
                    name: "vendor-selection.truths.json".into(),
                    content: r#"{
  "title": "Desktop vendor selection",
  "truth_key": "evaluate-vendor",
  "vendors": ["Acme AI", "Beta ML"]
}"#
                    .into(),
                },
                true,
            )
            .await
            .unwrap();

        assert!(result.converged);
        assert_eq!(app.list_decisions().len(), 1);
    }
}
