use clap::{Parser, Subcommand};
use std::env;
use std::sync::Arc;
use tap_agent::{Agent, TapAgent};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod error;
mod output;
mod tap_integration;

use error::Result;
use output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "tap-cli",
    about = "Command-line interface for TAP Agent operations",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    /// Enable debug logging
    #[arg(long, short, global = true)]
    debug: bool,

    /// Agent DID for operations (uses default key if not specified)
    #[arg(long, global = true)]
    agent_did: Option<String>,

    /// Custom TAP root directory [default: ~/.tap]
    #[arg(long, global = true)]
    tap_root: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "json")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage agents
    Agent {
        #[command(subcommand)]
        cmd: commands::agent::AgentCommands,
    },
    /// Create and list transactions
    Transaction {
        #[command(subcommand)]
        cmd: commands::transaction::TransactionCommands,
    },
    /// Transaction lifecycle actions
    Action {
        #[command(subcommand)]
        cmd: commands::transaction_actions::ActionCommands,
    },
    /// Communication (ping, message)
    #[command(name = "comm")]
    Communication {
        #[command(subcommand)]
        cmd: commands::communication::CommunicationCommands,
    },
    /// DID operations (generate, lookup, keys)
    Did {
        #[command(subcommand)]
        cmd: commands::did::DidCommands,
    },
    /// Customer management
    Customer {
        #[command(subcommand)]
        cmd: commands::customer::CustomerCommands,
    },
    /// Message delivery tracking
    Delivery {
        #[command(subcommand)]
        cmd: commands::delivery::DeliveryCommands,
    },
    /// Received message inspection
    Received {
        #[command(subcommand)]
        cmd: commands::received::ReceivedCommands,
    },
}

#[tokio::main]
async fn main() {
    let mut cli = Cli::parse();

    // Apply environment variables as fallback
    if cli.tap_root.is_none() {
        cli.tap_root = env::var("TAP_ROOT")
            .ok()
            .or_else(|| env::var("TAP_HOME").ok());
    }

    if cli.agent_did.is_none() {
        cli.agent_did = env::var("TAP_AGENT_DID").ok();
    }

    if let Some(ref tap_root) = cli.tap_root {
        env::set_var("TAP_HOME", tap_root);
    }

    let format = cli.format.parse::<OutputFormat>().unwrap_or_else(|_| {
        eprintln!("Warning: unknown format '{}', using json", cli.format);
        OutputFormat::Json
    });

    // Initialize logging to stderr
    let level = if cli.debug { "debug" } else { "warn" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("tap_cli={},tap_node=warn", level).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true),
        )
        .init();

    // DID commands don't need TapIntegration
    if let Commands::Did { ref cmd } = cli.command {
        if let Err(e) = commands::did::handle(cmd, format).await {
            output::print_error(format, &e.to_string());
            std::process::exit(1);
        }
        return;
    }

    // All other commands need TapIntegration
    let (agent, agent_did) = match resolve_agent(&cli).await {
        Ok(result) => result,
        Err(e) => {
            output::print_error(format, &e.to_string());
            std::process::exit(1);
        }
    };

    let tap_integration = match tap_integration::TapIntegration::new(
        Some(&agent_did),
        cli.tap_root.as_deref(),
        Some(agent),
    )
    .await
    {
        Ok(ti) => ti,
        Err(e) => {
            output::print_error(format, &format!("Failed to initialize TAP: {}", e));
            std::process::exit(1);
        }
    };

    info!("TAP CLI initialized with agent DID: {}", agent_did);

    let result = match cli.command {
        Commands::Agent { ref cmd } => commands::agent::handle(cmd, format, &tap_integration).await,
        Commands::Transaction { ref cmd } => {
            commands::transaction::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Action { ref cmd } => {
            commands::transaction_actions::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Communication { ref cmd } => {
            commands::communication::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Customer { ref cmd } => {
            commands::customer::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Delivery { ref cmd } => {
            commands::delivery::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Received { ref cmd } => {
            commands::received::handle(cmd, format, &agent_did, &tap_integration).await
        }
        Commands::Did { .. } => unreachable!(),
    };

    if let Err(e) = result {
        output::print_error(format, &e.to_string());
        std::process::exit(1);
    }
}

async fn resolve_agent(cli: &Cli) -> Result<(Arc<TapAgent>, String)> {
    if let Some(ref did) = cli.agent_did {
        info!("Using provided agent DID: {}", did);
        match TapAgent::from_stored_keys(Some(did.clone()), true).await {
            Ok(agent) => Ok((Arc::new(agent), did.clone())),
            Err(e) => {
                error!("Failed to load agent with DID {}: {}", did, e);
                Err(e.into())
            }
        }
    } else {
        match TapAgent::from_stored_keys(None, true).await {
            Ok(agent) => {
                let did = agent.get_agent_did().to_string();
                info!("Loaded agent from stored keys with DID: {}", did);
                Ok((Arc::new(agent), did))
            }
            Err(e) => {
                info!("No stored keys found ({}), creating new agent...", e);

                use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
                use tap_agent::config::AgentConfig;
                use tap_agent::did::{DIDGenerationOptions, KeyType};
                use tap_agent::key_manager::KeyManager;
                use tap_agent::storage::KeyStorage;

                let default_key_path = KeyStorage::default_key_path().ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Could not determine default key path",
                    )
                })?;
                let key_manager_builder =
                    AgentKeyManagerBuilder::new().load_from_path(default_key_path);
                let key_manager = key_manager_builder.build()?;

                let generated_key = key_manager.generate_key(DIDGenerationOptions {
                    key_type: KeyType::Ed25519,
                })?;

                info!("Generated new agent with DID: {}", generated_key.did);

                let config = AgentConfig::new(generated_key.did.clone()).with_debug(true);
                let agent = TapAgent::new(config, Arc::new(key_manager));

                Ok((Arc::new(agent), generated_key.did))
            }
        }
    }
}
