use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use converge_provider::{
    AgentRequirements, ChatBackendSelectionConfig, ComplianceLevel, CostClass, CostTier,
    LatencyClass, RequiredCapabilities, SelectionCriteria, TaskComplexity,
};
use converge_provider_adapters::select_healthy_chat_backend;
use governance_kernel::InMemoryStore;
use governance_truths::{AGENT_MODELS, AgentModelConfig, is_product_truth, product_truths};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use crate::experience::ExperienceRegistry;

/// Shared application state for governance HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<InMemoryStore>,
    pub experience: Arc<ExperienceRegistry>,
}

/// Build the full governance API router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/truths", get(list_truths))
        .route("/v1/truths/{key}/execute", post(execute_truth))
        .route("/v1/decisions", get(list_decisions))
        .route("/v1/vendors", get(list_vendors))
        .route("/v1/audit", get(list_audit))
        .route("/v1/experience/{truth_key}", get(get_experience))
        .route(
            "/v1/agents/available-models",
            get(list_available_agent_models),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn list_truths() -> Json<Vec<TruthListItem>> {
    let items = product_truths()
        .map(|t| TruthListItem {
            key: t.key.into(),
            display_name: t.display_name.into(),
            summary: t.summary.into(),
            packs: t.packs.iter().map(|p| (*p).into()).collect(),
        })
        .collect();
    Json(items)
}

async fn execute_truth(
    State(state): State<AppState>,
    axum::extract::Path(key): axum::extract::Path<String>,
    Json(request): Json<ExecuteTruthRequest>,
) -> Result<Json<crate::truth_runtime::TruthExecutionResult>, (StatusCode, String)> {
    if !is_product_truth(&key) {
        return Err((
            StatusCode::NOT_FOUND,
            format!("truth '{key}' is not a product workflow; use vendor-selection"),
        ));
    }

    let persist = request.persist_projection.unwrap_or(true);
    // The converge Engine future is !Send (trait-object suggestors), so run on
    // a blocking thread with a local async runtime to satisfy axum's Send bound.
    let store_inner = (*state.store).clone();
    let experience = state.experience.clone();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(crate::truth_runtime::execute_truth(
                &store_inner,
                &key,
                request.inputs,
                persist,
                &experience,
            ))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn list_decisions(State(state): State<AppState>) -> impl IntoResponse {
    let decisions = state
        .store
        .read(|k| {
            k.recent_decisions(20)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Json(decisions)
}

async fn list_vendors(State(state): State<AppState>) -> impl IntoResponse {
    let vendors = state
        .store
        .read(|k| k.vendors_list().into_iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Json(vendors)
}

async fn list_audit(State(state): State<AppState>) -> impl IntoResponse {
    let entries = state
        .store
        .read(|k| k.recent_audit(50).into_iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Json(entries)
}

async fn get_experience(
    State(state): State<AppState>,
    Path(truth_key): Path<String>,
) -> impl IntoResponse {
    Json(state.experience.snapshot(&truth_key))
}

async fn list_available_agent_models() -> Result<Json<Vec<AgentModelOptions>>, (StatusCode, String)>
{
    let mut result = Vec::new();

    for agent_config in AGENT_MODELS {
        let config = agent_selection_config(agent_config);
        let selected = match select_healthy_chat_backend(&config).await {
            Ok(s) => Some(ModelOption {
                provider: s.provider().to_string(),
                model: s.model().to_string(),
                recommended: true,
            }),
            Err(e) => {
                tracing::warn!(agent = %agent_config.agent_id, error = %e, "no healthy provider");
                None
            }
        };

        result.push(AgentModelOptions {
            agent_id: agent_config.agent_id.to_string(),
            agent_name: agent_config.agent_name.to_string(),
            description: agent_config.description.to_string(),
            selected,
        });
    }

    Ok(Json(result))
}

fn agent_selection_config(agent_config: &AgentModelConfig) -> ChatBackendSelectionConfig {
    ChatBackendSelectionConfig::default()
        .with_criteria(criteria_from_agent_requirements(&agent_config.requirements))
}

fn criteria_from_agent_requirements(requirements: &AgentRequirements) -> SelectionCriteria {
    let cost = match requirements.max_cost_class {
        CostClass::Free | CostClass::VeryLow | CostClass::Low => CostTier::Minimal,
        CostClass::Medium => CostTier::Standard,
        CostClass::High | CostClass::VeryHigh => CostTier::Premium,
    };

    let latency = match requirements.max_latency_ms {
        0..=100 => LatencyClass::Realtime,
        101..=2_000 => LatencyClass::Interactive,
        2_001..=30_000 => LatencyClass::Background,
        _ => LatencyClass::Batch,
    };

    let complexity = if requirements.requires_content_generation {
        TaskComplexity::Generation
    } else if requirements.requires_reasoning || requirements.min_quality >= 0.8 {
        TaskComplexity::Reasoning
    } else {
        TaskComplexity::Classification
    };

    let capabilities = RequiredCapabilities {
        tool_use: requirements.requires_tool_use,
        vision: requirements.requires_vision,
        min_context_tokens: requirements.min_context_tokens,
        structured_output: requirements.requires_structured_output,
        code: requirements.requires_code,
        multilingual: requirements.requires_multilingual,
        web_search: requirements.requires_web_search,
        content_generation: requirements.requires_content_generation,
        business_acumen: requirements.requires_business_acumen,
    };

    let mut criteria = SelectionCriteria::default()
        .with_cost(cost)
        .with_latency(latency)
        .with_complexity(complexity)
        .with_capabilities(capabilities);

    if requirements.compliance != ComplianceLevel::None {
        criteria = criteria.with_compliance(requirements.compliance);
    }

    criteria
}

#[derive(Deserialize)]
struct ExecuteTruthRequest {
    inputs: HashMap<String, String>,
    persist_projection: Option<bool>,
}

#[derive(Serialize)]
struct TruthListItem {
    key: String,
    display_name: String,
    summary: String,
    packs: Vec<String>,
}

#[derive(Serialize)]
struct ModelOption {
    provider: String,
    model: String,
    recommended: bool,
}

#[derive(Serialize)]
struct AgentModelOptions {
    agent_id: String,
    agent_name: String,
    description: String,
    selected: Option<ModelOption>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request};
    use converge_provider::DataSovereignty;
    use governance_kernel::InMemoryStore;
    use tower::util::ServiceExt;

    fn test_state() -> AppState {
        AppState {
            store: Arc::new(InMemoryStore::new()),
            experience: Arc::new(ExperienceRegistry::new()),
        }
    }

    #[test]
    fn agent_selection_preserves_structured_output_and_context_requirements() {
        let requirements = AgentRequirements {
            max_cost_class: CostClass::Low,
            max_latency_ms: 5_000,
            min_quality: 0.75,
            requires_reasoning: false,
            requires_web_search: false,
            requires_tool_use: true,
            requires_vision: false,
            requires_code: false,
            requires_multilingual: false,
            requires_content_generation: false,
            requires_business_acumen: false,
            requires_structured_output: true,
            min_context_tokens: Some(4_000),
            data_sovereignty: DataSovereignty::Any,
            compliance: ComplianceLevel::None,
        };

        let criteria = criteria_from_agent_requirements(&requirements);

        assert_eq!(criteria.cost, CostTier::Minimal);
        assert_eq!(criteria.latency, LatencyClass::Background);
        assert!(criteria.capabilities.tool_use);
        assert!(criteria.capabilities.structured_output);
        assert_eq!(criteria.capabilities.min_context_tokens, Some(4_000));
    }

    #[test]
    fn reasoning_agent_uses_reasoning_complexity() {
        let requirements = AgentRequirements {
            max_cost_class: CostClass::Medium,
            max_latency_ms: 10_000,
            min_quality: 0.85,
            requires_reasoning: true,
            requires_web_search: false,
            requires_tool_use: false,
            requires_vision: false,
            requires_code: false,
            requires_multilingual: false,
            requires_content_generation: false,
            requires_business_acumen: false,
            requires_structured_output: true,
            min_context_tokens: Some(8_000),
            data_sovereignty: DataSovereignty::Any,
            compliance: ComplianceLevel::None,
        };

        let criteria = criteria_from_agent_requirements(&requirements);

        assert_eq!(criteria.cost, CostTier::Standard);
        assert_eq!(criteria.latency, LatencyClass::Background);
        assert_eq!(criteria.complexity, TaskComplexity::Reasoning);
        assert!(criteria.capabilities.structured_output);
    }

    #[tokio::test]
    async fn list_truths_exposes_only_vendor_selection() {
        let response = build_router(test_state())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/v1/truths")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let truths = payload.as_array().unwrap();

        assert_eq!(truths.len(), 1);
        assert_eq!(
            truths[0].get("key").and_then(|value| value.as_str()),
            Some("vendor-selection")
        );
    }

    #[tokio::test]
    async fn archived_truths_are_not_public_http_workflows() {
        let response = build_router(test_state())
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/truths/evaluate-vendor/execute")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "inputs": {
                                "vendors": "Acme AI, Beta ML"
                            }
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
