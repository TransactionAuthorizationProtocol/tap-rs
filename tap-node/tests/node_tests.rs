//! Tests for the TAP Node

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::crypto::DebugSecretsResolver;
use tap_agent::key_manager::Secret;
use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};
use tap_node::event::EventBus;
use tap_node::{EventSubscriber, NodeEvent};

/// A simplified test secrets resolver
#[derive(Debug)]
#[allow(dead_code)]
struct TestSecretsResolver {
    secrets: HashMap<String, Secret>,
}

impl TestSecretsResolver {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
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

/// A test event subscriber that tracks events
#[derive(Debug, Default)]
struct TestEventSubscriber {
    count: std::sync::atomic::AtomicUsize,
}

#[async_trait::async_trait]
impl EventSubscriber for TestEventSubscriber {
    async fn handle_event(&self, _event: NodeEvent) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl TestEventSubscriber {
    fn get_count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Test that the event logger can be created and events can be logged
#[tokio::test]
async fn test_event_logger_creation() {
    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Create a logger config
    let logger_config = EventLoggerConfig {
        destination: LogDestination::Console,
        structured: false,
        log_level: log::Level::Info,
    };

    // Create an event logger
    let event_logger = Arc::new(EventLogger::new(logger_config));

    // Create a test subscriber
    let subscriber = Arc::new(TestEventSubscriber::default());

    // Register the subscribers with the event bus
    event_bus.subscribe(subscriber.clone()).await;
    event_bus.subscribe(event_logger.clone()).await;

    // Publish a message received event
    event_bus
        .publish_agent_registered("did:example:alice".to_string())
        .await;

    // Verify that the subscriber received the event
    assert_eq!(subscriber.get_count(), 1);
}

/// Test that multiple events can be logged
#[tokio::test]
async fn test_multiple_events() {
    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Create a logger config
    let logger_config = EventLoggerConfig {
        destination: LogDestination::Console,
        structured: false,
        log_level: log::Level::Info,
    };

    // Create an event logger
    let event_logger = Arc::new(EventLogger::new(logger_config));

    // Create a test subscriber
    let subscriber = Arc::new(TestEventSubscriber::default());

    // Register the subscribers with the event bus
    event_bus.subscribe(subscriber.clone()).await;
    event_bus.subscribe(event_logger.clone()).await;

    // Publish multiple events
    for i in 0..5 {
        // Create a test message
        let test_message = format!("Test message {}", i);

        // Publish a message
        event_bus
            .publish_agent_message(format!("did:example:bob{}", i), test_message.into_bytes())
            .await;
    }

    // Verify that the subscriber received all events
    assert_eq!(subscriber.get_count(), 5);
}
