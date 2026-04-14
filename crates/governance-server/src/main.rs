use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use governance_kernel::InMemoryStore;
use governance_server::truth_runtime;
use serde::{Deserialize, Serialize};

type AppState = Arc<InMemoryStore>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let store = InMemoryStore::new();
    let state: AppState = Arc::new(store);

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/truths", get(list_truths))
        .route("/v1/truths/{key}/execute", post(execute_truth))
        .route("/v1/decisions", get(list_decisions))
        .route("/v1/vendors", get(list_vendors))
        .route("/v1/audit", get(list_audit))
        .with_state(state);

    let addr = std::env::var("GOVERNANCE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    tracing::info!("governance server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok"
}

async fn list_truths() -> Json<Vec<TruthListItem>> {
    let items = governance_truths::TRUTHS
        .iter()
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
    State(store): State<AppState>,
    axum::extract::Path(key): axum::extract::Path<String>,
    Json(request): Json<ExecuteTruthRequest>,
) -> Result<Json<truth_runtime::TruthExecutionResult>, (StatusCode, String)> {
    let persist = request.persist_projection.unwrap_or(true);
    // The converge Engine future is !Send (trait-object suggestors), so run on
    // a blocking thread with a local async runtime to satisfy axum's Send bound.
    let store_inner = (*store).clone();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(truth_runtime::execute_truth(
                &store_inner,
                &key,
                request.inputs,
                persist,
            ))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn list_decisions(State(store): State<AppState>) -> impl IntoResponse {
    let decisions = store
        .read(|k| {
            k.recent_decisions(20)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Json(decisions)
}

async fn list_vendors(State(store): State<AppState>) -> impl IntoResponse {
    let vendors = store
        .read(|k| k.vendors_list().into_iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Json(vendors)
}

async fn list_audit(State(store): State<AppState>) -> impl IntoResponse {
    let entries = store
        .read(|k| k.recent_audit(50).into_iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    Json(entries)
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
