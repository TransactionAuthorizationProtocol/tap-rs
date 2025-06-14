use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::{Connectable, TapMessageBody};
use tap_msg::message::{Agent, Connect, Party, Payment, Transfer};

#[test]
fn test_transfer_connectable() {
    // Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id = connect_message.id.clone();

    // Create a Transfer message
    let mut transfer = create_test_transfer();

    // Test initial state (no connection)
    assert!(!transfer.has_connection());
    assert_eq!(transfer.connection_id(), None);

    // Connect the transfer to the connect message
    transfer.with_connection(&connection_id);

    // Test connected state
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(connection_id.as_str()));

    // Convert to DIDComm message and verify the connection is preserved
    let _transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");

    // The connection should be stored in the connection_id field
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(connection_id.as_str()));
    assert_eq!(transfer.connection_id, Some(connection_id.clone()));
}

#[test]
fn test_payment_request_connectable() {
    // Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id = connect_message.id.clone();

    // Create a Payment message
    let mut payment = create_test_payment_request();

    // Test initial state (no connection)
    assert!(!payment.has_connection());
    assert_eq!(payment.connection_id(), None);

    // Connect the payment to the connect message
    payment.with_connection(&connection_id);

    // Test connected state
    assert!(payment.has_connection());
    assert_eq!(payment.connection_id(), Some(connection_id.as_str()));

    // Convert to DIDComm message and verify the connection is preserved
    let _payment_message = payment
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");

    // The connection should be stored in the connection_id field
    assert!(payment.has_connection());
    assert_eq!(payment.connection_id(), Some(connection_id.as_str()));
    assert_eq!(payment.connection_id, Some(connection_id.clone()));
}

#[test]
fn test_message_connectable() {
    // Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id = connect_message.id.clone();

    // Create a Transfer message and convert to DIDComm message
    let transfer = create_test_transfer();
    let mut transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");

    // Test initial state (no connection)
    assert!(!transfer_message.has_connection());
    assert_eq!(transfer_message.connection_id(), None);

    // Connect the message to the connect message
    transfer_message.with_connection(&connection_id);

    // Test connected state
    assert!(transfer_message.has_connection());
    assert_eq!(
        transfer_message.connection_id(),
        Some(connection_id.as_str())
    );

    // The connection should be stored in the pthid field
    assert_eq!(transfer_message.pthid, Some(connection_id));
}

#[test]
fn test_connection_round_trip() {
    // Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id = connect_message.id.clone();

    // Create a Transfer message, connect it, and convert to DIDComm message
    let mut transfer = create_test_transfer();
    transfer.with_connection(&connection_id);
    let transfer_message = transfer
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");

    // Convert back to Transfer and verify the connection is preserved
    let round_trip_transfer = Transfer::from_didcomm(&transfer_message)
        .expect("Failed to convert DIDComm message back to Transfer");

    assert!(round_trip_transfer.has_connection());
    assert_eq!(
        round_trip_transfer.connection_id(),
        Some(connection_id.as_str())
    );
}

#[test]
fn test_multiple_connections() {
    // Create two Connect messages
    let connect1 = create_test_connect();
    let connect_message1 = connect1
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id1 = connect_message1.id.clone();

    let connect2 = create_test_connect();
    let connect_message2 = connect2
        .to_didcomm("did:example:sender")
        .expect("Failed to convert to DIDComm message");
    let connection_id2 = connect_message2.id.clone();

    // Create a Transfer message and connect it to the first connect message
    let mut transfer = create_test_transfer();
    transfer.with_connection(&connection_id1);

    // Verify it's connected to the first connect message
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(connection_id1.as_str()));

    // Connect it to the second connect message
    transfer.with_connection(&connection_id2);

    // Verify it's now connected to the second connect message
    assert!(transfer.has_connection());
    assert_eq!(transfer.connection_id(), Some(connection_id2.as_str()));
}

// Helper functions to create test messages

fn create_test_connect() -> Connect {
    Connect {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        agent_id: Some("did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string()),
        agent: None,
        principal: None,
        for_: Some("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()),
        role: None,
        constraints: None,
    }
}

fn create_test_transfer() -> Transfer {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let originator = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let beneficiary = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    let agents = vec![Agent::new(
        "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
        "agent",
        "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
    )];

    Transfer {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        asset,
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents,
        settlement_id: None,
        memo: None,
        connection_id: None,
        metadata: HashMap::new(),
    }
}

fn create_test_payment_request() -> Payment {
    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let merchant = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let customer = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    Payment {
        asset: Some(asset),
        amount: "100.0".to_string(),
        currency_code: Some("USD".to_string()),
        supported_assets: None,
        transaction_id: uuid::Uuid::new_v4().to_string(),
        memo: None,
        expiry: None,
        invoice: None,
        connection_id: None,
        metadata: HashMap::new(),
        merchant,
        customer: Some(customer),
        agents: vec![],
    }
}
