use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod types;

// ---------------------------------------------------------------------------
// Domain types — the governance world
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub website: Option<String>,
    pub capabilities: Vec<String>,
    pub certifications: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: Uuid,
    pub name: String,
    pub category: PolicyCategory,
    pub description: String,
    pub severity: Severity,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyCategory {
    Gdpr,
    AiAct,
    InternalSecurity,
    DataResidency,
    CostGovernance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub policy_rule_id: Uuid,
    pub status: ComplianceStatus,
    pub evidence: String,
    pub checked_by: Actor,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComplianceStatus {
    Pass,
    Fail,
    NeedsReview,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub dimension: String,
    pub score_bps: u16,
    pub rationale: String,
    pub scored_by: Actor,
    pub scored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub monthly_cost_minor: i64,
    pub currency_code: String,
    pub assumptions: String,
    pub estimated_by: Actor,
    pub estimated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: Uuid,
    pub truth_key: String,
    pub recommendation: String,
    pub confidence_bps: u16,
    pub rationale: String,
    pub vendor_id: Option<Uuid>,
    pub needs_human_review: bool,
    pub decided_by: Actor,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub action: String,
    pub actor: Actor,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub kind: ActorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActorKind {
    Human,
    Agent,
    System,
}

impl Actor {
    pub fn system() -> Self {
        Self {
            id: "system".into(),
            name: "System".into(),
            kind: ActorKind::System,
        }
    }

    pub fn agent(name: &str) -> Self {
        Self {
            id: format!("agent:{name}"),
            name: name.into(),
            kind: ActorKind::Agent,
        }
    }
}

// ---------------------------------------------------------------------------
// Domain events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum DomainEvent {
    VendorRegistered {
        vendor: Vendor,
        actor: Actor,
    },
    ComplianceChecked {
        check: ComplianceCheck,
        actor: Actor,
    },
    RiskScored {
        score: RiskScore,
        actor: Actor,
    },
    CostEstimated {
        estimate: CostEstimate,
        actor: Actor,
    },
    DecisionRecorded {
        decision: DecisionRecord,
        actor: Actor,
    },
    AuditRecorded {
        entry: AuditEntry,
    },
}

/// Adapter boundary for routing persisted domain events into an experience stream.
///
/// The kernel emits these events as part of its local mutation pipeline.
/// Adapters can forward them to durable observability stores, message queues,
/// or replay systems without influencing governance semantics.
pub trait DomainEventStream: Send + Sync {
    /// Record one or more emitted domain events.
    ///
    /// Implementations should treat failures as non-fatal for the kernel write
    /// path because domain persistence is authoritative; a failed stream write
    /// should not block or rollback a successful governance mutation.
    fn record_events(&self, _events: &[DomainEvent]) {}
}

/// Default no-op domain event stream.
#[derive(Debug, Default)]
pub struct NoopDomainEventStream;

impl DomainEventStream for NoopDomainEventStream {}

// ---------------------------------------------------------------------------
// Kernel — in-memory projection store
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct GovernanceKernel {
    pub vendors: HashMap<Uuid, Vendor>,
    pub policy_rules: HashMap<Uuid, PolicyRule>,
    pub compliance_checks: Vec<ComplianceCheck>,
    pub risk_scores: Vec<RiskScore>,
    pub cost_estimates: Vec<CostEstimate>,
    pub decisions: Vec<DecisionRecord>,
    pub audit_trail: Vec<AuditEntry>,
    pending_events: Vec<DomainEvent>,
}

impl GovernanceKernel {
    pub fn register_vendor(&mut self, name: String, description: String, actor: &Actor) -> Vendor {
        let vendor = Vendor {
            id: Uuid::new_v4(),
            name,
            description,
            website: None,
            capabilities: vec![],
            certifications: vec![],
            created_at: Utc::now(),
        };
        self.vendors.insert(vendor.id, vendor.clone());
        self.audit(
            "register-vendor",
            actor,
            &format!("Registered vendor: {}", vendor.name),
        );
        self.pending_events.push(DomainEvent::VendorRegistered {
            vendor: vendor.clone(),
            actor: actor.clone(),
        });
        vendor
    }

    pub fn add_policy_rule(&mut self, rule: PolicyRule) {
        self.policy_rules.insert(rule.id, rule);
    }

    pub fn record_compliance_check(&mut self, check: ComplianceCheck, actor: &Actor) {
        self.audit(
            "compliance-check",
            actor,
            &format!(
                "Compliance check for vendor {} on rule {}: {:?}",
                check.vendor_id, check.policy_rule_id, check.status
            ),
        );
        self.pending_events.push(DomainEvent::ComplianceChecked {
            check: check.clone(),
            actor: actor.clone(),
        });
        self.compliance_checks.push(check);
    }

    pub fn record_risk_score(&mut self, score: RiskScore, actor: &Actor) {
        self.audit(
            "risk-score",
            actor,
            &format!(
                "Risk score for vendor {}: {} = {} bps",
                score.vendor_id, score.dimension, score.score_bps
            ),
        );
        self.pending_events.push(DomainEvent::RiskScored {
            score: score.clone(),
            actor: actor.clone(),
        });
        self.risk_scores.push(score);
    }

    pub fn record_cost_estimate(&mut self, estimate: CostEstimate, actor: &Actor) {
        self.audit(
            "cost-estimate",
            actor,
            &format!(
                "Cost estimate for vendor {}: {} {}/month",
                estimate.vendor_id, estimate.monthly_cost_minor, estimate.currency_code
            ),
        );
        self.pending_events.push(DomainEvent::CostEstimated {
            estimate: estimate.clone(),
            actor: actor.clone(),
        });
        self.cost_estimates.push(estimate);
    }

    pub fn record_decision(&mut self, decision: DecisionRecord, actor: &Actor) {
        self.audit(
            "decision",
            actor,
            &format!(
                "Decision for {}: {} (confidence: {} bps, needs review: {})",
                decision.truth_key,
                decision.recommendation,
                decision.confidence_bps,
                decision.needs_human_review
            ),
        );
        self.pending_events.push(DomainEvent::DecisionRecorded {
            decision: decision.clone(),
            actor: actor.clone(),
        });
        self.decisions.push(decision);
    }

    // Queries

    pub fn vendors_list(&self) -> Vec<&Vendor> {
        self.vendors.values().collect()
    }

    pub fn compliance_for_vendor(&self, vendor_id: Uuid) -> Vec<&ComplianceCheck> {
        self.compliance_checks
            .iter()
            .filter(|c| c.vendor_id == vendor_id)
            .collect()
    }

    pub fn risk_scores_for_vendor(&self, vendor_id: Uuid) -> Vec<&RiskScore> {
        self.risk_scores
            .iter()
            .filter(|s| s.vendor_id == vendor_id)
            .collect()
    }

    pub fn recent_decisions(&self, limit: usize) -> Vec<&DecisionRecord> {
        self.decisions.iter().rev().take(limit).collect()
    }

    pub fn recent_audit(&self, limit: usize) -> Vec<&AuditEntry> {
        self.audit_trail.iter().rev().take(limit).collect()
    }

    // Internal

    fn audit(&mut self, action: &str, actor: &Actor, details: &str) {
        self.audit_trail.push(AuditEntry {
            id: Uuid::new_v4(),
            action: action.into(),
            actor: actor.clone(),
            details: details.into(),
            timestamp: Utc::now(),
        });
    }

    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

// ---------------------------------------------------------------------------
// Store — transactional wrapper (clone-and-swap)
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("not found: {kind} {id}")]
    NotFound { kind: &'static str, id: String },
    #[error("conflict: {0}")]
    Conflict(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("lock poisoned")]
    LockPoisoned,
    #[error("{0}")]
    Kernel(#[from] KernelError),
}

pub struct StoreWriteResult<T> {
    pub value: T,
    pub events: Vec<DomainEvent>,
}

#[derive(Clone)]
pub struct InMemoryStore {
    kernel: Arc<RwLock<GovernanceKernel>>,
    domain_event_stream: Arc<dyn DomainEventStream>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            kernel: Arc::new(RwLock::new(GovernanceKernel::default())),
            domain_event_stream: Arc::new(NoopDomainEventStream),
        }
    }

    /// Configure an adapter for emitted domain events.
    pub fn with_domain_event_stream(
        mut self,
        domain_event_stream: Arc<dyn DomainEventStream>,
    ) -> Self {
        self.domain_event_stream = domain_event_stream;
        self
    }

    pub fn read<R>(&self, f: impl FnOnce(&GovernanceKernel) -> R) -> Result<R, StoreError> {
        let kernel = self.kernel.read().map_err(|_| StoreError::LockPoisoned)?;
        Ok(f(&kernel))
    }

    /// Clone-and-swap: the closure runs against a snapshot.
    /// On success the snapshot replaces the live state.
    /// On failure the live state is untouched.
    pub fn write_with_events<R>(
        &self,
        f: impl FnOnce(&mut GovernanceKernel) -> Result<R, KernelError>,
    ) -> Result<StoreWriteResult<R>, StoreError> {
        let mut kernel = self.kernel.write().map_err(|_| StoreError::LockPoisoned)?;
        let mut snapshot = kernel.clone();
        let value = f(&mut snapshot)?;
        let events = snapshot.drain_events();
        *kernel = snapshot;

        let stream = self.domain_event_stream.clone();
        drop(kernel);
        stream.record_events(&events);
        Ok(StoreWriteResult { value, events })
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    struct CapturingDomainEventStream {
        events: Arc<Mutex<Vec<DomainEvent>>>,
    }

    impl DomainEventStream for CapturingDomainEventStream {
        fn record_events(&self, events: &[DomainEvent]) {
            self.events
                .lock()
                .expect("capture stream lock poisoned")
                .extend(events.iter().cloned());
        }
    }

    // --- Unit tests ---

    #[test]
    fn write_with_events_forwarded_to_domain_stream() {
        let stream = Arc::new(CapturingDomainEventStream::default());
        let store = InMemoryStore::new().with_domain_event_stream(stream.clone());

        store
            .write_with_events(|k| {
                k.record_decision(
                    DecisionRecord {
                        id: Uuid::new_v4(),
                        truth_key: "test-truth".into(),
                        recommendation: "ok".into(),
                        confidence_bps: 10_000,
                        rationale: "unit test".into(),
                        vendor_id: None,
                        needs_human_review: false,
                        decided_by: Actor::system(),
                        decided_at: Utc::now(),
                    },
                    &Actor::system(),
                );
                Ok(())
            })
            .expect("write should succeed");

        let captured = stream.events.lock().expect("capture stream lock poisoned");
        assert_eq!(captured.len(), 1, "decision should emit one domain event");
        assert!(matches!(captured[0], DomainEvent::DecisionRecorded { .. }));
    }

    #[test]
    fn register_vendor_and_query() {
        let store = InMemoryStore::new();
        let result = store
            .write_with_events(|k| {
                k.register_vendor("Acme AI".into(), "LLM provider".into(), &Actor::system());
                Ok(())
            })
            .unwrap();

        assert_eq!(result.events.len(), 1);

        let vendors = store.read(|k| k.vendors_list().len()).unwrap();
        assert_eq!(vendors, 1);
    }

    #[test]
    fn failed_write_rolls_back() {
        let store = InMemoryStore::new();
        store
            .write_with_events(|k| {
                k.register_vendor("Good".into(), "ok".into(), &Actor::system());
                Ok(())
            })
            .unwrap();

        let result: Result<StoreWriteResult<()>, StoreError> = store.write_with_events(|k| {
            k.register_vendor("Bad".into(), "will fail".into(), &Actor::system());
            Err(KernelError::Validation("forced failure".into()))
        });
        assert!(result.is_err());

        let count = store.read(|k| k.vendors.len()).unwrap();
        assert_eq!(count, 1, "rollback should leave only the first vendor");
    }

    // --- Negative tests ---

    #[test]
    fn compliance_for_nonexistent_vendor_returns_empty() {
        let store = InMemoryStore::new();
        let checks = store
            .read(|k| k.compliance_for_vendor(Uuid::new_v4()).len())
            .unwrap();
        assert_eq!(checks, 0);
    }

    #[test]
    fn risk_scores_for_nonexistent_vendor_returns_empty() {
        let store = InMemoryStore::new();
        let scores = store
            .read(|k| k.risk_scores_for_vendor(Uuid::new_v4()).len())
            .unwrap();
        assert_eq!(scores, 0);
    }

    #[test]
    fn recent_decisions_on_empty_store() {
        let store = InMemoryStore::new();
        let decisions = store.read(|k| k.recent_decisions(10).len()).unwrap();
        assert_eq!(decisions, 0);
    }

    #[test]
    fn recent_audit_on_empty_store() {
        let store = InMemoryStore::new();
        let entries = store.read(|k| k.recent_audit(10).len()).unwrap();
        assert_eq!(entries, 0);
    }

    #[test]
    fn drain_events_clears_pending() {
        let mut kernel = GovernanceKernel::default();
        kernel.register_vendor("A".into(), "desc".into(), &Actor::system());
        assert!(!kernel.drain_events().is_empty());
        assert!(
            kernel.drain_events().is_empty(),
            "second drain should be empty"
        );
    }

    // --- Soak tests ---

    #[test]
    fn soak_repeated_vendor_registration() {
        let store = InMemoryStore::new();
        for i in 0..100 {
            store
                .write_with_events(|k| {
                    k.register_vendor(format!("Vendor-{i}"), format!("desc-{i}"), &Actor::system());
                    Ok(())
                })
                .unwrap();
        }
        let count = store.read(|k| k.vendors.len()).unwrap();
        assert_eq!(count, 100);
        let audit_count = store.read(|k| k.audit_trail.len()).unwrap();
        assert_eq!(audit_count, 100);
    }

    #[test]
    fn soak_interleaved_reads_and_writes() {
        let store = InMemoryStore::new();
        for i in 0..50 {
            store
                .write_with_events(|k| {
                    k.register_vendor(format!("V-{i}"), "desc".into(), &Actor::system());
                    Ok(())
                })
                .unwrap();

            let count = store.read(|k| k.vendors.len()).unwrap();
            assert_eq!(count, i + 1);

            let audit = store.read(|k| k.recent_audit(1000).len()).unwrap();
            assert_eq!(audit, i + 1);
        }
    }

    #[test]
    fn soak_rollback_leaves_no_trace() {
        let store = InMemoryStore::new();
        for _ in 0..50 {
            let _: Result<StoreWriteResult<()>, StoreError> = store.write_with_events(|k| {
                k.register_vendor("Ghost".into(), "will fail".into(), &Actor::system());
                Err(KernelError::Validation("intentional".into()))
            });
        }
        let count = store.read(|k| k.vendors.len()).unwrap();
        assert_eq!(count, 0, "50 rollbacks should leave zero vendors");
        let audit = store.read(|k| k.audit_trail.len()).unwrap();
        assert_eq!(audit, 0, "50 rollbacks should leave zero audit entries");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_actor() -> impl Strategy<Value = Actor> {
        prop_oneof![
            Just(Actor::system()),
            "[a-z]{3,10}".prop_map(|name| Actor::agent(&name)),
        ]
    }

    fn arb_severity() -> impl Strategy<Value = Severity> {
        prop_oneof![
            Just(Severity::Critical),
            Just(Severity::High),
            Just(Severity::Medium),
            Just(Severity::Low),
            Just(Severity::Info),
        ]
    }

    fn arb_compliance_status() -> impl Strategy<Value = ComplianceStatus> {
        prop_oneof![
            Just(ComplianceStatus::Pass),
            Just(ComplianceStatus::Fail),
            Just(ComplianceStatus::NeedsReview),
            Just(ComplianceStatus::NotApplicable),
        ]
    }

    proptest! {
        #[test]
        fn vendor_registration_always_creates_audit_entry(
            name in "[A-Za-z ]{1,50}",
            desc in "[A-Za-z ]{0,100}",
            actor in arb_actor(),
        ) {
            let store = InMemoryStore::new();
            store
                .write_with_events(|k| {
                    k.register_vendor(name.clone(), desc, &actor);
                    Ok(())
                })
                .unwrap();

            let vendor_count = store.read(|k| k.vendors.len()).unwrap();
            let audit_count = store.read(|k| k.audit_trail.len()).unwrap();
            prop_assert_eq!(vendor_count, 1);
            prop_assert_eq!(audit_count, 1);
        }

        #[test]
        fn confidence_bps_within_range(bps in 0u16..=10_000u16) {
            let decision = DecisionRecord {
                id: Uuid::new_v4(),
                truth_key: "test".into(),
                recommendation: "ok".into(),
                confidence_bps: bps,
                rationale: "test".into(),
                vendor_id: None,
                needs_human_review: false,
                decided_by: Actor::system(),
                decided_at: Utc::now(),
            };
            let json = serde_json::to_string(&decision).unwrap();
            let parsed: DecisionRecord = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(parsed.confidence_bps, bps);
        }

        #[test]
        fn risk_score_bps_roundtrips(score_bps in 0u16..=10_000u16, dim in "[a-z]{3,20}") {
            let score = RiskScore {
                id: Uuid::new_v4(),
                vendor_id: Uuid::new_v4(),
                dimension: dim.clone(),
                score_bps,
                rationale: "test".into(),
                scored_by: Actor::system(),
                scored_at: Utc::now(),
            };
            let json = serde_json::to_string(&score).unwrap();
            let parsed: RiskScore = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(parsed.score_bps, score_bps);
            prop_assert_eq!(parsed.dimension, dim);
        }

        #[test]
        fn compliance_status_roundtrips(status in arb_compliance_status()) {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: ComplianceStatus = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(parsed, status);
        }

        #[test]
        fn severity_roundtrips(sev in arb_severity()) {
            let json = serde_json::to_string(&sev).unwrap();
            let parsed: Severity = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(parsed, sev);
        }

        #[test]
        fn failed_writes_never_change_state(
            n in 1usize..20,
        ) {
            let store = InMemoryStore::new();
            for _ in 0..n {
                let _: Result<StoreWriteResult<()>, StoreError> = store.write_with_events(|k| {
                    k.register_vendor("Ghost".into(), "fail".into(), &Actor::system());
                    Err(KernelError::Validation("forced".into()))
                });
            }
            let count = store.read(|k| k.vendors.len()).unwrap();
            prop_assert_eq!(count, 0);
        }

        #[test]
        fn multiple_vendors_all_queryable(count in 1usize..30) {
            let store = InMemoryStore::new();
            for i in 0..count {
                store
                    .write_with_events(|k| {
                        k.register_vendor(format!("V{i}"), "d".into(), &Actor::system());
                        Ok(())
                    })
                    .unwrap();
            }
            let actual = store.read(|k| k.vendors.len()).unwrap();
            prop_assert_eq!(actual, count);
        }

        #[test]
        fn actor_kind_serialization(kind in prop_oneof![
            Just(ActorKind::Human),
            Just(ActorKind::Agent),
            Just(ActorKind::System),
        ]) {
            let json = serde_json::to_string(&kind).unwrap();
            let parsed: ActorKind = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(parsed, kind);
        }
    }
}
