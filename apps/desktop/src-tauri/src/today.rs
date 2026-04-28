use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use governance_kernel::InMemoryStore;
use governance_server::experience::{ExperienceRegistry, ExperienceSnapshot};
use governance_server::llm_helpers::load_env;
use governance_server::truth_runtime::TruthExecutionResult;
use governance_server::truth_runtime::vendor_selection::{self, ModelOverrides};
use serde::{Deserialize, Serialize};

const VENDORS_JSON: &str =
    include_str!("../../../../examples/vendor-selection/demo-ai-vendors.json");
const BUYER_BRIEF: &str = include_str!("../../../../examples/vendor-selection/buyer-brief.md");
const STATIC_FACTS: &str = include_str!("../../../../examples/vendor-selection/static-facts.json");

const VENDORS_PATH: &str = "examples/vendor-selection/demo-ai-vendors.json";
const BUYER_BRIEF_PATH: &str = "examples/vendor-selection/buyer-brief.md";
const STATIC_FACTS_PATH: &str = "examples/vendor-selection/static-facts.json";
const TRUTH_KEY: &str = "vendor-selection";

const REPLAY_SCHEMA_VERSION: u32 = 1;
const REPLAY_FILE_NAME: &str = "today-live-session.json";
const REPLAY_DELAYS_MS: [u64; 6] = [5_200, 3_600, 3_200, 2_400, 2_200, 2_000];

#[derive(Debug, Serialize, Deserialize)]
pub struct TodayRunResponse {
    pub stage: String,
    pub result: TruthExecutionResult,
    pub experience: ExperienceSnapshot,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodayReplaySession {
    pub schema_version: u32,
    pub recorded_at: String,
    pub source_hash: String,
    pub mode: String,
    pub runs: Vec<TodayRecordedRun>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodayRecordedRun {
    pub stage: String,
    pub result: TruthExecutionResult,
    pub experience: ExperienceSnapshot,
    pub compressed_delay_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_elapsed_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TodayReplayStatus {
    pub available: bool,
    pub path: String,
    pub mode: Option<String>,
    pub recorded_at: Option<String>,
    pub source_hash: Option<String>,
    pub source_matches: bool,
    pub run_count: usize,
    pub model_summary: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn reset_experience() -> Result<(), String> {
    let path = experience_path();
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to reset {}: {error}", path.display())),
    }
}

pub fn experience_snapshot() -> ExperienceSnapshot {
    ExperienceRegistry::with_path(experience_path()).snapshot(TRUTH_KEY)
}

pub fn replay_status() -> TodayReplayStatus {
    match load_replay_session() {
        Ok(session) => status_from_session(session, None),
        Err(error) => TodayReplayStatus {
            available: false,
            path: recording_path().display().to_string(),
            mode: None,
            recorded_at: None,
            source_hash: None,
            source_matches: false,
            run_count: 0,
            model_summary: Vec::new(),
            error: if recording_path().is_file() {
                Some(error)
            } else {
                None
            },
        },
    }
}

pub fn load_replay_session() -> Result<TodayReplaySession, String> {
    let path = recording_path();
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read replay session {}: {error}", path.display()))?;
    let session = serde_json::from_str::<TodayReplaySession>(&contents)
        .map_err(|error| format!("failed to parse replay session {}: {error}", path.display()))?;

    if session.schema_version != REPLAY_SCHEMA_VERSION {
        return Err(format!(
            "unsupported replay schema {} in {}; expected {}",
            session.schema_version,
            path.display(),
            REPLAY_SCHEMA_VERSION
        ));
    }

    Ok(session)
}

pub fn clear_replay_session() -> Result<(), String> {
    let path = recording_path();
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove replay session {}: {error}",
            path.display()
        )),
    }
}

