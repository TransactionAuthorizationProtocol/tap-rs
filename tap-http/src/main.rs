//! Binary executable for the TAP HTTP server.

use env_logger::Env;
use std::env;
use std::error::Error;
use std::process;
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tracing::{error, info};

// For command line argument parsing
struct Args {
    host: String,
    port: u16,
    endpoint: String,
    timeout: u64,
    verbose: bool,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut args = pico_args::Arguments::from_env();

        // Check for help flag first
        if args.contains(["-h", "--help"]) {
            print_help();
            process::exit(0);
        }

        // Check for version flag
        if args.contains("--version") {
            println!("tap-http {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }

        let result = Args {
            host: args
                .opt_value_from_str(["-h", "--host"])?
                .unwrap_or_else(|| {
                    env::var("TAP_HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
                }),
            port: args
                .opt_value_from_str(["-p", "--port"])?
                .unwrap_or_else(|| {
                    env::var("TAP_HTTP_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(8000)
                }),
            endpoint: args
                .opt_value_from_str(["-e", "--endpoint"])?
                .unwrap_or_else(|| {
                    env::var("TAP_HTTP_DIDCOMM_ENDPOINT").unwrap_or_else(|_| "/didcomm".to_string())
                }),
            timeout: args
                .opt_value_from_str(["-t", "--timeout"])?
                .unwrap_or_else(|| {
                    env::var("TAP_HTTP_TIMEOUT")
                        .ok()
                        .and_then(|t| t.parse::<u64>().ok())
                        .unwrap_or(30)
                }),
            verbose: args.contains(["-v", "--verbose"]),
        };

        // Check for any remaining arguments (which would be invalid)
        let remaining = args.finish();
        if !remaining.is_empty() {
            return Err(format!("Unknown arguments: {:?}", remaining).into());
        }

        Ok(result)
    }
}

fn print_help() {
    println!("TAP HTTP Server");
    println!("---------------");
    println!("A HTTP server for the Transaction Authorization Protocol (TAP)");
    println!();
    println!("USAGE:");
    println!("    tap-http [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --host <HOST>            Host to bind to [default: 127.0.0.1]");
    println!("    -p, --port <PORT>            Port to listen on [default: 8000]");
    println!("    -e, --endpoint <ENDPOINT>    Path for the DIDComm endpoint [default: /didcomm]");
    println!("    -t, --timeout <SECONDS>      Request timeout in seconds [default: 30]");
    println!("    -v, --verbose                Enable verbose logging");
    println!("    --help                       Print help information");
    println!("    --version                    Print version information");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    TAP_HTTP_HOST                Host to bind to");
    println!("    TAP_HTTP_PORT                Port to listen on");
    println!("    TAP_HTTP_DIDCOMM_ENDPOINT    Path for the DIDComm endpoint");
    println!("    TAP_HTTP_TIMEOUT             Request timeout in seconds");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments first (to check for --verbose)
    let args = Args::parse().unwrap_or_else(|e| {
        eprintln!("Error parsing arguments: {}", e);
        process::exit(1);
    });

    // Initialize logging with appropriate level
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    info!("Starting TAP HTTP server");

    // Create config from parsed arguments
    let config = TapHttpConfig {
        host: args.host,
        port: args.port,
        didcomm_endpoint: args.endpoint,
        request_timeout_secs: args.timeout,
        rate_limit: None,
        tls: None,
    };

    // Log the configuration
    info!("Server configuration:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  DIDComm endpoint: {}", config.didcomm_endpoint);
    info!("  Request timeout: {} seconds", config.request_timeout_secs);

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

// Parse configuration is now handled by the Args struct
