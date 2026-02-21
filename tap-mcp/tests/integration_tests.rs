//! Integration tests for TAP-MCP
//!
//! These tests validate the complete MCP protocol implementation
//! and TAP functionality end-to-end.

use assert_matches::assert_matches;
use serde_json::{json, Value};
use serial_test::serial;
use std::path::PathBuf;
use tempfile::TempDir;

use tap_mcp::error::Result;
use tap_mcp::mcp::protocol::*;
use tap_mcp::mcp::McpServer;
use tap_mcp::tap_integration::TapIntegration;

/// Test helper to create a temporary TAP environment
struct TestEnvironment {
    _temp_dir: TempDir,
    tap_root: PathBuf,
    _old_tap_home: Option<String>,
}

impl TestEnvironment {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let tap_root = temp_dir.path().join("tap");

        std::fs::create_dir_all(&tap_root)?;

        // Save old TAP_HOME and set to test directory
        // This ensures KeyStorage::save_default/load_default use the temp directory
        let old_tap_home = std::env::var("TAP_HOME").ok();
        std::env::set_var("TAP_HOME", &tap_root);

        Ok(Self {
            _temp_dir: temp_dir,
            tap_root,
            _old_tap_home: old_tap_home,
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

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore old TAP_HOME environment variable
        if let Some(ref old_value) = self._old_tap_home {
            std::env::set_var("TAP_HOME", old_value);
        } else {
            std::env::remove_var("TAP_HOME");
        }
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
#[serial]
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
#[serial]
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
        assert_eq!(tools.len(), 36); // All 36 tools including decision tools

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

        assert!(tool_names.contains(&"tap_create_agent"));
        assert!(tool_names.contains(&"tap_list_agents"));
        assert!(tool_names.contains(&"tap_create_transfer"));
        assert!(tool_names.contains(&"tap_authorize"));
        assert!(tool_names.contains(&"tap_reject"));
        assert!(tool_names.contains(&"tap_cancel"));
        assert!(tool_names.contains(&"tap_settle"));
        assert!(tool_names.contains(&"tap_list_transactions"));
        assert!(tool_names.contains(&"tap_trust_ping"));
        assert!(tool_names.contains(&"tap_basic_message"));
        assert!(tool_names.contains(&"tap_list_deliveries_by_recipient"));
        assert!(tool_names.contains(&"tap_list_deliveries_by_message"));
        assert!(tool_names.contains(&"tap_list_deliveries_by_thread"));
        assert!(tool_names.contains(&"tap_list_customers"));
        assert!(tool_names.contains(&"tap_list_connections"));
        assert!(tool_names.contains(&"tap_create_customer"));
        assert!(tool_names.contains(&"tap_list_received"));
        assert!(tool_names.contains(&"tap_get_pending_received"));
        assert!(tool_names.contains(&"tap_view_raw_received"));
        assert!(tool_names.contains(&"tap_query_database"));
        assert!(tool_names.contains(&"tap_get_database_schema"));
        assert!(tool_names.contains(&"tap_revert"));
        assert!(tool_names.contains(&"tap_add_agents"));
        assert!(tool_names.contains(&"tap_remove_agent"));
        assert!(tool_names.contains(&"tap_replace_agent"));
        assert!(tool_names.contains(&"tap_update_policies"));
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_create_agent_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create agent (auto-generates DID)
    let create_request = client.create_call_tool_request(
        "tap_create_agent",
        json!({
            "label": "Test Agent"
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
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("created"));
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_list_agents_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create an agent first
    server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "Test List Agent"
            }),
        ))
        .await?;