pub async fn record_replay_session() -> Result<TodayReplaySession, String> {
    reset_experience()?;

    let mut runs = Vec::new();
    for (index, stage) in [
        "analysis",
        "approved",
        "negative-control",
        "learning",
        "learning",
        "learning",
    ]
    .iter()
    .enumerate()
    {
        let started = SystemTime::now();
        let response = run_stage(stage, true).await?;
        let original_elapsed_ms = started
            .elapsed()
            .ok()
            .and_then(|elapsed| u64::try_from(elapsed.as_millis()).ok());
        runs.push(TodayRecordedRun {
            stage: response.stage,
            result: response.result,
            experience: response.experience,
            compressed_delay_ms: REPLAY_DELAYS_MS[index],
            original_elapsed_ms,
        });
    }

    let session = TodayReplaySession {
        schema_version: REPLAY_SCHEMA_VERSION,
        recorded_at: recorded_at_now(),
        source_hash: current_source_hash(),
        mode: "recorded-live".to_string(),
        runs,
    };
    write_replay_session(&session)?;
    Ok(session)
}

pub async fn build_offline_replay_session() -> Result<TodayReplaySession, String> {
    reset_experience()?;

    let mut runs = Vec::new();
    for (index, stage) in [
        "analysis",
        "approved",
        "negative-control",
        "learning",
        "learning",
        "learning",
    ]
    .iter()
    .enumerate()
    {
        let response = run_stage(stage, false).await?;
        runs.push(TodayRecordedRun {
            stage: response.stage,
            result: response.result,
            experience: response.experience,
            compressed_delay_ms: REPLAY_DELAYS_MS[index],
            original_elapsed_ms: None,
        });
    }

    let session = TodayReplaySession {
        schema_version: REPLAY_SCHEMA_VERSION,
        recorded_at: recorded_at_now(),
        source_hash: current_source_hash(),
        mode: "offline-presentation-fixture".to_string(),
        runs,
    };
    write_replay_session(&session)?;
    Ok(session)
}

pub async fn run_stage(stage: &str, live: bool) -> Result<TodayRunResponse, String> {
    if live {
        load_repo_env();
        load_env();
        require_live_provider_env()?;
    }

    let normalized_stage = normalize_stage(stage)?;
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::with_path(experience_path());
    let inputs = inputs_for_stage(normalized_stage, live)?;
    let result = if live {
        vendor_selection::execute_with_model_overrides(
            &store,
            &inputs,
            true,
            Some(&experience),
            &today_model_overrides(),
        )
        .await
    } else {
        vendor_selection::execute_with_experience(&store, &inputs, true, Some(&experience)).await
    }?;

    if live {
        require_real_llm_calls(&result, normalized_stage)?;
    }

    Ok(TodayRunResponse {
        stage: normalized_stage.to_string(),
        result,
        experience: experience.snapshot(TRUTH_KEY),
    })
}

fn normalize_stage(stage: &str) -> Result<&'static str, String> {
    match stage {
        "analysis" | "before-hitl" => Ok("analysis"),
        "approved" | "promote" | "after-hitl" => Ok("approved"),
        "negative-control" | "advisory" => Ok("negative-control"),
        "learning" | "learning-loop" => Ok("learning"),
        other => Err(format!("unknown today demo stage: {other}")),
    }
}

