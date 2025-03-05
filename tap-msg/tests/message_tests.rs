extern crate tap_msg;

use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::types::{Participant, TapMessageEnvelope, TapMessageType, Transfer};

#[test]
fn test_create_message() {
    // Create a basic TAP message
    let asset =
        AssetId::from_str("eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
    };

    let body = Transfer {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100.00".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        memo: Some("Test transaction".to_string()),
        metadata: HashMap::new(),
    };

    let json_body = serde_json::to_value(&body).unwrap();

    let message = TapMessageEnvelope {
        message_type: TapMessageType::Transfer,
        id: "msg123".to_string(),
        version: "1.0".to_string(),
        created_time: "2021-01-01T00:00:00Z".to_string(),
        expires_time: Some("2021-01-02T00:00:00Z".to_string()),
        body: Some(json_body),
        attachments: None,
        metadata: Default::default(),
        from_did: Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()),
        to_did: Some("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string()),
    };

    // Verify the message was created correctly
    assert_eq!(message.id, "msg123");
    assert_eq!(message.version, "1.0");
    assert!(matches!(message.message_type, TapMessageType::Transfer));
}
