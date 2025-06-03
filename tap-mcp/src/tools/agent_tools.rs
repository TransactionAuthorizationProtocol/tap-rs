//! Agent management tools

use super::schema;
use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Tool for creating new TAP agents
pub struct CreateAgentTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating an agent
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateAgentParams {
    #[serde(default)]
    label: Option<String>,
}

/// Response for creating an agent
#[derive(Debug, Serialize)]
struct CreateAgentResponse {
    #[serde(rename = "@id")]
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    created_at: String,
}

impl CreateAgentTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for CreateAgentTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: CreateAgentParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!("Creating new agent with auto-generated DID");

        // Create a DID generator and generate a new key
        use std::sync::Arc;
        use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
        use tap_agent::config::AgentConfig;
        use tap_agent::did::{DIDGenerationOptions, DIDKeyGenerator, KeyType};
        use tap_agent::storage::KeyStorage;
        use tap_agent::TapAgent;

        let generator = DIDKeyGenerator::new();
        let did_options = DIDGenerationOptions {
            key_type: KeyType::Ed25519,
        };

        // Generate a new DID key
        let generated_key = generator
            .generate_did(did_options)
            .map_err(|e| Error::tool_execution(format!("Failed to generate DID: {}", e)))?;

        let generated_did = generated_key.did.clone();
        debug!("Generated new DID for agent: {}", generated_did);

        // Save the key to storage with optional label
        let stored_key = if let Some(ref label) = params.label {
            KeyStorage::from_generated_key_with_label(&generated_key, label)
        } else {
            KeyStorage::from_generated_key(&generated_key)
        };

        // Load existing storage or create a new one
        let mut storage = match KeyStorage::load_default() {
            Ok(storage) => storage,
            Err(_) => KeyStorage::new(),
        };

        // Add the key to storage
        storage.add_key(stored_key);

        // Save the updated storage
        storage
            .save_default()
            .map_err(|e| Error::tool_execution(format!("Failed to save key to storage: {}", e)))?;

        debug!("Key saved to storage for DID: {}", generated_did);

        // Now create the TapAgent with the saved key
        let default_key_path = KeyStorage::default_key_path().ok_or_else(|| {
            Error::tool_execution("Could not determine default key path".to_string())
        })?;

        let key_manager_builder = AgentKeyManagerBuilder::new().load_from_path(default_key_path);
        let key_manager = key_manager_builder
            .build()
            .map_err(|e| Error::tool_execution(format!("Failed to build key manager: {}", e)))?;

        // Create agent config
        let config = AgentConfig::new(generated_did.clone()).with_debug(true);

        // Create the agent
        let agent = TapAgent::new(config, Arc::new(key_manager));

        // Register the agent with the TapNode
        match self
            .tap_integration()
            .node()
            .register_agent(Arc::new(agent))
            .await
        {
            Ok(()) => {
                info!("Created and registered agent with DID: {}", generated_did);

                let response = CreateAgentResponse {
                    id: generated_did,
                    label: params.label,
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to create agent: {}", e);
                Ok(error_text_response(format!(
                    "Failed to create agent: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_create_agent".to_string(),
            description: "Creates a new TAP agent with auto-generated DID and stores the keys in ~/.tap/keys.json. Returns the generated DID. Roles and party associations are specified per transaction, not during agent creation.".to_string(),
            input_schema: schema::create_agent_schema(),
        }
    }
}

/// Tool for listing TAP agents
pub struct ListAgentsTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing agents
#[derive(Debug, Deserialize)]
struct ListAgentsParams {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

/// Response for listing agents
#[derive(Debug, Serialize)]
struct ListAgentsResponse {
    agents: Vec<ListAgentInfo>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct ListAgentInfo {
    #[serde(rename = "@id")]
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    policies: Vec<Value>,
    metadata: Value,
}

impl ListAgentsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListAgentsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListAgentsParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => ListAgentsParams {
                limit: default_limit(),
                offset: 0,
            },
        };

        debug!(
            "Listing agents with limit={}, offset={}",
            params.limit, params.offset
        );

        match self.tap_integration().list_agents().await {
            Ok(agents) => {
                let total = agents.len();

                // Apply pagination
                let paginated_agents: Vec<_> = agents
                    .into_iter()
                    .skip(params.offset as usize)
                    .take(params.limit as usize)
                    .map(|agent| ListAgentInfo {
                        id: agent.id,
                        label: agent.metadata.get("label").cloned(),
                        policies: agent
                            .policies
                            .into_iter()
                            .map(serde_json::Value::String)
                            .collect(),
                        metadata: if agent.metadata.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::to_value(agent.metadata).unwrap_or(serde_json::Value::Null)
                        },
                    })
                    .collect();

                let response = ListAgentsResponse {
                    agents: paginated_agents,
                    total,
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to list agents: {}", e);
                Ok(error_text_response(format!("Failed to list agents: {}", e)))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_agents".to_string(),
            description: "Lists all configured agents from ~/.tap/keys.json. Agents are identified by their DIDs. Roles and party associations are transaction-specific and not stored with agents.".to_string(),
            input_schema: schema::list_agents_schema(),
        }
    }
}
