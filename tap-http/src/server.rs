//! HTTP server implementation for TAP DIDComm messages.
//!
//! This module provides a complete HTTP server implementation for the Transaction Authorization
//! Protocol (TAP). The server exposes endpoints for:
//!
//! - Processing DIDComm messages for TAP operations
//! - Health checks for monitoring system availability
//!
//! The server is built using the Warp web framework and provides graceful shutdown capabilities.

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
}

/// Helper function to provide the TAP Node to route handlers.
fn with_node(
    node: Arc<TapNode>,
) -> impl Filter<Extract = (Arc<TapNode>,), Error = Infallible> + Clone {
    warp::any().map(move || node.clone())
}

/// Handler for rejections.
async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let message;
    let status;

    if err.is_not_found() {
        message = "Not Found";
        status = warp::http::StatusCode::NOT_FOUND;
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        message = "Payload too large";
        status = warp::http::StatusCode::PAYLOAD_TOO_LARGE;
    } else {
        error!("Unhandled rejection: {:?}", err);
        message = "Internal Server Error";
        status = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": message
        })),
        status,
    ))
}
