use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tap_msg::didcomm::PlainMessage;
use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};
use tap_node::event::EventBus;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::test]
async fn test_console_logging() {
    // Test configuration for console logging
    let config = EventLoggerConfig {
        destination: LogDestination::Console,
        structured: false,
        log_level: log::Level::Info,
    };

    // Create the event logger
    let logger = Arc::new(EventLogger::new(config));

    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Subscribe the logger to events
    event_bus.subscribe(logger).await;

    // Publish a test event - this would appear in console logs
    // We can only verify this doesn't panic, as console output is not captured by the test
    event_bus
        .publish_agent_registered("did:example:test123".to_string())
        .await;

    // Give the event bus time to process the event
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_file_logging() {
    // Create a temporary file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("test_log.txt");
    let log_path_str = log_path.to_str().unwrap().to_string();

    // Test configuration for file logging
    let config = EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path_str.clone(),
            max_size: None,
            rotate: false,
        },
        structured: false,
        log_level: log::Level::Info,
    };

    // Create the event logger
    let logger = Arc::new(EventLogger::new(config));

    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Subscribe the logger to events
    event_bus.subscribe(logger).await;

    // Publish a test event
    event_bus
        .publish_agent_registered("did:example:test123".to_string())
        .await;

    // Give the event bus time to process the event
    sleep(Duration::from_millis(100)).await;

    // Verify the log file was created and contains the expected data
    assert!(Path::new(&log_path_str).exists());
    let log_content = fs::read_to_string(&log_path_str).unwrap();
    assert!(log_content.contains("AGENT REGISTERED: did:example:test123"));
}

#[tokio::test]
async fn test_structured_json_logging() {
    // Create a temporary file for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let log_path = temp_dir.path().join("test_structured_log.json");
    let log_path_str = log_path.to_str().unwrap().to_string();

    // Test configuration for structured JSON file logging
    let config = EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path_str.clone(),
            max_size: None,
            rotate: false,
        },
        structured: true,
        log_level: log::Level::Info,
    };

    // Create the event logger
    let logger = Arc::new(EventLogger::new(config));

    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Subscribe the logger to events
    event_bus.subscribe(logger).await;

    // Publish a test event
    event_bus
        .publish_agent_registered("did:example:test123".to_string())
        .await;

    // Give the event bus time to process the event
    sleep(Duration::from_millis(100)).await;

    // Verify the log file was created and contains valid JSON
    assert!(Path::new(&log_path_str).exists());
    let log_content = fs::read_to_string(&log_path_str).unwrap();
    let log_json: Value = serde_json::from_str(&log_content).unwrap();

    // Check that the JSON structure is correct
    assert_eq!(log_json["event_type"], "agent_registered");
    assert_eq!(log_json["data"]["did"], "did:example:test123");
    assert!(log_json["timestamp"].is_string());
}

#[tokio::test]
async fn test_custom_logging() {
    // Create a shared container for log messages
    let log_messages = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_messages_clone = log_messages.clone();

    // Create a custom logging function
    let custom_logger = Arc::new(move |msg: &str| {
        let log_messages = log_messages.clone();
        let msg = msg.to_string();
        tokio::spawn(async move {
            let mut logs = log_messages.lock().await;
            logs.push(msg);
        });
    });

    // Test configuration for custom logging
    let config = EventLoggerConfig {
        destination: LogDestination::Custom(custom_logger),
        structured: false,
        log_level: log::Level::Info,
    };

    // Create the event logger
    let logger = Arc::new(EventLogger::new(config));

    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Subscribe the logger to events
    event_bus.subscribe(logger).await;

    // Publish various test events
    event_bus
        .publish_agent_registered("did:example:agent1".to_string())
        .await;
    event_bus
        .publish_agent_unregistered("did:example:agent2".to_string())
        .await;
    event_bus
        .publish_did_resolved("did:example:agent3".to_string(), true)
        .await;

    // Give the event bus time to process the events
    sleep(Duration::from_millis(200)).await;

    // Verify the custom logger received all events
    let logs = log_messages_clone.lock().await;
    assert_eq!(logs.len(), 3);
    assert!(logs[0].contains("AGENT REGISTERED: did:example:agent1"));
    assert!(logs[1].contains("AGENT UNREGISTERED: did:example:agent2"));
    assert!(logs[2].contains("DID RESOLVED: did=did:example:agent3, success=true"));
}

