//! Binary executable for the TAP HTTP server.

use base64::Engine;
use env_logger::Env;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tap_agent::storage::KeyStorage;
use tap_agent::{Agent, AgentConfig, Secret, SecretMaterial, SecretType, TapAgent};
use tap_http::event::{EventLoggerConfig, LogDestination};
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tracing::{debug, error, info, warn};

// For command line argument parsing
struct Args {
    host: String,
    port: u16,
    endpoint: String,
    timeout: u64,
    verbose: bool,
    agent_did: Option<String>,
    agent_key: Option<String>,
    logs_dir: Option<String>,
    structured_logs: bool,
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
            agent_did: args
                .opt_value_from_str("--agent-did")?
                .or_else(|| env::var("TAP_AGENT_DID").ok()),
            agent_key: args
                .opt_value_from_str("--agent-key")?
                .or_else(|| env::var("TAP_AGENT_KEY").ok()),
            logs_dir: args
                .opt_value_from_str("--logs-dir")?
                .or_else(|| env::var("TAP_LOGS_DIR").ok()),
            structured_logs: args.contains("--structured-logs")
                || env::var("TAP_STRUCTURED_LOGS").is_ok(),
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
    println!("    --agent-did <DID>            DID for the TAP agent (optional)");
    println!("    --agent-key <KEY>            Private key for the TAP agent (required if agent-did is provided)");
    println!("    --logs-dir <DIR>             Directory for event logs [default: ./logs]");
    println!("    --structured-logs            Use structured JSON logging [default: true]");
    println!("    -v, --verbose                Enable verbose logging");
    println!("    --help                       Print help information");
    println!("    --version                    Print version information");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    TAP_HTTP_HOST                Host to bind to");
    println!("    TAP_HTTP_PORT                Port to listen on");
    println!("    TAP_HTTP_DIDCOMM_ENDPOINT    Path for the DIDComm endpoint");
    println!("    TAP_HTTP_TIMEOUT             Request timeout in seconds");
    println!("    TAP_AGENT_DID                DID for the TAP agent");
    println!("    TAP_AGENT_KEY                Private key for the TAP agent");
    println!("    TAP_LOGS_DIR                 Directory for event logs");
    println!("    TAP_STRUCTURED_LOGS          Use structured JSON logging");
    println!();
    println!("NOTES:");
    println!("    - If no agent DID and key are provided, the server will:");
    println!("      1. Try to load keys from ~/.tap/keys.json (created by 'tap-agent-cli generate --save')");
    println!("      2. Fall back to an ephemeral agent if no stored keys exist");
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

    // Verify random number generator by creating two agents and comparing DIDs
    // Only in verbose mode to not spam normal output
    if args.verbose {
        let (_test_agent1, test_did1) = TapAgent::from_ephemeral_key().await?;
        let (_test_agent2, test_did2) = TapAgent::from_ephemeral_key().await?;
        info!("Test DID 1: {}", test_did1);
        info!("Test DID 2: {}", test_did2);
        if test_did1 == test_did2 {
            // This should never happen with proper randomness
            error!("WARNING: Generated identical DIDs! This indicates an issue with the random number generator.");
        } else {
            info!("Verified that agent DIDs are unique");
        }
    }

    // Create the actual agent
    let agent = TapAgent::from_stored_keys(None, true).await.unwrap();
    let agent_did = agent.get_agent_did();

    let agent_arc = Arc::new(agent);
    info!("Using agent with DID: {}", agent_did);

    // Print the DID to stdout for easy copying
    println!("TAP HTTP Server started with agent DID: {}", agent_did);

    // If using ephemeral agent, print a note that it's not persistent
    if args.agent_did.is_none() && args.agent_key.is_none() {
        match KeyStorage::default_key_path() {
            Some(path) if !path.exists() => {
                println!("Note: Using an ephemeral agent that won't persist across restarts.");
                println!("To create a persistent agent, run: `tap-agent-cli generate --save`");
                println!("Or provide --agent-did and --agent-key arguments");
            }
            _ => {}
        }
    }

    // Create config from parsed arguments
    let mut config = TapHttpConfig {
        host: args.host,
        port: args.port,
        didcomm_endpoint: args.endpoint,
        request_timeout_secs: args.timeout,
        rate_limit: None,
        tls: None,
        event_logger: None,
    };

    // Configure event logging
    let logs_dir = args.logs_dir.unwrap_or_else(|| "./logs".to_string());
    let log_path = PathBuf::from(&logs_dir).join("tap-http.log");

    config.event_logger = Some(EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path.to_string_lossy().to_string(),
            max_size: Some(10 * 1024 * 1024), // 10 MB
            rotate: true,
        },
        structured: args.structured_logs,
        log_level: tracing::Level::INFO,
    });

    // Log the configuration
    info!("Server configuration:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  DIDComm endpoint: {}", config.didcomm_endpoint);
    info!("  Request timeout: {} seconds", config.request_timeout_secs);
    info!("  Agent DID: {}", agent_did);
    debug!("  Event logging: {}", log_path.to_string_lossy());
    debug!("  Structured logs: {}", args.structured_logs);

    // Create node configuration with the agent
    let node_config = NodeConfig::default();
    // Register the agent after creating the node

    // Create TAP Node
    let node = TapNode::new(node_config);

    // Register the agent with the node
    if let Err(e) = node.register_agent(agent_arc.clone()).await {
        error!("Failed to register agent: {}", e);
        return Err(e.into());
    }

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
