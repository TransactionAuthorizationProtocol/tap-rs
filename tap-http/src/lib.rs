//! HTTP server implementation for the Transaction Authorization Protocol (TAP).
//!
//! This crate provides an HTTP server for handling DIDComm messages as part of the
//! Transaction Authorization Protocol (TAP). It leverages the new optimized message
//! routing architecture of TAP Node.
//!
//! # Architecture
//!
//! The HTTP server acts as a gateway between external clients and the TAP Node. It:
//!
//! - **Validates Security**: Ensures only signed or encrypted messages are accepted
//! - **Parses Messages**: Converts HTTP requests to JSON for TAP Node processing
//! - **Routes Efficiently**: Leverages TAP Node's optimized message routing
//! - **Provides Monitoring**: Health checks and event logging capabilities
//!
//! # Message Processing Flow
//!
//! 1. **HTTP Request**: Client sends POST to `/didcomm` with DIDComm message
//! 2. **Security Validation**: Content-Type header validated (must be signed or encrypted)
//! 3. **JSON Parsing**: Request body parsed as JSON Value
//! 4. **Node Processing**: JSON passed to TAP Node's `receive_message()`
//! 5. **Optimized Routing**: TAP Node handles verification/decryption and agent routing
//! 6. **HTTP Response**: Result returned to client
//!
//! # Security Features
//!
//! - **No Plain Messages**: Plain DIDComm messages are rejected for security
//! - **Content-Type Validation**: Strict validation of message security types
//! - **Event Logging**: All message processing events are logged for audit
//!
//! # Key Components
//!
//! - **Handler**: Request/response processing with validation
//! - **Server**: Warp-based HTTP server with configurable endpoints
//! - **Client**: HTTP client for outgoing message delivery
//! - **Event Bus**: Comprehensive event logging and monitoring
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use tap_http::{TapHttpConfig, TapHttpServer};
//! use tap_node::{TapNode, NodeConfig};
//! use tap_agent::TapAgent;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create TAP Node
//!     let node_config = NodeConfig::default();
//!     let mut node = TapNode::new(node_config);
//!     
//!     // Register an agent
//!     let (agent, _did) = TapAgent::from_ephemeral_key().await?;
//!     node.register_agent(Arc::new(agent)).await?;
//!
//!     // Configure and create the HTTP server
//!     let config = TapHttpConfig::default();
//!     let mut server = TapHttpServer::new(config, node);
//!
//!     // Start the server
//!     server.start().await?;
//!
//!     // Server now accepts signed and encrypted DIDComm messages
//!     // Messages are efficiently routed by the TAP Node
//!
//!     Ok(())
//! }
//! ```

// Public modules
pub mod client;
pub mod config;
pub mod error;
pub mod event;
pub mod handler;
pub mod server;

// Re-exports
pub use client::DIDCommClient;
pub use config::TapHttpConfig;
pub use error::{Error, Result};
pub use server::TapHttpServer;
