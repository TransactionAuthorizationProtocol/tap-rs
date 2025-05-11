#![cfg(target_arch = "wasm32")]

use tap_wasm::{Message, MessageType};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_message_creation() {
    let message_id = "msg_test_123";
    let message = Message::new(message_id.to_string(), "Transfer".to_string(), "1.0".to_string());
    
    assert_eq!(message.id(), message_id);
    assert_eq!(message.message_type(), "Transfer");
    assert_eq!(message.version(), "1.0");
}

#[wasm_bindgen_test]
fn test_message_properties() {
    let message_id = "msg_test_456";
    let mut message = Message::new(message_id.to_string(), "Transfer".to_string(), "1.0".to_string());
    
    // Set properties
    message.set_from_did(Some("did:example:sender".to_string()));
    message.set_to_did(Some("did:example:recipient".to_string()));
    
    // Check properties
    assert_eq!(message.from_did(), Some("did:example:sender".to_string()));
    assert_eq!(message.to_did(), Some("did:example:recipient".to_string()));
    
    // Update properties
    message.set_message_type("Authorize".to_string());
    message.set_version("1.1".to_string());
    
    // Check updated properties
    assert_eq!(message.message_type(), "Authorize");
    assert_eq!(message.version(), "1.1");
}