use clap::Parser;
use std::env;
use std::sync::Arc;
use tap_agent::{Agent, TapAgent};
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

    /// Custom TAP root directory [default: ~/.tap]
    #[arg(long)]
    tap_root: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();
    
    // Apply TAP_ROOT environment variable as fallback if not provided via CLI
    if args.tap_root.is_none() {
        args.tap_root = env::var("TAP_ROOT").ok();
    }

    // Initialize logging to stderr (stdout is reserved for MCP protocol)
    let level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("tap_mcp={},tap_node=info", level).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
        )
        .init();

    info!("Starting TAP-MCP server v{}", env!("CARGO_PKG_VERSION"));

    // Determine agent - use provided DID or load/create from storage
    let (agent, agent_did) = if let Some(did) = args.agent_did {
        info!("Using provided agent DID: {}", did);
        // Try to load the specified agent
        match TapAgent::from_stored_keys(Some(did.clone()), true).await {
            Ok(agent) => (Arc::new(agent), did),
            Err(e) => {
                error!("Failed to load agent with DID {}: {}", did, e);
                return Err(e.into());
            }
        }
    } else {
        // Try to load from storage first, create if none exist
        match TapAgent::from_stored_keys(None, true).await {
            Ok(agent) => {
                let did = agent.get_agent_did().to_string();
                info!("Loaded agent from stored keys with DID: {}", did);
                (Arc::new(agent), did)
            }
            Err(e) => {
                info!("No stored keys found ({}), creating new agent...", e);
                
                // Create agent with persistent storage
                use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
                use tap_agent::config::AgentConfig;
                use tap_agent::did::{DIDGenerationOptions, KeyType};
                use tap_agent::key_manager::KeyManager;
                use tap_agent::storage::KeyStorage;
                
                // Create a key manager with storage enabled and generate a new key
                let default_key_path = KeyStorage::default_key_path().ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not determine default key path",
                    )
                })?;
                let key_manager_builder =
                    AgentKeyManagerBuilder::new().load_from_path(default_key_path);
                let key_manager = key_manager_builder.build()?;

                // Generate a new key
                let generated_key = key_manager.generate_key(DIDGenerationOptions {
                    key_type: KeyType::Ed25519,
                })?;

                info!("Generated new agent with DID: {}", generated_key.did);

                // Create agent config and build agent
                let config = AgentConfig::new(generated_key.did.clone()).with_debug(true);
                let agent = TapAgent::new(config, Arc::new(key_manager));
                
                info!("New key saved to storage successfully");
                (Arc::new(agent), generated_key.did)
            }
        }
    };

    // Initialize TAP integration with TapNode
    let tap_integration = tap_integration::TapIntegration::new(
        Some(&agent_did),
        args.tap_root.as_deref(),
        Some(agent.clone()),
    )
    .await?;

    info!("TAP integration initialized using TapNode with DID-based storage at ~/.tap/{}", 
        agent_did.replace(':', "_"));

    // Create and run MCP server
    let mcp_server = mcp::McpServer::new(tap_integration).await?;

    info!("Starting MCP server on stdio");
    if let Err(e) = mcp_server.run().await {
        error!("MCP server error: {}", e);
        return Err(e);
    }

    Ok(())
}
