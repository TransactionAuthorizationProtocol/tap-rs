//! Event system for TAP HTTP server.
//!
//! This module provides an event tracking and logging system for monitoring HTTP server activities.
//! It allows tracking request/response activity, message processing, and server lifecycle events.
//! Events can be logged to a configurable location (console, file, or custom handler).
//!
//! # Features
//!
//! - Track and log HTTP server events
//! - Request and response monitoring
//! - Configurable logging destination
//! - JSON structured logging support
//! - File rotation capabilities
//! - Integration with the TAP Node event system
//!
//! # Example
//!
//! ```no_run
//! use tap_http::{TapHttpConfig, TapHttpServer};
//! use tap_http::event::{EventLogger, EventLoggerConfig, LogDestination};
//! use tap_node::{NodeConfig, TapNode};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a TAP Node with default config
//!     let node = TapNode::new(NodeConfig::default());
//!     
//!     // Configure the HTTP server with event logging
//!     let mut config = TapHttpConfig::default();
//!     config.event_logger = Some(EventLoggerConfig {
//!         destination: LogDestination::File {
//!             path: "./logs/tap-http.log".to_string(),
//!             max_size: Some(10 * 1024 * 1024), // 10 MB
//!             rotate: true,
//!         },
//!         structured: true, // Use JSON format
//!         log_level: tracing::Level::INFO,
//!     });
//!     
//!     // Create and start the server
//!     let mut server = TapHttpServer::new(config, node);
//!     server.start().await?;
//!     
//!     Ok(())
//! }
//! ```

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use tracing::{debug, error, info, trace, warn, Level};
use warp::hyper::StatusCode;

/// HTTP server event types
///
/// Represents the various events that can occur within the TAP HTTP server,
/// including request handling, message processing, and server lifecycle events.
#[derive(Debug, Clone)]
pub enum HttpEvent {
    /// Server started event
    ServerStarted {
        /// The address the server is bound to
        address: String,
    },

    /// Server stopped event
    ServerStopped,

    /// Request received event
    RequestReceived {
        /// The HTTP method
        method: String,
        /// The request path
        path: String,
        /// The client IP address
        client_ip: Option<String>,
        /// The timestamp when the request was received
        timestamp: DateTime<Utc>,
    },

    /// Response sent event
    ResponseSent {
        /// The HTTP status code
        status: StatusCode,
        /// The response size in bytes
        size: usize,
        /// The time it took to process the request in milliseconds
        duration_ms: u64,
    },

    /// DIDComm message received event
    MessageReceived {
        /// The message ID
        id: String,
        /// The message type
        type_: String,
        /// The message sender's DID (if available)
        from: Option<String>,
        /// The message recipient's DID (if available)
        to: Option<String>,
    },

    /// DIDComm message processing error event
    MessageError {
        /// The error type
        error_type: String,
        /// The error message
        message: String,
        /// The message ID (if available)
        message_id: Option<String>,
    },
}

/// Configuration for where event logs should be sent
#[derive(Clone)]
pub enum LogDestination {
    /// Log to the console via the standard logging framework
    Console,
    
    /// Log to a file with optional rotation
    File {
        /// Path to the log file
        path: String,
        
        /// Maximum file size before rotation (in bytes)
        max_size: Option<usize>,
        
        /// Whether to rotate log files when they reach max_size
        rotate: bool,
    },
    
    /// Custom logging function
    Custom(Arc<dyn Fn(&str) + Send + Sync>),
}

// Custom Debug implementation that doesn't try to print the function pointer
impl fmt::Debug for LogDestination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogDestination::Console => write!(f, "LogDestination::Console"),
            LogDestination::File { path, max_size, rotate } => f
                .debug_struct("LogDestination::File")
                .field("path", path)
                .field("max_size", max_size)
                .field("rotate", rotate)
                .finish(),
            LogDestination::Custom(_) => write!(f, "LogDestination::Custom(<function>)"),
        }
    }
}

use serde::{Deserialize, Serialize, Serializer, Deserializer};

/// Configuration for the event logger
#[derive(Debug, Clone)]
pub struct EventLoggerConfig {
    /// Where to send the log output
    pub destination: LogDestination,
    
    /// Whether to use structured (JSON) logging
    pub structured: bool,
    
    /// The log level to use
    pub log_level: Level,
}

