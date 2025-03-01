#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tap_core::message::{TapMessage, TapMessageType, Validate};
use std::collections::HashMap;

// Define a custom structure for arbitrary generation
#[derive(Arbitrary, Debug)]
struct FuzzTapMessage {
    message_type_index: u8,
    id: String,
    version: String,
    created_time: String,
    expires_time: Option<String>,
    has_body: bool,
    body_json: String,
    has_attachments: bool,
    metadata_keys: Vec<String>,
    metadata_values: Vec<String>,
}

// Map arbitrary data to a TapMessage
fn create_tap_message(data: FuzzTapMessage) -> TapMessage {
    // Map the arbitrary index to a TapMessageType
    let message_type = match data.message_type_index % 5 {
        0 => TapMessageType::TransactionProposal,
        1 => TapMessageType::IdentityExchange,
        2 => TapMessageType::TravelRuleInfo,
        3 => TapMessageType::AuthorizationResponse,
        _ => TapMessageType::Error,
    };

    // Start with a basic message
    let mut message = TapMessage {
        message_type,
        id: data.id,
        version: data.version,
        created_time: data.created_time,
        expires_time: data.expires_time,
        body: None,
        attachments: None,
        metadata: HashMap::new(),
    };

    // Try to add a body if required
    if data.has_body {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&data.body_json) {
            message.body = Some(json_value);
        }
    }

    // Try to add metadata
    let metadata_count = std::cmp::min(data.metadata_keys.len(), data.metadata_values.len());
    for i in 0..metadata_count {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&data.metadata_values[i]) {
            message.metadata.insert(data.metadata_keys[i].clone(), json_value);
        }
    }

    message
}

// The actual fuzz target
fuzz_target!(|data: FuzzTapMessage| {
    // Create a TAP message from fuzz data
    let message = create_tap_message(data);
    
    // Try to validate the message (this is what we're testing)
    let _ = message.validate();
    
    // Try to serialize and deserialize the message
    if let Ok(serialized) = serde_json::to_string(&message) {
        let _ = serde_json::from_str::<TapMessage>(&serialized);
    }
});
