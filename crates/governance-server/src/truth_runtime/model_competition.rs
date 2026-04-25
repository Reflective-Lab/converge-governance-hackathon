use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use chrono::Utc;
use converge_provider::{
    ProviderBenchmarkRun, ProviderCallSample, ProviderLeaderboardEntry, ScoredProviderRun,
    build_provider_overall_leaderboard, build_provider_role_leaderboard, score_provider_runs,
};
use governance_kernel::InMemoryStore;
use governance_telemetry::LlmCallTelemetry;
use serde::{Deserialize, Serialize};

use super::TruthExecutionResult;
use super::vendor_selection::{ModelOverrides, execute_with_model_overrides};
use crate::experience::ExperienceRegistry;

// ---------------------------------------------------------------------------
// Competition models
// ---------------------------------------------------------------------------

pub const COMPETITION_MODELS: &[&str] = &[
    // Tier: frontier
    "anthropic/claude-opus-4.7",
    "openai/gpt-5.4-pro",
    "x-ai/grok-4.20-multi-agent",
    "google/gemini-3.1-pro-preview",
    // Tier: strong
    "qwen/qwen3.6-plus",
    "minimax/minimax-m2.7",
    "z-ai/glm-5.1",
    "moonshotai/kimi-k2.6",
    "arcee-ai/trinity-large-thinking",
    "nvidia/nemotron-3-super-120b-a12b",
    "writer/palmyra-x5",
    // Tier: mid
    "openai/gpt-5.4-nano",
    "google/gemini-3.1-flash-lite-preview",
    "qwen/qwen3.5-27b",
    "z-ai/glm-5-turbo",
    "minimax/minimax-m2.5",
    "moonshotai/kimi-k2.5",
    "mistralai/mistral-small-2603",
    "bytedance-seed/seed-2.0-lite",
    // Tier: compact
    "google/gemma-4-31b-it",
    "arcee-ai/trinity-large-preview",
];

// Known-good defaults for the roles we're NOT testing
const DEFAULT_COMPLIANCE: &str = "meta-llama/llama-3.1-8b-instruct";
const DEFAULT_COST: &str = "google/gemini-2.0-flash-001";
const DEFAULT_RISK: &str = "google/gemini-2.0-flash-001";
const DEFAULT_SYNTHESIS: &str = "anthropic/claude-sonnet-4";
const COMPETITION_EXPERIENCE_PATH_ENV: &str = "GOVERNANCE_COMPETITION_EXPERIENCE_PATH";
const DEFAULT_COMPETITION_EXPERIENCE_PATH: &str = "data/model_competition_experience_store.json";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CompetitionRole {
    Compliance,
    Cost,
    Risk,
    Synthesis,
}

impl CompetitionRole {
    pub fn all() -> &'static [CompetitionRole] {
        &[
            CompetitionRole::Compliance,
            CompetitionRole::Cost,
            CompetitionRole::Risk,
            CompetitionRole::Synthesis,
        ]
    }

    fn context_prefixes(&self) -> &[&str] {
        match self {
            CompetitionRole::Compliance => &["compliance:screen:"],
            CompetitionRole::Cost => &["cost:analysis"],
            CompetitionRole::Risk => &["risk:assessment"],
            CompetitionRole::Synthesis => &["decision:synthesis"],
        }
    }

    fn build_overrides(&self, model: &str) -> ModelOverrides {
        let mut overrides = ModelOverrides {
            compliance: Some(DEFAULT_COMPLIANCE.to_string()),
            cost: Some(DEFAULT_COST.to_string()),
            risk: Some(DEFAULT_RISK.to_string()),
            synthesis: Some(DEFAULT_SYNTHESIS.to_string()),
        };
        match self {
            CompetitionRole::Compliance => overrides.compliance = Some(model.to_string()),
            CompetitionRole::Cost => overrides.cost = Some(model.to_string()),
            CompetitionRole::Risk => overrides.risk = Some(model.to_string()),
            CompetitionRole::Synthesis => overrides.synthesis = Some(model.to_string()),
        }
        overrides
    }
}

