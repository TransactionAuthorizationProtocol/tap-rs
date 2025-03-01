use std::collections::HashMap;
use tap_core::didcomm::pack_tap_message;
use tap_core::error::Result;
use tap_core::message::types::{TapMessage, TapMessageType, TransactionProposalBody};
use std::str::FromStr;
use tap_caip::{ChainId, AccountId, AssetId};

#[tokio::test]
async fn test_pack_tap_message() -> Result<()> {
    // Create a valid transaction proposal
    let transaction_id = "123e4567-e89b-12d3-a456-426614174000";
    let body = TransactionProposalBody {
        transaction_id: transaction_id.to_string(),
        network: ChainId::from_str("eip155:1").unwrap(),
        sender: AccountId::from_str("eip155:1:0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        recipient: AccountId::from_str("eip155:1:0xabcdef1234567890abcdef1234567890abcdef12").unwrap(),
        asset: AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: HashMap::new(),
    };

    let message = TapMessage::new(TapMessageType::TransactionProposal)
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
