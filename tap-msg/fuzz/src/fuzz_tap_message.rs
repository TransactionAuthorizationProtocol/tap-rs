#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use tap_msg::message::{TapMessage, TapMessageType, Validate};
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::{ChainId, AccountId, AssetId};

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
    // CAIP-related fields for transaction proposals
    chain_id_str: String,
    sender_address: String,
    recipient_address: String,
    asset_id_str: String,
    amount: String,
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
        from_did: None,
        to_did: None,
    };

    // Try to add a body if required
    if data.has_body {
        // For TransactionProposal, try to create a valid CAIP-formatted body
        if let TapMessageType::TransactionProposal = message.message_type {
            // Attempt to create valid CAIP identifiers
            let chain_id = ChainId::from_str(&format!("ethereum:{}", data.chain_id_str.chars().take(5).collect::<String>()));
            let sender = AccountId::from_str(&format!("ethereum:{}:0x{}", 
                data.chain_id_str.chars().take(5).collect::<String>(), 
                data.sender_address.chars().take(40).collect::<String>()));
            let recipient = AccountId::from_str(&format!("ethereum:{}:0x{}", 
                data.chain_id_str.chars().take(5).collect::<String>(), 
                data.recipient_address.chars().take(40).collect::<String>()));
            let asset = AssetId::from_str(&format!("ethereum:{}/erc20:0x{}", 
                data.chain_id_str.chars().take(5).collect::<String>(),
                data.asset_id_str.chars().take(40).collect::<String>()));
            
            // If all identifiers are valid, create a transaction proposal body
            if let (Ok(chain_id), Ok(sender), Ok(recipient), Ok(asset)) = (chain_id, sender, recipient, asset) {
                let tx_proposal = serde_json::json!({
                    "transaction_id": data.id,
                    "network": chain_id.to_string(),
                    "sender": sender.to_string(),
                    "recipient": recipient.to_string(),
                    "asset": asset.to_string(),
                    "amount": data.amount,
                    "memo": "Fuzz test transaction"
                });
                message.body = Some(tx_proposal);
            } else {
                // Fallback to the general approach if CAIP identifiers are invalid
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&data.body_json) {
                    message.body = Some(json_value);
                }
            }
        } else {
            // For other message types, use the general approach
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&data.body_json) {
                message.body = Some(json_value);
            }
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
    
    // If it's a transaction proposal, test CAIP validation specifically
    if let TapMessageType::TransactionProposal = message.message_type {
        if let Some(body) = &message.body {
            // This shouldn't panic, but may fail depending on the generated data
            if let Ok(serde_json::Value::String(network)) = body.get("network").map(|v| Ok(v.clone())).unwrap_or(Ok(serde_json::Value::Null)) {
                if let Ok(serde_json::Value::String(sender)) = body.get("sender").map(|v| Ok(v.clone())).unwrap_or(Ok(serde_json::Value::Null)) {
                    if let Ok(serde_json::Value::String(recipient)) = body.get("recipient").map(|v| Ok(v.clone())).unwrap_or(Ok(serde_json::Value::Null)) {
                        if let Ok(serde_json::Value::String(asset)) = body.get("asset").map(|v| Ok(v.clone())).unwrap_or(Ok(serde_json::Value::Null)) {
                            // Try to parse and validate CAIP identifiers
                            let _ = ChainId::from_str(&network);
                            let _ = AccountId::from_str(&sender);
                            let _ = AccountId::from_str(&recipient);
                            let _ = AssetId::from_str(&asset);
                        }
                    }
                }
            }
        }
    }
});
