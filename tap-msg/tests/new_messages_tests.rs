extern crate tap_msg;

use chrono::{Duration, Utc};
use serde_json::json;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{
    Attachment, AttachmentData, AuthorizationRequired, Connect, ConnectionConstraints,
    OutOfBand, Participant, Payment, PaymentBuilder, TransactionLimits,
};

#[test]
fn test_payment_request_with_asset() {
    // Create a Payment message with asset
    let asset = "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse::<AssetId>()
        .unwrap();

    let merchant = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
    };

    let transaction_id = uuid::Uuid::new_v4().to_string();
    
    // Create the Payment using the builder pattern
    let body = PaymentBuilder::default()
        .transaction_id(transaction_id)
        .asset(asset)
        .amount("100000000".to_string())
        .originator(merchant.clone())
        .beneficiary(agent.clone())
        .add_agent(agent.clone())
        .build();

    // Validate the message
    assert!(body.validate().is_ok());

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
            ["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6"]
                .iter()
                .copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#paymentrequest");
    assert!(message.created_time.is_some());
    assert_eq!(
        message.from,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()
    );
    assert_eq!(
        message.to,
        vec![
            "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string()
        ]
    );
}

#[test]
fn test_payment_request_with_currency() {
    // Create a Payment message with currency
    let merchant = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    let agent = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
    };

    let body = Payment::with_currency(
        "USD".to_string(),
        "100.00".to_string(),
        merchant.clone(),
        vec![agent.clone()],
    );

    // Add supported assets
    let mut body_with_supported_assets = body.clone();
    body_with_supported_assets.supported_assets = Some(vec![
        "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
        "bip122:000000000019d6689c085ae165831e93/slip44:0".to_string(),
    ]);

    // Validate the message
    assert!(body.validate().is_ok());
    assert!(body_with_supported_assets.validate().is_ok());

    // Test validation fails with neither asset nor currency
    let mut invalid_body = body.clone();
    invalid_body.currency = None;
    assert!(invalid_body.validate().is_err());
}

#[test]
fn test_connect_message() {
    // Create a Connect message
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let agent_id = "did:example:b2b-service".to_string();
    let for_id = "did:example:business-customer".to_string();
    let role = Some("ServiceAgent");
    
    // Create transaction limits
    let transaction_limits = TransactionLimits {
        max_amount: Some("10000.00".to_string()),
        max_total_amount: Some("50000.00".to_string()),
        max_transactions: Some(100),
    };
    
    let constraints = ConnectionConstraints {
        transaction_limits: Some(transaction_limits),
    };
    
    // Create the Connect message
    let mut body = Connect::new(&transaction_id, &agent_id, &for_id, role);
    body.constraints = Some(constraints);

    // Validate the message
    assert!(body.validate().is_ok());

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            "did:example:b2b-service",
            ["did:example:vasp"].iter().copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#connect");
    assert!(message.created_time.is_some());
    assert_eq!(message.from, "did:example:b2b-service".to_string());
    assert_eq!(message.to, vec!["did:example:vasp".to_string()]);

    // Test validation fails with empty for_
    let mut invalid_body = body.clone();
    invalid_body.for_ = "".to_string();
    assert!(invalid_body.validate().is_err());

    // Test minimal validation
    let minimal_body = Connect::new(
        "test-transaction-id",
        "did:example:b2b-service",
        "did:example:business-customer",
        None
    );
    assert!(minimal_body.validate().is_ok());
}

#[test]
fn test_authorization_required_message() {
    // Create an AuthorizationRequired message
    // Calculate a future expiry date (e.g., 1 day from now)
    let future_expiry = Utc::now() + Duration::days(1);
    let expires_str = future_expiry.to_rfc3339();

    let body = AuthorizationRequired::new(
        "https://vasp.com/authorize?request=abc123".to_string(),
        expires_str.clone(), // Use the future date string
    );

    // Validate the message
    assert!(body.validate().is_ok());

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            "did:example:vasp",
            ["did:example:b2b-service"].iter().copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(
        message.type_,
        "https://tap.rsvp/schema/1.0#authorizationrequired"
    );
    assert!(message.created_time.is_some());
    assert_eq!(message.from, "did:example:vasp".to_string());
    assert_eq!(
        message.to,
        vec!["did:example:b2b-service".to_string()]
    );

    // Test validation fails with empty authorization_url
    let invalid_body = AuthorizationRequired::new("".to_string(), expires_str);
    assert!(invalid_body.validate().is_err());

    // Test validation fails with invalid expiry date format
    let invalid_expiry_body = AuthorizationRequired::new(
        "https://vasp.com/authorize?request=abc123".to_string(),
        "invalid-date-format".to_string(),
    );
    assert!(invalid_expiry_body.validate().is_err());
}

