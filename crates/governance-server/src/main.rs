use std::sync::Arc;

use governance_kernel::InMemoryStore;
use governance_server::experience::{ExperienceRegistry, InMemoryExperienceStream};
use governance_server::{AppState, build_router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let experience_stream = Arc::new(InMemoryExperienceStream::default());
    let store = InMemoryStore::new().with_domain_event_stream(experience_stream);
    let experience = Arc::new(ExperienceRegistry::new());
    let state = AppState {
        store: Arc::new(store),
        experience,
    };

    let app = build_router(state);

    let addr = std::env::var("GOVERNANCE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    tracing::info!("governance server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
