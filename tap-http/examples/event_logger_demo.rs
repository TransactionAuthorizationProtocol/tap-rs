//! Event Logger Demo for TAP HTTP Server
//!
//! This example demonstrates how to configure and use the TAP HTTP server with
//! event logging. It shows how to:
//!
//! 1. Configure the server with different event logging destinations
//! 2. Start the server and observe events being logged
//! 3. Use a custom event subscriber for specialized event handling

use std::time::Duration;

use async_trait::async_trait;
use tap_http::event::{EventLoggerConfig, HandleEvent, HttpEvent, LogDestination};
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tokio::time::sleep;
use tracing::{info, Level};

/// Custom event subscriber that prints events to the console
struct ConsoleEventSubscriber;

#[async_trait]
impl HandleEvent<'_> for ConsoleEventSubscriber {
    async fn handle_event_async(&self, event: HttpEvent) {
        info!("Custom event subscriber received event: {:?}", event);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting TAP HTTP server with event logging");

    // Create a TAP Node
    let node = TapNode::new(NodeConfig::default());

    // Configure the HTTP server with event logging to ./logs directory
    let mut config = TapHttpConfig::default();

    // Configure event logger to log to ./logs directory
    config.event_logger = Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: "./logs/tap-http.log".to_string(),
            max_size: Some(10 * 1024 * 1024), // 10 MB
            rotate: true,
        },
        structured: true, // Use JSON format
        log_level: Level::INFO,
    });

    // Create the HTTP server
    let mut server = TapHttpServer::new(config, node);

    // Add a custom event subscriber
    let custom_subscriber = ConsoleEventSubscriber;
    server.event_bus().subscribe(custom_subscriber);

    // Start the server
    server.start().await?;

    info!("Server started. Events are being logged to ./logs/tap-http.log");
    info!("Press Ctrl+C to stop the server...");

    // Simulate running for a while
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutting down server...");
        }
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Stop the server
    server.stop().await?;

    // Give time for shutdown events to be logged
    sleep(Duration::from_millis(100)).await;

    info!("Server stopped. Event logging demo completed.");

    Ok(())
}
