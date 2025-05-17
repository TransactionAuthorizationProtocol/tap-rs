//! HTTP server implementation for the Transaction Authorization Protocol (TAP).
//!
//! This crate provides a HTTP server for handling DIDComm messages as part of the
//! Transaction Authorization Protocol (TAP). It includes:
//!
//! - A Warp-based HTTP server with DIDComm and health check endpoints
//! - Request/response handling for DIDComm messages
//! - Integration with the TAP Node for message processing
//! - Outgoing message delivery via HTTP
//!
//! # Example
//!
//! ```rust,no_run
//! use tap_http::{TapHttpConfig, TapHttpServer};
//! use tap_node::{TapNode, NodeConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a TAP Node for message processing
//!     let node = TapNode::new(NodeConfig::default());
//!
//!     // Configure and create the HTTP server
//!     let config = TapHttpConfig::default();
//!     let mut server = TapHttpServer::new(config, node);
//!
//!     // Start the server
//!     server.start().await?;
//!
//!     // Wait for a shutdown signal
//!     tokio::signal::ctrl_c().await?;
//!
//!     // Gracefully shut down the server
//!     server.stop().await?;
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
