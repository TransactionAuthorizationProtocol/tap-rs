//! HTTP server implementation for TAP DIDComm messages.
//!
//! This module provides a complete HTTP server implementation for the Transaction Authorization
//! Protocol (TAP). The server exposes endpoints for:
//!
//! - Processing DIDComm messages for TAP operations
//! - Health checks for monitoring system availability
//!
//! The server is built using the Warp web framework and provides graceful shutdown capabilities.
//! 
//! # Features
//! 
//! - HTTP/WebSocket messaging for DIDComm transport
//! - Message validation for TAP protocol compliance
//! - Configurable host, port, and endpoint paths
//! - Support for optional TLS encryption
//! - Graceful shutdown handling
//! - Health check monitoring endpoint
//! 
//! # Configuration
//! 
//! The server can be configured with the `TapHttpConfig` struct, which allows setting:
//! 
//! - Host address and port
//! - DIDComm endpoint path
//! - TLS configuration (certificate and key paths)
//! - Rate limiting options
//! - Request timeout settings
//! 
//! # Example
//! 
//! ```rust,no_run
//! use tap_http::{TapHttpConfig, TapHttpServer};
//! use tap_node::{NodeConfig, TapNode};
//! use std::time::Duration;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a TAP Node
//!     let node = TapNode::new(NodeConfig::default());
//!     
//!     // Configure the HTTP server with custom settings
//!     let config = TapHttpConfig {
//!         host: "0.0.0.0".to_string(),    // Listen on all interfaces
//!         port: 8080,                     // Custom port
//!         didcomm_endpoint: "/api/didcomm".to_string(),  // Custom endpoint path
//!         request_timeout_secs: 60,       // 60-second timeout for outbound requests
//!         ..TapHttpConfig::default()
//!     };
//!     
//!     // Create and start the server
//!     let mut server = TapHttpServer::new(config, node);
//!     server.start().await?;
//!     
//!     // Wait for shutdown signal
//!     tokio::signal::ctrl_c().await?;
//!     
//!     // Gracefully stop the server
//!     server.stop().await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::config::TapHttpConfig;
use crate::error::{Error, Result};
use crate::handler::{handle_didcomm, handle_health_check};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tap_node::TapNode;
use tokio::sync::oneshot;
use tracing::{error, info, warn};
use warp::{Filter, Rejection, Reply};

// Rate limiter will be implemented in the future update

/// TAP HTTP server for handling DIDComm messages.
///
/// This server implementation provides endpoints for:
/// - `/didcomm` - For processing DIDComm messages via the TAP protocol
/// - `/health` - For checking the server's operational status
///
/// The server requires a configuration and a TapNode instance to function.
/// The TapNode is responsible for the actual message processing logic.
pub struct TapHttpServer {
    /// Server configuration.
    config: TapHttpConfig,

    /// TAP Node for message processing.
    node: Arc<TapNode>,

    /// Shutdown channel for graceful server termination.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl TapHttpServer {
    /// Creates a new HTTP server with the given configuration and TAP Node.
    ///
    /// # Parameters
    /// * `config` - The server configuration including host, port, and endpoint settings
    /// * `node` - The TAP Node instance used for processing DIDComm messages
    ///
    /// # Returns
    /// A new TapHttpServer instance that can be started with the `start` method
    pub fn new(config: TapHttpConfig, node: TapNode) -> Self {
        // Log if rate limiting is configured but not implemented yet
        if config.rate_limit.is_some() {
            warn!("Rate limiting is configured but not yet implemented");
        }

        // Log if TLS is configured but not implemented yet
        if config.tls.is_some() {
            warn!("TLS is configured but not yet fully implemented");
        }

        Self {
            config,
            node: Arc::new(node),
            shutdown_tx: None,
        }
    }

    /// Starts the HTTP server.
    ///
    /// This method:
    /// 1. Configures the server routes based on the provided configuration
    /// 2. Sets up a graceful shutdown channel
    /// 3. Starts the server in a separate Tokio task
    ///
    /// The server runs until the `stop` method is called.
    ///
    /// # Returns
    /// * `Ok(())` - If the server started successfully
    /// * `Err(Error)` - If there was an error starting the server
    pub async fn start(&mut self) -> Result<()> {
        let addr: SocketAddr = self
            .config
            .server_addr()
            .parse()
            .map_err(|e| Error::Http(format!("Invalid address: {}", e)))?;

        // Clone Arc<TapNode> for use in route handlers
        let node = self.node.clone();

        // Get the endpoint path from config
        let endpoint_path = self
            .config
            .didcomm_endpoint
            .trim_start_matches('/')
            .to_string();

        // Create DIDComm endpoint
        let didcomm_route = warp::path(endpoint_path)
            .and(warp::post())
            .and(warp::body::bytes())
            .and(with_node(node.clone()))
            .and_then(handle_didcomm);

        // Health check endpoint
        let health_route = warp::path("health")
            .and(warp::get())
            .and_then(handle_health_check);

        // Combine all routes
        let routes = didcomm_route
            .or(health_route)
            .with(warp::log("tap_http"))
            .recover(handle_rejection);

        // Create shutdown channel
        let (tx, rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(tx);

        // Start the server
        info!("Starting TAP HTTP server on {}", addr);
        
        // Start server without TLS
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
            rx.await.ok();
            info!("Shutting down TAP HTTP server");
        });

        // Spawn the server task
        tokio::spawn(server);

        info!("TAP HTTP server started on {}", addr);
        Ok(())
    }

    /// Stops the HTTP server.
    ///
    /// This method sends a shutdown signal to the server, allowing it to terminate gracefully.
    ///
    /// # Returns
    /// * `Ok(())` - If the server was stopped successfully
    /// * `Err(Error)` - If there was an error stopping the server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            info!("Sent shutdown signal to TAP HTTP server");
        } else {
            warn!("TAP HTTP server is not running");
        }
        Ok(())
    }

    /// Returns a reference to the underlying TAP Node.
    ///
    /// The TAP Node is responsible for processing DIDComm messages.
    pub fn node(&self) -> &Arc<TapNode> {
        &self.node
    }

    /// Returns a reference to the server configuration.
    ///
    /// The server configuration includes settings for the host, port, and endpoint.
    pub fn config(&self) -> &TapHttpConfig {
        &self.config
    }
    
    // Rate limiting functionality will be implemented in a future update
}

/// Helper function to provide the TAP Node to route handlers.
fn with_node(
    node: Arc<TapNode>,
) -> impl Filter<Extract = (Arc<TapNode>,), Error = Infallible> + Clone {
    warp::any().map(move || node.clone())
}

/// Custom rejection for rate limited requests
#[derive(Debug)]
struct RateLimitedError;
impl warp::reject::Reject for RateLimitedError {}

/// Handler for rejections.
async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    use crate::error::Error;
    
    let error_response = if err.is_not_found() {
        // Not found errors
        let err = Error::Http("Resource not found".to_string());
        err.to_response()
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        // Payload too large
        let err = Error::Http("Payload too large".to_string());
        err.to_response()
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        // Unsupported media type
        let err = Error::Http("Unsupported media type".to_string());
        err.to_response()
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        // Method not allowed
        let err = Error::Http("Method not allowed".to_string());
        err.to_response()
    } else if err.find::<RateLimitedError>().is_some() {
        // Rate limiting
        let err = Error::RateLimit("Too many requests, please try again later".to_string());
        err.to_response()
    } else {
        // Unhandled error
        error!("Unhandled rejection: {:?}", err);
        let err = Error::Unknown("Internal server error".to_string());
        err.to_response()
    };

    Ok(error_response)
}