extern crate tap_msg;

use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::types::Authorizable;
use tap_msg::message::types::UpdateParty;
use tap_msg::{Participant, Transfer};

#[test]
fn test_update_party_creation() {
    // Create a transfer ID (simulating an existing transfer)
    let transfer_id = "12345-67890-abcdef";

    // Create a participant that will be updated
    let updated_participant = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("new_role".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create an UpdateParty message
    let update_party = UpdateParty::new(transfer_id, "beneficiary", updated_participant.clone());

    // Verify fields
    assert_eq!(update_party.transfer_id, transfer_id);
    assert_eq!(update_party.party_type, "beneficiary");
    assert_eq!(update_party.party.id, updated_participant.id);
    assert_eq!(update_party.party.role, updated_participant.role);
    assert_eq!(
        update_party.context,
        Some("https://tap.rsvp/schema/1.0".to_string())
    );

    // Add a note
    let update_party_with_note = UpdateParty {
        note: Some("Updating party information".to_string()),
        ..update_party
    };

    assert_eq!(
        update_party_with_note.note,
        Some("Updating party information".to_string())
    );
}

#[test]
fn test_update_party_validation() {
    // Test with valid data
    let valid_update = UpdateParty::new(
        "transfer-123",
        "beneficiary",
        Participant {
            id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        },
    );

    assert!(valid_update.validate().is_ok());

    // Test with empty transfer_id
    let invalid_transfer_id = UpdateParty {
        transfer_id: "".to_string(),
        ..valid_update.clone()
    };

    assert!(invalid_transfer_id.validate().is_err());

    // Test with empty party_type
    let invalid_party_type = UpdateParty {
        party_type: "".to_string(),
        ..valid_update.clone()
    };

    assert!(invalid_party_type.validate().is_err());

    // Test with empty party.id
    let invalid_party = UpdateParty {
        party: Participant {
            id: "".to_string(),
            role: Some("beneficiary".to_string()),
            policies: None,
            leiCode: None,
        },
        ..valid_update.clone()
    };

    assert!(invalid_party.validate().is_err());
}

#[test]
fn test_update_party_didcomm_conversion() {
    // Create a valid UpdateParty message
    let update_party = UpdateParty::new(
        "transfer-456",
        "originator",
        Participant {
            id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
            role: Some("updated_role".to_string()),
            policies: None,
            leiCode: None,
        },
    );

    // Test conversion to DIDComm
    let didcomm_message = update_party
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
    assert_eq!(round_trip.transfer_id, update_party.transfer_id);
    assert_eq!(round_trip.party_type, update_party.party_type);
    assert_eq!(round_trip.party.id, update_party.party.id);
    assert_eq!(round_trip.party.role, update_party.party.role);
    assert_eq!(round_trip.context, update_party.context);
}

#[test]
fn test_authorizable_with_update_party() {
    // Create a test Transfer first
    let transfer = create_test_transfer();

    // Create a participant to update
    let updated_participant = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("updated_role".to_string()),
        policies: None,
        leiCode: None,
    };

    // Use the Authorizable trait to create an UpdateParty message
    let update_party = transfer.update_party(
        "beneficiary".to_string(),
        updated_participant.clone(),
        Some("Updated via Authorizable trait".to_string()),
        HashMap::new(),
    );

    // Verify the created UpdateParty message
    assert_eq!(update_party.party_type, "beneficiary");
    assert_eq!(update_party.party.id, updated_participant.id);
    assert_eq!(update_party.party.role, updated_participant.role);
    assert_eq!(
        update_party.note,
        Some("Updated via Authorizable trait".to_string())
    );

    // Now test the same with a DIDComm message
    let message = transfer
        .to_didcomm()
        .expect("Failed to convert to DIDComm message");

    let update_party_from_message = message.update_party(
        "originator".to_string(),
        updated_participant.clone(),
        Some("Updated via DIDComm message".to_string()),
        HashMap::new(),
    );

    // Verify the created UpdateParty message from DIDComm
    assert_eq!(update_party_from_message.party_type, "originator");
    assert_eq!(update_party_from_message.party.id, updated_participant.id);
    assert_eq!(
        update_party_from_message.party.role,
        updated_participant.role
    );
    assert_eq!(
        update_party_from_message.note,
        Some("Updated via DIDComm message".to_string())
    );
}

// Helper function to create a test Transfer message
fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

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

    let agents = vec![Participant {
        id: "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string(),
        role: None,
        leiCode: None,
        policies: None,
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