fn inputs_for_stage(stage: &str, live: bool) -> Result<HashMap<String, String>, String> {
    let static_facts = serde_json::from_str::<serde_json::Value>(STATIC_FACTS)
        .map_err(|error| format!("invalid embedded static facts: {error}"))?;
    let static_fact_paths = vec![STATIC_FACTS_PATH.to_string()];
    let static_fact_bundle = vec![serde_json::json!({
        "path": STATIC_FACTS_PATH,
        "content": static_facts,
    })];

    let mut inputs = HashMap::from([
        ("vendors_json".to_string(), VENDORS_JSON.to_string()),
        ("vendors_json_path".to_string(), VENDORS_PATH.to_string()),
        (
            "source_document_path".to_string(),
            BUYER_BRIEF_PATH.to_string(),
        ),
        ("source_document".to_string(), BUYER_BRIEF.to_string()),
        ("min_score".to_string(), "75".to_string()),
        ("max_risk".to_string(), "30".to_string()),
        ("max_vendors".to_string(), "3".to_string()),
        ("demo_mode".to_string(), "governed".to_string()),
        ("principal_authority".to_string(), "supervisory".to_string()),
        (
            "static_facts_paths_json".to_string(),
            serde_json::to_string(&static_fact_paths)
                .map_err(|error| format!("failed to encode static fact paths: {error}"))?,
        ),
        (
            "static_facts_json".to_string(),
            serde_json::to_string(&static_fact_bundle)
                .map_err(|error| format!("failed to encode static facts: {error}"))?,
        ),
    ]);

    if live {
        inputs.insert("live_mode".to_string(), "true".to_string());
    }

    match stage {
        "analysis" => {
            inputs.insert("human_approval_present".to_string(), "false".to_string());
        }
        "approved" => {
            inputs.insert("human_approval_present".to_string(), "true".to_string());
        }
        "negative-control" => {
            inputs.insert("principal_authority".to_string(), "advisory".to_string());
        }
        "learning" => {}
        _ => {}
    }

    Ok(inputs)
}

fn today_model_overrides() -> ModelOverrides {
    let default_model = default_demo_model();
    ModelOverrides {
        compliance: Some(role_model_env("DEMO_COMPLIANCE_MODEL", &default_model)),
        cost: Some(role_model_env("DEMO_COST_MODEL", &default_model)),
        risk: Some(role_model_env("DEMO_RISK_MODEL", &default_model)),
        synthesis: Some(role_model_env("DEMO_SYNTHESIS_MODEL", &default_model)),
    }
}

fn default_demo_model() -> String {
    std::env::var("DEMO_LIVE_MODEL").unwrap_or_else(|_| {
        if env_has_value("OPENAI_API_KEY") {
            "gpt-4o-mini".to_string()
        } else if env_has_value("GEMINI_API_KEY") {
            "gemini-2.5-flash".to_string()
        } else if env_has_value("ANTHROPIC_API_KEY") {
            "claude-sonnet-4-6".to_string()
        } else {
            "openai/gpt-4o-mini".to_string()
        }
    })
}

fn role_model_env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn experience_path() -> PathBuf {
    std::env::temp_dir().join("governance-desktop-today-experience.json")
}

fn recording_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../public/demo")
        .join(REPLAY_FILE_NAME)
}

fn write_replay_session(session: &TodayReplaySession) -> Result<(), String> {
    let path = recording_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create replay session directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let json = serde_json::to_string_pretty(session)
        .map_err(|error| format!("failed to serialize replay session: {error}"))?;
    std::fs::write(&path, json)
        .map_err(|error| format!("failed to write replay session {}: {error}", path.display()))
}

fn status_from_session(session: TodayReplaySession, error: Option<String>) -> TodayReplayStatus {
    let source_hash = current_source_hash();
    let model_summary = model_summary(&session.runs);
    let run_count = session.runs.len();
    let mode = session.mode;
    let recorded_at = session.recorded_at;
    let stored_source_hash = session.source_hash;
    TodayReplayStatus {
        available: true,
        path: recording_path().display().to_string(),
        mode: Some(mode),
        recorded_at: Some(recorded_at),
        source_matches: stored_source_hash == source_hash,
        source_hash: Some(stored_source_hash),
        run_count,
        model_summary,
        error,
    }
}

fn model_summary(runs: &[TodayRecordedRun]) -> Vec<String> {
    let mut seen = HashSet::<String>::new();
    let mut summary = Vec::new();

    for run in runs {
        for call in run.result.llm_calls.as_deref().unwrap_or_default() {
            let label = format!("{} -> {}", call.context, call.model);
            if seen.insert(label.clone()) {
                summary.push(label);
            }
            if summary.len() >= 8 {
                return summary;
            }
        }
    }

    summary
}

