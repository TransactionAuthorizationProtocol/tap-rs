//! Simple integration tests for payment link functionality
//!
//! These tests focus on the core OOB and payment link features without
//! requiring complex Payment message construction.

use serde_json::json;
use std::sync::Arc;
use tap_agent::agent::{Agent, TapAgent};
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_agent::payment_link::{PaymentLinkConfig, DEFAULT_PAYMENT_SERVICE_URL};
use tap_agent::{OutOfBandInvitation, PaymentLink};

/// Helper function to create a test agent
async fn create_test_agent(did: &str) -> TapAgent {
    let secret = Secret {
        id: "default".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=",
                "d": "nWGxne/9WmC6hEr+BQh+uDpW6n7dZsN4c4C9rFfIz3Y="
            }),
        },
    };

    let mut builder = AgentKeyManagerBuilder::new();
    builder = builder.add_secret(did.to_string(), secret);
    let key_manager = builder.build().expect("Failed to build key manager");

    let config = AgentConfig::new(did.to_string());
    TapAgent::new(config, Arc::new(key_manager))
}

/// Simple test message for OOB testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestMessage {
    pub content: String,
    pub amount: String,
}

impl tap_msg::TapMessageBody for TestMessage {
    fn message_type() -> &'static str {
        "https://tap.rsvp/schema/1.0#TestMessage"
    }

    fn validate(&self) -> Result<(), tap_msg::error::Error> {
        if self.content.is_empty() {
            return Err(tap_msg::error::Error::Validation(
                "Content cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_basic_oob_creation_and_parsing() {
    let agent = create_test_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").await;

    let test_message = TestMessage {
        content: "Test payment request".to_string(),
        amount: "100.00".to_string(),
    };

    // Create OOB invitation
    let oob_url = agent
        .create_oob_invitation(
            &test_message,
            "tap.payment",
            "Process test payment",
            "https://example.com/pay",
        )
        .await
        .expect("Failed to create OOB invitation");

    // Verify URL structure
    assert!(oob_url.starts_with("https://example.com/pay"));
    assert!(oob_url.contains("?_oob="));

    // Parse the OOB invitation
    let oob_invitation = agent
        .parse_oob_invitation(&oob_url)
        .expect("Failed to parse OOB invitation");

    // Verify OOB structure
    assert_eq!(oob_invitation.from, agent.get_agent_did());
    assert_eq!(oob_invitation.body.goal_code, "tap.payment");
    assert_eq!(oob_invitation.body.goal, "Process test payment");
    assert!(oob_invitation.is_payment_invitation());

    // Verify signed attachment exists
    let signed_attachment = oob_invitation.get_signed_attachment();
    assert!(signed_attachment.is_some());

    let attachment = signed_attachment.unwrap();
    assert_eq!(
        attachment.media_type.as_deref(),
        Some("application/didcomm-signed+json")
    );
}

#[tokio::test]
async fn test_oob_attachment_structure() {
    let agent = create_test_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").await;

    let test_message = TestMessage {
        content: "Hello from sender".to_string(),
        amount: "250.50".to_string(),
    };

    // Create OOB invitation
    let oob_url = agent
        .create_oob_invitation(
            &test_message,
            "tap.payment",
            "Test message transfer",
            "https://test.example/msg",
        )
        .await
        .expect("Failed to create OOB invitation");

    // Parse the invitation
    let oob_invitation = agent
        .parse_oob_invitation(&oob_url)
        .expect("Failed to parse OOB invitation");

    // Verify OOB structure is correct
    assert_eq!(oob_invitation.from, agent.get_agent_did());
    assert_eq!(oob_invitation.body.goal_code, "tap.payment");
    assert_eq!(oob_invitation.body.goal, "Test message transfer");
    assert!(oob_invitation.is_payment_invitation());

    // Verify signed attachment exists and has correct structure
    let signed_attachment = oob_invitation.get_signed_attachment();
    assert!(signed_attachment.is_some());

    let attachment = signed_attachment.unwrap();
    assert_eq!(
        attachment.media_type.as_deref(),
        Some("application/didcomm-signed+json")
    );
    assert!(attachment.id.is_some());

    // Verify the attachment contains data
    match &attachment.data {
        tap_msg::didcomm::AttachmentData::Json { value } => {
            // The attachment should contain the signed message data
            assert!(!value.json.is_null());
        }
        _ => panic!("Expected JSON attachment data"),
    }
}

#[tokio::test]
async fn test_payment_link_configuration() {
    let agent = create_test_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").await;

    let test_message = TestMessage {
        content: "Premium subscription payment".to_string(),
        amount: "99.99".to_string(),
    };

    // Test default configuration
    let default_url = agent
        .create_oob_invitation(
            &test_message,
            "tap.payment",
            "Default payment",
            DEFAULT_PAYMENT_SERVICE_URL,
        )
        .await
        .expect("Failed to create default OOB");

    assert!(default_url.starts_with(DEFAULT_PAYMENT_SERVICE_URL));

    // Test custom configuration with metadata
    let custom_url = agent
        .create_oob_invitation(
            &test_message,
            "tap.payment",
            "Custom payment with metadata",
            "https://custom-service.example/checkout",
        )
        .await
        .expect("Failed to create custom OOB");

    assert!(custom_url.starts_with("https://custom-service.example/checkout"));

    // Parse and verify custom metadata can be added to the OOB
    let oob_invitation = agent.parse_oob_invitation(&custom_url).unwrap();
    assert_eq!(oob_invitation.body.goal, "Custom payment with metadata");
}

#[tokio::test]
async fn test_url_round_trip() {
    let agent = create_test_agent("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK").await;

    let test_message = TestMessage {
        content: "Round trip test".to_string(),
        amount: "42.00".to_string(),
    };

    // Create OOB URL
    let original_url = agent
        .create_oob_invitation(
            &test_message,
            "tap.payment",
            "Round trip test",
            "https://roundtrip.example/test",
        )
        .await
        .expect("Failed to create OOB");

    // Parse OOB invitation from URL
    let oob_invitation =
        OutOfBandInvitation::from_url(&original_url).expect("Failed to parse OOB from URL");

    // Recreate URL from OOB invitation
    let recreated_url = oob_invitation
        .to_url("https://roundtrip.example/test")
        .expect("Failed to recreate URL");

    // Parse both URLs and verify content matches
    let original_oob = OutOfBandInvitation::from_url(&original_url).unwrap();
    let recreated_oob = OutOfBandInvitation::from_url(&recreated_url).unwrap();

    assert_eq!(original_oob.from, recreated_oob.from);
    assert_eq!(original_oob.body.goal_code, recreated_oob.body.goal_code);
    assert_eq!(original_oob.body.goal, recreated_oob.body.goal);
    assert_eq!(original_oob.id, recreated_oob.id);
}

#[test]
fn test_payment_link_config_builder() {
    // Test PaymentLinkConfig functionality
    let config = PaymentLinkConfig::new()
        .with_service_url("https://custom.example/pay")
        .with_metadata("order_id", json!("ORDER-123"))
        .with_metadata("store_id", json!("STORE-456"))
        .with_goal("Complete your order");

    assert_eq!(config.service_url, "https://custom.example/pay");
    assert_eq!(config.metadata.get("order_id"), Some(&json!("ORDER-123")));
    assert_eq!(config.metadata.get("store_id"), Some(&json!("STORE-456")));
    assert_eq!(config.goal, Some("Complete your order".to_string()));
}

#[test]
fn test_oob_validation() {
    // Test OOB validation with various configurations
    let valid_oob =
        OutOfBandInvitation::builder("did:example:test", "tap.payment", "Valid payment").build();

    assert!(valid_oob.validate().is_ok());

    // Test invalid goal code
    let invalid_goal_oob =
        OutOfBandInvitation::builder("did:example:test", "invalid.goal", "Invalid goal").build();

    assert!(invalid_goal_oob.validate().is_err());

    // Test missing didcomm/v2 in accept
    let mut no_didcomm_oob =
        OutOfBandInvitation::builder("did:example:test", "tap.payment", "No DIDComm").build();

    no_didcomm_oob.body.accept = vec!["other/format".to_string()];
    assert!(no_didcomm_oob.validate().is_err());
}

#[test]
fn test_error_handling() {
    // Test various error conditions

    // Invalid URL
    let result = OutOfBandInvitation::from_url("not-a-url");
    assert!(result.is_err());

    // URL without _oob parameter
    let result = OutOfBandInvitation::from_url("https://example.com/pay?other=param");
    assert!(result.is_err());

    // Malformed base64
    let result = OutOfBandInvitation::from_url("https://example.com/pay?_oob=invalid-base64!");
    assert!(result.is_err());

    // PaymentLink error cases
    let result = PaymentLink::from_url("https://example.com/invalid");
    assert!(result.is_err());
}
