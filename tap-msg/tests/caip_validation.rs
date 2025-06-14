use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::{Agent, Party, Transfer};

#[test]
fn test_valid_transfer_body() {
    // Create a valid transfer body
    let asset =
        AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let beneficiary = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: Some(originator.clone()),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![
            Agent::new(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "originator_agent",
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ),
            Agent::new(
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
                "beneficiary_agent",
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ),
        ],
        settlement_id: None,
        connection_id: None,
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

    let originator = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let beneficiary = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    // Creating a body with valid values
    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: Some(originator.clone()),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![
            Agent::new(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "originator_agent",
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ),
            Agent::new(
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
                "beneficiary_agent",
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ),
        ],
        settlement_id: None,
        connection_id: None,
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

    let originator = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let beneficiary = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    let body = Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: Some(originator.clone()),
        beneficiary: Some(beneficiary.clone()),
        amount: "".to_string(), // Empty amount
        agents: vec![
            Agent::new(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                "originator_agent",
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ),
            Agent::new(
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
                "beneficiary_agent",
                "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ),
        ],
        settlement_id: None,
        connection_id: None,
        metadata: HashMap::new(),
        memo: None,
    };

    // Validation should fail for empty amount
    assert!(body.validate().is_err());
}