#[test]
fn test_out_of_band_message() {
    // Create an OutOfBand message
    let attachment_data = AttachmentData {
        base64: None,
        json: Some(json!({
            "key": "value"
        })),
    };

    let attachment = Attachment {
        id: "123".to_string(),
        media_type: "application/json".to_string(),
        data: Some(attachment_data),
    };

    let body = OutOfBand::new(
        Some("payment-request".to_string()),
        Some("Request a payment".to_string()),
        vec![attachment],
    );

    // Add accept and handshake_protocols
    let mut body_with_accept = body.clone();
    body_with_accept.accept = Some(vec!["application/json".to_string()]);
    body_with_accept.handshake_protocols =
        Some(vec!["https://didcomm.org/connections/1.0".to_string()]);

    // Validate the message
    assert!(body.validate().is_ok());
    assert!(body_with_accept.validate().is_ok());

    // Convert to DIDComm message
    let message = body
        .to_didcomm_with_route(
            "did:example:sender",
            ["did:example:recipient"].iter().copied(),
        )
        .unwrap();

    // Verify the message was created correctly
    assert!(!message.id.is_empty());
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#outofband");
    assert!(message.created_time.is_some());
    assert_eq!(message.from, "did:example:sender".to_string());
    assert_eq!(message.to, vec!["did:example:recipient".to_string()]);

    // Test validation fails with invalid attachment
    let invalid_attachment = Attachment {
        id: "".to_string(), // Empty ID
        media_type: "application/json".to_string(),
        data: None,
    };

    let invalid_body = OutOfBand::new(None, None, vec![invalid_attachment]);
    assert!(invalid_body.validate().is_err());

    // Test validation fails with empty media_type
    let invalid_media_type_attachment = Attachment {
        id: "123".to_string(),
        media_type: "".to_string(), // Empty media type
        data: None,
    };

    let invalid_media_type_body = OutOfBand::new(None, None, vec![invalid_media_type_attachment]);
    assert!(invalid_media_type_body.validate().is_err());
}

#[test]
fn test_invalid_out_of_band_message() {
    // Test case 1: Invalid attachment (missing id)
    let invalid_attachment = Attachment {
        id: "".to_string(),
        media_type: "application/json".to_string(),
        data: None,
    };
    let invalid_body = OutOfBand::new(None, None, vec![invalid_attachment]);
    assert!(invalid_body.validate().is_err());

    // Test case 2: Invalid attachment media type
    let invalid_media_type_attachment = Attachment {
        id: "attachment1".to_string(),
        media_type: "".to_string(),
        data: None,
    };
    let invalid_media_type_body = OutOfBand::new(None, None, vec![invalid_media_type_attachment]);
    assert!(invalid_media_type_body.validate().is_err());
}

#[test]
fn test_payment_request_message() {
    let asset = AssetId::from_str("eip155:1/slip44:60").unwrap(); // Ethereum

    let payment_request_body = Payment {
        asset: Some(asset),
        amount: "1000000000000000000".to_string(), // 1 ETH
        // note: Some("Test payment request".to_string()), // Invalid field
        // thid: None, // Invalid field
        // pthid: None, // Invalid field
        expiry: None, // Correct field name
        merchant: Participant::new("did:example:merchant"),
        customer: Some(Participant::new("did:example:customer")), // Wrap in Some()
        // attachments: None, // Invalid field
        agents: vec![Participant::new("did:example:agent1")],
        currency: None,
        supported_assets: None,
        invoice: None,
        metadata: Default::default(),
    };

    let message_result = payment_request_body.to_didcomm(Some("did:example:sender"));

    assert!(message_result.is_ok());
    let message = message_result.unwrap();

    // Verify message properties
    assert!(!message.id.is_empty()); // Check that an ID was generated
    assert_eq!(message.typ, "application/didcomm-plain+json"); // Verify type based on TapMessageBody implementation
    assert!(!message.body.is_null()); // Check if body is not null

    let body_json = message.body;
    let body: Payment = serde_json::from_value(body_json).unwrap();
    assert_eq!(body.amount, "1000000000000000000"); // Verify body content
}
