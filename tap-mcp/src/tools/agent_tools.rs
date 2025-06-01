//! Agent management tools

use super::schema;
use super::{error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error};

/// Tool for creating new TAP agents
pub struct CreateAgentTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for creating an agent
#[derive(Debug, Deserialize)]
struct CreateAgentParams {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
    #[serde(default)]
    policies: Option<Vec<Value>>,
    #[serde(default)]
    metadata: Option<Value>,
}

/// Response for creating an agent
#[derive(Debug, Serialize)]
struct CreateAgentResponse {
    agent: AgentResponse,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct AgentResponse {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
}

impl CreateAgentTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self {
            tap_integration,
        }
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
            None => return Ok(error_text_response("Missing required parameters".to_string())),
        };

        debug!("Creating agent: id={}, role={}, for={}", params.id, params.role, params.for_party);

        match self
            .tap_integration()
            .create_agent(
                params.id.clone(),
                params.role.clone(),
                params.for_party.clone(),
                params.policies,
                params.metadata,
            )
            .await
        {
            Ok(agent_info) => {
                let response = CreateAgentResponse {
                    agent: AgentResponse {
                        id: agent_info.id,
                        role: agent_info.role,
                        for_party: agent_info.for_party,
                    },
                    created_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response)
                    .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to create agent: {}", e);
                Ok(error_text_response(format!("Failed to create agent: {}", e)))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap.create_agent".to_string(),
            description: "Creates a new TAP agent with specified configuration and stores it in ~/.tap/agents".to_string(),
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
    #[serde(default)]
    filter: Option<AgentFilter>,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

#[derive(Debug, Deserialize)]
struct AgentFilter {
    role: Option<String>,
    for_party: Option<String>,
}

fn default_limit() -> u32 {
    50
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
    role: String,
    #[serde(rename = "for")]
    for_party: String,
    policies: Vec<Value>,
    metadata: Value,
}

impl ListAgentsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self {
            tap_integration,
        }
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
                filter: None,
                limit: default_limit(),
                offset: 0,
            },
        };

        debug!("Listing agents with limit={}, offset={}", params.limit, params.offset);

        match self.tap_integration().list_agents().await {
            Ok(agents) => {
                // Apply filters
                let filtered_agents: Vec<_> = agents
                    .into_iter()
                    .filter(|agent| {
                        if let Some(ref filter) = params.filter {
                            if let Some(ref role) = filter.role {
                                if agent.role != *role {
                                    return false;
                                }
                            }
                            if let Some(ref for_party) = filter.for_party {
                                if agent.for_party != *for_party {
                                    return false;
                                }
                            }
                        }
                        true
                    })
                    .collect();

                let total = filtered_agents.len();

                // Apply pagination
                let paginated_agents: Vec<_> = filtered_agents
                    .into_iter()
                    .skip(params.offset as usize)
                    .take(params.limit as usize)
                    .map(|agent| ListAgentInfo {
                        id: agent.id,
                        role: agent.role,
                        for_party: agent.for_party,
                        policies: agent.policies,
                        metadata: agent.metadata,
                    })
                    .collect();

                let response = ListAgentsResponse {
                    agents: paginated_agents,
                    total,
                };

                let response_json = serde_json::to_string_pretty(&response)
                    .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

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
            name: "tap.list_agents".to_string(),
            description: "Lists all configured agents from the ~/.tap/agents directory with optional filtering".to_string(),
            input_schema: schema::list_agents_schema(),
        }
    }
}