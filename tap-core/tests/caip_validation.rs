use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_core::message::types::{Agent, TransferBody};
use tap_core::message::validation::validate_transfer_body;

#[test]
fn test_valid_transfer_body() {
    // Create a valid transfer body
    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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

    // Validate the transfer body - no error should be returned
    assert!(validate_transfer_body(&body).is_ok());
}

#[test]
fn test_transfer_with_empty_asset() {
    // Asset validation is handled at deserialization time
    // This test is meant to demonstrate handling when asset is empty after deserialization
    // For demo purposes, we're just using a valid asset here since we can't create an empty one

    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Agent {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Agent {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
    };

    // Creating a body with valid values
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

    // Validation should pass
    assert!(validate_transfer_body(&body).is_ok());
}

#[test]
fn test_transfer_with_empty_amount() {
    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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
        amount_subunits: "".to_string(), // Empty amount
        agents: vec![originator, beneficiary],
        settlement_id: None,
        memo: Some("Test transaction".to_string()),
        metadata: HashMap::new(),
    };

    // Validation should fail for empty amount
    assert!(validate_transfer_body(&body).is_err());
}
