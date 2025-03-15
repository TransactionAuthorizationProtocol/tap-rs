use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::types::Authorizable;
use tap_msg::message::types::UpdateParty;
use tap_msg::{Participant, Transfer};

#[test]
fn test_transfer_authorizable() {
    // Create a Transfer message
    let transfer = create_test_transfer();

    // Test authorize method
    let auth = transfer.authorize(Some("Authorization approved".to_string()), HashMap::new());
    assert_eq!(auth.note, Some("Authorization approved".to_string()));

    // Test reject method
    let reject = transfer.reject(
        "REJECT-001".to_string(),
        "Rejected due to compliance issues".to_string(),
        Some("Additional rejection note".to_string()),
        HashMap::new(),
    );
    assert_eq!(reject.code, "REJECT-001");
    assert_eq!(reject.description, "Rejected due to compliance issues");
    assert_eq!(reject.note, Some("Additional rejection note".to_string()));

    // Test settle method
    let settle = transfer.settle(
        "tx-12345".to_string(),
        Some("0x1234567890abcdef".to_string()),
        Some(1234567),
        Some("Settlement note".to_string()),
        HashMap::new(),
    );
    assert_eq!(settle.transaction_id, "tx-12345");
    assert_eq!(
        settle.transaction_hash,
        Some("0x1234567890abcdef".to_string())
    );
    assert_eq!(settle.block_height, Some(1234567));
    assert_eq!(settle.note, Some("Settlement note".to_string()));
}

#[test]
fn test_didcomm_message_authorizable() {
    // Create a Transfer message and convert to DIDComm message
    let transfer = create_test_transfer();
    let message = transfer
        .to_didcomm()
        .expect("Failed to convert to DIDComm message");

    // Test authorize method
    let auth = message.authorize(Some("Authorization approved".to_string()), HashMap::new());
    assert_eq!(auth.note, Some("Authorization approved".to_string()));

    // Test reject method
    let reject = message.reject(
        "REJECT-001".to_string(),
        "Rejected due to compliance issues".to_string(),
        Some("Additional rejection note".to_string()),
        HashMap::new(),
    );
    assert_eq!(reject.code, "REJECT-001");
    assert_eq!(reject.description, "Rejected due to compliance issues");
    assert_eq!(reject.note, Some("Additional rejection note".to_string()));

    // Test settle method
    let settle = message.settle(
        "tx-12345".to_string(),
        Some("0x1234567890abcdef".to_string()),
        Some(1234567),
        Some("Settlement note".to_string()),
        HashMap::new(),
    );
    assert_eq!(settle.transaction_id, "tx-12345");
    assert_eq!(
        settle.transaction_hash,
        Some("0x1234567890abcdef".to_string())
    );
    assert_eq!(settle.block_height, Some(1234567));
    assert_eq!(settle.note, Some("Settlement note".to_string()));
}

#[test]
fn test_full_flow() {
    // Create a Transfer message
    let transfer = create_test_transfer();
    let original_message = transfer
        .to_didcomm()
        .expect("Failed to convert to DIDComm message");

    // Generate authorize response
    let auth = original_message.authorize(Some("Transfer approved".to_string()), HashMap::new());

    // Convert authorize to DIDComm message
    let auth_message = auth
        .to_didcomm()
        .expect("Failed to convert authorize to DIDComm message");
    assert_eq!(auth_message.type_, "https://tap.rsvp/schema/1.0#authorize");

    // Generate settle response
    let settle = original_message.settle(
        "txid-12345".to_string(),
        Some("0xabcdef1234567890".to_string()),
        Some(9876543),
        Some("Settlement completed".to_string()),
        HashMap::new(),
    );

    // Convert settle to DIDComm message
    let settle_message = settle
        .to_didcomm()
        .expect("Failed to convert settle to DIDComm message");
    assert_eq!(settle_message.type_, "https://tap.rsvp/schema/1.0#settle");
}

// Helper function to create a test Transfer message
#[test]
fn test_update_party_message() {
    // Create a test Transfer message first
    let transfer = create_test_transfer();

    // Get the transfer_id (in a real scenario, this would be available)
    let transfer_message = transfer
        .to_didcomm()
        .expect("Failed to convert transfer to DIDComm");
    let transfer_id = transfer_message.id.clone();

    // Create a participant that will be updated
    let updated_participant = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("new_role".to_string()),
        policies: None,
        lei: None,
    };

    // Create an UpdateParty message
    let update_party = UpdateParty::new(&transfer_id, "beneficiary", updated_participant.clone());

    // Add a note
    let update_party_with_note = UpdateParty {
        note: Some("Updating party information".to_string()),
        ..update_party
    };

    // Validate the message
    assert!(update_party_with_note.validate().is_ok());

    // Test conversion to DIDComm
    let didcomm_message = update_party_with_note
        .to_didcomm()
        .expect("Failed to convert UpdateParty to DIDComm");

    // Verify fields
    assert_eq!(
        didcomm_message.type_,
        "https://tap.rsvp/schema/1.0#updateparty"
    );

    // Test from_didcomm
    let round_trip = UpdateParty::from_didcomm(&didcomm_message)
        .expect("Failed to convert DIDComm to UpdateParty");

    // Verify round-trip conversion
    assert_eq!(round_trip.transfer_id, transfer_id);
    assert_eq!(round_trip.party_type, "beneficiary");
    assert_eq!(round_trip.party.id, updated_participant.id);
    assert_eq!(round_trip.party.role, updated_participant.role);
    assert_eq!(
        round_trip.note,
        Some("Updating party information".to_string())
    );

    // Test using update_party from Authorizable trait
    let update_party_from_authorizable = transfer.update_party(
        "beneficiary".to_string(),
        updated_participant,
        Some("Updated via Authorizable trait".to_string()),
        HashMap::new(),
    );

    assert_eq!(update_party_from_authorizable.party_type, "beneficiary");
    assert_eq!(
        update_party_from_authorizable.note,
        Some("Updated via Authorizable trait".to_string())
    );
}

fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("originator".to_string()),
        policies: None,
        lei: None,
    };

    let beneficiary = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("beneficiary".to_string()),
        policies: None,
        lei: None,
    };

    let agents = vec![Participant {
        id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
        role: None,
        policies: None,
        lei: None,
    }];

    Transfer {
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        memo: Some("Test transfer".to_string()),
        metadata: HashMap::new(),
    }
}