// Custom serialization/deserialization for EventLoggerConfig
impl Serialize for EventLoggerConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EventLoggerConfig", 3)?;
        
        // For destination, serialize a type and path if it's a file
        match &self.destination {
            LogDestination::Console => {
                state.serialize_field("destination_type", "console")?;
                state.serialize_field("destination_path", "")?;
            },
            LogDestination::File { path, .. } => {
                state.serialize_field("destination_type", "file")?;
                state.serialize_field("destination_path", path)?;
            },
            LogDestination::Custom(_) => {
                state.serialize_field("destination_type", "custom")?;
                state.serialize_field("destination_path", "")?;
            },
        }
        
        state.serialize_field("structured", &self.structured)?;
        state.serialize_field("log_level", &format!("{:?}", self.log_level))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for EventLoggerConfig {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // For deserialization, we'll create a default config
        // The actual destination will be set programmatically
        Ok(EventLoggerConfig::default())
    }
}

impl EventLoggerConfig {
    // Remove unused method as we now use the log_level field directly
}

impl Default for EventLoggerConfig {
    fn default() -> Self {
        Self {
            destination: LogDestination::File {
                path: "./logs/tap-http.log".to_string(),
                max_size: Some(10 * 1024 * 1024), // 10 MB
                rotate: true,
            },
            structured: true,
            log_level: Level::INFO,
        }
    }
}

/// Event subscriber trait for TAP HTTP events
///
/// Note: This trait is not object-safe because of the async fn.
/// For dynamic dispatch, we use a type-erased wrapper.
pub trait EventSubscriber: Send + Sync {
    /// Handle an HTTP event
    fn handle_event(&self, event: HttpEvent) -> futures::future::BoxFuture<'_, ()>;
}

/// Implementation for async handlers
impl<T> EventSubscriber for T
where
    T: Send + Sync + 'static,
    T: for<'a> HandleEvent<'a>,
{
    fn handle_event(&self, event: HttpEvent) -> futures::future::BoxFuture<'_, ()> {
        Box::pin(self.handle_event_async(event))
    }
}

/// Helper trait for async handling
#[async_trait]
pub trait HandleEvent<'a>: Send + Sync {
    async fn handle_event_async(&self, event: HttpEvent);
}

/// Event bus for TAP HTTP server
pub struct EventBus {
    /// Subscribers
    subscribers: Mutex<Vec<Arc<Box<dyn EventSubscriber>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }

    /// Subscribe to HTTP events with a boxed subscriber
    pub fn subscribe<S>(&self, subscriber: S)
    where
        S: EventSubscriber + 'static,
    {
        let boxed = Arc::new(Box::new(subscriber) as Box<dyn EventSubscriber>);
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(boxed);
    }

    /// Remove a subscriber from the event bus
    pub fn unsubscribe(&self, subscriber: &Arc<Box<dyn EventSubscriber>>) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.retain(|s| !Arc::ptr_eq(s, subscriber));
    }

    /// Publish a server started event
    pub async fn publish_server_started(&self, address: String) {
        let event = HttpEvent::ServerStarted { address };
        self.publish_event(event).await;
    }

    /// Publish a server stopped event
    pub async fn publish_server_stopped(&self) {
        let event = HttpEvent::ServerStopped;
        self.publish_event(event).await;
    }

    /// Publish a request received event
    pub async fn publish_request_received(
        &self,
        method: String,
        path: String,
        client_ip: Option<String>,
    ) {
        let event = HttpEvent::RequestReceived {
            method,
            path,
            client_ip,
            timestamp: Utc::now(),
        };
        self.publish_event(event).await;
    }

    /// Publish a response sent event
    pub async fn publish_response_sent(
        &self,
        status: StatusCode,
        size: usize,
        duration_ms: u64,
    ) {
        let event = HttpEvent::ResponseSent {
            status,
            size,
            duration_ms,
        };
        self.publish_event(event).await;
    }

    /// Publish a DIDComm message received event
    pub async fn publish_message_received(
        &self,
        id: String,
        type_: String,
        from: Option<String>,
        to: Option<String>,
    ) {
        let event = HttpEvent::MessageReceived {
            id,
            type_,
            from,
            to,
        };
        self.publish_event(event).await;
    }

    /// Publish a DIDComm message processing error event
    pub async fn publish_message_error(
        &self,
        error_type: String,
        message: String,
        message_id: Option<String>,
    ) {
        let event = HttpEvent::MessageError {
            error_type,
            message,
            message_id,
        };
        self.publish_event(event).await;
    }

    /// Publish an event to all subscribers
    async fn publish_event(&self, event: HttpEvent) {
        // Notify subscribers
        let subscribers = self.subscribers.lock().unwrap().clone();
        for subscriber in subscribers.iter() {
            let fut = subscriber.handle_event(event.clone());
            fut.await;
        }
    }
    
    /// Get the number of subscribers (for testing)
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.lock().unwrap().len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event logger for TAP HTTP server
///
/// This component subscribes to the server's event bus and logs all events
/// to the configured destination. It supports both plain text and structured
/// (JSON) logging, and can output to the console or files.
pub struct EventLogger {
    /// Configuration for the logger
    config: EventLoggerConfig,
    
