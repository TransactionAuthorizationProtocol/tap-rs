use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::time::Duration;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tokio::time::sleep;

// Helper function to create a mock TapNode for testing
fn create_mock_node() -> TapNode {
    let mut node_config = NodeConfig::default();
    node_config.storage_path = None; // Disable storage for tests
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

    // Create a DIDComm test message (intentionally invalid to test error handling)
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
        .post(format!("http://127.0.0.1:{}/didcomm", port))
        .header("Content-Type", "application/didcomm-encrypted+json")
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
    // Our validation now returns a 400 Bad Request error for invalid message types
    assert_eq!(status, 400);

    // Read the response body to ensure it has the right error message
    let body = response.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();

    // Verify this is an error response
    assert_eq!(json["status"], "error");
    // Verify the error type is related to validation
    assert!(json["error"]["type"]
        .as_str()
        .unwrap()
        .contains("validation"));

    // Stop the server
    server.stop().await.expect("Server should stop");
}