    // List agents
    let list_request = client.create_call_tool_request("tap_list_agents", json!({}));

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
#[serial]
async fn test_create_transfer_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // First create an agent
    let agent_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "Test Agent"
            }),
        ))
        .await?;

    // Extract agent DID from response
    let agent_result_value = agent_response.result.unwrap();
    let agent_content = agent_result_value["content"][0]["text"].as_str().unwrap();
    let agent_result: Value = serde_json::from_str(agent_content)?;
    let agent_did = agent_result["@id"].as_str().unwrap();

    // Create transfer
    let transfer_request = client.create_call_tool_request(
        "tap_create_transfer",
        json!({
            "agent_did": agent_did,
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
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let content_text = result["content"][0]["text"].as_str().unwrap();

        if is_error {
            // Network delivery may fail in test environments
            assert!(
                content_text.contains("Failed to send transfer message")
                    || content_text.contains("Failed to deliver"),
                "Unexpected error: {}",
                content_text
            );
            return Ok(());
        }

        let transfer_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(transfer_result["status"], "sent");
        assert!(transfer_result["transaction_id"].is_string());
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_authorize_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // First create an agent
    let agent_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "Test Agent for Authorization"
            }),
        ))
        .await?;

    // Extract agent DID from response
    let agent_result_value = agent_response.result.unwrap();
    let agent_content = agent_result_value["content"][0]["text"].as_str().unwrap();
    let agent_result: Value = serde_json::from_str(agent_content)?;
    let agent_did = agent_result["@id"].as_str().unwrap();

    // Create a transfer to establish the transaction context
    let transfer_request = client.create_call_tool_request(
        "tap_create_transfer",
        json!({
            "agent_did": agent_did,
            "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
            "amount": "75.00",
            "originator": {
                "@id": agent_did
            },
            "beneficiary": {
                "@id": "did:example:beneficiary"
            },
            "agents": [],
            "memo": "Test transfer for authorization"
        }),
    );

    let transfer_response = server.handle_request_direct(transfer_request).await?;
    let transfer_result = transfer_response.result.unwrap();
    let is_error = transfer_result["isError"].as_bool().unwrap_or(false);
    let transfer_content = transfer_result["content"][0]["text"].as_str().unwrap();

    if is_error {
        // Network delivery may fail in test environments
        assert!(
            transfer_content.contains("Failed to send transfer message")
                || transfer_content.contains("Failed to deliver"),
            "Unexpected error: {}",
            transfer_content
        );
        return Ok(());
    }

    let transfer_data: Value = serde_json::from_str(transfer_content)?;
    let transaction_id = transfer_data["transaction_id"].as_str().unwrap();

    // Now authorize the transaction
    let auth_request = client.create_call_tool_request(
        "tap_authorize",
        json!({
            "agent_did": agent_did,
            "transaction_id": transaction_id,
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
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let content_text = result["content"][0]["text"].as_str().unwrap();

        if is_error {
            // Expected for now - authorize messages don't have recipient info
            assert_eq!(content_text, "No recipient found for authorize message");
            return Ok(());
        }

        let auth_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(auth_result["status"], "sent");
        assert_eq!(auth_result["transaction_id"], transaction_id);
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_reject_tool() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // First create an agent
    let agent_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "Test Agent for Rejection"
            }),
        ))
        .await?;

    // Extract agent DID from response
    let agent_result_value = agent_response.result.unwrap();
    let agent_content = agent_result_value["content"][0]["text"].as_str().unwrap();
    let agent_result: Value = serde_json::from_str(agent_content)?;
    let agent_did = agent_result["@id"].as_str().unwrap();

    // Create a transfer to establish the transaction context
    let transfer_request = client.create_call_tool_request(
        "tap_create_transfer",
        json!({
            "agent_did": agent_did,
            "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
            "amount": "50.00",
            "originator": {
                "@id": agent_did
            },
            "beneficiary": {
                "@id": "did:example:beneficiary"
            },
            "agents": [],
            "memo": "Test transfer for rejection"
        }),
    );

    let transfer_response = server.handle_request_direct(transfer_request).await?;
    let transfer_result = transfer_response.result.unwrap();
    let is_error = transfer_result["isError"].as_bool().unwrap_or(false);
    let transfer_content = transfer_result["content"][0]["text"].as_str().unwrap();

    if is_error {
        // Network delivery may fail in test environments
        assert!(
            transfer_content.contains("Failed to send transfer message")
                || transfer_content.contains("Failed to deliver"),
            "Unexpected error: {}",
            transfer_content
        );
        return Ok(());
    }

    let transfer_data: Value = serde_json::from_str(transfer_content)?;
    let transaction_id = transfer_data["transaction_id"].as_str().unwrap();

    // Now reject the transaction
    let reject_request = client.create_call_tool_request(
        "tap_reject",
        json!({
            "agent_did": agent_did,
            "transaction_id": transaction_id,
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
        let is_error = result["isError"].as_bool().unwrap_or(false);
        let content_text = result["content"][0]["text"].as_str().unwrap();

        if is_error {
            // Expected for now - reject messages don't have recipient info
            assert_eq!(content_text, "No recipient found for reject message");
            return Ok(());
        }

        let reject_result: Value = serde_json::from_str(content_text)?;
        assert_eq!(reject_result["status"], "sent");
        assert_eq!(
            reject_result["reason"],
            "Insufficient compliance verification"
        );
        assert_eq!(reject_result["transaction_id"], transaction_id);
    }

    Ok(())
}

#[tokio::test]
#[serial]
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
        assert_eq!(resources.len(), 6); // agents, messages, deliveries, database-schema, schemas, received

        let resource_uris: Vec<&str> = resources
            .iter()
            .map(|r| r["uri"].as_str().unwrap())
            .collect();

        assert!(resource_uris.contains(&"tap://agents"));
        assert!(resource_uris.contains(&"tap://messages"));
        assert!(resource_uris.contains(&"tap://deliveries"));
        assert!(resource_uris.contains(&"tap://database-schema"));
        assert!(resource_uris.contains(&"tap://schemas"));
        assert!(resource_uris.contains(&"tap://received"));
    }

    Ok(())
}

