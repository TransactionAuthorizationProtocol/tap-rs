use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tap_http::event::{EventLoggerConfig, HttpEvent, LogDestination};
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tempfile::tempdir;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::test(flavor = "multi_thread")]
async fn test_event_logging_config() {
    // Create a temporary directory for the log file
    let temp_dir = tempdir().unwrap();
    let log_path = temp_dir.path().join("test-events.log");
    let log_path_str = log_path.to_str().unwrap().to_string();

    // Create configuration with event logging
    let mut config = TapHttpConfig::default();
    config.event_logger = Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path_str.clone(),
            max_size: None,
            rotate: false,
        },
        structured: true,
        log_level: tracing::Level::INFO,
    });

    // Create a TapNode without storage for tests
    let mut node_config = NodeConfig::default();
    node_config.storage_path = None;
    let node = TapNode::new(node_config);

    // Create the HTTP server
    let server = TapHttpServer::new(config, node);

    // Verify the event logger was set up correctly
    assert!(server.event_bus().subscriber_count() > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_server_events() {
    // Create a custom subscriber to capture events
    struct TestSubscriber {
        events: Arc<Mutex<Vec<HttpEvent>>>,
    }

    // Implement the HandleEvent trait for TestSubscriber
    #[async_trait::async_trait]
    impl tap_http::event::HandleEvent<'_> for TestSubscriber {
        async fn handle_event_async(&self, event: HttpEvent) {
            self.events.lock().await.push(event);
        }
    }

    // Create a TapNode without storage for tests
    let mut node_config = NodeConfig::default();
    node_config.storage_path = None;
    let node = TapNode::new(node_config);

    // Create configuration (without file logging)
    let config = TapHttpConfig::default();

    // Create the HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Add our test subscriber
    let events = Arc::new(Mutex::new(Vec::new()));
    let subscriber = TestSubscriber {
        events: events.clone(),
    };
    server.event_bus().subscribe(subscriber);

    // Start the server
    server.start().await.unwrap();

    // Wait a moment for events to be processed
    sleep(Duration::from_millis(100)).await;

    // Stop the server
    server.stop().await.unwrap();

    // Wait a moment for events to be processed
    sleep(Duration::from_millis(100)).await;

    // Verify the events were captured
    let captured_events = events.lock().await;

    // We should have at least a server_started and server_stopped event
    assert!(captured_events.len() >= 2);

    // Verify first event is server_started
    match &captured_events[0] {
        HttpEvent::ServerStarted { address } => {
            assert!(address.starts_with("127.0.0.1:"));
        }
        _ => panic!("First event should be ServerStarted"),
    }

    // Verify last event is server_stopped
    match &captured_events[captured_events.len() - 1] {
        HttpEvent::ServerStopped => {}
        _ => panic!("Last event should be ServerStopped"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_json_event_logging() {
    // Create a temporary directory for the log file
    let temp_dir = tempdir().unwrap();
    let log_path = temp_dir.path().join("json-events.log");
    let log_path_str = log_path.to_str().unwrap().to_string();

    // Create configuration with JSON event logging and custom port
    let mut config = TapHttpConfig::default();
    // Use a different port to avoid conflicts
    config.port = 8001;
    config.event_logger = Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path_str.clone(),
            max_size: None,
            rotate: false,
        },
        structured: true,
        log_level: tracing::Level::INFO,
    });

    // Create a TapNode without storage for tests
    let mut node_config = NodeConfig::default();
    node_config.storage_path = None;
    let node = TapNode::new(node_config);

    // Create the HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Start the server
    server.start().await.unwrap();

    // Wait a moment for events to be processed
    sleep(Duration::from_millis(100)).await;

    // Stop the server
    server.stop().await.unwrap();

    // Wait a moment for events to be processed
    sleep(Duration::from_millis(100)).await;

    // Verify the log file was created
    assert!(Path::new(&log_path_str).exists());

    // Verify the log file contents
    let log_content = fs::read_to_string(&log_path_str).unwrap();

    // There should be at least one line in the log
    let log_lines: Vec<&str> = log_content.trim().split('\n').collect();
    assert!(!log_lines.is_empty());

    // Verify each line is valid JSON
    for line in log_lines {
        let json: Value = serde_json::from_str(line).unwrap();

        // Verify the JSON structure
        assert!(json["timestamp"].is_string());
        assert!(json["event_type"].is_string());
        assert!(json["data"].is_object());
    }
}
