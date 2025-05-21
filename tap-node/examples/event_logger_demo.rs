//! Example of using the TAP event logger

use serde_json::json;
use std::sync::Arc;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_node::event::logger::EventLogger;
use tap_node::event::EventBus;
use tap_node::{EventSubscriber, NodeEvent};

/// Subscriber that prints events to the console
#[derive(Debug)]
struct ConsoleSubscriber;

#[async_trait::async_trait]
impl EventSubscriber for ConsoleSubscriber {
    async fn handle_event(&self, event: NodeEvent) {
        println!("Event: {:?}", event);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an event bus
    let event_bus = Arc::new(tap_node::event::EventBus::new());

    // Configure the event logger
    let logger_config = tap_node::event::logger::EventLoggerConfig {
        destination: tap_node::event::logger::LogDestination::Console,
        structured: false,
        log_level: log::Level::Info,
    };

    // Create an event logger
    let event_logger = Arc::new(EventLogger::new(logger_config));

    // Create an event subscriber
    let console_subscriber = Arc::new(ConsoleSubscriber);

    // Subscribe to the event bus
    event_bus.subscribe(console_subscriber).await;
    event_bus.subscribe(event_logger.clone()).await;

    // Let's simulate some TAP events

    // Use one of the provided event publishing methods
    event_bus
        .publish_agent_registered("did:example:alice".to_string())
        .await;

    // Create a DID resolved event
    event_bus
        .publish_did_resolved("did:example:bob".to_string(), true)
        .await;

    // Create an agent message event
    let message_bytes = serde_json::to_string(&json!({
        "id": "msg-456",
        "type": "tap.transfer.reply",
        "from": "did:example:bob",
        "to": "did:example:alice"
    }))
    .unwrap()
    .into_bytes();

    event_bus
        .publish_agent_message("did:example:bob".to_string(), message_bytes)
        .await;

    // Simulate setting up TAP agents with an event logger
    simulate_agent_setup(&event_logger).await;

    // Wait a bit to let the logs print
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}

/// Simulate setting up TAP agents with event logging
async fn simulate_agent_setup(event_logger: &Arc<EventLogger>) {
    // First, create mocked crypto components

    // TestDIDResolver - a mock DID resolver
    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestDIDResolver;

    // Replace TestDIDResolver with MultiResolver
    // which already implements SyncDIDResolver
    let _did_resolver = Arc::new(tap_agent::did::MultiResolver::default());

    // In a real implementation, we would:
    // 1. We already created a DID resolver above

    // 2. Create an agent key manager builder
    let mut key_manager_builder = tap_agent::agent_key_manager::AgentKeyManagerBuilder::new();

    // 3. Add a test secret
    let secret = Secret {
        id: "did:example:alice".to_string(),
        type_: SecretType::JsonWebKey2020,
        secret_material: SecretMaterial::JWK {
            private_key_jwk: serde_json::json!({
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "test1234",
                "d": "test1234"
            }),
        },
    };

    // 4. Add the secret to the builder
    key_manager_builder = key_manager_builder.add_secret("did:example:alice".to_string(), secret);

    // 5. Build the key manager
    let _agent_key_manager = key_manager_builder
        .build()
        .expect("Failed to build key manager");

    // 4. Create an agent configuration
    let _config = tap_agent::config::AgentConfig::new("did:example:alice".to_string())
        .with_security_mode("SIGNED")
        .with_debug(true);

    // 5. Create an agent with the event logger
    // In a real implementation, we would:
    // let agent = tap_agent::agent::DefaultAgent::new(config, message_packer);

    // Use the event bus to publish an agent registered event
    let event_bus = Arc::new(EventBus::new());
    event_bus.subscribe(event_logger.clone()).await;

    // Log agent creation by publishing an agent registered event
    event_bus
        .publish_agent_registered("did:example:alice".to_string())
        .await;

    println!("Agent setup simulation completed");
}
