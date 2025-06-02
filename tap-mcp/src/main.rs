use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod mcp;
mod resources;
mod tap_integration;
mod tools;

use error::Result;

#[derive(Parser)]
#[command(
    name = "tap-mcp",
    about = "Model Context Protocol server for TAP Node functionality",
    version = env!("CARGO_PKG_VERSION")
)]
struct Args {
    /// Enable debug logging
    #[arg(long, short)]
    debug: bool,

    /// Agent DID for database organization (e.g., did:web:example.com)
    #[arg(long)]
    agent_did: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("tap_mcp={},tap_node=info", level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting TAP-MCP server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize TAP integration with TapNode
    let tap_integration = tap_integration::TapIntegration::new(args.agent_did.as_deref()).await?;

    info!("TAP integration initialized using TapNode with DID-based storage");

    // Create and run MCP server
    let mcp_server = mcp::McpServer::new(tap_integration).await?;

    info!("Starting MCP server on stdio");
    if let Err(e) = mcp_server.run().await {
        error!("MCP server error: {}", e);
        return Err(e);
    }

    Ok(())
}
