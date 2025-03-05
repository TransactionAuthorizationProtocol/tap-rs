extern crate tap_msg;

use std::collections::HashMap;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::types::{Participant, Transfer};

#[test]
fn test_create_message() {
    // Create a Transfer message
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
    };

    let body = Transfer {
        asset,
        originator: originator.clone(),
        beneficiary: Some(beneficiary.clone()),
        amount: "100000000".to_string(),
        agents: vec![originator, beneficiary],
        settlement_id: None,
        memo: Some("Test transaction".to_string()),
        metadata: HashMap::new(),
    };

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"),
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#transfer");
    assert!(message.created_time.is_some());
    assert_eq!(
        message.from,
        Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string())
    );
    assert_eq!(
        message.to,
        Some(vec![
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string()
        ])
    );
}
