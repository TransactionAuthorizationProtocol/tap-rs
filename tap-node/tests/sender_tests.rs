use tap_node::error::Result;
use tap_node::message::sender::{HttpPlainMessageSender, PlainMessageSender};

#[cfg(feature = "reqwest")]
#[tokio::test]
async fn test_http_message_sender_native() -> Result<()> {
    // Create a test message and recipient
    let message = "test message".to_string();
    let recipient = vec!["did:example:123".to_string()];

    // Create an HTTP message sender with a test URL
    let sender = HttpPlainMessageSender::new("http://localhost:8080".to_string());

    // For testing, we expect this to return an error since there's no real server
    let result = sender.send(message, recipient).await;
    assert!(
        result.is_err(),
        "Expected error when no server is available"
    );

    // The specific error should mention connection
    if let Err(e) = result {
        let err_string = format!("{:?}", e);
        assert!(
            err_string.contains("Request error")
                || err_string.contains("Failed to send message to some recipients"),
            "Unexpected error: {}",
            err_string
        );
    }

    Ok(())
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_http_message_sender_wasm() -> Result<()> {
    // Create a test message and recipient
    let message = "test message".to_string();
    let recipient = vec!["did:example:123".to_string()];

    // Create an HTTP message sender with a test URL
    let sender = HttpPlainMessageSender::new("http://localhost:8080".to_string());

    // For testing, we expect this to return an error since there's no real server
    let result = sender.send(message, recipient).await;
    assert!(
        result.is_err(),
        "Expected error when no server is available"
    );

    // The specific error should mention connection
    if let Err(e) = result {
        let err_string = format!("{:?}", e);
        assert!(
            err_string.contains("Fetch error")
                || err_string.contains("Failed to send message to some recipients"),
            "Unexpected error: {}",
            err_string
        );
    }

    Ok(())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "reqwest")))]
#[tokio::test]
async fn test_http_message_sender_fallback() -> Result<()> {
    // Create a test message and recipient
    let message = "test message".to_string();
    let recipient = vec!["did:example:123".to_string()];

    // Create an HTTP message sender with a test URL
    let sender = HttpPlainMessageSender::new("http://localhost:8080".to_string());

    // For testing purposes, the fallback implementation should succeed
    let result = sender.send(message, recipient).await;
    assert!(
        result.is_ok(),
        "Expected success with fallback implementation"
    );

    Ok(())
}

#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
#[wasm_bindgen_test::wasm_bindgen_test]
async fn test_http_message_sender_wasm_fallback() -> Result<()> {
    // Create a test message and recipient
    let message = "test message".to_string();
    let recipient = vec!["did:example:123".to_string()];

    // Create an HTTP message sender with a test URL
    let sender = HttpPlainMessageSender::new("http://localhost:8080".to_string());

    // For testing purposes, the fallback implementation should succeed
    let result = sender.send(message, recipient).await;
    assert!(
        result.is_ok(),
        "Expected success with fallback implementation"
    );

    Ok(())
}
