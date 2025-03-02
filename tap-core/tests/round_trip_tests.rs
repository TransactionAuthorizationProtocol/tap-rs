use std::collections::HashMap;
use std::result::Result;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_core::didcomm::{pack_tap_message, unpack_didcomm_message};
use tap_core::message::{Agent, TapMessage, TapMessageType, TransferBody};

/// Test the round-trip conversion between TAP messages and DIDComm messages.
///
/// This test verifies that a TAP message can be:
/// 1. Packed into a DIDComm message
/// 2. Unpacked back into a TAP message
/// 3. The original and unpacked messages should be identical
#[tokio::test]
async fn test_tap_didcomm_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Create a valid transfer message
    let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP";

    let originator = Agent {
        id: from_did.to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Agent {
        id: to_did.to_string(),
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

    // Create the original TAP message
    let original_message = TapMessage::new(TapMessageType::Transfer)
        .with_id("msg-id-1")
        .with_body(&body);

    // DIDComm parameters
    let to_dids = [to_did];

    // Pack the TAP message into a DIDComm message
    let packed_message = pack_tap_message(&original_message, Some(from_did), &to_dids).await?;

    // Verify the packed message is not empty
    assert!(!packed_message.is_empty());

    // Unpack the DIDComm message back into a TAP message
    let (unpacked_message, _metadata) = unpack_didcomm_message(&packed_message).await?;

    // Verify the message type is preserved
    assert_eq!(unpacked_message.message_type, TapMessageType::Transfer);

    // Verify the message ID is preserved
    assert_eq!(unpacked_message.id, original_message.id);

    // Deserialize and verify the body of both messages
    let original_body: TransferBody = serde_json::from_value(original_message.body.unwrap())?;
    let unpacked_body: TransferBody = serde_json::from_value(unpacked_message.body.unwrap())?;

    // Verify that key fields in the body are preserved
    assert_eq!(unpacked_body.amount_subunits, original_body.amount_subunits);
    assert_eq!(unpacked_body.asset, original_body.asset);
    assert_eq!(unpacked_body.memo, original_body.memo);

    Ok(())
}

#[test]
fn test_serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Create a valid transfer message
    let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

    let originator = Agent {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Agent {
        id: "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP".to_string(),
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

    // Create a TapMessage with a message body
    let message = TapMessage {
        message_type: TapMessageType::Transfer,
        id: "msg-12345".to_string(),
        version: "1.0".to_string(),
        created_time: "2023-05-01T12:00:00Z".to_string(),
        expires_time: Some("2023-05-02T12:00:00Z".to_string()),
        body: Some(serde_json::to_value(&body)?),
        attachments: None,
        metadata: HashMap::new(),
        from_did: Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()),
        to_did: Some("did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&message)?;

    // Deserialize back to a TapMessage
    let deserialized_message: TapMessage = serde_json::from_str(&json)?;

    // Verify core message attributes are preserved
    assert_eq!(deserialized_message.message_type, TapMessageType::Transfer);
    assert_eq!(deserialized_message.id, "msg-12345");
    assert_eq!(deserialized_message.version, "1.0");

    // Verify body was correctly preserved
    assert!(deserialized_message.body.is_some());
    let deserialized_body: TransferBody =
        serde_json::from_value(deserialized_message.body.clone().unwrap())?;
    assert_eq!(deserialized_body.amount_subunits, "100000000");
    assert_eq!(deserialized_body.memo, Some("Test transaction".to_string()));

    // Assert DIDs are preserved
    assert!(deserialized_message.from_did.is_some());
    assert!(deserialized_message.to_did.is_some());

    Ok(())
}
