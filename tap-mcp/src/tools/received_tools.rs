//! Tools for working with received messages

use crate::error::Result;
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use crate::tools::{default_limit, error_text_response, ToolContent, ToolHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tap_node::storage::{ReceivedStatus, SourceType};
use tracing::{debug, error};

/// Input for listing received messages
#[derive(Debug, Deserialize)]
pub struct ListReceivedInput {
    /// The DID of the agent whose received messages to list
    pub agent_did: String,
    /// Maximum number of messages to return
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Number of messages to skip for pagination
    #[serde(default)]
    pub offset: u32,
    /// Filter by source type
    pub source_type: Option<String>,
    /// Filter by status
    pub status: Option<String>,
}

/// Output for received message listing
#[derive(Debug, Serialize)]
pub struct ReceivedMessage {
    /// Unique ID of the received record
    pub id: i64,
    /// Message ID extracted from the raw message (if available)
    pub message_id: Option<String>,
    /// Source type (https, internal, websocket, return_path, pickup)
    pub source_type: String,
    /// Source identifier (URL, agent DID, etc.)
    pub source_identifier: Option<String>,
    /// Processing status (pending, processed, failed)
    pub status: String,
    /// Error message if processing failed
    pub error_message: Option<String>,
    /// When the message was received
    pub received_at: String,
    /// When the message was processed
    pub processed_at: Option<String>,
    /// ID of the processed message in messages table
    pub processed_message_id: Option<String>,
}

/// Tool for listing received messages
pub struct ListReceivedTool {
    tap_integration: Arc<TapIntegration>,
}

impl ListReceivedTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for ListReceivedTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let input: ListReceivedInput = match arguments {
            Some(args) => serde_json::from_value(args)?,
            None => {
                return Ok(error_text_response(
                    "Missing required arguments".to_string(),
                ));
            }
        };

        debug!("Listing received messages for agent: {}", input.agent_did);

        // Parse source type if provided
        let source_type = input.source_type.as_ref().and_then(|s| match s.as_str() {
            "https" => Some(SourceType::Https),
            "internal" => Some(SourceType::Internal),
            "websocket" => Some(SourceType::WebSocket),
            "return_path" => Some(SourceType::ReturnPath),
            "pickup" => Some(SourceType::Pickup),
            _ => None,
        });

        // Parse status if provided
        let status = input.status.as_ref().and_then(|s| match s.as_str() {
            "pending" => Some(ReceivedStatus::Pending),
            "processed" => Some(ReceivedStatus::Processed),
            "failed" => Some(ReceivedStatus::Failed),
            _ => None,
        });

        // Get agent storage
        let storage = match self
            .tap_integration
            .storage_for_agent(&input.agent_did)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get agent storage: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    input.agent_did, e
                )));
            }
        };

        // List received messages
        let messages = match storage
            .list_received(input.limit, input.offset, source_type, status)
            .await
        {
            Ok(msgs) => msgs,
            Err(e) => {
                error!("Failed to list received messages: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to list received messages: {}",
                    e
                )));
            }
        };

        let received_messages: Vec<ReceivedMessage> = messages
            .into_iter()
            .map(|m| ReceivedMessage {
                id: m.id,
                message_id: m.message_id,
                source_type: format!("{:?}", m.source_type).to_lowercase(),
                source_identifier: m.source_identifier,
                status: format!("{:?}", m.status).to_lowercase(),
                error_message: m.error_message,
                received_at: m.received_at,
                processed_at: m.processed_at,
                processed_message_id: m.processed_message_id,
            })
            .collect();

        let text = format!(
            "Found {} received messages for agent {}",
            received_messages.len(),
            input.agent_did
        );

        Ok(CallToolResult {
            content: vec![
                ToolContent::Text { text },
                ToolContent::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "messages": received_messages,
                        "total": received_messages.len(),
                        "agent_did": input.agent_did,
                        "limit": input.limit,
                        "offset": input.offset,
                        "filters": {
                            "source_type": input.source_type,
                            "status": input.status
                        }
                    }))
                    .unwrap_or_else(|_| "Failed to serialize JSON".to_string()),
                },
            ],
            is_error: Some(false),
        })
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_list_received".to_string(),
            description: "Lists raw received messages with filtering and pagination support. Shows all incoming messages (JWE, JWS, or plain) before processing.".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["agent_did"],
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose received messages to list"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of messages to return",
                        "default": 50,
                        "minimum": 1,
                        "maximum": 1000
                    },
                    "offset": {
                        "type": "number",
                        "description": "Number of messages to skip for pagination",
                        "default": 0,
                        "minimum": 0
                    },
                    "source_type": {
                        "type": "string",
                        "description": "Filter by source type",
                        "enum": ["https", "internal", "websocket", "return_path", "pickup"]
                    },
                    "status": {
                        "type": "string",
                        "description": "Filter by processing status",
                        "enum": ["pending", "processed", "failed"]
                    }
                },
                "additionalProperties": false
            }),
        }
    }
}

