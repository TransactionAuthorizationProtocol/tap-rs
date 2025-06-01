use clap::Parser;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod mcp;
mod tap_integration;
mod tools;
mod resources;
mod error;

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

    /// Path to TAP root directory (defaults to ~/.tap)
    #[arg(long)]
    tap_root: Option<String>,

    /// Database path for tap-node (defaults to ~/.tap/tap-node.db)
    #[arg(long)]
    db_path: Option<String>,
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

    // Initialize TAP integration
    let tap_integration = tap_integration::TapIntegration::new(
        args.tap_root.as_deref(),
        args.db_path.as_deref(),
    ).await?;

    info!("TAP integration initialized");

    // Create and run MCP server
    let mcp_server = mcp::McpServer::new(tap_integration).await?;
    
    info!("Starting MCP server on stdio");
    if let Err(e) = mcp_server.run().await {
        error!("MCP server error: {}", e);
        return Err(e);
    }

    Ok(())
}