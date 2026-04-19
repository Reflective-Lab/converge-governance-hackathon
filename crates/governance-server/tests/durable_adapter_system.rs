use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use governance_kernel::{DomainEvent, DomainEventStream, InMemoryStore};
use governance_server::{AppState, build_router};
use reqwest::StatusCode as ReqwestStatusCode;
use tower::util::ServiceExt;

#[derive(Debug)]
struct DurableFileDomainEventStream {
    path: PathBuf,
    lock: Mutex<()>,
}

impl DurableFileDomainEventStream {
    fn event_kind(event: &DomainEvent) -> &'static str {
        match event {
            DomainEvent::VendorRegistered { .. } => "vendor-registered",
            DomainEvent::ComplianceChecked { .. } => "compliance-checked",
            DomainEvent::RiskScored { .. } => "risk-scored",
            DomainEvent::CostEstimated { .. } => "cost-estimated",
            DomainEvent::DecisionRecorded { .. } => "decision-recorded",
            DomainEvent::AuditRecorded { .. } => "audit-recorded",
        }
    }
}

impl DomainEventStream for DurableFileDomainEventStream {
    fn record_events(&self, events: &[DomainEvent]) {
        if events.is_empty() {
            return;
        }

        let _guard = match self.lock.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            Ok(file) => file,
            Err(_) => return,
        };

        for event in events {
            let _ = writeln!(file, "{}", Self::event_kind(event));
        }
    }
}

#[tokio::test]
async fn run_system_truth_endpoint_with_durable_adapter() {
    let path = temp_path("governance-events");
    let _ = fs::remove_file(&path);

    let adapter = Arc::new(DurableFileDomainEventStream {
        path: path.clone(),
        lock: Mutex::new(()),
    });
    let store: AppState = Arc::new(InMemoryStore::new().with_domain_event_stream(adapter));

    let app = build_router(store.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/truths/authorize-vendor-commitment/execute")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "inputs": {
                            "principal_id": "user:procurement-lead",
                            "principal_authority": "supervisory",
                            "commitment_id": "commitment:vendor-a-2026-04",
                            "vendor_name": "Vendor A",
                            "commitment_type": "contract",
                            "action": "commit",
                            "amount_minor": "75000",
                            "currency_code": "USD",
                            "human_approval_present": "true",
                            "required_gates_met": "true"
                        },
                        "persist_projection": true,
                    })
                    .to_string(),
                ))
                .expect("request should be constructable"),
        )
        .await
        .expect("endpoint should respond");

    assert_eq!(response.status(), StatusCode::OK);

    let decisions = store.read(|kernel| kernel.decisions.len()).unwrap();
    assert_eq!(decisions, 1, "truth execution should persist a decision");

    let durable_rows = fs::read_to_string(&path).unwrap_or_default();
    let lines: Vec<_> = durable_rows
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert!(
        !lines.is_empty(),
        "durable adapter should receive at least one event"
    );
    assert!(
        lines.iter().any(|line| *line == "decision-recorded"),
        "event stream should include decision-recorded"
    );

    let _ = fs::remove_file(&path);
}

#[tokio::test]
async fn run_system_truth_endpoint_over_tcp_with_durable_adapter() {
    let path = temp_path("governance-events-tcp");
    let _ = fs::remove_file(&path);

    let adapter = Arc::new(DurableFileDomainEventStream {
        path: path.clone(),
        lock: Mutex::new(()),
    });
    let store: AppState = Arc::new(InMemoryStore::new().with_domain_event_stream(adapter));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_router(store.clone());

    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let response = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/truths/authorize-vendor-commitment/execute",
            addr
        ))
        .json(&truth_request_payload())
        .send()
        .await
        .expect("endpoint should respond")
        .status();

    assert_eq!(response, ReqwestStatusCode::OK);

    let decisions = store.read(|kernel| kernel.decisions.len()).unwrap();
    assert_eq!(decisions, 1, "truth execution should persist a decision");

    let lines = wait_for_durable_events(&path, 1).await;
    assert!(
        lines.iter().any(|line| line == "decision-recorded"),
        "durable adapter should include decision-recorded"
    );

    server.abort();
    let _ = server.await;

    let _ = fs::remove_file(&path);
}

fn truth_request_payload() -> serde_json::Value {
    serde_json::json!({
        "inputs": {
            "principal_id": "user:procurement-lead",
            "principal_authority": "supervisory",
            "commitment_id": "commitment:vendor-a-2026-04",
            "vendor_name": "Vendor A",
            "commitment_type": "contract",
            "action": "commit",
            "amount_minor": "75000",
            "currency_code": "USD",
            "human_approval_present": "true",
            "required_gates_met": "true"
        },
        "persist_projection": true,
    })
}

async fn wait_for_durable_events(path: &Path, minimum_count: usize) -> Vec<String> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);

    loop {
        let lines = read_event_lines(path);
        if lines.len() >= minimum_count {
            return lines;
        }

        if tokio::time::Instant::now() >= deadline {
            return lines;
        }

        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn read_event_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn temp_path(prefix: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|ts| ts.as_nanos())
        .unwrap_or_default();

    std::env::temp_dir().join(format!("{prefix}-{ts}.jsonl"))
}