fn recorded_at_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

fn current_source_hash() -> String {
    let mut hasher = StableHasher::new();
    hasher.write(VENDORS_JSON.as_bytes());
    hasher.write(BUYER_BRIEF.as_bytes());
    hasher.write(STATIC_FACTS.as_bytes());
    format!("{:016x}", hasher.finish())
}

struct StableHasher(u64);

impl StableHasher {
    fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

fn load_repo_env() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for path in [
        manifest_dir.join("../../../.env"),
        manifest_dir.join("../.env"),
        manifest_dir.join(".env"),
    ] {
        if path.is_file() {
            let _ = dotenv::from_path(path);
        }
    }
}

fn require_live_provider_env() -> Result<(), String> {
    let has_openrouter = env_has_value("OPENROUTER_API_KEY");
    let has_direct_provider = env_has_value("ANTHROPIC_API_KEY")
        || env_has_value("OPENAI_API_KEY")
        || env_has_value("GEMINI_API_KEY");
    let has_kong = env_has_value("KONG_AI_GATEWAY_URL") && env_has_value("KONG_API_KEY");

    if has_openrouter || has_direct_provider || has_kong {
        return Ok(());
    }

    Err(
        "no live LLM credentials are visible to the desktop process. Add OPENROUTER_API_KEY, ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY, or Kong credentials to the repo-root .env, then relaunch the Tauri app."
            .to_string(),
    )
}

fn env_has_value(name: &str) -> bool {
    std::env::var(name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn require_real_llm_calls(result: &TruthExecutionResult, stage: &str) -> Result<(), String> {
    let calls = result.llm_calls.as_deref().unwrap_or_default();
    if calls.is_empty() {
        return Err(format!(
            "the {stage} stage did not record any real LLM calls. Check OPENROUTER_API_KEY or direct provider credentials before running the live desktop demo."
        ));
    }

    let fallback_contexts = calls
        .iter()
        .filter(|call| {
            call.provider == "none"
                || call.model == "deterministic-fallback"
                || call
                    .metadata
                    .get("fallback")
                    .is_some_and(|value| value == "true")
        })
        .map(|call| call.context.as_str())
        .collect::<Vec<_>>();

    if !fallback_contexts.is_empty() {
        return Err(format!(
            "the {stage} stage fell back instead of using real LLM calls: {}. Check provider credentials and model availability.",
            fallback_contexts.join(", ")
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn today_stages_match_demo_policy_story() {
        reset_experience().unwrap();

        let analysis = run_stage("analysis", false).await.unwrap();
        let analysis_policy = policy(&analysis);
        assert_eq!(analysis_policy["outcome"], "Escalate");
        assert_eq!(analysis_policy["selected_vendor"], "Mistral");
        assert_eq!(analysis_policy["human_approval_present"], false);

        let approved = run_stage("approved", false).await.unwrap();
        let approved_policy = policy(&approved);
        assert_eq!(approved_policy["outcome"], "Promote");
        assert_eq!(approved_policy["selected_vendor"], "Mistral");
        assert_eq!(approved_policy["human_approval_present"], true);

        let negative = run_stage("negative-control", false).await.unwrap();
        let negative_policy = policy(&negative);
        assert_eq!(negative_policy["outcome"], "Reject");
        assert_eq!(negative_policy["principal_authority"], "advisory");

        reset_experience().unwrap();
    }

    #[test]
    fn replay_metadata_is_stable_enough_for_status() {
        let hash = current_source_hash();
        assert_eq!(hash.len(), 16);
        assert!(recording_path().ends_with(REPLAY_FILE_NAME));
    }

    fn policy(response: &TodayRunResponse) -> &serde_json::Value {
        response
            .result
            .projection
            .as_ref()
            .and_then(|projection| projection.details.as_ref())
            .and_then(|details| details.get("policy"))
            .expect("policy details should be projected")
    }
}
