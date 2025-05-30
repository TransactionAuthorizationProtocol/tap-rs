extern crate tap_msg;

use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Participant, Transfer, UpdateParty};

#[test]
fn test_update_party_creation() {
    // Create a transfer ID (simulating an existing transfer)
    let transaction_id = "12345-67890-abcdef";

    // Create a participant that will be updated
    let mut updated_participant =
        Participant::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");
    updated_participant.role = Some("new_role".to_string());

    // Create an UpdateParty message
    let update_party = UpdateParty {
        transaction_id: transaction_id.to_string(),
        party_type: "beneficiary".to_string(),
        party: updated_participant.clone(),
        context: None,
    };

    // Verify fields
    assert_eq!(update_party.transaction_id, transaction_id);
    assert_eq!(update_party.party_type, "beneficiary");
    assert_eq!(update_party.party.id, updated_participant.id);
    assert_eq!(update_party.party.role, updated_participant.role);
    assert_eq!(update_party.context, None);

    // Convert to DIDComm
    let didcomm_message = update_party
        .to_didcomm("did:example:1234567890abcdef")
        .expect("Failed to convert UpdateParty to DIDComm");

    assert_eq!(
        didcomm_message.type_,
        "https://tap.rsvp/schema/1.0#UpdateParty"
    );
}

#[test]
fn test_update_party_validation() {
    // Test with valid data
    let valid_update = UpdateParty {
        transaction_id: "transfer-123".to_string(),
        party_type: "beneficiary".to_string(),
        party: {
            let mut p =
                Participant::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");
            p.role = Some("beneficiary".to_string());
            p
        },
        context: None,
    };

    assert!(valid_update.validate().is_ok());

    // Test with empty transaction_id
    let invalid_transaction_id = UpdateParty {
        transaction_id: "".to_string(),
        ..valid_update.clone()
    };

    assert!(invalid_transaction_id.validate().is_err());

    // Test with empty party_type
    let invalid_party_type = UpdateParty {
        party_type: "".to_string(),
        ..valid_update.clone()
    };

    assert!(invalid_party_type.validate().is_err());

    // Test with empty party.id
    let invalid_party = UpdateParty {
        party: Participant::new(""),
        ..valid_update.clone()
    };

    assert!(invalid_party.validate().is_err());
}

#[test]
fn test_update_party_didcomm_conversion() {
    // Create a valid UpdateParty message
    let update_party = UpdateParty {
        transaction_id: "transfer-456".to_string(),
        party_type: "originator".to_string(),
        party: {
            let mut p =
                Participant::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");
            p.role = Some("updated_role".to_string());
            p
        },
        context: None,
    };

    // Convert to DIDComm
    let didcomm_message = update_party
        .to_didcomm("did:example:1234567890abcdef")
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
    assert_eq!(round_trip.transaction_id, update_party.transaction_id);
    assert_eq!(round_trip.party_type, update_party.party_type);
    assert_eq!(round_trip.party.id, update_party.party.id);
    assert_eq!(round_trip.party.role, update_party.party.role);
    assert_eq!(round_trip.context, update_party.context);
}

#[test]
fn test_update_party_beneficiary() {
    let transaction_id = "transfer-456".to_string();
    let mut updated_participant =
        Participant::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");
    updated_participant.leiCode = Some("UPDATEDLEICODE456".to_string());

    let update_party = UpdateParty {
        transaction_id: transaction_id.clone(),
        party_type: "beneficiary".to_string(),
        party: updated_participant.clone(),
        context: None,
    };

    // Verify fields
    assert_eq!(update_party.transaction_id, transaction_id);
    assert_eq!(update_party.party_type, "beneficiary");
    assert_eq!(update_party.party.id, updated_participant.id);
    assert_eq!(
        update_party.party.leiCode,
        Some("UPDATEDLEICODE456".to_string())
    );
    assert_eq!(update_party.context, None);
}

#[test]
fn test_update_party_intermediary() {
    let transaction_id = "transfer-789".to_string();
    let mut updated_participant =
        Participant::new("did:key:z6Mkff4Y1wG9Bf7qY9LqfXQ3n8Yk5tW6RzX2n5k3f8j7sJg");
    updated_participant.leiCode = Some("UPDATEDLEICODE789".to_string());

    let update_party = UpdateParty {
        transaction_id: transaction_id.clone(),
        party_type: "intermediary".to_string(),
        party: updated_participant.clone(),
        context: None,
    };

    // Verify fields
    assert_eq!(update_party.transaction_id, transaction_id);
    assert_eq!(update_party.party_type, "intermediary");
    assert_eq!(update_party.party.id, updated_participant.id);
    assert_eq!(
        update_party.party.leiCode,
        Some("UPDATEDLEICODE789".to_string())
    );
    assert_eq!(update_party.context, None);
}

#[test]
fn test_authorizable_with_update_party() {
    // Create a test Transfer first
    let transfer = create_test_transfer();

    // Convert transfer to a DIDComm message *first* to get its ID
    let transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert initial transfer to DIDComm message");

    // Create a participant to update (e.g., the beneficiary)
    let mut updated_participant_1 =
        Participant::new("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH");
    updated_participant_1.leiCode = Some("5493008UI88NI01JAE46".to_string());

    // Create the first UpdateParty message manually, referencing the transfer message ID
    let update_party = UpdateParty {
        transaction_id: transfer_message.id.clone(), // Use the ID from the message
        party_type: "beneficiary".to_string(),
        party: updated_participant_1.clone(),
        context: None,
    };

    assert_eq!(update_party.transaction_id, transfer_message.id);

    // Now test creating UpdateParty for a different participant (e.g., originator)
    // We can reuse the same transfer_message ID if it's for the same transfer

    let mut updated_participant_2 =
        Participant::new("did:key:z6MkrPhff2T6RCEc3m4Q4v1nhfFbFf8aGKvFhXGf3g1jX8nN");
    updated_participant_2.leiCode = Some("UPDATEDLEICODE123".to_string());

    // Create the second UpdateParty message
    let update_party_from_message = UpdateParty {
        transaction_id: transfer_message.id.clone(), // Still references the same transfer
        party_type: "originator".to_string(),
        party: updated_participant_2.clone(),
        context: None,
    };

    // Verify the created UpdateParty message from DIDComm
    assert_eq!(update_party_from_message.party_type, "originator");
    assert_eq!(update_party_from_message.party.id, updated_participant_2.id);
    assert_eq!(
        update_party_from_message.party.leiCode,
        updated_participant_2.leiCode
    );
}

// Helper function to create a test Transfer message
fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let mut originator =
        Participant::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");
    originator.role = Some("originator".to_string());

    let mut beneficiary =
        Participant::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");
    beneficiary.role = Some("beneficiary".to_string());

    let agents = vec![Participant::new(
        "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
    )];

    Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator,
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        metadata: HashMap::new(),
        memo: None,
    }
}
