use anyhow::Result;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

mod backends;
mod decode;
mod network;
mod pool;
mod server;
mod signer;
mod tools;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    // CRITICAL: Log to stderr only. stdout is the MCP stdio transport.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting polkadot-mcp server");

    // Load signer from env if available (read-only mode if not set)
    let signer = signer::load_from_env()?;
    if signer.is_some() {
        tracing::info!("Signer loaded — transaction tools enabled");
    } else {
        tracing::info!("No signer configured — read-only mode");
    }

    // Build MCP server (all networks available)
    let server = server::PolkadotMcp::new(signer);

    // Start stdio transport
    let transport = rmcp::transport::io::stdio();
    let service = server
        .serve(transport)
        .await
        .inspect_err(|e| tracing::error!("Failed to start server: {}", e))
        .expect("Failed to start MCP server");
    service.waiting().await?;

    Ok(())
}
