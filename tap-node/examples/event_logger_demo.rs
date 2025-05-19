//! Example of using the TAP event logger

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tap_agent::crypto::DebugSecretsResolver;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_node::event::logger::{EventLogger, EventSubscriber};
use tap_node::event::Event;

/// Subscriber that prints events to the console
#[derive(Debug)]
struct ConsoleSubscriber;

impl EventSubscriber for ConsoleSubscriber {
    fn on_event(&self, event: &Event) {
        println!("Event: {:?}", event);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an event logger
    let event_logger = EventLogger::new();

    // Create an event subscriber
    let console_subscriber = ConsoleSubscriber;

    // Register the subscriber with the logger
    event_logger.register_subscriber(Box::new(console_subscriber));

    // Let's simulate some TAP events

    // Message received event
    let message_received = Event::MessageReceived {
        from: Some("did:example:alice".to_string()),
        to: "did:example:bob".to_string(),
        message_id: "msg-123".to_string(),
        message_type: "tap.transfer".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // Log the event
    event_logger.log_event(message_received);

    // Message sent event
    let message_sent = Event::MessageSent {
        from: "did:example:bob".to_string(),
        to: vec!["did:example:alice".to_string()],
        message_id: "msg-456".to_string(),
        message_type: "tap.transfer.reply".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // Log the event
    event_logger.log_event(message_sent);

    // Simulate setting up TAP agents with an event logger
    simulate_agent_setup(&event_logger);

    // Wait a bit to let the logs print
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}

/// Simulate setting up TAP agents with event logging
fn simulate_agent_setup(event_logger: &EventLogger) {
    // First, create mocked crypto components

    // TestDIDResolver - a mock DID resolver
    #[derive(Debug)]
    struct TestDIDResolver;

    // TestSecretsResolver - a mock secrets resolver
    #[derive(Debug)]
    struct TestSecretsResolver {
        secrets: HashMap<String, Secret>,
    }

    impl TestSecretsResolver {
        fn new() -> Self {
            let mut secrets = HashMap::new();

            // Add a test secret
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

            secrets.insert("did:example:alice".to_string(), secret);

            Self { secrets }
        }
    }

    impl DebugSecretsResolver for TestSecretsResolver {
        fn get_secret_by_id(&self, id: &str) -> Option<Secret> {
            self.secrets.get(id).cloned()
        }

        fn get_secrets_map(&self) -> &HashMap<String, Secret> {
            &self.secrets
        }
    }

    // In a real implementation, we would:
    // 1. Create a DID resolver
    let did_resolver = Arc::new(TestDIDResolver);

    // 2. Create a secrets resolver
    let secrets_resolver = Arc::new(TestSecretsResolver::new());

    // 3. Create a message packer
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
        true,
    ));

    // 4. Create an agent configuration
    let _config = tap_agent::config::AgentConfig::new("did:example:alice".to_string())
        .with_security_mode("SIGNED")
        .with_debug(true);

    // 5. Create an agent with the event logger
    // In a real implementation, we would:
    // let agent = tap_agent::agent::DefaultAgent::new(config, message_packer);

    // Log agent creation event
    event_logger.log_event(Event::AgentCreated {
        did: "did:example:alice".to_string(),
        timestamp: chrono::Utc::now(),
    });

    println!("Agent setup simulation completed");
}