#[tokio::test]
#[serial]
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
#[serial]
async fn test_read_specific_schema_resource() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Read specific schema resource - Payment
    let read_request = client.create_read_resource_request("tap://schemas/Payment");
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
        assert_eq!(schema_content["uri"], "tap://schemas/Payment");
        assert_eq!(schema_content["mimeType"], "application/json");

        let schema_text = schema_content["text"].as_str().unwrap();
        let schema: Value = serde_json::from_str(schema_text)?;

        // Should only contain Payment schema, not all schemas
        assert!(schema["schemas"]["Payment"].is_object());
        assert!(
            schema["schemas"]["Transfer"].is_null()
                || !schema["schemas"]
                    .as_object()
                    .unwrap()
                    .contains_key("Transfer")
        );
        assert_eq!(schema["schemas"].as_object().unwrap().len(), 1);

        // Verify it's the Payment schema
        assert_eq!(
            schema["schemas"]["Payment"]["message_type"],
            "https://tap.rsvp/schema/1.0#Payment"
        );
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_complete_transaction_flow() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // 1. Create agent
    let agent_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "Alice Settlement Agent"
            }),
        ))
        .await?;

    // Extract agent DID from response
    let agent_result_value = agent_response.result.unwrap();
    let agent_content = agent_result_value["content"][0]["text"].as_str().unwrap();
    let agent_result: Value = serde_json::from_str(agent_content)?;
    let alice_agent_id = agent_result["@id"].as_str().unwrap();

    // 2. Create transfer
    let transfer_response = server.handle_request_direct(client.create_call_tool_request(
        "tap_create_transfer",
        json!({
            "agent_did": alice_agent_id,
            "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
            "amount": "250.00",
            "originator": {"@id": "did:example:alice"},
            "beneficiary": {"@id": "did:example:bob"},
            "agents": [{"@id": alice_agent_id, "role": "SettlementAddress", "for": "did:example:alice"}]
        })
    )).await?;

    // Extract transaction ID
    let transfer_result_value = transfer_response.result.unwrap();
    let is_error = transfer_result_value["isError"].as_bool().unwrap_or(false);
    let transfer_content = transfer_result_value["content"][0]["text"]
        .as_str()
        .unwrap();

    if is_error {
        // Network delivery may fail in test environments
        assert!(
            transfer_content.contains("Failed to send transfer message")
                || transfer_content.contains("Failed to deliver"),
            "Unexpected error: {}",
            transfer_content
        );
        return Ok(());
    }

    let transfer_result: Value = serde_json::from_str(transfer_content)?;
    let transaction_id = transfer_result["transaction_id"].as_str().unwrap();

    // 3. Authorize transaction
    let auth_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_authorize",
            json!({
                "agent_did": alice_agent_id,
                "transaction_id": transaction_id,
                "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87"
            }),
        ))
        .await?;

    let auth_result_value = auth_response.result.unwrap();
    let is_error = auth_result_value["isError"].as_bool().unwrap_or(false);
    let auth_content = auth_result_value["content"][0]["text"].as_str().unwrap();

    if is_error {
        // Expected for now - authorize messages don't have recipient info
        assert_eq!(auth_content, "No recipient found for authorize message");
    } else {
        let auth_result: Value = serde_json::from_str(auth_content)?;
        assert_eq!(auth_result["status"], "sent");
    }

    // 4. Settle transaction
    let settle_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_settle",
            json!({
                "agent_did": alice_agent_id,
                "transaction_id": transaction_id,
                "settlement_id": "eip155:1:tx/0xabcd1234567890abcdef1234567890abcdef1234",
                "amount": "250.00"
            }),
        ))
        .await?;

    let settle_result_value = settle_response.result.unwrap();
    let is_error = settle_result_value["isError"].as_bool().unwrap_or(false);
    let settle_content = settle_result_value["content"][0]["text"].as_str().unwrap();

    if is_error {
        // Expected for now - settle messages don't have recipient info
        assert_eq!(settle_content, "No recipient found for settle message");
    } else {
        let settle_result: Value = serde_json::from_str(settle_content)?;
        assert_eq!(settle_result["status"], "sent");
    }

    // 5. List transactions to verify
    let list_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_list_transactions",
            json!({
                "agent_did": alice_agent_id
            }),
        ))
        .await?;

    let list_result_value = list_response.result.unwrap();
    let list_content = list_result_value["content"][0]["text"].as_str().unwrap();
    let list_result: Value = serde_json::from_str(list_content)?;
    assert!(list_result["total"].as_u64().unwrap() >= 1); // Only transfer message succeeds

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_error_handling() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Test invalid tool call
    let invalid_request = client.create_call_tool_request("tap_invalid_tool", json!({}));

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
        "tap_create_agent",
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

