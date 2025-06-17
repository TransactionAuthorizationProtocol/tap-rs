use std::collections::HashMap;
use std::result::Result;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::{Agent, Party, TapMessageBody, Transfer};

/// Test the round-trip conversion between TAP messages and DIDComm messages.
///
/// This test verifies that a TAP message body can be:
/// 1. Packed into a DIDComm message
/// 2. Unpacked back into a TAP message body
/// 3. The original and unpacked messages should be identical
#[tokio::test]
async fn test_tap_didcomm_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Create a valid transfer message
    let asset = AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7")?;

    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP";

    let originator = Party::new(from_did);

    let beneficiary = Party::new(to_did);

    let body = Transfer {
        transaction_id: Some(uuid::Uuid::new_v4().to_string()),
        asset: asset.clone(),
        originator: Some(originator.clone()),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![
            Agent::new(from_did, "originator", from_did),
            Agent::new(to_did, "beneficiary", to_did),
        ],
        settlement_id: None,
        connection_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Pack the message using the direct conversion method
    let to_dids = [to_did];
    let didcomm_message = body.to_didcomm_with_route(from_did, to_dids.iter().copied())?;

    // Serialize to JSON string
    let packed_message = serde_json::to_string(&didcomm_message)?;

    // Deserialize back to a PlainMessage
    let unpacked_message: PlainMessage = serde_json::from_str(&packed_message)?;

    // Extract the message metadata
    assert_eq!(unpacked_message.from, from_did.to_string());
    assert_eq!(unpacked_message.to, vec![to_did.to_string()]);
    assert_eq!(unpacked_message.type_, Transfer::message_type());

    // Extract the message body
    let unpacked_body = Transfer::from_didcomm(&unpacked_message)?;

    // Verify the body matches the original
    assert_eq!(unpacked_body.asset.to_string(), asset.to_string());
    assert_eq!(unpacked_body.originator.unwrap().id, from_did);
    assert_eq!(unpacked_body.beneficiary.as_ref().unwrap().id, to_did);
    assert_eq!(unpacked_body.amount, "100.00");

    Ok(())
}
