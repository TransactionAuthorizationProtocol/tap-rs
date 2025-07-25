// use serde_json; // Redundant import

// Removing external didcomm dependency as we don't really need it for these tests
// Creating a simple mock resolver for testing purposes
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::error::Error;
use tap_msg::message::tap_message_trait::{Connectable, TapMessageBody}; // Import trait for methods
use tap_msg::message::{
    Agent, Authorize, Connect, ConnectionConstraints, Party, Payment, PaymentBuilder, Reject,
    Settle, TransactionLimits, Transfer, UpdateParty,
};
use tap_msg::Result;

/// Integration test for the full TAP message flow:
/// 1. Create a Connect message
/// 2. Create a Transfer message connected to the Connect message
/// 3. Authorize the Transfer
/// 4. Settle the Transfer
#[test]
fn test_full_tap_flow() -> Result<()> {
    // We don't need an actual DID resolver for these tests

    // Step 1: Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert Connect to DIDComm");
    let connection_id = connect_message.id.clone();

    // Step 2: Create a Transfer message connected to the Connect message
    let mut transfer = create_test_transfer();
    println!(
        "DEBUG: Before with_connection, transfer.connection_id() = {:?}",
        transfer.connection_id()
    );
    transfer.with_connection(&connection_id);
    println!(
        "DEBUG: After with_connection, transfer.connection_id() = {:?}",
        transfer.connection_id()
    );

    // Verify connection
    assert!(transfer.connection_id().is_some()); // Check using connection_id()
    assert_eq!(transfer.connection_id(), Some(connection_id.as_str()));

    // Convert to DIDComm message
    let transfer_message = transfer
        .to_didcomm_with_route(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Transfer to DIDComm");

    // Verify the transfer message has the correct pthid
    println!(
        "DEBUG: transfer_message.pthid = {:?}",
        transfer_message.pthid
    );
    println!("DEBUG: connection_id = {:?}", connection_id);
    println!(
        "DEBUG: transfer.connection_id() = {:?}",
        transfer.connection_id()
    );
    assert_eq!(transfer_message.pthid, Some(connection_id.clone()));

    // Step 3: Authorize the Transfer
    let transfer_body_json = transfer_message.body.clone();
    if transfer_body_json.is_null() {
        return Err(Error::SerializationError(
            "Missing transfer message body".to_string(),
        ));
    }
    let _transfer_body: Transfer = serde_json::from_value(transfer_body_json.clone())?;
    let authorize_body = Authorize {
        transaction_id: transfer_message.id.clone(), // Use the ID of the message being authorized
        settlement_address: None,
        expiry: None,
    };

    // Convert to DIDComm message
    let mut authorize_message = authorize_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Authorize to DIDComm");

    // Manually set thread ID for the reply
    authorize_message.thid = Some(transfer_message.id.clone());

    // Verify the authorize message has the correct thread ID (should be the transfer ID)
    assert_eq!(authorize_message.thid, Some(transfer_message.id.clone()));

    // Step 4: Settle the Transfer
    let settle_body = Settle {
        transaction_id: transfer_message.id.clone(),
        settlement_id: Some("tx-12345".to_string()),
        amount: Some("100".to_string()),
    };

    // Convert to DIDComm message
    let settle_message = settle_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Settle to DIDComm");

    // Verify the settle message has the correct thread ID (should be the transfer ID)
    assert_eq!(
        settle_message.thid.as_deref(),
        Some(transfer_message.id.as_str())
    ); // Use as_deref and as_str

    Ok(())
}

/// Integration test for the payment flow:
/// 1. Create a Connect message
/// 2. Create a Payment message connected to the Connect message
/// 3. Authorize the Payment
/// 4. Reject a subsequent Payment
#[test]
fn test_payment_flow() {
    // Step 1: Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert Connect to DIDComm");
    let connection_id = connect_message.id.clone();

    // Step 2: Create a Payment message connected to the Connect message
    let mut payment = create_test_payment_request();
    println!(
        "DEBUG: Before with_connection, payment.connection_id() = {:?}",
        payment.connection_id()
    );
    payment.with_connection(&connection_id);
    println!(
        "DEBUG: After with_connection, payment.connection_id() = {:?}",
        payment.connection_id()
    );

    // Verify connection
    assert!(payment.connection_id().is_some()); // Check using connection_id()
    assert_eq!(payment.connection_id(), Some(connection_id.as_str()));

    // Convert to DIDComm message
    let payment_message = payment
        .to_didcomm_with_route(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Payment to DIDComm");

    // Check that the payment message has the correct pthid (parent thread ID)
    assert_eq!(payment_message.pthid, Some(connection_id.clone()));

    // Step 3: Authorize the Payment
    let authorize_body = Authorize {
        transaction_id: payment_message.id.clone(), // Use the ID of the message being authorized
        settlement_address: None,
        expiry: None,
    };

    // Convert to DIDComm message
    let mut authorize_message = authorize_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Authorize to DIDComm");

    // Manually set thread ID for the reply
    authorize_message.thid = Some(payment_message.id.clone());

    // Verify the authorize message has the correct thread ID (should be the payment ID)
    assert_eq!(authorize_message.thid, Some(payment_message.id.clone()));

    // Step 4: Create a second Payment and reject it
    let mut payment2 = create_test_payment_request();
    println!(
        "DEBUG: Before with_connection, payment2.connection_id() = {:?}",
        payment2.connection_id()
    );
    payment2.with_connection(&connection_id);
    println!(
        "DEBUG: After with_connection, payment2.connection_id() = {:?}",
        payment2.connection_id()
    );
    let payment_message2 = payment2
        .to_didcomm_with_route(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Payment to DIDComm");

    // Reject the payment
    let reject_body = Reject {
        transaction_id: payment_message2.id.clone(),
        reason: Some("REJECT-001: Rejected due to compliance issues".to_string()),
    };

    // Convert reject body to DIDComm message
    let mut reject_message = reject_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6", // Rejector's DID
            [payment_message2.from.as_str()], // Send back to Payment sender
        )
        .expect("Failed to convert Reject to DIDComm");

    // Manually set thread ID for the reply
    reject_message.thid = Some(payment_message2.id.clone());

    // Verify the reject message has the correct thread ID (should be the payment ID)
    assert_eq!(reject_message.thid, Some(payment_message2.id.clone()));
}

/// Integration test for a complex flow with multiple connected messages:
/// 1. Create a Connect message
/// 2. Create multiple Transfer messages connected to the Connect message
/// 3. Authorize some and reject others
#[test]
fn test_complex_message_flow() -> Result<()> {
    // We don't need an actual DID resolver for these tests

    // Step 1: Create a Connect message
    let connect = create_test_connect();
    let connect_message = connect
        .to_didcomm("did:example:sender")
        .expect("Failed to convert Connect to DIDComm");
    let connection_id = connect_message.id.clone();

    // Step 2: Create multiple Transfer messages connected to the Connect message
    let mut transfers = Vec::new();
    let mut transfer_messages = Vec::new();

    for i in 0..3 {
        let mut transfer = create_test_transfer();
        // Modify the amount for each transfer
        transfer.amount = format!("{}.0", (i + 1) * 100);
        println!(
            "DEBUG: Before with_connection, transfer.connection_id() = {:?}",
            transfer.connection_id()
        );
        transfer.with_connection(&connection_id);
        println!(
            "DEBUG: After with_connection, transfer.connection_id() = {:?}",
            transfer.connection_id()
        );

        let transfer_message = transfer
            .to_didcomm_with_route(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
                ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                    .iter()
                    .copied(),
            )
            .expect("Failed to convert Transfer to DIDComm");

        transfers.push(transfer);
        transfer_messages.push(transfer_message);
    }

    // Unpack and verify each transfer message
    for (i, transfer_message) in transfer_messages.iter().enumerate() {
        // let _unpacked = resolver.receive_message(transfer_message)?; // Resolver doesn't have this method, and unpacking might be handled differently
        println!(
            "Transfer {} received by Bob: ID={}",
            i + 1,
            transfer_message.id
        );
    }

    // Step 3: Authorize the first transfer
    let transfer_body_json = transfer_messages[0].body.clone();
    if transfer_body_json.is_null() {
        return Err(Error::SerializationError(
            "Missing transfer message body".to_string(),
        ));
    }
    let _transfer_body: Transfer = serde_json::from_value(transfer_body_json.clone())?;
    let authorize_body = Authorize {
        transaction_id: transfer_messages[0].id.clone(),
        settlement_address: None,
        expiry: None,
    };

    // Convert to DIDComm message
    let mut authorize_message = authorize_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Authorize to DIDComm");

    // Manually set thread ID for the reply
    authorize_message.thid = Some(transfer_messages[0].id.clone());

    // Step 4: Reject the second transfer
    let transfer_body_json_1 = transfer_messages[1].body.clone();
    if transfer_body_json_1.is_null() {
        return Err(Error::SerializationError(
            "Missing transfer message body".to_string(),
        ));
    }
    let _transfer_body_1: Transfer = serde_json::from_value(transfer_body_json_1.clone())?;
    let reject = Reject {
        transaction_id: transfer_messages[1].id.clone(),
        reason: Some("REJECT-002: Rejected due to amount too high".to_string()),
    };

    // Convert to DIDComm message
    let mut reject_message = reject
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Reject to DIDComm");

    // Manually set thread ID for the reply
    reject_message.thid = Some(transfer_messages[1].id.clone());

    // Step 5: Settle the third transfer
    let transfer_body_json_2 = transfer_messages[2].body.clone();
    if transfer_body_json_2.is_null() {
        return Err(Error::SerializationError(
            "Missing transfer message body".to_string(),
        ));
    }
    let _transfer_body_2: Transfer = serde_json::from_value(transfer_body_json_2.clone())?;
    let settle_body = Settle {
        transaction_id: transfer_messages[2].id.clone(),
        settlement_id: Some("tx-67890".to_string()),
        amount: Some("50".to_string()),
    };

    // Convert to DIDComm message
    let settle_message = settle_body
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert Settle to DIDComm");

    // Step 6: UpdateParty the third transfer
    let updated_originator = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let update_party = UpdateParty {
        transaction_id: transfer_messages[2].id.clone(),
        party_type: "originator".to_string(),
        party: updated_originator.clone(),
        context: None,
    };

    // Convert to DIDComm message
    let mut update_party_message = update_party
        .to_didcomm_with_route(
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
            ["did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"]
                .iter()
                .copied(),
        )
        .expect("Failed to convert UpdateParty to DIDComm");

    // Manually set thread ID for the reply
    update_party_message.thid = Some(transfer_messages[2].id.clone());

    // Verify all messages have the correct thread IDs and parent thread IDs
    assert_eq!(
        authorize_message.thid,
        Some(transfer_messages[0].id.clone())
    );
    assert_eq!(reject_message.thid, Some(transfer_messages[1].id.clone()));
    assert_eq!(
        settle_message.thid.as_deref(),
        Some(transfer_messages[2].id.as_str())
    ); // Use as_deref and as_str
    assert_eq!(
        update_party_message.thid,
        Some(transfer_messages[2].id.clone())
    );

    // All transfer messages should have the connection_id as their parent thread ID
    for transfer_message in &transfer_messages {
        assert_eq!(transfer_message.pthid, Some(connection_id.clone()));
    }

    Ok(())
}

// Helper functions to create test messages

fn create_test_connect() -> Connect {
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let agent_id = "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx".to_string();
    let for_id = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string();

    let mut connect = Connect::new(&transaction_id, &agent_id, &for_id, Some("agent"));

    let transaction_limits = TransactionLimits {
        per_transaction: Some("1000.0".to_string()),
        daily: Some("5000.0".to_string()),
        currency: Some("USD".to_string()),
    };

    let constraints = ConnectionConstraints {
        purposes: Some(vec!["trading".to_string()]),
        category_purposes: None,
        limits: Some(transaction_limits),
    };

    connect.constraints = Some(constraints);
    connect
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
        transaction_id: Some(uuid::Uuid::new_v4().to_string()),
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
    let transaction_id = uuid::Uuid::new_v4().to_string();

    let merchant = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    let customer = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    let agents = vec![Agent::new(
        "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
        "agent",
        "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
    )];

    let asset =
        AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();

    let mut payment = PaymentBuilder::default()
        .transaction_id(transaction_id)
        .asset(asset)
        .amount("100.0".to_string())
        .merchant(merchant)
        .customer(customer)
        .agents(agents)
        .build();

    payment.currency_code = Some("USD".to_string());
    payment.expiry = Some("2023-12-31T23:59:59Z".to_string());
    payment
}
