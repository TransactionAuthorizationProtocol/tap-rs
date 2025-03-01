use serde_json::json;
use std::time::Duration;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tokio::time::sleep;

#[tokio::test]
async fn test_server_startup() {
    // Configure server to use a random high port to avoid conflicts
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Use a random available port
        ..TapHttpConfig::default()
    };

    // Create TAP Node
    let node_config = NodeConfig::default();
    let node = TapNode::new(node_config);

    // Create and start HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Server should start successfully
    server.start().await.expect("Server should start");

    // Wait a moment for server to fully start
    sleep(Duration::from_millis(100)).await;

    // Stop the server
    server.stop().await.expect("Server should stop");
}

#[tokio::test]
async fn test_health_endpoint() {
    // Use a specific port for this test
    let port = 9090;
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create TAP Node
    let node_config = NodeConfig::default();
    let node = TapNode::new(node_config);

    // Create and start HTTP server
    let mut server = TapHttpServer::new(config, node);
    server.start().await.expect("Server should start");

    // Wait a moment for server to fully start
    sleep(Duration::from_millis(100)).await;

    // Make request to health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .expect("Health check request should succeed");

    // Check response
    assert_eq!(response.status(), 200);
    let body = response
        .json::<serde_json::Value>()
        .await
        .expect("Response should be valid JSON");
    assert_eq!(body["status"], "ok");

    // Stop the server
    server.stop().await.expect("Server should stop");
}

#[tokio::test]
async fn test_didcomm_endpoint() {
    // Use a specific port for this test
    let port = 9091;
    let config = TapHttpConfig {
        host: "127.0.0.1".to_string(),
        port,
        ..TapHttpConfig::default()
    };

    // Create TAP Node
    let node_config = NodeConfig::default();
    let node = TapNode::new(node_config);

    // Create and start HTTP server
    let mut server = TapHttpServer::new(config, node);
    server.start().await.expect("Server should start");

    // Wait a moment for server to fully start
    sleep(Duration::from_millis(100)).await;

    // Create a mock DIDComm message
    // This is a simplified structure of a DIDComm message - in real usage we would
    // create a proper packed message using didcomm library
    let didcomm_msg = json!({
        "id": "1234567890",
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
        .post(&format!("http://127.0.0.1:{}/didcomm", port))
        .header("Content-Type", "application/didcomm-encrypted+json")
        .json(&didcomm_msg)
        .timeout(Duration::from_secs(2))
        .send()
        .await;

    // We expect a 400 Bad Request because our test message isn't properly formatted
    // This test verifies the endpoint exists and responds, even though with an error
    assert!(response.is_ok());

    let status = response.unwrap().status();
    assert_eq!(status, 400);

    // Stop the server
    server.stop().await.expect("Server should stop");
}
