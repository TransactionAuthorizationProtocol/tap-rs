use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::error::Result;
use tap_msg::message::{Participant, TapMessageBody, Transfer};

#[tokio::test]
async fn test_pack_tap_body() -> Result<()> {
    // Create a valid transfer message body
    let asset =
        AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: asset.clone(),
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Test packing the message body using the new to_didcomm_with_route method
    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkwYyuTCaaDKnMGHpMkteuFpj1KrsFgGXwW3nXdT7k3RQP";
    let to_dids = [to_did];

    let packed_msg = body.to_didcomm_with_route(Some(from_did), to_dids.iter().copied())?;

    // Verify the packed message
    assert_eq!(packed_msg.from, Some(from_did.to_string()));
    assert_eq!(packed_msg.to, Some(vec![to_did.to_string()]));
    assert_eq!(packed_msg.type_, Transfer::message_type());

    // Verify the body contains our data
    let body_value = packed_msg.body.as_object().unwrap();
    assert!(body_value.contains_key("asset"));
    assert!(body_value.contains_key("amount"));
    assert!(body_value.contains_key("originator"));

    // Now test extracting the body back using from_didcomm
    let extracted_body = Transfer::from_didcomm(&packed_msg)?;

    // Verify the extracted body matches the original
    assert_eq!(extracted_body.asset, asset);
    assert_eq!(extracted_body.amount, body.amount);
    assert_eq!(extracted_body.originator.id, body.originator.id);
    assert_eq!(
        extracted_body.beneficiary.as_ref().unwrap().id,
        body.beneficiary.as_ref().unwrap().id
    );

    Ok(())
}

#[tokio::test]
async fn test_extract_tap_body() -> Result<()> {
    // Create a valid transfer message body
    let asset =
        AssetId::from_str("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset: asset.clone(),
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "1.00".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Pack the message body
    let from_did = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
    let to_did = "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6";
    let to_dids = [to_did];

    let message = body.to_didcomm_with_route(Some(from_did), to_dids.iter().copied())?;

    // Extract the body using from_didcomm
    let extracted: Transfer = Transfer::from_didcomm(&message)?;

    // Verify extraction was successful
    assert_eq!(extracted.asset, asset);
    assert_eq!(extracted.amount, "1.00");

    Ok(())
}

// TODO: Add more comprehensive tests for:
// - Unpacking messages
// - Handling different message types
// - Error cases