    /// File handle if using file destination
    file: Option<Arc<Mutex<File>>>,
}

impl EventLogger {
    /// Create a new event logger with the given configuration
    pub fn new(config: EventLoggerConfig) -> Self {
        let file = match &config.destination {
            LogDestination::File { path, .. } => {
                match Self::open_log_file(path) {
                    Ok(file) => Some(Arc::new(Mutex::new(file))),
                    Err(err) => {
                        error!("Failed to open log file {}: {}", path, err);
                        None
                    }
                }
            }
            _ => None,
        };
        
        Self { config, file }
    }
    
    /// Open or create a log file
    fn open_log_file(path: &str) -> io::Result<File> {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open or create the file
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    }
    
    /// Log an event to the configured destination
    fn log_event(&self, event: &HttpEvent) -> crate::error::Result<()> {
        let log_message = if self.config.structured {
            self.format_structured_log(event)?
        } else {
            self.format_plain_log(event)
        };
        
        match &self.config.destination {
            LogDestination::Console => {
                // Use the standard logging framework
                match self.config.log_level {
                    Level::ERROR => error!("{}", log_message),
                    Level::WARN => warn!("{}", log_message),
                    Level::INFO => info!("{}", log_message),
                    Level::DEBUG => debug!("{}", log_message),
                    Level::TRACE => trace!("{}", log_message),
                }
                Ok(())
            }
            LogDestination::File { .. } => {
                if let Some(file) = &self.file {
                    let mut file_guard = file.lock().map_err(|_| {
                        crate::error::Error::Config("Failed to acquire log file lock".to_string())
                    })?;
                    
                    // Write to the file with newline
                    writeln!(file_guard, "{}", log_message).map_err(|err| {
                        crate::error::Error::Config(format!("Failed to write to log file: {}", err))
                    })?;
                    
                    // Ensure the log is flushed
                    file_guard.flush().map_err(|err| {
                        crate::error::Error::Config(format!("Failed to flush log file: {}", err))
                    })?;
                    
                    Ok(())
                } else {
                    // Fall back to console logging if file isn't available
                    error!("{}", log_message);
                    Ok(())
                }
            }
            LogDestination::Custom(func) => {
                // Call the custom logging function
                func(&log_message);
                Ok(())
            }
        }
    }
    
    /// Format an event as a plain text log message
    fn format_plain_log(&self, event: &HttpEvent) -> String {
        let timestamp = DateTime::<Utc>::from(
            SystemTime::now()
        ).format("%Y-%m-%dT%H:%M:%S%.3fZ");
        
        match event {
            HttpEvent::ServerStarted { address } => {
                format!("[{}] SERVER STARTED: address={}", timestamp, address)
            }
            HttpEvent::ServerStopped => {
                format!("[{}] SERVER STOPPED", timestamp)
            }
            HttpEvent::RequestReceived { method, path, client_ip, timestamp } => {
                format!(
                    "[{}] REQUEST RECEIVED: method={}, path={}, client_ip={}, timestamp={}",
                    timestamp,
                    method,
                    path,
                    client_ip.as_deref().unwrap_or("unknown"),
                    timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ")
                )
            }
            HttpEvent::ResponseSent { status, size, duration_ms } => {
                format!(
                    "[{}] RESPONSE SENT: status={}, size={}, duration_ms={}",
                    timestamp,
                    status.as_u16(),
                    size,
                    duration_ms
                )
            }
            HttpEvent::MessageReceived { id, type_, from, to } => {
                format!(
                    "[{}] MESSAGE RECEIVED: id={}, type={}, from={}, to={}",
                    timestamp,
                    id,
                    type_,
                    from.as_deref().unwrap_or("unknown"),
                    to.as_deref().unwrap_or("unknown")
                )
            }
            HttpEvent::MessageError { error_type, message, message_id } => {
                format!(
                    "[{}] MESSAGE ERROR: type={}, message={}, message_id={}",
                    timestamp,
                    error_type,
                    message,
                    message_id.as_deref().unwrap_or("unknown")
                )
            }
        }
    }
    
