#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    governance_server::live_due_diligence::run_from_cli().await
}
