extern crate tap_msg;

use chrono::{Duration, Utc};
use serde_json::json;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{
    Attachment, AuthorizationRequired, Connect, ConnectionConstraints, OutOfBand, Participant,
    Payment, PaymentBuilder, SimpleAttachmentData, TransactionLimits,
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
    assert_eq!(message.type_, "https://tap.rsvp/schema/1.0#payment");
    assert!(message.created_time.is_some());
    assert_eq!(
        message.from,
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()
    );
    assert_eq!(
        message.to,
        vec!["did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string()]
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

    // Create a payment with USD currency
    let fiat_asset =
        AssetId::from_str("fiat:USD").unwrap_or(AssetId::from_str("eip155:1/slip44:60").unwrap());
    let body = PaymentBuilder::default()
        .asset(fiat_asset.clone())
        .currency_code("USD".to_string())
        .amount("100.00".to_string())
        .transaction_id(uuid::Uuid::new_v4().to_string())
        .originator(merchant.clone())
        .beneficiary(agent.clone())
        .add_agent(agent.clone())
        .build();

    // Add supported assets via metadata
    let mut body_with_supported_assets = body.clone();
    body_with_supported_assets.metadata.insert(
        "supported_assets".to_string(),
        serde_json::to_value(vec![
            "eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(),
            "bip122:000000000019d6689c085ae165831e93/slip44:0".to_string(),
        ])
        .unwrap(),
    );

    // Validate the message
    assert!(body.validate().is_ok());
    assert!(body_with_supported_assets.validate().is_ok());

    // Test validation with minimal required fields
    let invalid_asset = AssetId::from_str("invalid:asset").unwrap_or(fiat_asset);
    let invalid_body = PaymentBuilder::default()
        .asset(invalid_asset)
        .amount("".to_string()) // Empty amount should fail validation
        .transaction_id(uuid::Uuid::new_v4().to_string())
        .originator(merchant.clone())
        .beneficiary(agent.clone())
        .build();
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
        None,
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
    assert_eq!(message.to, vec!["did:example:b2b-service".to_string()]);

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
    let _attachment_data = SimpleAttachmentData {
        base64: None,
        json: Some(json!({
            "key": "value"
        })),
    };

    let _attachment = Attachment {
        id: Some("123".to_string()),
        media_type: Some("application/json".to_string()),
        data: tap_msg::didcomm::AttachmentData::Json {
            value: tap_msg::didcomm::JsonAttachmentData {
                json: json!({
                    "key": "value"
                }),
                jws: None,
            },
        },
        description: None,
        filename: None,
        format: None,
        lastmod_time: None,
        byte_count: None,
    };

    let body = OutOfBand::new(
        "payment-request".to_string(),
        "Request a payment".to_string(),
        "https://example.com/service".to_string(),
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
    let _invalid_attachment = Attachment {
        id: Some("".to_string()), // Empty ID
        media_type: Some("application/json".to_string()),
        data: tap_msg::didcomm::AttachmentData::Json {
            value: tap_msg::didcomm::JsonAttachmentData {
                json: json!({}),
                jws: None,
            },
        },
        description: None,
        filename: None,
        format: None,
        lastmod_time: None,
        byte_count: None,
    };

    let invalid_body = OutOfBand::new(
        "".to_string(), // Empty goal_code - will fail validation
        "Invalid test".to_string(),
        "https://example.com/service".to_string(),
    );
    assert!(invalid_body.validate().is_err());

    // Test validation fails with empty media_type
    let _invalid_media_type_attachment = Attachment {
        id: Some("123".to_string()),
        media_type: Some("".to_string()), // Empty media type
        data: tap_msg::didcomm::AttachmentData::Json {
            value: tap_msg::didcomm::JsonAttachmentData {
                json: json!({}),
                jws: None,
            },
        },
        description: None,
        filename: None,
        format: None,
        lastmod_time: None,
        byte_count: None,
    };

    let invalid_media_type_body = OutOfBand::new(
        "test".to_string(),
        "Invalid media type test".to_string(),
        "".to_string(), // Empty service - will fail validation
    );
    assert!(invalid_media_type_body.validate().is_err());
}

#[test]
fn test_invalid_out_of_band_message() {
    // Test case 1: Invalid attachment (missing id)
    let _invalid_attachment = Attachment {
        id: Some("".to_string()),
        media_type: Some("application/json".to_string()),
        data: tap_msg::didcomm::AttachmentData::Json {
            value: tap_msg::didcomm::JsonAttachmentData {
                json: json!({}),
                jws: None,
            },
        },
        description: None,
        filename: None,
        format: None,
        lastmod_time: None,
        byte_count: None,
    };
    let invalid_body = OutOfBand::new(
        "".to_string(), // Empty goal_code - will fail validation
        "Invalid test".to_string(),
        "https://example.com/service".to_string(),
    );
    assert!(invalid_body.validate().is_err());

    // Test case 2: Invalid attachment media type
    let _invalid_media_type_attachment = Attachment {
        id: Some("attachment1".to_string()),
        media_type: Some("".to_string()),
        data: tap_msg::didcomm::AttachmentData::Json {
            value: tap_msg::didcomm::JsonAttachmentData {
                json: json!({}),
                jws: None,
            },
        },
        description: None,
        filename: None,
        format: None,
        lastmod_time: None,
        byte_count: None,
    };
    let invalid_media_type_body = OutOfBand::new(
        "test".to_string(),
        "Invalid media type test".to_string(),
        "".to_string(), // Empty service - will fail validation
    );
    assert!(invalid_media_type_body.validate().is_err());
}

#[test]
fn test_payment_request_message() {
    let asset = AssetId::from_str("eip155:1/slip44:60").unwrap(); // Ethereum

    let payment_request_body = PaymentBuilder::default()
        .asset(asset)
        .amount("1000000000000000000".to_string()) // 1 ETH
        .transaction_id(uuid::Uuid::new_v4().to_string())
        .originator(Participant::new("did:example:merchant"))
        .beneficiary(Participant::new("did:example:customer"))
        .add_agent(Participant::new("did:example:agent1"))
        .build();

    let message_result = payment_request_body.to_didcomm("did:example:sender");

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