#[tokio::test]
#[serial]
async fn test_message_listing_shows_both_directions() -> Result<()> {
    let env = TestEnvironment::new()?;
    let mut server = env.create_server().await?;
    let mut client = McpTestClient::new();

    // Initialize
    server
        .handle_request_direct(client.create_initialize_request())
        .await?;

    // Create an agent
    let create_agent_response = server
        .handle_request_direct(client.create_call_tool_request(
            "tap_create_agent",
            json!({
                "label": "TestAgent"
            }),
        ))
        .await?;

    let create_agent_result = create_agent_response.result.unwrap();
    let agent_content = create_agent_result["content"][0]["text"].as_str().unwrap();
    let agent_result: Value = serde_json::from_str(agent_content)?;
    let agent_did = agent_result["@id"].as_str().unwrap();

    // Read messages resource for this agent (should show both incoming and outgoing by default)
    let resource_uri = format!("tap://messages?agent_did={}", agent_did);
    let list_resources_response = server
        .handle_request_direct(client.create_list_resources_request())
        .await?;

    // Check that messages resource is listed
    let resources_result = list_resources_response.result.unwrap();
    let resources = resources_result["resources"].as_array().unwrap();
    let messages_resource = resources
        .iter()
        .find(|r| r["uri"] == "tap://messages")
        .unwrap();

    // Verify the description includes agent_did parameter
    let description = messages_resource["description"].as_str().unwrap();
    assert!(description.contains("agent_did"));
    assert!(description.contains("direction"));

    // Test reading the messages resource directly (this tests our implementation)
    // Note: In a real test, we'd send messages first, but this tests the query parameter handling
    let read_resource_request = client.create_read_resource_request(&resource_uri);
    let read_response = server.handle_request_direct(read_resource_request).await?;

    assert_matches!(
        read_response,
        JsonRpcResponse {
            result: Some(_),
            error: None,
            ..
        }
    );

    if let Some(result) = read_response.result {
        let content = result["contents"][0]["text"].as_str().unwrap();
        let json_content: Value = serde_json::from_str(content)?;

        // Should include applied_filters with agent_did and direction should be null (meaning both)
        let applied_filters = &json_content["applied_filters"];
        assert_eq!(applied_filters["agent_did"], agent_did);
        assert_eq!(applied_filters["direction"], serde_json::Value::Null);

        // Should have a messages array (even if empty)
        assert!(json_content["messages"].is_array());
    }

    // Test with explicit direction filter
    let resource_uri_incoming =
        format!("tap://messages?agent_did={}&direction=incoming", agent_did);
    let read_incoming_request = client.create_read_resource_request(&resource_uri_incoming);
    let read_incoming_response = server.handle_request_direct(read_incoming_request).await?;

    if let Some(result) = read_incoming_response.result {
        let content = result["contents"][0]["text"].as_str().unwrap();
        let json_content: Value = serde_json::from_str(content)?;
        let applied_filters = &json_content["applied_filters"];
        assert_eq!(applied_filters["direction"], "incoming");
    }

    Ok(())
}
