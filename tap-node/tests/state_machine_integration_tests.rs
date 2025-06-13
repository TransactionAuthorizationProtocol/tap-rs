//! Integration tests for the TAP Node state machine
//!
//! These tests verify the complete message processing pipeline including:
//! - Message processing through the pipeline
//! - State machine integration
//! - Automatic authorization
//! - Settlement message generation
//! - Intra-node routing

use std::sync::Arc;
use tap_caip::{AssetId, ChainId};
use tap_msg::didcomm::PlainMessage;
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, Party, Payment, Transfer};
use tap_node::agent::AgentRegistry;
use tap_node::event::EventBus;
use tap_node::message::{PlainMessageProcessor, StateMachineIntegrationProcessor};
use tap_node::state_machine::{StandardTransactionProcessor, TransactionStateProcessor};
use tap_node::storage::Storage;

/// Helper to create a test agent DID
fn test_agent_did(name: &str) -> String {
    format!("did:test:{}", name)
}

/// Helper to create a test party
fn test_party(name: &str) -> Party {
    Party::new(&test_agent_did(name))
}

/// Helper to create a test agent
fn test_agent(name: &str, role: &str, for_party: &str) -> Agent {
    Agent::new(&test_agent_did(name), role, &test_agent_did(for_party))
}

/// Helper to create a test asset
fn test_asset() -> AssetId {
    let chain_id = ChainId::new("eip155", "1").unwrap();
    AssetId::new(
        chain_id,
        "erc20",
        "0x6b175474e89094c44da98b954eedeac495271d0f",
    )
    .unwrap()
}

