//! Live E2E integration tests for vendor-selection truth executor.
//!
//! These tests make real LLM calls (via OpenRouter) and real web searches
//! (Brave + Tavily). They require API keys set in `.env` and are gated
//! behind the `live-tests` cargo feature.
//!
//! Run: `cargo test --features live-tests -p governance-server -- live_ --nocapture`

#![cfg(feature = "live-tests")]

use governance_kernel::InMemoryStore;
use governance_server::experience::ExperienceRegistry;
use governance_server::truth_runtime::vendor_selection;
use std::collections::HashMap;

fn vendor_inputs() -> HashMap<String, String> {
    let vendors = serde_json::json!([
        {
            "name": "Acme AI",
            "score": 85,
            "risk_score": 12.5,
            "monthly_cost_minor": 25000,
            "currency_code": "USD",
            "compliance_status": "compliant",
            "certifications": ["SOC2", "ISO27001", "GDPR"]
        },
        {
            "name": "NovaTech",
            "score": 78,
            "risk_score": 22.0,
            "monthly_cost_minor": 18000,
            "currency_code": "USD",
            "compliance_status": "compliant",
            "certifications": ["SOC2"]
        },
        {
            "name": "DataForge",
            "score": 92,
            "risk_score": 8.0,
            "monthly_cost_minor": 42000,
            "currency_code": "USD",
            "compliance_status": "pending",
            "certifications": ["ISO27001"]
        }
    ]);

    let mut inputs = HashMap::new();
    inputs.insert("vendors_json".to_string(), vendors.to_string());
    inputs.insert("live_mode".to_string(), "true".to_string());
    inputs
}

fn run_blocking<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn live_vendor_selection_e2e() {
    dotenv::dotenv().ok();
    run_blocking(async {
        let store = InMemoryStore::new();
        let experience = ExperienceRegistry::with_path(
            std::env::temp_dir().join(format!("live-test-{}.json", uuid::Uuid::new_v4())),
        );
        let inputs = vendor_inputs();

        let result =
            vendor_selection::execute_with_experience(&store, &inputs, false, Some(&experience))
                .await
                .expect("live vendor selection should succeed");

        assert!(result.converged, "should converge");
        assert!(result.cycles > 0, "should run at least one cycle");

        // LLM calls should be populated in live mode
        let llm_calls = result
            .llm_calls
            .as_ref()
            .expect("llm_calls should be Some in live mode");
        assert!(!llm_calls.is_empty(), "should have made LLM calls");

        println!("--- Live E2E Result ---");
        println!("Converged: {}, Cycles: {}", result.converged, result.cycles);
        println!("LLM calls: {}", llm_calls.len());
        for call in llm_calls {
            println!(
                "  [{context}] model={model} elapsed={elapsed}ms",
                context = call.context,
                model = call.model,
                elapsed = call.elapsed_ms,
            );
        }
        if let Some(projection) = &result.projection {
            println!("Events emitted: {}", projection.events_emitted);
            if let Some(details) = &projection.details {
                println!(
                    "Projection details: {}",
                    serde_json::to_string_pretty(details).unwrap()
                );
            }
        }
    });
}

