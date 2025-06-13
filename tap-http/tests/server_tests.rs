use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tokio::time::sleep;

// Helper function to create a mock TapNode for testing
fn create_mock_node() -> TapNode {
    let node_config = NodeConfig {
        storage_path: None, // Disable storage for tests
        ..Default::default()
    };
    TapNode::new(node_config)
}

// Helper function to find an unused port
fn find_unused_port() -> Option<u16> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    match TcpListener::bind(addr) {
        Ok(listener) => listener.local_addr().map(|addr| addr.port()).ok(),
        Err(_) => None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_startup() {
    // Create a mock TapNode and find an available port
    let node = create_mock_node();
    let port = find_unused_port().expect("Unable to find unused port");

    // Configure server with the available port
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create and start HTTP server
    let mut server = TapHttpServer::new(config, node);
    server.start().await.expect("Server should start");

    // Wait a moment for server to fully start
    sleep(Duration::from_millis(100)).await;

    // Stop the server
    server.stop().await.expect("Server should stop");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_health_endpoint() {
    // Create a mock TapNode and find an available port
    let node = create_mock_node();
    let port = find_unused_port().expect("Unable to find unused port");

    // Configure server with the available port
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Start the server
    server.start().await.expect("Server should start");

    // Wait for server to fully start
    sleep(Duration::from_millis(500)).await;

    // Make request to health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // Ensure we got a response
    assert!(
        response.is_ok(),
        "Failed to connect to health endpoint: {:?}",
        response.err()
    );

    let response = response.unwrap();
    assert_eq!(response.status(), 200);

    // Parse response body
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    // Verify response contents
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());

    // Stop the server
    server.stop().await.expect("Server should stop");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_didcomm_endpoint() {
    // Create a mock TapNode and find an available port
    let node = create_mock_node();
    let port = find_unused_port().expect("Unable to find unused port");

    // Configure server with the available port
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Start the server
    server.start().await.expect("Server should start");

    // Wait for server to fully start
    sleep(Duration::from_millis(500)).await;

    // Create a plain DIDComm test message (should be rejected for security reasons)
    let didcomm_msg = json!({
        "id": "1234567890",
        "typ": "application/didcomm-plain+json",
        "type": "https://didcomm.org/basicmessage/2.0/message",
        "body": {
            "messageType": "TAP_AUTHORIZATION_REQUEST",
            "version": "1.0",
            "ledgerId": "eip155:1",
            "authorizationRequest": {
                "transactionHash": "0x123456789abcdef",
                "sender": "eip155:1:0x1234567890123456789012345678901234567890",
                "receiver": "eip155:1:0x0987654321098765432109876543210987654321",
                "amount": "1000000000000000000"
            }
        },
        "from": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "to": ["did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp"]
    });

    // Make request to DIDComm endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .header("Content-Type", "application/didcomm-plain+json")
        .json(&didcomm_msg)
        .timeout(Duration::from_secs(5))
        .send()
        .await;

    // Ensure we got a response
    assert!(
        response.is_ok(),
        "Failed to connect to didcomm endpoint: {:?}",
        response.err()
    );

    let response = response.unwrap();
    let status = response.status();
    // Plain messages should be rejected with a 400 Bad Request for security reasons
    assert_eq!(status, 400);

    // Read the response body to ensure it has the right error message
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    // Verify this is an error response
    assert_eq!(json["status"], "error");
    // The error should be a validation error since plain messages are not allowed
    assert_eq!(json["error"]["type"], "validation_error");

    // Stop the server
    server.stop().await.expect("Server should stop");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_didcomm_endpoint_content_types() {
    // Create a mock TapNode and find an available port
    let node = create_mock_node();
    let port = find_unused_port().expect("Unable to find unused port");

    // Configure server with the available port
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Start the server
    server.start().await.expect("Server should start");

    // Wait for server to fully start
    sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();

    // Test 1: Encrypted message content type (should fail with no agent, but pass validation)
    let encrypted_msg = json!({
        "protected": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIn0=",
        "recipients": [{
            "header": {"kid": "did:key:test"},
            "encrypted_key": "test-key"
        }],
        "ciphertext": "test-ciphertext",
        "tag": "test-tag",
        "iv": "test-iv"
    });

    let response = client
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .header("Content-Type", "application/didcomm-encrypted+json")
        .json(&encrypted_msg)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();

    // Should get 500 (no agent to process), not 400 (validation error)
    let status = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(status, 500);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "error");
    // The message should indicate that no agent could process it
    let message = json["message"].as_str().unwrap_or("");
    assert!(
        message.contains("No agent could process"),
        "Expected 'No agent could process' but got: {}",
        message
    );

    // Test 2: Signed message content type (should fail with no agent, but pass validation)
    let signed_msg = json!({
        "payload": "eyJ0ZXN0IjoidGVzdCJ9",
        "signatures": [{
            "protected": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLXNpZ25lZCtqc29uIn0=",
            "signature": "test-signature"
        }]
    });

    let response = client
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .header(
            "Content-Type",
            "application/didcomm-signed+json; charset=utf-8",
        )
        .json(&signed_msg)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();

    // Should get 500 (no agent to process), not 400 (validation error)
    let status = response.status();
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(status, 500);
    assert_eq!(json["status"], "error");
    // The message should indicate that no agent could process it or verification failed
    let message = json["message"].as_str().unwrap_or("");
    assert!(
        message.contains("No agent could process") || message.contains("Verification error"),
        "Expected processing/verification error but got: {}",
        message
    );

    // Test 3: Invalid content type (should be rejected)
    let response = client
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .header("Content-Type", "application/json")
        .json(&json!({"test": "data"}))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "validation_error");

    // Test 4: Missing content type (should be rejected)
    let response = client
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .body(json!({"test": "data"}).to_string())
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["error"]["type"], "validation_error");

    // Stop the server
    server.stop().await.expect("Server should stop");
}