/// Input for getting pending received messages
#[derive(Debug, Deserialize)]
pub struct GetPendingReceivedInput {
    /// The DID of the agent whose pending messages to get
    pub agent_did: String,
    /// Maximum number of messages to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Tool for getting pending received messages
pub struct GetPendingReceivedTool {
    tap_integration: Arc<TapIntegration>,
}

impl GetPendingReceivedTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for GetPendingReceivedTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let input: GetPendingReceivedInput = match arguments {
            Some(args) => serde_json::from_value(args)?,
            None => {
                return Ok(error_text_response(
                    "Missing required arguments".to_string(),
                ));
            }
        };

        debug!(
            "Getting pending received messages for agent: {}",
            input.agent_did
        );

        // Get agent storage
        let storage = match self
            .tap_integration
            .storage_for_agent(&input.agent_did)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get agent storage: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    input.agent_did, e
                )));
            }
        };

        // Get pending messages
        let messages = match storage.get_pending_received(input.limit).await {
            Ok(msgs) => msgs,
            Err(e) => {
                error!("Failed to get pending received messages: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get pending received messages: {}",
                    e
                )));
            }
        };

        let received_messages: Vec<ReceivedMessage> = messages
            .into_iter()
            .map(|m| ReceivedMessage {
                id: m.id,
                message_id: m.message_id,
                source_type: format!("{:?}", m.source_type).to_lowercase(),
                source_identifier: m.source_identifier,
                status: format!("{:?}", m.status).to_lowercase(),
                error_message: m.error_message,
                received_at: m.received_at,
                processed_at: m.processed_at,
                processed_message_id: m.processed_message_id,
            })
            .collect();

        let text = format!(
            "Found {} pending messages for agent {}",
            received_messages.len(),
            input.agent_did
        );

        Ok(CallToolResult {
            content: vec![
                ToolContent::Text { text },
                ToolContent::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "messages": received_messages,
                        "total": received_messages.len(),
                        "agent_did": input.agent_did
                    }))
                    .unwrap_or_else(|_| "Failed to serialize JSON".to_string()),
                },
            ],
            is_error: Some(false),
        })
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_get_pending_received".to_string(),
            description: "Gets pending received messages that haven't been processed yet. Useful for debugging message processing issues.".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["agent_did"],
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose pending messages to get"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of messages to return",
                        "default": 50,
                        "minimum": 1,
                        "maximum": 1000
                    }
                },
                "additionalProperties": false
            }),
        }
    }
}

/// Input for viewing raw received message
#[derive(Debug, Deserialize)]
pub struct ViewRawReceivedInput {
    /// The DID of the agent who owns the received message
    pub agent_did: String,
    /// The ID of the received record
    pub received_id: i64,
}

/// Tool for viewing raw received message content
pub struct ViewRawReceivedTool {
    tap_integration: Arc<TapIntegration>,
}

impl ViewRawReceivedTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for ViewRawReceivedTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let input: ViewRawReceivedInput = match arguments {
            Some(args) => serde_json::from_value(args)?,
            None => {
                return Ok(error_text_response(
                    "Missing required arguments".to_string(),
                ));
            }
        };

        debug!(
            "Viewing raw received message {} for agent: {}",
            input.received_id, input.agent_did
        );

        // Get agent storage
        let storage = match self
            .tap_integration
            .storage_for_agent(&input.agent_did)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get agent storage: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    input.agent_did, e
                )));
            }
        };

        // Get the received record
        let received = match storage.get_received_by_id(input.received_id).await {
            Ok(Some(r)) => r,
            Ok(None) => {
                return Ok(error_text_response(format!(
                    "Received message {} not found",
                    input.received_id
                )));
            }
            Err(e) => {
                error!("Failed to get received message: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to get received message: {}",
                    e
                )));
            }
        };

        // Parse the raw message as JSON if possible
        let raw_json = serde_json::from_str::<Value>(&received.raw_message).ok();

        let text = format!(
            "Received message {} (status: {:?})",
            input.received_id, received.status
        );

        Ok(CallToolResult {
            content: vec![
                ToolContent::Text { text },
                ToolContent::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "id": received.id,
                        "message_id": received.message_id,
                        "source_type": format!("{:?}", received.source_type).to_lowercase(),
                        "source_identifier": received.source_identifier,
                        "status": format!("{:?}", received.status).to_lowercase(),
                        "error_message": received.error_message,
                        "received_at": received.received_at,
                        "processed_at": received.processed_at,
                        "processed_message_id": received.processed_message_id,
                        "raw_message": received.raw_message,
                        "raw_message_json": raw_json
                    }))
                    .unwrap_or_else(|_| "Failed to serialize JSON".to_string()),
                },
            ],
            is_error: Some(false),
        })
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_view_raw_received".to_string(),
            description: "Views the raw content of a received message. Shows the complete raw message as received (JWE, JWS, or plain JSON).".to_string(),
            input_schema: json!({
                "type": "object",
                "required": ["agent_did", "received_id"],
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent who owns the received message"
                    },
                    "received_id": {
                        "type": "number",
                        "description": "The ID of the received record"
                    }
                },
                "additionalProperties": false
            }),
        }
    }
}
