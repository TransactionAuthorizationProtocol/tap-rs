//! JSON-RPC 2.0 protocol types for external decision communication.
//!
//! Messages flow over stdin (tap-http → child) and stdout (child → tap-http)
//! using newline-delimited JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request (has an `id` — expects a response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 notification (no `id` — fire-and-forget)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Value,
}

/// JSON-RPC 2.0 error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub error: JsonRpcError,
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// A message read from stdout can be a request (tool call) or a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IncomingMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
}

// -----------------------------------------------------------------------
// tap-http → External Process (stdin)
// -----------------------------------------------------------------------

/// Initialize message sent when the external process starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub version: String,
    pub agent_dids: Vec<String>,
    pub subscribe_mode: String,
    pub capabilities: InitializeCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeCapabilities {
    pub tools: bool,
    pub decisions: bool,
}

/// Decision request sent to external process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequestParams {
    pub decision_id: i64,
    pub transaction_id: String,
    pub agent_did: String,
    pub decision_type: String,
    pub context: Value,
    pub created_at: String,
}

/// Event notification sent to external process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventNotificationParams {
    pub event_type: String,
    pub agent_did: Option<String>,
    pub data: Value,
    pub timestamp: String,
}

// -----------------------------------------------------------------------
// External Process → tap-http (stdout)
// -----------------------------------------------------------------------

/// Decision response from external process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}

/// Ready notification from external process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyParams {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

// -----------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------

impl JsonRpcRequest {
    pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcResponse {
    pub fn new(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

impl JsonRpcErrorResponse {
    pub fn new(id: Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            error: JsonRpcError {
                code,
                message: message.into(),
                data: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_decision_request_serialization() {
        let params = DecisionRequestParams {
            decision_id: 42,
            transaction_id: "txn-123".to_string(),
            agent_did: "did:key:z6Mk...".to_string(),
            decision_type: "authorization_required".to_string(),
            context: json!({
                "transaction_state": "Received",
                "pending_agents": ["did:key:z6Mk..."],
            }),
            created_at: "2026-02-21T12:00:00Z".to_string(),
        };

        let req = JsonRpcRequest::new(
            1,
            "tap/decision",
            Some(serde_json::to_value(&params).unwrap()),
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.method, "tap/decision");
        assert_eq!(parsed.id, json!(1));

        let parsed_params: DecisionRequestParams =
            serde_json::from_value(parsed.params.unwrap()).unwrap();
        assert_eq!(parsed_params.decision_id, 42);
        assert_eq!(parsed_params.transaction_id, "txn-123");
    }

    #[test]
    fn test_decision_response_deserialization() {
        let json = r#"{"action":"authorize","detail":{"settlement_address":"eip155:1:0xABC"}}"#;
        let resp: DecisionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.action, "authorize");
        assert_eq!(resp.detail.unwrap()["settlement_address"], "eip155:1:0xABC");
    }

    #[test]
    fn test_event_notification_serialization() {
        let params = EventNotificationParams {
            event_type: "message_received".to_string(),
            agent_did: Some("did:key:z6Mk...".to_string()),
            data: json!({"message_id": "msg-1", "message_type": "Transfer"}),
            timestamp: "2026-02-21T12:00:00Z".to_string(),
        };

        let notif =
            JsonRpcNotification::new("tap/event", Some(serde_json::to_value(&params).unwrap()));
        let json = serde_json::to_string(&notif).unwrap();

        // Should not have an id field
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("id").is_none());
        assert_eq!(parsed["method"], "tap/event");
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            version: "0.1.0".to_string(),
            agent_dids: vec!["did:key:z6Mk1".to_string(), "did:key:z6Mk2".to_string()],
            subscribe_mode: "decisions".to_string(),
            capabilities: InitializeCapabilities {
                tools: true,
                decisions: true,
            },
        };

        let json_str = serde_json::to_string(&params).unwrap();
        let parsed: InitializeParams = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.version, "0.1.0");
        assert_eq!(parsed.agent_dids.len(), 2);
        assert!(parsed.capabilities.tools);
    }

    #[test]
    fn test_incoming_message_request() {
        let json = r#"{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"tap_authorize","arguments":{}}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Request(req) => {
                assert_eq!(req.method, "tools/call");
                assert_eq!(req.id, json!(100));
            }
            _ => panic!("Expected request"),
        }
    }

    #[test]
    fn test_incoming_message_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"tap/ready","params":{"name":"my-engine"}}"#;
        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        match msg {
            IncomingMessage::Notification(notif) => {
                assert_eq!(notif.method, "tap/ready");
            }
            _ => panic!("Expected notification"),
        }
    }

    #[test]
    fn test_json_rpc_error_response() {
        let err = JsonRpcErrorResponse::new(json!(1), -32600, "Invalid request");
        let json_str = serde_json::to_string(&err).unwrap();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["error"]["code"], -32600);
        assert_eq!(parsed["error"]["message"], "Invalid request");
    }

    #[test]
    fn test_ready_params_deserialization() {
        let json = r#"{"version":"1.0.0","name":"my-compliance-engine"}"#;
        let params: ReadyParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.version, Some("1.0.0".to_string()));
        assert_eq!(params.name, Some("my-compliance-engine".to_string()));
    }

    #[test]
    fn test_ready_params_minimal() {
        let json = r#"{}"#;
        let params: ReadyParams = serde_json::from_str(json).unwrap();
        assert!(params.version.is_none());
        assert!(params.name.is_none());
    }
}
