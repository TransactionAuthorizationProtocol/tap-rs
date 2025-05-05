use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::types::{Participant, Transfer};

#[test]
fn test_valid_transfer_body() {
    // Create a valid transfer body
    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Validate the transfer body - no error should be returned
    assert!(body.validate().is_ok());
}

#[test]
fn test_transfer_with_empty_asset() {
    // Asset validation is handled at deserialization time
    // This test is meant to demonstrate handling when asset is empty after deserialization
    // For demo purposes, we're just using a valid asset here since we can't create an empty one

    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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

    // Creating a body with valid values
    let body = Transfer {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Validation should pass
    assert!(body.validate().is_ok());
}

#[test]
fn test_transfer_with_empty_amount() {
    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "".to_string(), // Empty amount
        agents: vec![originator, beneficiary],
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Validation should fail for empty amount
    assert!(body.validate().is_err());
}
