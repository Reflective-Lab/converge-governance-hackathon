use std::sync::{Arc, Mutex};

use chrono::Utc;
use governance_kernel::{
    Actor, AuditEntry, ComplianceCheck, CostEstimate, DecisionRecord, DomainEvent, DomainEvent::*,
    DomainEventStream, RiskScore, Vendor,
};
use serde_json::Value;

/// Minimal first-cut experience stream event.
///
/// This keeps the first implementation intentionally small: it records emitted
/// kernel events in an in-memory append-only buffer, preserving event order for
/// replay and diagnostics.
#[derive(Debug, Clone)]
pub struct ExperienceEvent {
    pub occurred_at: String,
    pub payload: Value,
}

/// In-memory stream adapter for kernel-domain events.
#[derive(Debug, Default)]
pub struct InMemoryExperienceStream {
    events: Arc<Mutex<Vec<ExperienceEvent>>>,
}

impl InMemoryExperienceStream {
    pub fn snapshot(&self) -> Vec<ExperienceEvent> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }
}

impl DomainEventStream for InMemoryExperienceStream {
    fn record_events(&self, events: &[DomainEvent]) {
        if events.is_empty() {
            return;
        }

        let mut buffer = match self.events.lock() {
            Ok(events) => events,
            Err(_) => return,
        };

        for event in events {
            buffer.push(ExperienceEvent {
                occurred_at: Utc::now().to_rfc3339(),
                payload: domain_event_payload(event),
            });
        }
    }
}

fn domain_event_payload(event: &DomainEvent) -> Value {
    match event {
        VendorRegistered { vendor, actor } => serde_json::json!({
            "type": "vendor-registered",
            "vendor": serialize_vendor(vendor),
            "actor": serialize_actor(actor),
        }),
        ComplianceChecked { check, actor } => serde_json::json!({
            "type": "compliance-checked",
            "check": serialize_compliance_check(check),
            "actor": serialize_actor(actor),
        }),
        RiskScored { score, actor } => serde_json::json!({
            "type": "risk-scored",
            "score": serialize_risk_score(score),
            "actor": serialize_actor(actor),
        }),
        CostEstimated { estimate, actor } => serde_json::json!({
            "type": "cost-estimated",
            "estimate": serialize_cost_estimate(estimate),
            "actor": serialize_actor(actor),
        }),
        DecisionRecorded { decision, actor } => serde_json::json!({
            "type": "decision-recorded",
            "decision": serialize_decision_record(decision),
            "actor": serialize_actor(actor),
        }),
        AuditRecorded { entry } => serde_json::json!({
            "type": "audit-recorded",
            "entry": serialize_audit_entry(entry),
        }),
    }
}

fn serialize_vendor(vendor: &Vendor) -> Value {
    serde_json::json!({
        "id": vendor.id.to_string(),
        "name": vendor.name,
    })
}

fn serialize_actor(actor: &Actor) -> Value {
    serde_json::json!({
        "id": actor.id,
        "name": actor.name,
        "kind": format!("{:?}", actor.kind),
    })
}

fn serialize_compliance_check(check: &ComplianceCheck) -> Value {
    serde_json::json!({
        "id": check.id.to_string(),
        "vendor_id": check.vendor_id.to_string(),
        "policy_rule_id": check.policy_rule_id.to_string(),
        "status": format!("{:?}", check.status),
        "checked_by": check.checked_by.id,
    })
}

fn serialize_risk_score(score: &RiskScore) -> Value {
    serde_json::json!({
        "id": score.id.to_string(),
        "vendor_id": score.vendor_id.to_string(),
        "dimension": score.dimension,
        "score_bps": score.score_bps,
        "scored_by": score.scored_by.id,
    })
}

fn serialize_cost_estimate(estimate: &CostEstimate) -> Value {
    serde_json::json!({
        "id": estimate.id.to_string(),
        "vendor_id": estimate.vendor_id.to_string(),
        "monthly_cost_minor": estimate.monthly_cost_minor,
        "currency_code": estimate.currency_code,
        "estimated_by": estimate.estimated_by.id,
    })
}

fn serialize_decision_record(decision: &DecisionRecord) -> Value {
    serde_json::json!({
        "id": decision.id.to_string(),
        "truth_key": decision.truth_key,
        "recommendation": decision.recommendation,
        "confidence_bps": decision.confidence_bps,
        "needs_human_review": decision.needs_human_review,
        "vendor_id": decision.vendor_id.map(|id| id.to_string()),
    })
}

fn serialize_audit_entry(entry: &AuditEntry) -> Value {
    serde_json::json!({
        "id": entry.id.to_string(),
        "action": entry.action,
        "actor_id": entry.actor.id,
        "actor_kind": format!("{:?}", entry.actor.kind),
    })
}
