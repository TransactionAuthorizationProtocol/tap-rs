//! Integration tests for TAP-MCP
//!
//! These tests validate the complete MCP protocol implementation
//! and TAP functionality end-to-end.

use assert_matches::assert_matches;
use serde_json::{json, Value};
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

use tap_mcp::error::Result;
use tap_mcp::mcp::protocol::*;
use tap_mcp::mcp::McpServer;
use tap_mcp::tap_integration::TapIntegration;

/// Test helper to create a temporary TAP environment
struct TestEnvironment {
    _temp_dir: TempDir,
    tap_root: PathBuf,
}

impl TestEnvironment {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let tap_root = temp_dir.path().join("tap");

        std::fs::create_dir_all(&tap_root)?;

        Ok(Self {
            _temp_dir: temp_dir,
            tap_root,
        })
    }

    async fn create_integration(&self) -> Result<TapIntegration> {
        TapIntegration::new_for_testing(
            Some(self.tap_root.to_str().unwrap()),
            "did:example:test-agent",
        )
        .await
    }

    async fn create_server(&self) -> Result<McpServer> {
        let integration = self.create_integration().await?;
        McpServer::new(integration).await
    }
}

/// Test helper for MCP protocol messages
struct McpTestClient {
    next_id: i64,
}

impl McpTestClient {
    fn new() -> Self {
        Self { next_id: 1 }
    }

    fn next_id(&mut self) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        json!(id)
    }

    fn create_initialize_request(&mut self) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            id: Some(self.next_id()),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        }
    }

    fn create_list_tools_request(&mut self) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            id: Some(self.next_id()),
            params: None,
        }
    }

    fn create_call_tool_request(&mut self, name: &str, arguments: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            id: Some(self.next_id()),
            params: Some(json!({
                "name": name,
                "arguments": arguments
            })),
        }
    }

    fn create_list_resources_request(&mut self) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources/list".to_string(),
            id: Some(self.next_id()),
            params: None,
        }
    }

    fn create_read_resource_request(&mut self, uri: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources/read".to_string(),
            id: Some(self.next_id()),
            params: Some(json!({
                "uri": uri
            })),
        }
    }
}

#[tokio::test]
async fn test_mcp_initialization() -> Result<()> {
    let env = TestEnvironment::new()?;
    let integration = env.create_integration().await?;
    let mut server = McpServer::new(integration).await?;
    let mut client = McpTestClient::new();

    // Test initialization
    let init_request = client.create_initialize_request();
    let response = server.handle_request_direct(init_request).await?;

    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert!(result["capabilities"]["tools"].is_object());
        assert!(result["capabilities"]["resources"].is_object());
    }

    Ok(())
}

#[tokio::test]
async fn test_list_tools() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize first
    let init_request = client.create_initialize_request();
    server.handle_request_direct(init_request).await?;

    // List tools
    let list_request = client.create_list_tools_request();
    let response = server.handle_request_direct(list_request).await?;

    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 8); // All 8 tools should be available

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

        assert!(tool_names.contains(&"tap.create_agent"));
        assert!(tool_names.contains(&"tap.list_agents"));
        assert!(tool_names.contains(&"tap.create_transfer"));
        assert!(tool_names.contains(&"tap.authorize"));
        assert!(tool_names.contains(&"tap.reject"));
        assert!(tool_names.contains(&"tap.cancel"));
        assert!(tool_names.contains(&"tap.settle"));
        assert!(tool_names.contains(&"tap.list_transactions"));
    }

    Ok(())
}

