//! Tests for WebSocket message sender functionality
//!
//! This file tests the WebSocketMessageSender implementation.
//!
//! Note: Since actual WebSocket functionality requires a server, these tests
//! focus on the API correctness and fallback behavior, rather than end-to-end
//! communication testing.

#[cfg(feature = "websocket")]
mod websocket_tests {
    use tap_node::{PlainMessageSender, WebSocketPlainMessageSender};

    #[tokio::test]
    async fn test_websocket_message_sender_url_construction() {
        // Test that the WebSocket URL is constructed correctly
        let sender = WebSocketPlainMessageSender::new("https://example.com".to_string());

        // The get_endpoint_url method is private, but we can indirectly test it
        // by checking the debug output which should contain the URL
        let sender_debug = format!("{:?}", sender);
        assert!(sender_debug.contains("WebSocketPlainMessageSender"));

        // Since we're not actually connecting, we can test the send function
        // which will fail due to connection issues, but should attempt to connect
        // to the correct URL
        let recipient = "did:example:123".to_string();
        let message = "test message".to_string();

        let result = sender.send(message, vec![recipient.clone()]).await;

        // The send should fail since there's no real server
        assert!(result.is_err());

        // Check that the error message contains information about the connection attempt
        let err = result.unwrap_err();
        let err_string = format!("{:?}", err);

        // The error should mention either WebSocket, connection failure, or the endpoint
        assert!(
            err_string.contains("WebSocket")
                || err_string.contains("connection")
                || err_string.contains("ws://")
                || err_string.contains("wss://"),
            "Error doesn't contain expected WebSocket information: {}",
            err_string
        );
    }

    #[tokio::test]
    async fn test_websocket_message_sender_options() {
        // Test custom options
        let sender = WebSocketPlainMessageSender::with_options("https://example.com".to_string());

        let sender_debug = format!("{:?}", sender);
        assert!(sender_debug.contains("base_url"));
    }
}

#[cfg(not(feature = "websocket"))]
mod fallback_tests {
    use tap_node::{PlainMessageSender, WebSocketPlainMessageSender};

    #[tokio::test]
    async fn test_websocket_message_sender_fallback() {
        // Test that the fallback implementation works and doesn't panic
        let sender = WebSocketPlainMessageSender::new("https://example.com".to_string());

        let recipient = "did:example:123".to_string();
        let message = "test message".to_string();

        let result = sender.send(message, vec![recipient]).await;

        // The fallback implementation should succeed, even though no real
        // connection is established
        assert!(result.is_ok());
    }
}
