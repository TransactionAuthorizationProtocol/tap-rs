//! # Event Logger for TAP Node
//!
//! This module provides an event logging system that captures all node events
//! and logs them to a configurable location. It implements the `EventSubscriber`
//! trait to receive events via callbacks from the event bus.
//!
//! The event logger supports different output formats and destinations, including:
//! - Console logging via the standard logging framework
//! - File-based logging with rotation support
//! - Structured JSON logging for machine readability
//!
//! ## Usage
//!
//! ```no_run
//! use std::sync::Arc;
//! use tap_node::{NodeConfig, TapNode};
//! use tap_node::event::logger::{EventLogger, EventLoggerConfig, LogDestination};
//!
//! async fn example() {
//!     // Create a new TAP node
//!     let node = TapNode::new(NodeConfig::default());
//!
//!     // Configure the event logger
//!     let logger_config = EventLoggerConfig {
//!         destination: LogDestination::File {
//!             path: "/var/log/tap-node/events.log".to_string(),
//!             max_size: Some(10 * 1024 * 1024), // 10 MB
//!             rotate: true,
//!         },
//!         structured: true, // Use JSON format
//!         log_level: log::Level::Info,
//!     };
//!
//!     // Create and subscribe the event logger
//!     let event_logger = Arc::new(EventLogger::new(logger_config));
//!     node.get_event_bus().subscribe(event_logger).await;
//!
//!     // The event logger will now receive and log all events
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
use tracing::{debug, error, info, trace, warn};

use crate::error::{Error, Result};
use crate::event::{EventSubscriber, NodeEvent};

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
            LogDestination::File {
                path,
                max_size,
                rotate,
            } => f
                .debug_struct("LogDestination::File")
                .field("path", path)
                .field("max_size", max_size)
                .field("rotate", rotate)
                .finish(),
            LogDestination::Custom(_) => write!(f, "LogDestination::Custom(<function>)"),
        }
    }
}

/// Configuration for the event logger
#[derive(Debug, Clone)]
pub struct EventLoggerConfig {
    /// Where to send the log output
    pub destination: LogDestination,

    /// Whether to use structured (JSON) logging
    pub structured: bool,

    /// The log level to use
    pub log_level: log::Level,
}

impl Default for EventLoggerConfig {
    fn default() -> Self {
        Self {
            destination: LogDestination::Console,
            structured: false,
            log_level: log::Level::Info,
        }
    }
}

/// Event logger for TAP Node
///
/// This component subscribes to the node's event bus and logs all events
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
            LogDestination::File { path, .. } => match Self::open_log_file(path) {
                Ok(file) => Some(Arc::new(Mutex::new(file))),
                Err(err) => {
                    error!("Failed to open log file {}: {}", path, err);
                    None
                }
            },
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
        OpenOptions::new().create(true).append(true).open(path)
    }

    /// Log an event to the configured destination
    fn log_event(&self, event: &NodeEvent) -> Result<()> {
        let log_message = if self.config.structured {
            self.format_structured_log(event)?
        } else {
            self.format_plain_log(event)
        };

        match &self.config.destination {
            LogDestination::Console => {
                // Use the standard logging framework
                match self.config.log_level {
                    log::Level::Error => error!("{}", log_message),
                    log::Level::Warn => warn!("{}", log_message),
                    log::Level::Info => info!("{}", log_message),
                    log::Level::Debug => debug!("{}", log_message),
                    log::Level::Trace => trace!("{}", log_message),
                }
                Ok(())
            }
            LogDestination::File { .. } => {
                if let Some(file) = &self.file {
                    let mut file_guard = file.lock().map_err(|_| {
                        Error::Configuration("Failed to acquire log file lock".to_string())
                    })?;

                    // Write to the file with newline
                    writeln!(file_guard, "{}", log_message).map_err(|err| {
                        Error::Configuration(format!("Failed to write to log file: {}", err))
                    })?;

                    // Ensure the log is flushed
                    file_guard.flush().map_err(|err| {
                        Error::Configuration(format!("Failed to flush log file: {}", err))
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
    fn format_plain_log(&self, event: &NodeEvent) -> String {
        let timestamp = DateTime::<Utc>::from(SystemTime::now()).format("%Y-%m-%dT%H:%M:%S%.3fZ");

        match event {
            NodeEvent::PlainMessageReceived { message } => {
                format!("[{}] MESSAGE RECEIVED: {}", timestamp, message)
            }
            NodeEvent::PlainMessageSent { message, from, to } => {
                format!(
                    "[{}] MESSAGE SENT: from={}, to={}, message={}",
                    timestamp, from, to, message
                )
            }
            NodeEvent::AgentRegistered { did } => {
                format!("[{}] AGENT REGISTERED: {}", timestamp, did)
            }
            NodeEvent::AgentUnregistered { did } => {
                format!("[{}] AGENT UNREGISTERED: {}", timestamp, did)
            }
            NodeEvent::DidResolved { did, success } => {
                format!(
                    "[{}] DID RESOLVED: did={}, success={}",
                    timestamp, did, success
                )
            }
            NodeEvent::AgentPlainMessage { did, message } => {
                format!(
                    "[{}] AGENT MESSAGE: did={}, message_length={}",
                    timestamp,
                    did,
                    message.len()
                )
            }
        }
    }

    /// Format an event as a structured (JSON) log message
    fn format_structured_log(&self, event: &NodeEvent) -> Result<String> {
        // Create common fields for all event types
        let timestamp = DateTime::<Utc>::from(SystemTime::now()).to_rfc3339();

        // Create event-specific fields
        let (event_type, event_data) = match event {
            NodeEvent::PlainMessageReceived { message } => (
                "message_received",
                json!({
                    "message": message,
                }),
            ),
            NodeEvent::PlainMessageSent { message, from, to } => (
                "message_sent",
                json!({
                    "from": from,
                    "to": to,
                    "message": message,
                }),
            ),
            NodeEvent::AgentRegistered { did } => (
                "agent_registered",
                json!({
                    "did": did,
                }),
            ),
            NodeEvent::AgentUnregistered { did } => (
                "agent_unregistered",
                json!({
                    "did": did,
                }),
            ),
            NodeEvent::DidResolved { did, success } => (
                "did_resolved",
                json!({
                    "did": did,
                    "success": success,
                }),
            ),
            NodeEvent::AgentPlainMessage { did, message } => (
                "agent_message",
                json!({
                    "did": did,
                    "message_length": message.len(),
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
        serde_json::to_string(&log_entry).map_err(Error::Serialization)
    }
}

#[async_trait]
impl EventSubscriber for EventLogger {
    async fn handle_event(&self, event: NodeEvent) {
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
