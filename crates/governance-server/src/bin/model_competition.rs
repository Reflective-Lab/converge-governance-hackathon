use std::collections::HashMap;

use governance_server::truth_runtime::model_competition::{
    COMPETITION_MODELS, CompetitionRole, run_competition,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    governance_server::llm_helpers::load_env();
    tracing_subscriber::fmt()
        .with_env_filter("governance_server=info")
        .init();

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

    let models: Vec<&str> = COMPETITION_MODELS.to_vec();
    let roles = CompetitionRole::all().to_vec();

    println!("=== Model Competition ===");
    println!("Models: {}", models.len());
    println!("Roles: {}", roles.len());
    println!("Total runs: {}", models.len() * roles.len());
    println!();

    let report = run_competition(&models, &roles, &inputs).await;

    println!("\n=== RESULTS ===");
    println!(
        "Completed: {} | Success: {} | Failed: {}",
        report.total_runs, report.successful_runs, report.failed_runs
    );

    for (role, entries) in &report.leaderboard_by_role {
        println!("\n--- {role} Leaderboard ---");
        println!(
            "{:<4} {:<40} {:>9} {:>8} {:>10} {:>10}",
            "Rank", "Model", "Composite", "Success", "Latency", "Tokens"
        );
        for e in entries {
            println!(
                "{:<4} {:<40} {:>9.4} {:>8.0}% {:>8.0}ms {:>10.0}",
                e.rank,
                e.model,
                e.composite,
                e.success_rate * 100.0,
                e.avg_latency_ms,
                e.avg_tokens,
            );
        }
    }

    println!("\n--- Overall Leaderboard ---");
    println!(
        "{:<4} {:<40} {:>9} {:>8} {:>10} {:>10} {:>5}",
        "Rank", "Model", "Composite", "Success", "Latency", "Tokens", "Runs"
    );
    for e in &report.leaderboard_overall {
        println!(
            "{:<4} {:<40} {:>9.4} {:>8.0}% {:>8.0}ms {:>10.0} {:>5}",
            e.rank,
            e.model,
            e.composite,
            e.success_rate * 100.0,
            e.avg_latency_ms,
            e.avg_tokens,
            e.runs,
        );
    }

    // Write JSON report
    let report_path = "competition_report.json";
    std::fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
    println!("\nFull report written to {report_path}");

    Ok(())
}
