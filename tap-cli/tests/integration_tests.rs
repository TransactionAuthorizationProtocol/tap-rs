use serial_test::serial;
use std::path::PathBuf;
use tempfile::TempDir;

use tap_cli::error::Result;
use tap_cli::output::OutputFormat;
use tap_cli::tap_integration::TapIntegration;

/// Test environment with isolated temp directory
struct TestEnv {
    _temp_dir: TempDir,
    tap_root: PathBuf,
    _old_tap_home: Option<String>,
}

impl TestEnv {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let tap_root = temp_dir.path().join("tap");
        std::fs::create_dir_all(&tap_root)?;

        let old_tap_home = std::env::var("TAP_HOME").ok();
        std::env::set_var("TAP_HOME", &tap_root);

        Ok(Self {
            _temp_dir: temp_dir,
            tap_root,
            _old_tap_home: old_tap_home,
        })
    }

    async fn create_integration(&self) -> Result<TapIntegration> {
        TapIntegration::new_for_testing(
            Some(self.tap_root.to_str().unwrap()),
            "did:example:test-agent",
        )
        .await
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        if let Some(ref old_value) = self._old_tap_home {
            std::env::set_var("TAP_HOME", old_value);
        } else {
            std::env::remove_var("TAP_HOME");
        }
    }
}

#[tokio::test]
#[serial]
async fn test_tap_integration_initialization() {
    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();
    let agents = integration.list_agents().await.unwrap();
    // At least one test agent should be registered
    assert!(!agents.is_empty());
}

#[tokio::test]
#[serial]
async fn test_agent_list() {
    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();

    let agents = integration.list_agents().await.unwrap();
    assert!(!agents.is_empty());

    // Agents should have DIDs
    for agent in &agents {
        assert!(!agent.id.is_empty());
    }
}

#[tokio::test]
#[serial]
async fn test_output_format_json() {
    let format = "json".parse::<OutputFormat>().unwrap();
    assert_eq!(format, OutputFormat::Json);
}

#[tokio::test]
#[serial]
async fn test_output_format_text() {
    let format = "text".parse::<OutputFormat>().unwrap();
    assert_eq!(format, OutputFormat::Text);
}

#[tokio::test]
#[serial]
async fn test_output_format_invalid() {
    let result = "xml".parse::<OutputFormat>();
    assert!(result.is_err());
}

#[tokio::test]
#[serial]
async fn test_transaction_list_empty() {
    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();
    let agents = integration.list_agents().await.unwrap();
    let agent_did = &agents[0].id;

    let storage = integration.storage_for_agent(agent_did).await.unwrap();
    let messages = storage.list_messages(50, 0, None).await.unwrap();
    assert!(messages.is_empty());
}