#[tokio::test]
async fn test_create_agent_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create agent
    let agent_id = format!("did:example:test-agent-{}", Uuid::new_v4());
    let create_request = client.create_call_tool_request(
        "tap.create_agent",
        json!({
            "@id": agent_id,
            "role": "SettlementAddress",
            "for": "did:example:test-party"
        }),
    );

    let response = server.handle_request_direct(create_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        assert_eq!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("created"),
            true
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_list_agents_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create an agent first
    let agent_id = format!("did:example:list-test-{}", Uuid::new_v4());
    server
        .handle_request_direct(client.create_call_tool_request(
            "tap.create_agent",
            json!({
                "@id": agent_id,
                "role": "Exchange",
                "for": "did:example:party"
            }),
        ))
        .await?;

    // List agents
    let list_request = client.create_call_tool_request("tap.list_agents", json!({}));

    let response = server.handle_request_direct(list_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let content_text = result["content"][0]["text"].as_str().unwrap();
        let agent_list: Value = serde_json::from_str(content_text)?;
        assert!(agent_list["total"].as_u64().unwrap() >= 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_create_transfer_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create transfer
    let transfer_request = client.create_call_tool_request(
        "tap.create_transfer",
        json!({
            "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
            "amount": "100.50",
            "originator": {
                "@id": "did:example:alice"
            },
            "beneficiary": {
                "@id": "did:example:bob"
            },
            "agents": [],
            "memo": "Test transfer"
        }),
    );

    let response = server.handle_request_direct(transfer_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let content_text = result["content"][0]["text"].as_str().unwrap();
        let transfer_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(transfer_result["status"], "created");
        assert!(transfer_result["transaction_id"].is_string());
    }

    Ok(())
}

#[tokio::test]
async fn test_authorize_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Authorize transaction
    let auth_request = client.create_call_tool_request(
        "tap.authorize",
        json!({
            "transaction_id": "test-tx-123",
            "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87"
        }),
    );

    let response = server.handle_request_direct(auth_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let content_text = result["content"][0]["text"].as_str().unwrap();
        let auth_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(auth_result["status"], "authorized");
        assert_eq!(auth_result["transaction_id"], "test-tx-123");
    }

    Ok(())
}

#[tokio::test]
async fn test_reject_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Reject transaction
    let reject_request = client.create_call_tool_request(
        "tap.reject",
        json!({
            "transaction_id": "test-tx-456",
            "reason": "Insufficient compliance verification"
        }),
    );

    let response = server.handle_request_direct(reject_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let content_text = result["content"][0]["text"].as_str().unwrap();
        let reject_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(reject_result["status"], "rejected");
        assert_eq!(
            reject_result["reason"],
            "Insufficient compliance verification"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_list_resources() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // List resources
    let list_request = client.create_list_resources_request();
    let response = server.handle_request_direct(list_request).await?;

    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let resources = result["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 3); // agents, messages, schemas

        let resource_uris: Vec<&str> = resources
            .iter()
            .map(|r| r["uri"].as_str().unwrap())
            .collect();

        assert!(resource_uris.contains(&"tap://agents"));
        assert!(resource_uris.contains(&"tap://messages"));
        assert!(resource_uris.contains(&"tap://schemas"));
    }

    Ok(())
}

#[tokio::test]
async fn test_read_schemas_resource() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Read schemas resource
    let read_request = client.create_read_resource_request("tap://schemas");
    let response = server.handle_request_direct(read_request).await?;

    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        let contents = result["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1);

        let schema_content = &contents[0];
        assert_eq!(schema_content["uri"], "tap://schemas");
        assert_eq!(schema_content["mimeType"], "application/json");

        let schemas_text = schema_content["text"].as_str().unwrap();
        let schemas: Value = serde_json::from_str(schemas_text)?;
        assert!(schemas["schemas"]["Transfer"].is_object());
        assert!(schemas["schemas"]["Authorize"].is_object());
        assert!(schemas["schemas"]["Reject"].is_object());
    }

    Ok(())
}

#[tokio::test]
async fn test_complete_transaction_flow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // 1. Create agents
    let alice_agent_id = format!("did:example:alice-agent-{}", Uuid::new_v4());
    server
        .handle_request_direct(client.create_call_tool_request(
            "tap.create_agent",
            json!({
                "@id": alice_agent_id,
                "role": "SettlementAddress",
                "for": "did:example:alice"
            }),
        ))
        .await?;

    // 2. Create transfer
    let transfer_response = server.handle_request_direct(client.create_call_tool_request(
        "tap.create_transfer",
        json!({
            "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
            "amount": "250.00",
            "originator": {"@id": "did:example:alice"},
            "beneficiary": {"@id": "did:example:bob"},
            "agents": [{"@id": alice_agent_id, "role": "SettlementAddress", "for": "did:example:alice"}]
        })
    )).await?;

    // Extract transaction ID
    let transfer_result_value = transfer_response.result.unwrap();
    let transfer_content = transfer_result_value["content"][0]["text"]
        .as_str()
        .unwrap();
    let transfer_result: Value = serde_json::from_str(transfer_content)?;
    let transaction_id = transfer_result["transaction_id"].as_str().unwrap();

    // 3. Authorize transaction
    let auth_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap.authorize",
            json!({
                "transaction_id": transaction_id,
                "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87"
            }),
        ))
        .await?;

    let auth_result_value = auth_response.result.unwrap();
    let auth_content = auth_result_value["content"][0]["text"].as_str().unwrap();
    let auth_result: Value = serde_json::from_str(auth_content)?;
    assert_eq!(auth_result["status"], "authorized");

    // 4. Settle transaction
    let settle_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap.settle",
            json!({
                "transaction_id": transaction_id,
                "settlement_id": "eip155:1:0xabcd1234567890abcdef1234567890abcdef1234",
                "amount": "250.00"
            }),
        ))
        .await?;

    let settle_result_value = settle_response.result.unwrap();
    let settle_content = settle_result_value["content"][0]["text"].as_str().unwrap();
    let settle_result: Value = serde_json::from_str(settle_content)?;
    assert_eq!(settle_result["status"], "settled");

    // 5. List transactions to verify
    let list_response = server
        .handle_request_direct(client.create_call_tool_request("tap.list_transactions", json!({})))
        .await?;

    let list_result_value = list_response.result.unwrap();
    let list_content = list_result_value["content"][0]["text"].as_str().unwrap();
    let list_result: Value = serde_json::from_str(list_content)?;
    assert!(list_result["total"].as_u64().unwrap() >= 3); // transfer, auth, settle messages

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Test invalid tool call
    let invalid_request = client.create_call_tool_request("tap.invalid_tool", json!({}));

    let response = server.handle_request_direct(invalid_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("not found"));
    }

    // Test invalid parameters
    let invalid_params_request = client.create_call_tool_request(
        "tap.create_agent",
        json!({
            "invalid": "parameters"
        }),
    );

    let response = server.handle_request_direct(invalid_params_request).await?;
    assert_matches!(
        response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = response.result {
        assert_eq!(result["isError"], true);
    }

    Ok(())
}
