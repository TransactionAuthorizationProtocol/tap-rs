//! Communication tools for Trust Ping and Basic Message protocols

use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use crate::tools::{error_text_response, success_text_response, ToolHandler};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{BasicMessage, TrustPing};
use tracing::{debug, error};

/// Tool for sending Trust Ping messages
pub struct TrustPingTool {
    tap_integration: Arc<TapIntegration>,
}

impl TrustPingTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for TrustPingTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let args =
            arguments.ok_or_else(|| Error::tool_execution("Arguments required".to_string()))?;

        // Parse arguments
        let from_did = args
            .get("from_did")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::tool_execution("from_did is required".to_string()))?;

        let to_did = args
            .get("to_did")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::tool_execution("to_did is required".to_string()))?;

        let response_requested = args
            .get("response_requested")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let comment = args
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        debug!(
            "Sending Trust Ping from {} to {}, response_requested: {}",
            from_did, to_did, response_requested
        );

        // Create Trust Ping message
        let mut ping = TrustPing::new().response_requested(response_requested);

        if let Some(comment_text) = comment {
            ping = TrustPing::with_comment(comment_text);
            ping = ping.response_requested(response_requested);
        }

        // Create PlainMessage
        let ping_message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).map_err(|e| {
                Error::tool_execution(format!("Failed to serialize Trust Ping: {}", e))
            })?,
            from: from_did.to_string(),
            to: vec![to_did.to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Send the message through the underlying TAP node
        match self
            .tap_integration
            .node()
            .send_message(from_did.to_string(), ping_message)
            .await
        {
            Ok(message_id) => {
                let response_text = if response_requested {
                    format!("Trust Ping sent successfully with ID: {}. Response requested from recipient.", message_id)
                } else {
                    format!(
                        "Trust Ping sent successfully with ID: {}. No response expected.",
                        message_id
                    )
                };
                Ok(success_text_response(response_text))
            }
            Err(e) => {
                error!("Failed to send Trust Ping: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send Trust Ping: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_trust_ping".to_string(),
            description: "Send a Trust Ping message to test connectivity with another agent"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from_did": {
                        "type": "string",
                        "description": "The DID of the sender agent"
                    },
                    "to_did": {
                        "type": "string",
                        "description": "The DID of the recipient agent"
                    },
                    "response_requested": {
                        "type": "boolean",
                        "description": "Whether a response is requested (default: true)",
                        "default": true
                    },
                    "comment": {
                        "type": "string",
                        "description": "Optional comment to include with the ping"
                    }
                },
                "required": ["from_did", "to_did"],
                "additionalProperties": false
            }),
        }
    }
}

/// Tool for sending Basic Message (text messages)
pub struct BasicMessageTool {
    tap_integration: Arc<TapIntegration>,
}

impl BasicMessageTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for BasicMessageTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let args =
            arguments.ok_or_else(|| Error::tool_execution("Arguments required".to_string()))?;

        // Parse arguments
        let from_did = args
            .get("from_did")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::tool_execution("from_did is required".to_string()))?;

        let to_did = args
            .get("to_did")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::tool_execution("to_did is required".to_string()))?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::tool_execution("content is required".to_string()))?;

        let locale = args
            .get("locale")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        debug!("Sending Basic Message from {} to {}", from_did, to_did);

        // Create Basic Message
        let basic_message = BasicMessage {
            content: content.to_string(),
            locale,
            sent_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            metadata: HashMap::new(),
        };

        // Create PlainMessage
        let message = PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: BasicMessage::message_type().to_string(),
            body: serde_json::to_value(&basic_message).map_err(|e| {
                Error::tool_execution(format!("Failed to serialize Basic Message: {}", e))
            })?,
            from: from_did.to_string(),
            to: vec![to_did.to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Send the message through the underlying TAP node
        match self
            .tap_integration
            .node()
            .send_message(from_did.to_string(), message)
            .await
        {
            Ok(message_id) => Ok(success_text_response(format!(
                "Basic Message sent successfully with ID: {}. Content: \"{}\"",
                message_id, content
            ))),
            Err(e) => {
                error!("Failed to send Basic Message: {}", e);
                Ok(error_text_response(format!(
                    "Failed to send Basic Message: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_basic_message".to_string(),
            description: "Send a basic text message to another agent".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "from_did": {
                        "type": "string",
                        "description": "The DID of the sender agent"
                    },
                    "to_did": {
                        "type": "string",
                        "description": "The DID of the recipient agent"
                    },
                    "content": {
                        "type": "string",
                        "description": "The text content of the message"
                    },
                    "locale": {
                        "type": "string",
                        "description": "Optional locale/language code (e.g., 'en-US')"
                    }
                },
                "required": ["from_did", "to_did", "content"],
                "additionalProperties": false
            }),
        }
    }
}