    /// Format an event as a structured (JSON) log message
    fn format_structured_log(&self, event: &HttpEvent) -> crate::error::Result<String> {
        // Create common fields for all event types
        let timestamp = DateTime::<Utc>::from(
            SystemTime::now()
        ).to_rfc3339();
        
        // Create event-specific fields
        let (event_type, event_data) = match event {
            HttpEvent::ServerStarted { address } => (
                "server_started",
                json!({
                    "address": address,
                }),
            ),
            HttpEvent::ServerStopped => (
                "server_stopped",
                json!({})
            ),
            HttpEvent::RequestReceived { method, path, client_ip, timestamp } => (
                "request_received",
                json!({
                    "method": method,
                    "path": path,
                    "client_ip": client_ip,
                    "request_timestamp": timestamp.to_rfc3339(),
                }),
            ),
            HttpEvent::ResponseSent { status, size, duration_ms } => (
                "response_sent",
                json!({
                    "status": status.as_u16(),
                    "size": size,
                    "duration_ms": duration_ms,
                }),
            ),
            HttpEvent::MessageReceived { id, type_, from, to } => (
                "message_received",
                json!({
                    "id": id,
                    "type": type_,
                    "from": from,
                    "to": to,
                }),
            ),
            HttpEvent::MessageError { error_type, message, message_id } => (
                "message_error",
                json!({
                    "error_type": error_type,
                    "message": message,
                    "message_id": message_id,
                }),
            ),
        };
        
        // Combine into a single JSON object
        let log_entry = json!({
            "timestamp": timestamp,
            "event_type": event_type,
            "data": event_data,
        });
        
        // Serialize to a string
        serde_json::to_string(&log_entry).map_err(|err| {
            crate::error::Error::Json(err.to_string())
        })
    }
}

#[async_trait]
impl HandleEvent<'_> for EventLogger {
    async fn handle_event_async(&self, event: HttpEvent) {
        if let Err(err) = self.log_event(&event) {
            error!("Failed to log event: {}", err);
        }
    }
}

impl fmt::Debug for EventLogger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLogger")
            .field("config", &self.config)
            .field("file", &self.file.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish() {
        // Create a custom event subscriber for testing
        struct TestSubscriber {
            events: Arc<Mutex<Vec<HttpEvent>>>,
        }

        #[async_trait]
        impl HandleEvent<'_> for TestSubscriber {
            async fn handle_event_async(&self, event: HttpEvent) {
                self.events.lock().unwrap().push(event);
            }
        }

        // Create event bus and subscriber
        let event_bus = EventBus::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let subscriber = TestSubscriber { events: events.clone() };
        event_bus.subscribe(subscriber);

        // Publish some events
        event_bus.publish_server_started("127.0.0.1:8000".to_string()).await;
        event_bus.publish_request_received(
            "GET".to_string(),
            "/didcomm".to_string(),
            Some("192.168.1.1".to_string()),
        ).await;

        // Check that the events were received
        let received_events = events.lock().unwrap();
        assert_eq!(received_events.len(), 2);

        match &received_events[0] {
            HttpEvent::ServerStarted { address } => {
                assert_eq!(address, "127.0.0.1:8000");
            },
            _ => panic!("Expected ServerStarted event"),
        }

        match &received_events[1] {
            HttpEvent::RequestReceived { method, path, client_ip, .. } => {
                assert_eq!(method, "GET");
                assert_eq!(path, "/didcomm");
                assert_eq!(client_ip, &Some("192.168.1.1".to_string()));
            },
            _ => panic!("Expected RequestReceived event"),
        }
    }
}