//! Tests for the TAP Node

use std::collections::HashMap;
use std::sync::Arc;
use tap_agent::crypto::DebugSecretsResolver;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_node::event::logger::{EventLogger, EventSubscriber};
use tap_node::event::Event;

/// A simplified test secrets resolver
#[derive(Debug)]
struct TestSecretsResolver {
    secrets: HashMap<String, Secret>,
}

impl TestSecretsResolver {
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

impl EventSubscriber for TestEventSubscriber {
    fn on_event(&self, _event: &Event) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl TestEventSubscriber {
    fn get_count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Test that the event logger can be created and events can be logged
#[test]
fn test_event_logger_creation() {
    // Create an event logger
    let event_logger = EventLogger::new();

    // Create a test subscriber
    let subscriber = Arc::new(TestEventSubscriber::default());
    let subscriber_clone = subscriber.clone();

    // Register the subscriber
    event_logger.register_subscriber(Box::new(move |event| {
        subscriber_clone.on_event(event);
    }));

    // Log an event
    event_logger.log_event(Event::MessageReceived {
        from: Some("did:example:alice".to_string()),
        to: "did:example:bob".to_string(),
        message_id: "msg-123".to_string(),
        message_type: "test.message".to_string(),
        timestamp: chrono::Utc::now(),
    });

    // Verify that the subscriber received the event
    assert_eq!(subscriber.get_count(), 1);
}

/// Test that multiple events can be logged
#[test]
fn test_multiple_events() {
    // Create an event logger
    let event_logger = EventLogger::new();

    // Create a test subscriber
    let subscriber = Arc::new(TestEventSubscriber::default());
    let subscriber_clone = subscriber.clone();

    // Register the subscriber
    event_logger.register_subscriber(Box::new(move |event| {
        subscriber_clone.on_event(event);
    }));

    // Log multiple events
    for i in 0..5 {
        event_logger.log_event(Event::MessageSent {
            from: "did:example:alice".to_string(),
            to: vec!["did:example:bob".to_string()],
            message_id: format!("msg-{}", i),
            message_type: "test.message".to_string(),
            timestamp: chrono::Utc::now(),
        });
    }

    // Verify that the subscriber received all events
    assert_eq!(subscriber.get_count(), 5);
}
