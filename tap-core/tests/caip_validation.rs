use std::str::FromStr;
use tap_caip::{AccountId, AssetId, ChainId};
use tap_core::message::types::TransactionProposalBody;

#[test]
fn test_valid_transaction_proposal() {
    // Create a valid transaction proposal with consistent CAIP identifiers
    let proposal = TransactionProposalBody {
        transaction_id: "tx123".to_string(),
        network: ChainId::from_str("ethereum:1").unwrap(),
        sender: AccountId::from_str("ethereum:1:0x1234567890123456789012345678901234567890")
            .unwrap(),
        recipient: AccountId::from_str("ethereum:1:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .unwrap(),
        asset: AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: Default::default(),
    };

    // Validation should pass
    assert!(proposal.validate_caip_consistency().is_ok());
}

#[test]
fn test_invalid_sender_network() {
    // Create a transaction proposal with sender on a different network
    let proposal = TransactionProposalBody {
        transaction_id: "tx123".to_string(),
        network: ChainId::from_str("ethereum:1").unwrap(),
        // Sender is on polygon, not ethereum:1
        sender: AccountId::from_str("polygon:137:0x1234567890123456789012345678901234567890")
            .unwrap(),
        recipient: AccountId::from_str("ethereum:1:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .unwrap(),
        asset: AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: Default::default(),
    };

    // Validation should fail
    assert!(proposal.validate_caip_consistency().is_err());
}

#[test]
fn test_invalid_recipient_network() {
    // Create a transaction proposal with recipient on a different network
    let proposal = TransactionProposalBody {
        transaction_id: "tx123".to_string(),
        network: ChainId::from_str("ethereum:1").unwrap(),
        sender: AccountId::from_str("ethereum:1:0x1234567890123456789012345678901234567890")
            .unwrap(),
        // Recipient is on polygon, not ethereum:1
        recipient: AccountId::from_str("polygon:137:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .unwrap(),
        asset: AssetId::from_str("ethereum:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: Default::default(),
    };

    // Validation should fail
    assert!(proposal.validate_caip_consistency().is_err());
}

#[test]
fn test_invalid_asset_network() {
    // Create a transaction proposal with asset on a different network
    let proposal = TransactionProposalBody {
        transaction_id: "tx123".to_string(),
        network: ChainId::from_str("ethereum:1").unwrap(),
        sender: AccountId::from_str("ethereum:1:0x1234567890123456789012345678901234567890")
            .unwrap(),
        recipient: AccountId::from_str("ethereum:1:0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .unwrap(),
        // Asset is on polygon, not ethereum:1
        asset: AssetId::from_str("polygon:137/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .unwrap(),
        amount: "100.0".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: Default::default(),
    };

    // Validation should fail
    assert!(proposal.validate_caip_consistency().is_err());
}
