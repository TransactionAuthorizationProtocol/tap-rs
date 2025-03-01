extern crate tap_core;

use tap_core::message::types::{TapMessage, TapMessageType, TransactionProposalBody};
use std::str::FromStr;
use caip::{ChainId, AccountId, AssetId};

#[test]
fn test_create_message() {
    // Create a basic TAP message
    let body = TransactionProposalBody {
        transaction_id: "tx123".to_string(),
        network: ChainId::from_str("eip155:1").unwrap(),
        sender: AccountId::from_str("eip155:1:0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        recipient: AccountId::from_str("eip155:1:0xabcdef1234567890abcdef1234567890abcdef12").unwrap(),
        asset: AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap(),
        amount: "100.00".to_string(),
        memo: Some("Test transaction".to_string()),
        tx_reference: None,
        metadata: Default::default(),
    };

    let json_body = serde_json::to_value(&body).unwrap();

    let message = TapMessage {
        message_type: TapMessageType::TransactionProposal,
        id: "msg123".to_string(),
        version: "1.0".to_string(),
        created_time: "2021-01-01T00:00:00Z".to_string(),
        expires_time: Some("2021-01-02T00:00:00Z".to_string()),
        body: Some(json_body),
        attachments: None,
        metadata: Default::default(),
        from_did: None,
        to_did: None,
    };

    // Convert the message to JSON and back
    let json = serde_json::to_string(&message).unwrap();
    let _deserialized: TapMessage = serde_json::from_str(&json).unwrap();
}
