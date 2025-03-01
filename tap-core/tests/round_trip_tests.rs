use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::{AccountId, AssetId, ChainId};
use tap_core::didcomm::{pack_tap_message, unpack_didcomm_message};
use tap_core::error::Result;
use tap_core::message::{TapMessage, TapMessageType, TransactionProposalBody};

/// Test the round-trip conversion between TAP messages and DIDComm messages.
///
/// This test verifies that a TAP message can be:
/// 1. Packed into a DIDComm message
/// 2. Unpacked back into a TAP message
/// 3. The original and unpacked messages should be identical
#[tokio::test]
async fn test_tap_didcomm_round_trip() -> Result<()> {
    // Create a valid transaction proposal
    let transaction_id = "123e4567-e89b-12d3-a456-426614174000";
    let body = TransactionProposalBody {
        transaction_id: transaction_id.to_string(),
        network: ChainId::from_str("eip155:1").unwrap(),
        sender: AccountId::from_str("eip155:1:0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        recipient: AccountId::from_str("eip155:1:0xabcdef1234567890abcdef1234567890abcdef12")
            .unwrap(),
        asset: AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: HashMap::new(),
    };

    // Create the original TAP message
    let original_message = TapMessage::new(TapMessageType::TransactionProposal)
        .with_id("msg-id-1")
        .with_body(&body);

    // DIDComm parameters
    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP";
    let to_dids = [to_did];

    // Pack the TAP message into a DIDComm message
    let packed_message = pack_tap_message(&original_message, Some(from_did), &to_dids).await?;

    // Verify the packed message is not empty
    assert!(!packed_message.is_empty());

    // Unpack the DIDComm message back into a TAP message
    let (unpacked_message, _metadata) = unpack_didcomm_message(&packed_message).await?;

    // Verify the message type is preserved
    assert_eq!(
        unpacked_message.message_type.to_string(),
        original_message.message_type.to_string()
    );

    // Verify the message ID is preserved
    assert_eq!(unpacked_message.id, original_message.id);

    // Verify the transaction ID in the body is preserved
    let original_body: TransactionProposalBody = original_message.body_as()?;
    let unpacked_body: TransactionProposalBody = unpacked_message.body_as()?;
    assert_eq!(unpacked_body.transaction_id, original_body.transaction_id);

    Ok(())
}

#[test]
fn test_serde_round_trip() {
    // Create a test TAP message
    let transaction_id = "123e4567-e89b-12d3-a456-426614174000";
    let body = TransactionProposalBody {
        transaction_id: transaction_id.to_string(),
        network: ChainId::from_str("eip155:1").unwrap(),
        sender: AccountId::from_str("eip155:1:0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        recipient: AccountId::from_str("eip155:1:0xabcdef1234567890abcdef1234567890abcdef12")
            .unwrap(),
        asset: AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: HashMap::new(),
    };

    let json_body = serde_json::to_value(&body).unwrap();

    let original = TapMessage {
        message_type: TapMessageType::TransactionProposal,
        id: "msg-id-1".to_string(),
        version: "1.0".to_string(),
        created_time: "2023-01-01T00:00:00Z".to_string(),
        expires_time: Some("2023-01-02T00:00:00Z".to_string()),
        body: Some(json_body),
        attachments: None,
        metadata: HashMap::new(),
        from_did: Some("did:example:123".to_string()),
        to_did: Some("did:example:456".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize back to a TapMessage
    let deserialized: TapMessage = serde_json::from_str(&json).unwrap();

    // Verify message was correctly reconstructed
    assert_eq!(deserialized.id, original.id);
    assert_eq!(deserialized.message_type, original.message_type);
    assert_eq!(deserialized.version, original.version);
    assert_eq!(deserialized.created_time, original.created_time);
    assert_eq!(deserialized.expires_time, original.expires_time);
    assert_eq!(deserialized.from_did, original.from_did);
    assert_eq!(deserialized.to_did, original.to_did);
}
