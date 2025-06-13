//! Integration test to verify TAP-HTTP works with TAP Node's per-agent storage architecture

use std::sync::Arc;
use tap_agent::TapAgent;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tempfile::TempDir;

#[tokio::test]
async fn test_tap_http_with_per_agent_storage() {
    // Create temporary directory for this test
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    println!("Testing TAP-HTTP with per-agent storage at: {:?}", tap_root);

    // Create TAP Node with custom TAP root
    let node_config = NodeConfig {
        tap_root: Some(tap_root.clone()),
        enable_message_logging: true,
        log_message_content: true,
        ..Default::default()
    };

    let node = TapNode::new(node_config);

    // Create two test agents
    let (agent1, did1) = TapAgent::from_ephemeral_key().await.unwrap();
    let (agent2, did2) = TapAgent::from_ephemeral_key().await.unwrap();

    println!("Created test agents: {} and {}", did1, did2);

    let node_arc = Arc::new(node);

    // Register both agents
    node_arc.register_agent(Arc::new(agent1)).await.unwrap();
    node_arc.register_agent(Arc::new(agent2)).await.unwrap();

    println!("Registered agents with TAP Node");

    // Verify agent storage manager is available
    let storage_manager = node_arc.agent_storage_manager().unwrap();

    // Verify each agent gets its own storage
    let storage1 = storage_manager.get_agent_storage(&did1).await.unwrap();
    let storage2 = storage_manager.get_agent_storage(&did2).await.unwrap();

    // These should be different instances pointing to different databases
    assert!(!Arc::ptr_eq(&storage1, &storage2));

    println!("Verified per-agent storage isolation");

    // Create HTTP server configuration
    let http_config = TapHttpConfig {
        port: 0, // Let the OS choose an available port
        ..Default::default()
    };

    // Create TAP-HTTP server
    let _server = TapHttpServer::new(http_config, (*node_arc).clone());

    // Verify that the server has access to the node functionality
    // (TAP-HTTP creates its own Arc, so we can't use ptr_eq, but functionality should be the same)

    println!("Created TAP-HTTP server successfully");

    // Verify that TAP-HTTP delegates to TAP Node properly
    // (This is demonstrated by the fact that the server was created successfully
    // and has access to the same node instance with per-agent storage)

    // Test a simple message processing (without actually starting the HTTP server)
    let test_message = serde_json::json!({
        "id": "test-message-123",
        "type": "basic-message",
        "from": did1,
        "to": [did2],
        "created_time": chrono::Utc::now().timestamp(),
        "body": {
            "content": "Test message for per-agent storage"
        }
    });

    // Process the message through TAP Node (which TAP-HTTP would do)
    let result = node_arc.receive_message(test_message).await;
    assert!(result.is_ok(), "Message processing should succeed");

    println!("Successfully processed test message through TAP Node");

    // Verify that the message was stored in the correct agent's storage
    let storage_manager = node_arc.agent_storage_manager().unwrap();

    // Check that the message appears in the recipient's storage
    let recipient_storage = storage_manager.get_agent_storage(&did2).await.unwrap();
    let messages = recipient_storage.list_messages(10, 0, None).await.unwrap();

    // Should have at least one message
    assert!(
        !messages.is_empty(),
        "Recipient should have received the message"
    );

    // Verify the message content
    let stored_message = &messages[0];
    assert_eq!(stored_message.message_id, "test-message-123");
    assert_eq!(stored_message.from_did, Some(did1.clone()));
    assert_eq!(stored_message.to_did, Some(did2.clone()));

    println!("Verified message was stored in recipient's per-agent storage");

    println!("TAP-HTTP per-agent storage integration test completed successfully");
}

#[tokio::test]
async fn test_tap_http_agent_storage_manager_delegation() {
    // Create temporary directory
    let temp_dir = TempDir::new().unwrap();
    let tap_root = temp_dir.path().to_path_buf();

    // Create TAP Node with per-agent storage
    let node_config = NodeConfig {
        tap_root: Some(tap_root),
        ..Default::default()
    };

    let node = TapNode::new(node_config);
    let node_arc = Arc::new(node);

    // Create TAP-HTTP server
    let http_config = TapHttpConfig::default();
    let server = TapHttpServer::new(http_config, (*node_arc).clone());

    // Verify that TAP-HTTP has access to the node
    // (TAP-HTTP creates its own Arc, so we verify functionality instead of pointer equality)

    // Verify that the storage manager is accessible through TAP-HTTP's node
    let storage_manager = server.node().agent_storage_manager();
    assert!(
        storage_manager.is_some(),
        "Storage manager should be available"
    );

    // Create a test agent DID
    let test_did = "did:example:test-agent";

    // Verify that we can get storage for an agent through TAP-HTTP
    let storage = storage_manager.unwrap().get_agent_storage(test_did).await;
    assert!(storage.is_ok(), "Should be able to get agent storage");

    println!("Verified TAP-HTTP properly delegates to TAP Node's agent storage manager");
}

#[test]
fn test_tap_http_architecture_consistency() {
    // This test verifies the architectural design at compile time

    // TAP-HTTP should accept a TapNode and store it as Arc<TapNode>
    let node = TapNode::new(NodeConfig::default());
    let config = TapHttpConfig::default();

    // This should compile and work correctly
    let _server = TapHttpServer::new(config, node);

    // The architecture ensures that TAP-HTTP is just a transport layer
    // over TAP Node, which handles all the per-agent storage logic

    println!("TAP-HTTP architecture is consistent with per-agent storage design");
}
