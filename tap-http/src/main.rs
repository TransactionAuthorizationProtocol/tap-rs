//! Binary executable for the TAP HTTP server.

use env_logger::Env;
use tracing::{error, info};
use std::env;
use std::error::Error;
use std::process;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting TAP HTTP server");

    // Parse command line arguments or use default configuration
    let config = parse_config().unwrap_or_else(|e| {
        error!("Configuration error: {}", e);
        process::exit(1);
    });

    // Create TAP Node
    let node_config = NodeConfig::default();
    let node = TapNode::new(node_config);

    // Create and start HTTP server
    let mut server = TapHttpServer::new(config, node);
    if let Err(e) = server.start().await {
        error!("Failed to start server: {}", e);
        process::exit(1);
    }

    // Wait for Ctrl-C to shut down
    tokio::signal::ctrl_c().await?;
    info!("Ctrl-C received, shutting down");

    // Stop the server
    if let Err(e) = server.stop().await {
        error!("Error during shutdown: {}", e);
    }

    info!("Server shutdown complete");
    Ok(())
}

/// Parse configuration from environment or command line.
fn parse_config() -> Result<TapHttpConfig, Box<dyn Error>> {
    let port = env::var("TAP_HTTP_PORT")
        .map(|p| p.parse::<u16>())
        .unwrap_or(Ok(8000))?;

    let host = env::var("TAP_HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let didcomm_endpoint =
        env::var("TAP_HTTP_DIDCOMM_ENDPOINT").unwrap_or_else(|_| "/didcomm".to_string());

    Ok(TapHttpConfig {
        host,
        port,
        didcomm_endpoint,
        rate_limit: None,
        tls: None,
        request_timeout_secs: 30,
    })
}