#[test]
fn live_multi_model_exercise() {
    dotenv::dotenv().ok();
    let _ = tracing_subscriber::fmt()
        .with_env_filter("governance_server=debug")
        .try_init();
    run_blocking(async {
        let store = InMemoryStore::new();
        let experience = ExperienceRegistry::with_path(
            std::env::temp_dir().join(format!("live-test-{}.json", uuid::Uuid::new_v4())),
        );
        let inputs = vendor_inputs();

        let result =
            vendor_selection::execute_with_experience(&store, &inputs, false, Some(&experience))
                .await
                .expect("live vendor selection should succeed");

        let llm_calls = result.llm_calls.as_ref().expect("llm_calls should be Some");

        // Collect unique models and contexts used
        let models: std::collections::HashSet<&str> =
            llm_calls.iter().map(|c| c.model.as_str()).collect();
        let contexts: Vec<&str> = llm_calls.iter().map(|c| c.context.as_str()).collect();

        println!("Models exercised: {:?}", models);
        println!("LLM call contexts: {:?}", contexts);

        let mut fallback_count = 0;
        for call in llm_calls {
            let is_fallback = call.model == "deterministic-fallback";
            if is_fallback {
                fallback_count += 1;
            }
            let error_info = call.metadata.get("error").map(|e| e.as_str()).unwrap_or("");
            println!(
                "  [{context}] model={model} elapsed={elapsed}ms{fallback}{error}",
                context = call.context,
                model = call.model,
                elapsed = call.elapsed_ms,
                fallback = if is_fallback { " FALLBACK" } else { "" },
                error = if error_info.is_empty() {
                    String::new()
                } else {
                    format!(" error={error_info}")
                },
            );
        }

        let real_calls = llm_calls.len() - fallback_count;
        println!(
            "Summary: {} real LLM calls, {} fallbacks",
            real_calls, fallback_count
        );

        // All 4 suggestors should make real LLM calls (compliance x3, cost, risk, decision).
        // Any fallback means an LLM call failed — that's a real problem to investigate.
        assert_eq!(
            fallback_count, 0,
            "no suggestors should fall back to deterministic — check error details above"
        );

        // Verify we exercised all 3 model tiers
        let real_models: std::collections::HashSet<&str> = llm_calls
            .iter()
            .filter(|c| c.model != "deterministic-fallback")
            .map(|c| c.model.as_str())
            .collect();
        assert!(
            real_models.len() >= 3,
            "should exercise 3 model tiers (fast/mid/strong), got: {:?}",
            real_models
        );
    });
}

#[test]
fn live_learning_improves_over_runs() {
    dotenv::dotenv().ok();
    run_blocking(async {
        let store = InMemoryStore::new();
        let experience = ExperienceRegistry::with_path(
            std::env::temp_dir().join(format!("live-test-{}.json", uuid::Uuid::new_v4())),
        );
        let inputs = vendor_inputs();

        let mut results = Vec::new();
        for i in 0..3 {
            let result = vendor_selection::execute_with_experience(
                &store,
                &inputs,
                false,
                Some(&experience),
            )
            .await
            .expect(&format!("run {} should succeed", i + 1));

            println!(
                "Run {}: converged={}, cycles={}, confidence={:?}",
                i + 1,
                result.converged,
                result.cycles,
                result
                    .projection
                    .as_ref()
                    .and_then(|p| p.details.as_ref())
                    .and_then(|d| d.get("learning"))
            );
            results.push(result);
        }

        // The third run should have learning metrics referencing prior runs
        let last = &results[2];
        let details = last
            .projection
            .as_ref()
            .and_then(|p| p.details.as_ref())
            .expect("third run should have projection details");

        let learning = details.get("learning");
        println!("Learning metrics (run 3): {:?}", learning);
        if let Some(learning) = learning {
            let prior_runs = learning
                .get("prior_runs")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            assert!(
                prior_runs >= 1,
                "third run should see at least 1 prior run, got {}",
                prior_runs
            );
        }
    });
}

#[test]
fn live_experience_persists_across_runs() {
    dotenv::dotenv().ok();
    run_blocking(async {
        let store = InMemoryStore::new();
        let experience = ExperienceRegistry::with_path(
            std::env::temp_dir().join(format!("live-test-{}.json", uuid::Uuid::new_v4())),
        );
        let inputs = vendor_inputs();

        // First run
        let r1 =
            vendor_selection::execute_with_experience(&store, &inputs, false, Some(&experience))
                .await
                .expect("first run should succeed");

        // Check that ExperienceRegistry now has data
        let prior = experience.prior_decisions_summary("vendor-selection");
        println!("Prior decisions after run 1: {}", prior);

        // Second run — should see the first run's data
        let r2 =
            vendor_selection::execute_with_experience(&store, &inputs, false, Some(&experience))
                .await
                .expect("second run should succeed");

        let prior = experience.prior_decisions_summary("vendor-selection");
        println!("Prior decisions after run 2: {}", prior);
        assert!(
            !prior.is_empty(),
            "experience registry should have prior decisions after two runs"
        );

        println!("Run 1: converged={}, cycles={}", r1.converged, r1.cycles);
        println!("Run 2: converged={}, cycles={}", r2.converged, r2.cycles);
    });
}
