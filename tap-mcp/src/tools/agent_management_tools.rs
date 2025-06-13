//! Agent management tools for TAP transactions

use super::schema;
use super::{error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{AddAgents, Agent, RemoveAgent, ReplaceAgent};
use tracing::{debug, error};

/// Tool for adding agents to a transaction
pub struct AddAgentsTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for adding agents
#[derive(Debug, Deserialize)]
struct AddAgentsParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    agents: Vec<AgentInfo>,
}

#[derive(Debug, Deserialize)]
struct AgentInfo {
    #[serde(rename = "@id")]
    id: String,
    role: String,
    #[serde(rename = "for")]
    for_party: String,
}

/// Response for adding agents
#[derive(Debug, Serialize)]
struct AddAgentsResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    agents_added: usize,
    added_at: String,
}

impl AddAgentsTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for AddAgentsTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: AddAgentsParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Adding {} agents to transaction: {}",
            params.agents.len(),
            params.transaction_id
        );

        // Create agents
        let agents: Vec<Agent> = params
            .agents
            .iter()
            .map(|agent_info| Agent::new(&agent_info.id, &agent_info.role, &agent_info.for_party))
            .collect();

        // Create add agents message
        let add_agents = AddAgents::new(&params.transaction_id, agents);

        // Validate the add agents message
        if let Err(e) = add_agents.validate() {
            return Ok(error_text_response(format!(
                "AddAgents validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match add_agents.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for add agents message".to_string(),
            ));
        };

        debug!(
            "Sending add agents from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "AddAgents message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = AddAgentsResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    agents_added: add_agents.agents.len(),
                    added_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send add agents message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send add agents message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_add_agents".to_string(),
            description: "Adds agents to a TAP transaction using the AddAgents message (TAIP-5)"
                .to_string(),
            input_schema: schema::add_agents_schema(),
        }
    }
}

/// Tool for removing an agent from a transaction
pub struct RemoveAgentTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for removing an agent
#[derive(Debug, Deserialize)]
struct RemoveAgentParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    agent_to_remove: String,
}

/// Response for removing an agent
#[derive(Debug, Serialize)]
struct RemoveAgentResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    removed_agent: String,
    removed_at: String,
}

impl RemoveAgentTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for RemoveAgentTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: RemoveAgentParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Removing agent {} from transaction: {}",
            params.agent_to_remove, params.transaction_id
        );

        // Create remove agent message
        let remove_agent = RemoveAgent::new(&params.transaction_id, &params.agent_to_remove);

        // Validate the remove agent message
        if let Err(e) = remove_agent.validate() {
            return Ok(error_text_response(format!(
                "RemoveAgent validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match remove_agent.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for remove agent message".to_string(),
            ));
        };

        debug!(
            "Sending remove agent from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "RemoveAgent message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = RemoveAgentResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    removed_agent: params.agent_to_remove,
                    removed_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send remove agent message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send remove agent message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_remove_agent".to_string(),
            description:
                "Removes an agent from a TAP transaction using the RemoveAgent message (TAIP-5)"
                    .to_string(),
            input_schema: schema::remove_agent_schema(),
        }
    }
}

/// Tool for replacing an agent in a transaction
pub struct ReplaceAgentTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for replacing an agent
#[derive(Debug, Deserialize)]
struct ReplaceAgentParams {
    agent_did: String, // The DID of the agent that will sign and send this message
    transaction_id: String,
    original_agent: String,
    new_agent: AgentInfo,
}

/// Response for replacing an agent
#[derive(Debug, Serialize)]
struct ReplaceAgentResponse {
    transaction_id: String,
    message_id: String,
    status: String,
    old_agent: String,
    new_agent: String,
    replaced_at: String,
}

impl ReplaceAgentTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    fn tap_integration(&self) -> &TapIntegration {
        &self.tap_integration
    }
}

#[async_trait::async_trait]
impl ToolHandler for ReplaceAgentTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ReplaceAgentParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Replacing agent {} with {} in transaction: {}",
            params.original_agent, params.new_agent.id, params.transaction_id
        );

        // Create new agent
        let replacement_agent = Agent::new(
            &params.new_agent.id,
            &params.new_agent.role,
            &params.new_agent.for_party,
        );

        // Create replace agent message
        let replace_agent = ReplaceAgent::new(
            &params.transaction_id,
            &params.original_agent,
            replacement_agent,
        );

        // Validate the replace agent message
        if let Err(e) = replace_agent.validate() {
            return Ok(error_text_response(format!(
                "ReplaceAgent validation failed: {}",
                e
            )));
        }

        // Create DIDComm message using the specified agent DID
        let didcomm_message = match replace_agent.to_didcomm(&params.agent_did) {
            Ok(msg) => msg,
            Err(e) => {
                return Ok(error_text_response(format!(
                    "Failed to create DIDComm message: {}",
                    e
                )));
            }
        };

        // Determine recipient from the message
        let recipient_did = if !didcomm_message.to.is_empty() {
            didcomm_message.to[0].clone()
        } else {
            return Ok(error_text_response(
                "No recipient found for replace agent message".to_string(),
            ));
        };

        debug!(
            "Sending replace agent from {} to {} for transaction: {}",
            params.agent_did, recipient_did, params.transaction_id
        );

        // Send the message through the TAP node
        match self
            .tap_integration()
            .node()
            .send_message(params.agent_did.clone(), didcomm_message.clone())
            .await
        {
            Ok(packed_message) => {
                debug!(
                    "ReplaceAgent message sent successfully to {}, packed message length: {}",
                    recipient_did,
                    packed_message.len()
                );

                let response = ReplaceAgentResponse {
                    transaction_id: params.transaction_id,
                    message_id: didcomm_message.id,
                    status: "sent".to_string(),
                    old_agent: params.original_agent,
                    new_agent: params.new_agent.id,
                    replaced_at: chrono::Utc::now().to_rfc3339(),
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to send replace agent message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send replace agent message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_replace_agent".to_string(),
            description:
                "Replaces an agent in a TAP transaction using the ReplaceAgent message (TAIP-5)"
                    .to_string(),
            input_schema: schema::replace_agent_schema(),
        }
    }
}