#[tokio::test]
#[serial]
async fn test_transfer_message_creation() {
    use tap_caip::AssetId;
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::{Party, Transfer};

    let asset: AssetId = "eip155:1/slip44:60".parse().unwrap();
    let originator = Party::new("did:example:originator");
    let beneficiary = Party::new("did:example:beneficiary");

    let transfer = Transfer {
        transaction_id: None,
        asset,
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: "100.00".to_string(),
        agents: vec![],
        memo: Some("Test transfer".to_string()),
        settlement_id: None,
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    assert!(transfer.validate().is_ok());

    let didcomm = transfer.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_payment_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::{Party, Payment};

    let merchant = Party::new("did:example:merchant");
    let payment = Payment::with_currency("USD".to_string(), "50.00".to_string(), merchant, vec![]);

    assert!(payment.validate().is_ok());

    let didcomm = payment.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_authorize_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::Authorize;

    let authorize = Authorize {
        transaction_id: "test-tx-123".to_string(),
        settlement_address: Some("eip155:1:0x1234567890abcdef".to_string()),
        expiry: None,
    };

    assert!(authorize.validate().is_ok());

    let didcomm = authorize.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_reject_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::Reject;

    let reject = Reject {
        transaction_id: "test-tx-123".to_string(),
        reason: Some("Compliance issue".to_string()),
    };

    assert!(reject.validate().is_ok());

    let didcomm = reject.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_settle_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::Settle;

    let settle = Settle {
        transaction_id: "test-tx-123".to_string(),
        settlement_id: Some("eip155:1:tx/0xabc123".to_string()),
        amount: None,
    };

    assert!(settle.validate().is_ok());

    let didcomm = settle.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_trust_ping_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::TrustPing;

    let ping = TrustPing::new();
    assert!(ping.response_requested);

    let mut didcomm = ping.to_didcomm("did:example:sender").unwrap();
    didcomm.to = vec!["did:example:recipient".to_string()];
    assert!(!didcomm.id.is_empty());
    assert_eq!(didcomm.to[0], "did:example:recipient");
}

#[tokio::test]
#[serial]
async fn test_basic_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::BasicMessage;

    let msg = BasicMessage::new("Hello, TAP!".to_string());
    assert_eq!(msg.content, "Hello, TAP!");

    let mut didcomm = msg.to_didcomm("did:example:sender").unwrap();
    didcomm.to = vec!["did:example:recipient".to_string()];
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_customer_crud() {
    use tap_node::storage::models::{Customer, SchemaType};

    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();
    let agents = integration.list_agents().await.unwrap();
    let agent_did = &agents[0].id;

    let storage = integration.storage_for_agent(agent_did).await.unwrap();

    // Create customer
    let customer = Customer {
        id: "did:example:customer1".to_string(),
        agent_did: agent_did.clone(),
        schema_type: SchemaType::Person,
        given_name: Some("Alice".to_string()),
        family_name: Some("Smith".to_string()),
        display_name: Some("Alice Smith".to_string()),
        legal_name: None,
        lei_code: None,
        mcc_code: None,
        address_country: Some("US".to_string()),
        address_locality: Some("New York".to_string()),
        postal_code: None,
        street_address: None,
        profile: serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Person",
            "givenName": "Alice",
            "familyName": "Smith"
        }),
        ivms101_data: None,
        verified_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    storage.upsert_customer(&customer).await.unwrap();

    // Retrieve customer
    let retrieved = storage.get_customer("did:example:customer1").await.unwrap();
    assert!(retrieved.is_some());
    let c = retrieved.unwrap();
    assert_eq!(c.given_name, Some("Alice".to_string()));
    assert_eq!(c.address_country, Some("US".to_string()));

    // List customers
    let customers = storage.list_customers(agent_did, 50, 0).await.unwrap();
    assert_eq!(customers.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_received_messages_empty() {
    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();
    let agents = integration.list_agents().await.unwrap();
    let agent_did = &agents[0].id;

    let storage = integration.storage_for_agent(agent_did).await.unwrap();
    let received = storage.list_received(50, 0, None, None).await.unwrap();
    assert!(received.is_empty());
}

#[tokio::test]
#[serial]
async fn test_pending_received_empty() {
    let env = TestEnv::new().unwrap();
    let integration = env.create_integration().await.unwrap();
    let agents = integration.list_agents().await.unwrap();
    let agent_did = &agents[0].id;

    let storage = integration.storage_for_agent(agent_did).await.unwrap();
    let pending = storage.get_pending_received(50).await.unwrap();
    assert!(pending.is_empty());
}

#[tokio::test]
#[serial]
async fn test_connect_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::{Connect, ConnectionConstraints};

    let mut connect = Connect::new(
        "connect-123",
        "did:example:sender",
        "did:example:party",
        Some("SettlementAgent"),
    );

    connect.constraints = Some(ConnectionConstraints {
        purposes: None,
        category_purposes: None,
        limits: None,
    });

    assert!(connect.validate().is_ok());

    let mut didcomm = connect.to_didcomm("did:example:sender").unwrap();
    didcomm.to = vec!["did:example:recipient".to_string()];
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_cancel_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::Cancel;

    let cancel = Cancel {
        transaction_id: "test-tx-123".to_string(),
        by: "did:example:originator".to_string(),
        reason: Some("Changed my mind".to_string()),
    };

    assert!(cancel.validate().is_ok());

    let didcomm = cancel.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}

#[tokio::test]
#[serial]
async fn test_revert_message_creation() {
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::Revert;

    let revert = Revert {
        transaction_id: "test-tx-123".to_string(),
        settlement_address: "eip155:1:0xabc".to_string(),
        reason: "Fraud detected".to_string(),
    };

    assert!(revert.validate().is_ok());

    let didcomm = revert.to_didcomm("did:example:sender").unwrap();
    assert!(!didcomm.id.is_empty());
}
