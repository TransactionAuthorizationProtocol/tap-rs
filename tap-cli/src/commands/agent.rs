use crate::error::{Error, Result};
use crate::output::{print_success, OutputFormat};
use crate::tap_integration::TapIntegration;
use clap::Subcommand;
use serde::Serialize;
use std::sync::Arc;
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::did::{DIDGenerationOptions, KeyType};
use tap_agent::key_manager::KeyManager;
use tap_agent::storage::KeyStorage;
use tap_agent::TapAgent;
use tracing::info;

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    /// Create a new agent with a generated DID
    Create {
        /// Label for the agent
        #[arg(long)]
        label: Option<String>,
    },
    /// List all registered agents
    List,
}

#[derive(Debug, Serialize)]
struct AgentCreatedResponse {
    did: String,
    label: Option<String>,
}

pub async fn handle(
    cmd: &AgentCommands,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    match cmd {
        AgentCommands::Create { label } => {
            handle_create(label.clone(), format, tap_integration).await
        }
        AgentCommands::List => handle_list(format, tap_integration).await,
    }
}

async fn handle_create(
    label: Option<String>,
    format: OutputFormat,
    tap_integration: &TapIntegration,
) -> Result<()> {
    let default_key_path = KeyStorage::default_key_path()
        .ok_or_else(|| Error::configuration("Could not determine default key path"))?;
    let key_manager_builder = AgentKeyManagerBuilder::new().load_from_path(default_key_path);
    let key_manager = key_manager_builder
        .build()
        .map_err(|e| Error::configuration(format!("Failed to build key manager: {}", e)))?;

    let generated_key = key_manager
        .generate_key(DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        })
        .map_err(|e| Error::command_failed(format!("Failed to generate key: {}", e)))?;

    // Set label if provided
    if let Some(ref label) = label {
        let mut storage = KeyStorage::load_default().unwrap_or_else(|_| KeyStorage::new());
        if let Some(key) = storage.keys.get_mut(&generated_key.did) {
            key.label = label.clone();
            let _ = storage.save_default();
        }
    }

    info!("Generated new agent with DID: {}", generated_key.did);

    // Register with the TapNode
    let config = AgentConfig::new(generated_key.did.clone()).with_debug(true);
    let new_key_manager = AgentKeyManagerBuilder::new()
        .load_from_default_storage()
        .build()
        .map_err(|e| Error::configuration(format!("Failed to reload key manager: {}", e)))?;
    let agent = TapAgent::new(config, Arc::new(new_key_manager));
    tap_integration
        .node()
        .register_agent(Arc::new(agent))
        .await
        .map_err(|e| Error::command_failed(format!("Failed to register agent: {}", e)))?;

    let response = AgentCreatedResponse {
        did: generated_key.did,
        label,
    };
    print_success(format, &response);
    Ok(())
}

async fn handle_list(format: OutputFormat, tap_integration: &TapIntegration) -> Result<()> {
    let agents = tap_integration.list_agents().await?;
    print_success(format, &agents);
    Ok(())
}
