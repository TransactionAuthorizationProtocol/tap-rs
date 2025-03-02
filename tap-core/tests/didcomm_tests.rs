use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_core::didcomm::pack_tap_message;
use tap_core::error::Result;
use tap_core::message::types::{Agent, TapMessage, TapMessageType, TransferBody};

#[tokio::test]
async fn test_pack_tap_message() -> Result<()> {
    // Create a valid transfer message
    let asset =
        AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();

    let originator = Agent {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Agent {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
    };

    let body = TransferBody {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount_subunits: "100000000".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        memo: Some("Test transaction".to_string()),
        metadata: HashMap::new(),
    };

    let message = TapMessage::new(TapMessageType::Transfer)
        .with_id("msg-id-1")
        .with_body(&body);

    // Test packing the message
    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP";
    let to_dids = [to_did];

    let packed_msg = pack_tap_message(&message, Some(from_did), &to_dids).await?;

    // Ensure the packed message is a valid JSON string
    let result = serde_json::from_str::<serde_json::Value>(&packed_msg)
        .map_err(|e| tap_core::error::Error::SerializationError(e.to_string()))?;
    assert!(result.is_object());

    // Check that key fields are present in the packed message
    assert!(result["id"].is_string());
    assert!(result["type"].is_string());
    assert!(result["body"].is_object());

    Ok(())
}

// TODO: Add more comprehensive tests for:
// - Unpacking messages
// - Handling different message types
// - Error cases