impl std::fmt::Display for CompetitionRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompetitionRole::Compliance => write!(f, "Compliance"),
            CompetitionRole::Cost => write!(f, "Cost"),
            CompetitionRole::Risk => write!(f, "Risk"),
            CompetitionRole::Synthesis => write!(f, "Synthesis"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub model: String,
    pub role: CompetitionRole,
    pub success: bool,
    pub used_fallback: bool,
    pub converged: bool,
    pub cycles: u32,
    pub elapsed_ms: u64,
    pub target_latency_ms: u64,
    pub target_tokens: u64,
    pub confidence: f64,
    pub error: Option<String>,
    pub llm_calls: Vec<LlmCallTelemetry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredRun {
    pub model: String,
    pub role: CompetitionRole,
    pub success_score: f64,
    pub latency_score: f64,
    pub quality_score: f64,
    pub cost_score: f64,
    pub composite: f64,
    #[serde(flatten)]
    pub raw: RunResult,
}

pub type LeaderboardEntry = ProviderLeaderboardEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionReport {
    pub started_at: String,
    pub finished_at: String,
    pub total_runs: usize,
    pub successful_runs: usize,
    pub failed_runs: usize,
    pub runs: Vec<ScoredRun>,
    pub leaderboard_by_role: HashMap<String, Vec<LeaderboardEntry>>,
    pub leaderboard_overall: Vec<LeaderboardEntry>,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

pub async fn run_competition(
    models: &[&str],
    roles: &[CompetitionRole],
    vendor_inputs: &HashMap<String, String>,
) -> CompetitionReport {
    let started_at = Utc::now().to_rfc3339();
    let mut raw_results: Vec<RunResult> = Vec::new();
    let experience_path = competition_experience_path();
    tracing::info!(
        "Using model competition experience store at {}",
        experience_path.display()
    );
    let experience = ExperienceRegistry::with_path(&experience_path);

    let total = models.len() * roles.len();
    for (i, &role) in roles.iter().enumerate() {
        for (j, &model) in models.iter().enumerate() {
            let run_num = i * models.len() + j + 1;
            tracing::info!("[{run_num}/{total}] {role} <- {model}",);

            let result = run_single(model, role, vendor_inputs, &experience).await;

            match &result.error {
                Some(e) => tracing::warn!("  FAIL: {e}"),
                None if result.used_fallback => tracing::warn!(
                    "  FALLBACK: {}ms, {} tokens",
                    result.target_latency_ms,
                    result.target_tokens,
                ),
                None => tracing::info!(
                    "  OK: {}ms, {} tokens, confidence={:.2}",
                    result.target_latency_ms,
                    result.target_tokens,
                    result.confidence,
                ),
            }

            raw_results.push(result);

            // Brief pause to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    let scored = score_runs(&raw_results);
    let successful_runs = scored.iter().filter(|r| r.raw.success).count();
    let failed_runs = scored.len() - successful_runs;

    let leaderboard_by_role = build_role_leaderboards(&scored, roles);
    let leaderboard_overall = build_overall_leaderboard(&scored, models);

    CompetitionReport {
        started_at,
        finished_at: Utc::now().to_rfc3339(),
        total_runs: scored.len(),
        successful_runs,
        failed_runs,
        runs: scored,
        leaderboard_by_role,
        leaderboard_overall,
    }
}

fn competition_experience_path() -> PathBuf {
    std::env::var(COMPETITION_EXPERIENCE_PATH_ENV)
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_COMPETITION_EXPERIENCE_PATH))
}

async fn run_single(
    model: &str,
    role: CompetitionRole,
    inputs: &HashMap<String, String>,
    experience: &ExperienceRegistry,
) -> RunResult {
    let store = InMemoryStore::new();
    let overrides = role.build_overrides(model);

    let mut live_inputs = inputs.clone();
    live_inputs.insert("live_mode".to_string(), "true".to_string());

    let started = Instant::now();
    let result =
        execute_with_model_overrides(&store, &live_inputs, false, Some(experience), &overrides)
            .await;

    let elapsed_ms = started.elapsed().as_millis() as u64;

    match result {
        Err(e) => RunResult {
            model: model.to_string(),
            role,
            success: false,
            used_fallback: false,
            converged: false,
            cycles: 0,
            elapsed_ms,
            target_latency_ms: 0,
            target_tokens: 0,
            confidence: 0.0,
            error: Some(e),
            llm_calls: vec![],
        },
        Ok(exec_result) => run_result_from_execution(model, role, elapsed_ms, exec_result),
    }
}

fn run_result_from_execution(
    model: &str,
    role: CompetitionRole,
    elapsed_ms: u64,
    exec_result: TruthExecutionResult,
) -> RunResult {
    let calls = exec_result.llm_calls.unwrap_or_default();
    let confidence = extract_decision_confidence(
        exec_result
            .projection
            .as_ref()
            .and_then(|p| p.details.as_ref()),
    );
    let benchmark = ProviderBenchmarkRun::from_call_samples(
        model,
        role.to_string(),
        elapsed_ms,
        exec_result.converged,
        confidence,
        role.context_prefixes(),
        calls.iter().map(provider_call_sample).collect(),
    );

    RunResult {
        model: model.to_string(),
        role,
        success: benchmark.success,
        used_fallback: benchmark.used_fallback,
        converged: exec_result.converged,
        cycles: exec_result.cycles,
        elapsed_ms,
        target_latency_ms: benchmark.target_latency_ms,
        target_tokens: benchmark.target_tokens,
        confidence,
        error: None,
        llm_calls: calls,
    }
}

fn provider_call_sample(call: &LlmCallTelemetry) -> ProviderCallSample {
    ProviderCallSample {
        context: call.context.clone(),
        provider: Some(call.provider.clone()),
        model: call.model.clone(),
        elapsed_ms: call.elapsed_ms,
        total_tokens: call.usage.as_ref().and_then(|usage| usage.total_tokens),
        fallback: call.model == "deterministic-fallback",
    }
}

fn extract_decision_confidence(details: Option<&serde_json::Value>) -> f64 {
    details
        .and_then(|d| d.get("recommendation"))
        .and_then(|d| d.get("confidence"))
        .and_then(|c| c.as_f64())
        .or_else(|| {
            details
                .and_then(|d| d.get("decision"))
                .and_then(|d| d.get("confidence"))
                .and_then(|c| c.as_f64())
        })
        .or_else(|| {
            details
                .and_then(|d| d.get("learning"))
                .and_then(|l| l.get("this_run_confidence"))
                .and_then(|c| c.as_f64())
        })
        .unwrap_or(0.0)
}

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

fn score_runs(runs: &[RunResult]) -> Vec<ScoredRun> {
    let benchmark_runs: Vec<ProviderBenchmarkRun> =
        runs.iter().map(benchmark_run_from_result).collect();

    score_provider_runs(&benchmark_runs)
        .into_iter()
        .zip(runs.iter())
        .map(|(scored, run)| ScoredRun {
            model: run.model.clone(),
            role: run.role,
            success_score: scored.success_score,
            latency_score: scored.latency_score,
            quality_score: scored.quality_score,
            cost_score: scored.cost_score,
            composite: scored.composite,
            raw: run.clone(),
        })
        .collect()
}

fn build_role_leaderboards(
    scored: &[ScoredRun],
    roles: &[CompetitionRole],
) -> HashMap<String, Vec<LeaderboardEntry>> {
    let mut result = HashMap::new();
    let provider_scored: Vec<ScoredProviderRun> =
        scored.iter().map(provider_scored_from_scored_run).collect();
    for &role in roles {
        let entries = build_provider_role_leaderboard(&provider_scored, &role.to_string())
            .into_iter()
            .collect();
        result.insert(role.to_string(), entries);
    }
    result
}

fn build_overall_leaderboard(scored: &[ScoredRun], models: &[&str]) -> Vec<LeaderboardEntry> {
    let provider_scored: Vec<ScoredProviderRun> =
        scored.iter().map(provider_scored_from_scored_run).collect();

    build_provider_overall_leaderboard(&provider_scored, models)
}

fn benchmark_run_from_result(run: &RunResult) -> ProviderBenchmarkRun {
    ProviderBenchmarkRun {
        model: run.model.clone(),
        role: run.role.to_string(),
        success: run.success,
        used_fallback: run.used_fallback,
        converged: run.converged,
        elapsed_ms: run.elapsed_ms,
        target_latency_ms: run.target_latency_ms,
        target_tokens: run.target_tokens,
        confidence: run.confidence,
        error: run.error.clone(),
        calls: run.llm_calls.iter().map(provider_call_sample).collect(),
    }
}

fn provider_scored_from_scored_run(scored: &ScoredRun) -> ScoredProviderRun {
    ScoredProviderRun {
        model: scored.model.clone(),
        role: scored.role.to_string(),
        success_score: scored.success_score,
        latency_score: scored.latency_score,
        quality_score: scored.quality_score,
        cost_score: scored.cost_score,
        composite: scored.composite,
        raw: benchmark_run_from_result(&scored.raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use governance_telemetry::LlmUsageSummary;

    #[test]
    fn extracts_confidence_from_current_vendor_selection_projection() {
        let details = serde_json::json!({
            "recommendation": {
                "recommendation": "Acme AI",
                "confidence": 0.87,
                "needs_human_review": false
            }
        });

        assert_eq!(extract_decision_confidence(Some(&details)), 0.87);
    }

    #[test]
    fn extracts_confidence_from_legacy_decision_projection() {
        let details = serde_json::json!({
            "decision": {
                "confidence": 0.73
            }
        });

        assert_eq!(extract_decision_confidence(Some(&details)), 0.73);
    }

    #[test]
    fn extracts_confidence_from_learning_fallback() {
        let details = serde_json::json!({
            "learning": {
                "this_run_confidence": 0.64
            }
        });

        assert_eq!(extract_decision_confidence(Some(&details)), 0.64);
    }

    #[test]
    fn scores_successful_runs_with_nonzero_composite() {
        let runs = vec![RunResult {
            model: "test-model".to_string(),
            role: CompetitionRole::Synthesis,
            success: true,
            used_fallback: false,
            converged: true,
            cycles: 4,
            elapsed_ms: 1000,
            target_latency_ms: 100,
            target_tokens: 50,
            confidence: 0.8,
            error: None,
            llm_calls: Vec::new(),
        }];

        let scored = score_runs(&runs);

        assert_eq!(scored.len(), 1);
        assert!(scored[0].composite > 0.0);
        assert_eq!(scored[0].success_score, 1.0);
        assert_eq!(scored[0].quality_score, 0.8);
    }

    #[test]
    fn converts_execution_result_into_successful_run_without_network() {
        let result = TruthExecutionResult {
            converged: true,
            cycles: 4,
            stop_reason: "Converged".to_string(),
            criteria_outcomes: Vec::new(),
            projection: Some(super::super::TruthProjection {
                events_emitted: 0,
                details: Some(serde_json::json!({
                    "recommendation": {
                        "confidence": 0.91
                    }
                })),
            }),
            llm_calls: Some(vec![LlmCallTelemetry {
                context: "decision:synthesis".to_string(),
                provider: "openrouter".to_string(),
                model: "test-model".to_string(),
                elapsed_ms: 1234,
                finish_reason: Some("Stop".to_string()),
                usage: Some(LlmUsageSummary {
                    prompt_tokens: Some(100),
                    completion_tokens: Some(25),
                    total_tokens: Some(125),
                }),
                metadata: HashMap::new(),
            }]),
        };

        let run = run_result_from_execution("test-model", CompetitionRole::Synthesis, 1500, result);

        assert!(run.success);
        assert!(!run.used_fallback);
        assert_eq!(run.confidence, 0.91);
        assert_eq!(run.target_latency_ms, 1234);
        assert_eq!(run.target_tokens, 125);
    }

    #[test]
    fn target_fallback_marks_run_unsuccessful_without_penalizing_other_roles() {
        let result = TruthExecutionResult {
            converged: true,
            cycles: 4,
            stop_reason: "Converged".to_string(),
            criteria_outcomes: Vec::new(),
            projection: Some(super::super::TruthProjection {
                events_emitted: 0,
                details: Some(serde_json::json!({
                    "recommendation": {
                        "confidence": 0.82
                    }
                })),
            }),
            llm_calls: Some(vec![
                LlmCallTelemetry {
                    context: "cost:analysis".to_string(),
                    provider: "none".to_string(),
                    model: "deterministic-fallback".to_string(),
                    elapsed_ms: 0,
                    finish_reason: Some("fallback".to_string()),
                    usage: None,
                    metadata: HashMap::new(),
                },
                LlmCallTelemetry {
                    context: "decision:synthesis".to_string(),
                    provider: "openrouter".to_string(),
                    model: "test-model".to_string(),
                    elapsed_ms: 900,
                    finish_reason: Some("Stop".to_string()),
                    usage: Some(LlmUsageSummary {
                        prompt_tokens: Some(80),
                        completion_tokens: Some(20),
                        total_tokens: Some(100),
                    }),
                    metadata: HashMap::new(),
                },
            ]),
        };

        let synthesis_run =
            run_result_from_execution("test-model", CompetitionRole::Synthesis, 1000, result);

        assert!(synthesis_run.success);
        assert!(!synthesis_run.used_fallback);
    }
}
