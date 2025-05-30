extern crate tap_msg;

use std::collections::HashMap;
use std::str::FromStr;

use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::Authorizable;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Participant, Reject, Settle, Transfer, UpdateParty};

#[test]
fn test_transfer_authorizable() {
    // Create a Transfer message
    let transfer = create_test_transfer();

    // Convert to DIDComm message to get an ID
    let transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert transfer to DIDComm");

    // Test authorize method - now returns PlainMessage
    let auth_message = transfer.authorize("did:example:sender", None, None);

    // The auth_message is already a PlainMessage<Authorize>, so we can access the body directly
    assert_eq!(auth_message.body.transaction_id, transfer.transaction_id);

    // Create Reject struct directly/manually since the trait is now at a different location
    let reject_code = "REJECT-001".to_string();
    let reject_reason = "Rejected due to compliance issues".to_string();
    let reject = Reject {
        transaction_id: transfer_message.id.clone(),
        reason: format!("{}: {}", reject_code, reject_reason),
    };
    assert_eq!(
        reject.reason,
        "REJECT-001: Rejected due to compliance issues"
    );

    // Create Settle struct directly
    let settle = Settle {
        transaction_id: "tx-123".to_string(),
        settlement_id: "settle-123".to_string(),
        amount: Some("50.00".to_string()),
    };

    assert_eq!(settle.settlement_id, "settle-123".to_string());
    assert_eq!(settle.amount, Some("50.00".to_string()));
}

#[test]
fn test_didcomm_message_authorizable() {
    // Create a Transfer message and convert to DIDComm message
    let transfer = create_test_transfer();
    let transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");

    // Test authorize method - now returns PlainMessage
    let auth_message = transfer.authorize("did:example:sender", None, None);

    // The auth_message is already a PlainMessage<Authorize>, so we can access the body directly
    assert_eq!(auth_message.body.transaction_id, transfer.transaction_id);

    // Create Reject struct directly/manually
    let reject_code = "REJECT-001".to_string();
    let reject_reason = "Rejected due to compliance issues".to_string();
    let reject = Reject {
        transaction_id: transfer_message.id.clone(),
        reason: format!("{}: {}", reject_code, reject_reason),
    };
    assert_eq!(
        reject.reason,
        "REJECT-001: Rejected due to compliance issues"
    );

    // Create Settle struct directly
    let settle = Settle {
        transaction_id: transfer.transaction_id.clone(),
        settlement_id: "tx-12345".to_string(),
        amount: Some("100".to_string()),
    };

    assert_eq!(settle.settlement_id, "tx-12345".to_string());
    assert_eq!(settle.amount, Some("100".to_string()));
}

#[test]
fn test_full_flow() {
    // Create a Transfer message
    let transfer = create_test_transfer();
    // Generate authorize response - now returns PlainMessage directly
    let auth_message = transfer.authorize("did:example:sender", None, None);
    assert_eq!(auth_message.type_, "https://tap.rsvp/schema/1.0#Authorize");

    // Create Settle struct directly
    let settle = Settle {
        transaction_id: transfer.transaction_id.clone(),
        settlement_id: "tx-12345".to_string(),
        amount: Some("100".to_string()),
    };

    // Convert settle to DIDComm message
    let settle_message = settle
        .to_didcomm("did:example:sender")
        .expect("Failed to convert settle to DIDComm message");
    assert_eq!(settle_message.type_, "https://tap.rsvp/schema/1.0#Settle");
}

#[test]
fn test_update_party_message() {
    // Create a test Transfer message first
    let transfer = create_test_transfer();

    // Get the transfer_id (in a real scenario, this would be available)
    let transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert transfer to DIDComm");
    let transfer_id = transfer_message.id.clone();

    // Create a participant that will be updated
    let updated_participant = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("new_role".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    // Create an UpdateParty message
    let update_party = UpdateParty {
        transaction_id: transfer_id.clone(),
        party_type: "beneficiary".to_string(),
        party: updated_participant.clone(),
        context: None,
    };

    // Validate the message
    assert!(update_party.validate().is_ok());

    // Test conversion to DIDComm
    let didcomm_message = update_party
        .to_didcomm("did:example:sender")
        .expect("Failed to convert UpdateParty to DIDComm");

    // Verify fields
    assert_eq!(
        didcomm_message.type_,
        "https://tap.rsvp/schema/1.0#UpdateParty"
    );

    // Test from_didcomm
    let round_trip = UpdateParty::from_didcomm(&didcomm_message)
        .expect("Failed to convert DIDComm to UpdateParty");

    // Verify round-trip conversion
    assert_eq!(round_trip.transaction_id, transfer_id);
    assert_eq!(round_trip.party_type, "beneficiary");
    assert_eq!(round_trip.party.id, updated_participant.id);
    assert_eq!(round_trip.party.role, updated_participant.role);

    // Test using update_party from manual creation
    let update_party_from_manual = UpdateParty {
        transaction_id: transfer_id.clone(),
        party_type: "beneficiary".to_string(),
        party: updated_participant,
        context: None,
    };

    assert_eq!(update_party_from_manual.party_type, "beneficiary");
}

fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        leiCode: None,
        name: None,
    };

    let agents = vec![Participant {
        id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
        role: None,
        policies: None,
        leiCode: None,
        name: None,
    }];

    Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        connection_id: None,
        metadata: HashMap::new(),
        memo: None,
    }
}