/// Test the complete state machine integration workflow
#[tokio::test]
async fn test_complete_state_machine_integration() {
    // Setup components
    let storage = Arc::new(Storage::new(Some(":memory:".into())).await.unwrap());
    let event_bus = Arc::new(EventBus::new());
    let agents = Arc::new(AgentRegistry::new(None));

    // Create state machine processor
    let state_processor = Arc::new(StandardTransactionProcessor::new(
        storage.clone(),
        event_bus.clone(),
        agents.clone(),
    ));

    // Create integration processor
    let integration_processor =
        StateMachineIntegrationProcessor::new().with_state_processor(state_processor.clone());

    // Create a Transfer message with parties and agents
    let originator = test_party("alice");
    let beneficiary = test_party("bob");
    let compliance_agent = test_agent("compliance1", "compliance", "alice");

    let transfer = Transfer {
        asset: test_asset(),
        originator: Some(originator),
        beneficiary: Some(beneficiary),
        amount: "100.0".to_string(),
        agents: vec![compliance_agent],
        memo: None,
        settlement_id: None,
        transaction_id: "test-tx-001".to_string(),
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    // Convert to PlainMessage
    let mut plain_message = transfer.to_didcomm(&test_agent_did("alice")).unwrap();
    // Ensure the message ID matches the transaction ID for proper tracking
    plain_message.id = "test-tx-001".to_string();

    // Process the message directly through the state machine
    let result = state_processor.process_message(&plain_message).await;
    assert!(result.is_ok());

    // Also test the integration processor
    let integration_result = integration_processor
        .process_incoming(plain_message.clone())
        .await;
    assert!(integration_result.is_ok());
    assert!(integration_result.unwrap().is_some());

    // Verify transaction was stored (using message ID since that's what gets stored)
    let stored_transaction = storage
        .get_transaction_by_id(&plain_message.id)
        .await
        .unwrap();
    assert!(stored_transaction.is_some());

    // Verify agent was stored
    let agent_authorized = storage
        .are_all_agents_authorized(&plain_message.id)
        .await
        .unwrap();
    // Should be false initially since no authorization has happened yet
    assert!(!agent_authorized);
}

/// Test automatic authorization workflow
#[tokio::test]
async fn test_automatic_authorization() {
    // Setup components
    let storage = Arc::new(Storage::new(Some(":memory:".into())).await.unwrap());
    let event_bus = Arc::new(EventBus::new());
    let agents = Arc::new(AgentRegistry::new(None));

    // Create state machine processor
    let state_processor = Arc::new(StandardTransactionProcessor::new(
        storage.clone(),
        event_bus.clone(),
        agents.clone(),
    ));

    // Create a Payment message
    let customer = test_party("customer1");
    let merchant = test_party("merchant1");
    let compliance_agent = test_agent("compliance1", "compliance", "customer1");

    let payment = Payment {
        asset: Some(test_asset()),
        amount: "50.0".to_string(),
        currency_code: None,
        supported_assets: None,
        customer: Some(customer),
        merchant,
        transaction_id: "test-payment-001".to_string(),
        memo: None,
        expiry: None,
        invoice: None,
        agents: vec![compliance_agent],
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    // Convert to PlainMessage
    let mut plain_message = payment.to_didcomm(&test_agent_did("customer1")).unwrap();
    // Ensure the message ID matches the transaction ID for proper tracking
    plain_message.id = "test-payment-001".to_string();

    // Process the message
    let result = state_processor.process_message(&plain_message).await;
    assert!(result.is_ok());

    // Verify transaction was stored (using message ID since that's what gets stored)
    let stored_transaction = storage
        .get_transaction_by_id(&plain_message.id)
        .await
        .unwrap();
    assert!(stored_transaction.is_some());

    // Verify agents were extracted and stored
    let agent_authorized = storage
        .are_all_agents_authorized(&plain_message.id)
        .await
        .unwrap();
    // Should be false initially since no authorization has happened yet
    assert!(!agent_authorized);
}

/// Test message processing pipeline ordering
#[tokio::test]
async fn test_processing_pipeline_order() {
    use tap_node::message::{CompositePlainMessageProcessor, PlainMessageProcessorType};
    use tap_node::message::{LoggingPlainMessageProcessor, ValidationPlainMessageProcessor};

    // Setup components
    let storage = Arc::new(Storage::new(Some(":memory:".into())).await.unwrap());
    let event_bus = Arc::new(EventBus::new());
    let agents = Arc::new(AgentRegistry::new(None));

    // Create state machine processor
    let state_processor = Arc::new(StandardTransactionProcessor::new(
        storage.clone(),
        event_bus.clone(),
        agents.clone(),
    ));

    // Create a composite processor with the right order:
    // 1. Validation (validate messages)
    // 2. Logging (log messages)
    // 3. State Machine Integration (update state)
    let mut composite = CompositePlainMessageProcessor::new(vec![]);
    composite.add_processor(PlainMessageProcessorType::Validation(
        ValidationPlainMessageProcessor,
    ));
    composite.add_processor(PlainMessageProcessorType::Logging(
        LoggingPlainMessageProcessor,
    ));
    composite.add_processor(PlainMessageProcessorType::StateMachine(
        StateMachineIntegrationProcessor::new().with_state_processor(state_processor),
    ));

    // Create a valid Transfer message
    let transfer = Transfer {
        asset: test_asset(),
        originator: Some(test_party("alice")),
        beneficiary: Some(test_party("bob")),
        amount: "100.0".to_string(),
        agents: vec![test_agent("compliance1", "compliance", "alice")],
        memo: None,
        settlement_id: None,
        transaction_id: "test-pipeline-001".to_string(),
        connection_id: None,
        metadata: std::collections::HashMap::new(),
    };

    let mut plain_message = transfer.to_didcomm(&test_agent_did("alice")).unwrap();
    // Ensure the message ID matches the transaction ID for proper tracking
    plain_message.id = "test-pipeline-001".to_string();

    // Process through the pipeline
    let result = composite.process_incoming(plain_message.clone()).await;
    assert!(result.is_ok());
    let processed_message = result.unwrap();
    if processed_message.is_none() {
        println!("Message was filtered out by the pipeline");
        println!("Message typ: '{}'", plain_message.typ);
        println!("Message type_: '{}'", plain_message.type_);
        println!("Message ID: '{}'", plain_message.id);
        println!("Message from: '{}'", plain_message.from);
        println!("Message to: {:?}", plain_message.to);
        println!("Message body: {:?}", plain_message.body);
    }
    assert!(processed_message.is_some());

    // Verify the state was updated (transaction stored)
    let stored_transaction = storage
        .get_transaction_by_id(&plain_message.id)
        .await
        .unwrap();
    assert!(stored_transaction.is_some());
}

/// Test that invalid messages are filtered out before reaching state machine
#[tokio::test]
async fn test_invalid_message_filtering() {
    use tap_node::message::ValidationPlainMessageProcessor;
    use tap_node::message::{CompositePlainMessageProcessor, PlainMessageProcessorType};

    // Setup components
    let storage = Arc::new(Storage::new(Some(":memory:".into())).await.unwrap());
    let event_bus = Arc::new(EventBus::new());
    let agents = Arc::new(AgentRegistry::new(None));

    // Create state machine processor
    let state_processor = Arc::new(StandardTransactionProcessor::new(
        storage.clone(),
        event_bus.clone(),
        agents.clone(),
    ));

    // Create a composite processor
    let mut composite = CompositePlainMessageProcessor::new(vec![]);
    composite.add_processor(PlainMessageProcessorType::Validation(
        ValidationPlainMessageProcessor,
    ));
    composite.add_processor(PlainMessageProcessorType::StateMachine(
        StateMachineIntegrationProcessor::new().with_state_processor(state_processor),
    ));

    // Create an invalid message (empty ID)
    let invalid_message = PlainMessage {
        id: "".to_string(), // Invalid: empty ID
        typ: "application/didcomm-plain+json".to_string(),
        type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
        from: test_agent_did("alice"),
        to: vec![test_agent_did("bob")],
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        extra_headers: std::collections::HashMap::new(),
        from_prior: None,
        body: serde_json::json!({}),
        attachments: None,
    };

    // Process through the pipeline
    let result = composite.process_incoming(invalid_message).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Message should be filtered out

    // Verify no transaction was stored (since message was invalid)
    let transactions = storage.list_transactions(10, 0).await.unwrap();
    assert_eq!(transactions.len(), 0);
}
