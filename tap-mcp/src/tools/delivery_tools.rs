//! Delivery management tools

use super::{default_limit, error_text_response, success_text_response, ToolHandler};
use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

/// Tool for listing deliveries by recipient
pub struct ListDeliveriesByRecipientTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing deliveries
#[derive(Debug, Deserialize)]
struct ListDeliveriesParams {
    agent_did: String, // The DID of the agent whose sent deliveries to list
    recipient_did: String,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
    #[serde(default)]
    status_filter: Option<String>, // "all", "pending", "delivered", "failed"
}

/// Delivery information
#[derive(Debug, Serialize)]
struct DeliveryInfo {
    id: i64,
    message_id: String,
    message_text: String,
    recipient_did: String,
    delivery_url: Option<String>,
    delivery_type: String,
    status: String,
    retry_count: i32,
    last_http_status_code: Option<i32>,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
    delivered_at: Option<String>,
}

/// Response for listing deliveries
#[derive(Debug, Serialize)]
struct ListDeliveriesResponse {
    deliveries: Vec<DeliveryInfo>,
    total: u32,
}

impl ListDeliveriesByRecipientTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListDeliveriesByRecipientTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListDeliveriesParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Listing deliveries from agent: {} to recipient: {}, limit: {}, offset: {}",
            params.agent_did, params.recipient_did, params.limit, params.offset
        );

        // Get agent-specific storage
        let storage = self
            .tap_integration
            .storage_for_agent(&params.agent_did)
            .await
            .map_err(|e| Error::configuration(format!("Failed to get storage for agent: {}", e)))?;

        // Query deliveries based on status filter
        let deliveries = match params.status_filter.as_deref() {
            Some("failed") => storage
                .get_failed_deliveries_for_recipient(
                    &params.recipient_did,
                    params.limit,
                    params.offset,
                )
                .await
                .map_err(|e| Error::tool_execution(format!("Failed to get deliveries: {}", e)))?,
            _ => {
                // For now, we only have get_failed_deliveries_for_recipient implemented
                // We need to add a general get_deliveries_by_recipient function
                storage
                    .get_deliveries_by_recipient(&params.recipient_did, params.limit, params.offset)
                    .await
                    .map_err(|e| {
                        Error::tool_execution(format!("Failed to get deliveries: {}", e))
                    })?
            }
        };

        // Convert to response format
        let delivery_infos: Vec<DeliveryInfo> = deliveries
            .into_iter()
            .map(|d| DeliveryInfo {
                id: d.id,
                message_id: d.message_id,
                message_text: d.message_text,
                recipient_did: d.recipient_did,
                delivery_url: d.delivery_url,
                delivery_type: d.delivery_type.to_string(),
                status: d.status.to_string(),
                retry_count: d.retry_count,
                last_http_status_code: d.last_http_status_code,
                error_message: d.error_message,
                created_at: d.created_at,
                updated_at: d.updated_at,
                delivered_at: d.delivered_at,
            })
            .collect();

        let total = delivery_infos.len() as u32;

        let response = ListDeliveriesResponse {
            deliveries: delivery_infos,
            total,
        };

        let json_response = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(json_response))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_deliveries_by_recipient".to_string(),
            description: "Lists TAP message deliveries for a specific recipient with filtering and pagination support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose sent deliveries to list"
                    },
                    "recipient_did": {
                        "type": "string",
                        "description": "The DID of the recipient whose deliveries to list"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of deliveries to return",
                        "default": 50,
                        "minimum": 1,
                        "maximum": 1000
                    },
                    "offset": {
                        "type": "number",
                        "description": "Number of deliveries to skip for pagination",
                        "default": 0,
                        "minimum": 0
                    },
                    "status_filter": {
                        "type": "string",
                        "description": "Filter by delivery status",
                        "enum": ["all", "pending", "delivered", "failed"]
                    }
                },
                "required": ["agent_did", "recipient_did"]
            }),
        }
    }
}

/// Tool for listing deliveries by message ID
pub struct ListDeliveriesByMessageTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing deliveries by message
#[derive(Debug, Deserialize)]
struct ListDeliveriesByMessageParams {
    agent_did: String, // The DID of the agent whose sent deliveries to list
    message_id: String,
}

impl ListDeliveriesByMessageTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListDeliveriesByMessageTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListDeliveriesByMessageParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Listing deliveries from agent: {} for message: {}",
            params.agent_did, params.message_id
        );

        // Get agent-specific storage
        let storage = self
            .tap_integration
            .storage_for_agent(&params.agent_did)
            .await
            .map_err(|e| Error::configuration(format!("Failed to get storage for agent: {}", e)))?;

        // Query deliveries for the specific message
        let deliveries = storage
            .get_deliveries_for_message(&params.message_id)
            .await
            .map_err(|e| Error::tool_execution(format!("Failed to get deliveries: {}", e)))?;

        // Convert to response format
        let delivery_infos: Vec<DeliveryInfo> = deliveries
            .into_iter()
            .map(|d| DeliveryInfo {
                id: d.id,
                message_id: d.message_id,
                message_text: d.message_text,
                recipient_did: d.recipient_did,
                delivery_url: d.delivery_url,
                delivery_type: d.delivery_type.to_string(),
                status: d.status.to_string(),
                retry_count: d.retry_count,
                last_http_status_code: d.last_http_status_code,
                error_message: d.error_message,
                created_at: d.created_at,
                updated_at: d.updated_at,
                delivered_at: d.delivered_at,
            })
            .collect();

        let total = delivery_infos.len() as u32;

        let response = ListDeliveriesResponse {
            deliveries: delivery_infos,
            total,
        };

        let json_response = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(json_response))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_deliveries_by_message".to_string(),
            description: "Lists TAP message deliveries for a specific message ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose sent deliveries to list"
                    },
                    "message_id": {
                        "type": "string",
                        "description": "The message ID to get deliveries for"
                    }
                },
                "required": ["agent_did", "message_id"]
            }),
        }
    }
}

/// Tool for listing deliveries by thread ID
pub struct ListDeliveriesByThreadTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for listing deliveries by thread
#[derive(Debug, Deserialize)]
struct ListDeliveriesByThreadParams {
    agent_did: String, // The DID of the agent whose sent deliveries to list
    thread_id: String,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

impl ListDeliveriesByThreadTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait::async_trait]
impl ToolHandler for ListDeliveriesByThreadTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: ListDeliveriesByThreadParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Listing deliveries from agent: {} for thread: {}, limit: {}, offset: {}",
            params.agent_did, params.thread_id, params.limit, params.offset
        );

        // Get agent-specific storage
        let storage = self
            .tap_integration
            .storage_for_agent(&params.agent_did)
            .await
            .map_err(|e| Error::configuration(format!("Failed to get storage for agent: {}", e)))?;

        // Query deliveries for the specific thread
        let deliveries = storage
            .get_deliveries_for_thread(&params.thread_id, params.limit, params.offset)
            .await
            .map_err(|e| Error::tool_execution(format!("Failed to get deliveries: {}", e)))?;

        // Convert to response format
        let delivery_infos: Vec<DeliveryInfo> = deliveries
            .into_iter()
            .map(|d| DeliveryInfo {
                id: d.id,
                message_id: d.message_id,
                message_text: d.message_text,
                recipient_did: d.recipient_did,
                delivery_url: d.delivery_url,
                delivery_type: d.delivery_type.to_string(),
                status: d.status.to_string(),
                retry_count: d.retry_count,
                last_http_status_code: d.last_http_status_code,
                error_message: d.error_message,
                created_at: d.created_at,
                updated_at: d.updated_at,
                delivered_at: d.delivered_at,
            })
            .collect();

        let total = delivery_infos.len() as u32;

        let response = ListDeliveriesResponse {
            deliveries: delivery_infos,
            total,
        };

        let json_response = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(json_response))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_deliveries_by_thread".to_string(),
            description: "Lists TAP message deliveries for all messages in a specific thread"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose sent deliveries to list"
                    },
                    "thread_id": {
                        "type": "string",
                        "description": "The thread ID to get deliveries for"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of deliveries to return",
                        "default": 50,
                        "minimum": 1,
                        "maximum": 1000
                    },
                    "offset": {
                        "type": "number",
                        "description": "Number of deliveries to skip for pagination",
                        "default": 0,
                        "minimum": 0
                    }
                },
                "required": ["agent_did", "thread_id"]
            }),
        }
    }
}