// Test all event types with both structured and plain logging
#[tokio::test]
async fn test_all_event_types() {
    // Create a shared container for log messages
    let log_messages = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_messages_clone = log_messages.clone();

    // Create a custom logging function
    let custom_logger = Arc::new(move |msg: &str| {
        let log_messages = log_messages.clone();
        let msg = msg.to_string();
        tokio::spawn(async move {
            let mut logs = log_messages.lock().await;
            logs.push(msg);
        });
    });

    // Test configuration for custom logging with structured output
    let config = EventLoggerConfig {
        destination: LogDestination::Custom(custom_logger),
        structured: true,
        log_level: log::Level::Info,
    };

    // Create the event logger
    let logger = Arc::new(EventLogger::new(config));

    // Create an event bus
    let event_bus = Arc::new(EventBus::new());

    // Subscribe the logger to events
    event_bus.subscribe(logger).await;

    // Create a test message for testing message events
    let test_message = Message {
        id: "msg-123".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "test-message".to_string(),
        body: serde_json::json!("Hello world"),
        from: None,
        to: None,
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: std::collections::HashMap::new(),
    };

    // Publish events of each type
    event_bus
        .publish_message_received(test_message.clone())
        .await;

    event_bus
        .publish_message_sent(
            test_message.clone(),
            "did:example:sender".to_string(),
            "did:example:receiver".to_string(),
        )
        .await;

    event_bus
        .publish_agent_registered("did:example:agent1".to_string())
        .await;

    event_bus
        .publish_agent_unregistered("did:example:agent2".to_string())
        .await;

    event_bus
        .publish_did_resolved("did:example:agent3".to_string(), true)
        .await;

    event_bus
        .publish_agent_message("did:example:agent4".to_string(), vec![1, 2, 3, 4])
        .await;

    // Give the event bus time to process all events
    sleep(Duration::from_millis(200)).await;

    // Verify all events were logged
    let logs = log_messages_clone.lock().await;
    assert_eq!(logs.len(), 6); // We published 6 different events

    // Parse each log entry to verify structured logging works for all event types
    for log in logs.iter() {
        // Each log should be valid JSON
        let parsed: Value = serde_json::from_str(log).unwrap();

        // Verify common structure
        assert!(parsed["timestamp"].is_string());
        assert!(parsed["event_type"].is_string());
        assert!(parsed["data"].is_object());

        // Check each event type
        match parsed["event_type"].as_str().unwrap() {
            "message_received" => {
                assert!(parsed["data"]["message"].is_object());
            }
            "message_sent" => {
                assert!(parsed["data"]["message"].is_object());
                assert_eq!(parsed["data"]["from"], "did:example:sender");
                assert_eq!(parsed["data"]["to"], "did:example:receiver");
            }
            "agent_registered" => {
                assert_eq!(parsed["data"]["did"], "did:example:agent1");
            }
            "agent_unregistered" => {
                assert_eq!(parsed["data"]["did"], "did:example:agent2");
            }
            "did_resolved" => {
                assert_eq!(parsed["data"]["did"], "did:example:agent3");
                assert_eq!(parsed["data"]["success"], true);
            }
            "agent_message" => {
                assert_eq!(parsed["data"]["did"], "did:example:agent4");
                assert_eq!(parsed["data"]["message_length"], 4);
            }
            _ => panic!("Unknown event type: {}", parsed["event_type"]),
        }
    }
}
