use std::sync::{Arc, Mutex};

use chrono::Utc;
use converge_kernel::ExperienceEventObserver;
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

impl ExperienceEventObserver for InMemoryExperienceStream {
    fn on_event(&self, event: &converge_kernel::ExperienceEvent) {
        if let Ok(mut buffer) = self.events.lock() {
            buffer.push(ExperienceEvent {
                occurred_at: Utc::now().to_rfc3339(),
                payload: serde_json::json!({
                    "source": "converge-engine",
                    "kind": format!("{:?}", event.kind()),
                }),
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

// ---------------------------------------------------------------------------
// Experience registry — persistent across runs and server restarts
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DEFAULT_STORE_PATH: &str = "data/experience_store.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub cycles: u32,
    pub elapsed_ms: u64,
    pub vendor_count: usize,
    pub converged: bool,
    pub confidence: f64,
    pub recommended_vendor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_document_path: Option<String>,
    #[serde(default)]
    pub static_fact_count: usize,
    #[serde(default)]
    pub static_fact_paths: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct RunSummaryInput<'a> {
    pub cycles: u32,
    pub elapsed_ms: u64,
    pub vendor_count: usize,
    pub converged: bool,
    pub confidence: f64,
    pub recommended_vendor: &'a str,
    pub source_document_path: Option<&'a str>,
    pub static_fact_count: usize,
    pub static_fact_paths: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperienceSnapshot {
    pub truth_key: String,
    pub run_count: usize,
    pub summaries: Vec<RunSummary>,
    pub aggregate: ExperienceAggregate,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperienceAggregate {
    pub convergence_rate: f64,
    pub avg_cycles: f64,
    pub avg_confidence: f64,
    pub avg_elapsed_ms: u64,
    pub recommendation_frequencies: Vec<RecommendationFrequency>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecommendationFrequency {
    pub recommendation: String,
    pub count: usize,
    pub share: f64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PersistedState {
    run_summaries: HashMap<String, Vec<RunSummary>>,
}

#[derive(Debug)]
pub struct ExperienceRegistry {
    streams: std::sync::Mutex<HashMap<String, Arc<InMemoryExperienceStream>>>,
    run_summaries: std::sync::Mutex<HashMap<String, Vec<RunSummary>>>,
    store_path: PathBuf,
}

impl Default for ExperienceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperienceRegistry {
    pub fn new() -> Self {
        Self::with_path(DEFAULT_STORE_PATH)
    }

    pub fn with_path(path: impl AsRef<Path>) -> Self {
        let store_path = path.as_ref().to_path_buf();
        let loaded = Self::load_from_disk(&store_path);
        Self {
            streams: std::sync::Mutex::new(HashMap::new()),
            run_summaries: std::sync::Mutex::new(loaded),
            store_path,
        }
    }

    fn load_from_disk(path: &Path) -> HashMap<String, Vec<RunSummary>> {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str::<PersistedState>(&contents) {
                Ok(state) => {
                    let total: usize = state.run_summaries.values().map(|v| v.len()).sum();
                    tracing::info!(
                        "Loaded {} prior runs across {} truth keys from {}",
                        total,
                        state.run_summaries.len(),
                        path.display(),
                    );
                    state.run_summaries
                }
                Err(e) => {
                    tracing::warn!("Failed to parse experience store {}: {e}", path.display());
                    HashMap::new()
                }
            },
            Err(_) => {
                tracing::info!(
                    "No existing experience store at {}, starting fresh",
                    path.display()
                );
                HashMap::new()
            }
        }
    }

    fn flush_to_disk(&self) {
        let summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        let state = PersistedState {
            run_summaries: summaries.clone(),
        };
        drop(summaries);

        if let Some(parent) = self.store_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(&state) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&self.store_path, json) {
                    tracing::warn!("Failed to write experience store: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to serialize experience store: {e}");
            }
        }
    }

    pub fn get_or_create(&self, truth_key: &str) -> Arc<InMemoryExperienceStream> {
        let mut streams = self.streams.lock().unwrap_or_else(|e| e.into_inner());
        streams
            .entry(truth_key.to_string())
            .or_insert_with(|| Arc::new(InMemoryExperienceStream::default()))
            .clone()
    }

    pub fn record_run_summary(&self, truth_key: &str, input: RunSummaryInput<'_>) {
        let summary = RunSummary {
            run_id: uuid::Uuid::new_v4().to_string(),
            cycles: input.cycles,
            elapsed_ms: input.elapsed_ms,
            vendor_count: input.vendor_count,
            converged: input.converged,
            confidence: input.confidence,
            recommended_vendor: input.recommended_vendor.to_string(),
            source_document_path: input.source_document_path.map(ToString::to_string),
            static_fact_count: input.static_fact_count,
            static_fact_paths: input.static_fact_paths,
            timestamp: Utc::now().to_rfc3339(),
        };
        let mut summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        summaries
            .entry(truth_key.to_string())
            .or_default()
            .push(summary);
        drop(summaries);
        self.flush_to_disk();
    }

    pub fn run_count(&self, truth_key: &str) -> usize {
        let summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        summaries.get(truth_key).map(|v| v.len()).unwrap_or(0)
    }

    pub fn all_summaries(&self, truth_key: &str) -> Vec<RunSummary> {
        let summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        summaries.get(truth_key).cloned().unwrap_or_default()
    }

    pub fn snapshot(&self, truth_key: &str) -> ExperienceSnapshot {
        let mut runs = self.all_summaries(truth_key);
        runs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let run_count = runs.len();
        let aggregate = if runs.is_empty() {
            ExperienceAggregate {
                convergence_rate: 0.0,
                avg_cycles: 0.0,
                avg_confidence: 0.0,
                avg_elapsed_ms: 0,
                recommendation_frequencies: Vec::new(),
            }
        } else {
            let converged = runs.iter().filter(|run| run.converged).count();
            let avg_cycles =
                runs.iter().map(|run| run.cycles as f64).sum::<f64>() / run_count as f64;
            let avg_confidence =
                runs.iter().map(|run| run.confidence).sum::<f64>() / run_count as f64;
            let avg_elapsed_ms =
                runs.iter().map(|run| run.elapsed_ms).sum::<u64>() / run_count as u64;

            let mut counts = HashMap::<String, usize>::new();
            for run in &runs {
                if !run.recommended_vendor.trim().is_empty() {
                    *counts.entry(run.recommended_vendor.clone()).or_default() += 1;
                }
            }
            let mut recommendation_frequencies: Vec<_> = counts
                .into_iter()
                .map(|(recommendation, count)| RecommendationFrequency {
                    recommendation,
                    count,
                    share: count as f64 / run_count as f64,
                })
                .collect();
            recommendation_frequencies.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.recommendation.cmp(&b.recommendation))
            });

            ExperienceAggregate {
                convergence_rate: converged as f64 / run_count as f64,
                avg_cycles: (avg_cycles * 10.0).round() / 10.0,
                avg_confidence: (avg_confidence * 1000.0).round() / 1000.0,
                avg_elapsed_ms,
                recommendation_frequencies,
            }
        };

        ExperienceSnapshot {
            truth_key: truth_key.to_string(),
            run_count,
            summaries: runs,
            aggregate,
        }
    }

    pub fn prior_decisions_summary(&self, truth_key: &str) -> String {
        let summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        let runs = match summaries.get(truth_key) {
            Some(runs) if !runs.is_empty() => runs,
            _ => return String::new(),
        };

        let mut lines = vec![format!("Prior runs: {}", runs.len())];
        for (i, run) in runs.iter().rev().take(5).enumerate() {
            lines.push(format!(
                "  Run -{}: recommended={}, confidence={:.2}, cycles={}, converged={}, elapsed={}ms, source={}, static_facts={}",
                i + 1,
                run.recommended_vendor,
                run.confidence,
                run.cycles,
                run.converged,
                run.elapsed_ms,
                run.source_document_path.as_deref().unwrap_or("none"),
                run.static_fact_count,
            ));
        }
        lines.join("\n")
    }

    pub fn learning_metrics(
        &self,
        truth_key: &str,
        current_cycles: u32,
        current_confidence: f64,
    ) -> Value {
        let summaries = self.run_summaries.lock().unwrap_or_else(|e| e.into_inner());
        let runs = match summaries.get(truth_key) {
            Some(runs) if !runs.is_empty() => runs,
            _ => {
                return serde_json::json!({
                    "prior_runs": 0,
                    "status": "first_run"
                });
            }
        };

        // Exclude the current run (just recorded) from prior stats
        let prior_count = runs.len().saturating_sub(1);
        if prior_count == 0 {
            return serde_json::json!({
                "prior_runs": 0,
                "status": "first_run"
            });
        }

        let prior_runs = &runs[..prior_count];
        let avg_cycles: f64 =
            prior_runs.iter().map(|r| r.cycles as f64).sum::<f64>() / prior_count as f64;
        let avg_confidence: f64 =
            prior_runs.iter().map(|r| r.confidence).sum::<f64>() / prior_count as f64;
        let avg_elapsed: f64 =
            prior_runs.iter().map(|r| r.elapsed_ms as f64).sum::<f64>() / prior_count as f64;

        let cycle_improvement = if avg_cycles > 0.0 {
            ((avg_cycles - current_cycles as f64) / avg_cycles * 100.0).round()
        } else {
            0.0
        };
        let confidence_improvement = if avg_confidence > 0.0 {
            ((current_confidence - avg_confidence) / avg_confidence * 100.0).round()
        } else {
            0.0
        };

        // Check recommendation consistency
        let most_recommended = prior_runs
            .iter()
            .filter(|r| !r.recommended_vendor.is_empty())
            .fold(HashMap::<&str, usize>::new(), |mut acc, r| {
                *acc.entry(&r.recommended_vendor).or_default() += 1;
                acc
            });
        let consistent_vendor = most_recommended
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(vendor, count)| (*vendor, *count));

        let mut result = serde_json::json!({
            "prior_runs": prior_count,
            "avg_prior_cycles": (avg_cycles * 10.0).round() / 10.0,
            "this_run_cycles": current_cycles,
            "avg_prior_confidence": (avg_confidence * 1000.0).round() / 1000.0,
            "this_run_confidence": (current_confidence * 1000.0).round() / 1000.0,
            "avg_prior_elapsed_ms": avg_elapsed.round() as u64,
            "cycle_improvement_pct": cycle_improvement,
            "confidence_improvement_pct": confidence_improvement,
        });

        if let Some((vendor, count)) = consistent_vendor {
            result.as_object_mut().unwrap().insert(
                "consistent_recommendation".to_string(),
                serde_json::json!({
                    "vendor": vendor,
                    "count": count,
                    "total_prior_runs": prior_count,
                }),
            );
        }

        if let Some(last_prior) = prior_runs.last() {
            result.as_object_mut().unwrap().insert(
                "latest_prior_source".to_string(),
                serde_json::json!({
                    "source_document_path": last_prior.source_document_path.as_deref(),
                    "static_fact_count": last_prior.static_fact_count,
                    "static_fact_paths": &last_prior.static_fact_paths,
                }),
            );
        }

        result
    }
}
