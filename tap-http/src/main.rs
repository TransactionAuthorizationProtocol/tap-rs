//! Binary executable for the TAP HTTP server.

use base64::Engine;
use env_logger::Env;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tap_agent::key_manager::{Secret, SecretMaterial, SecretType};
use tap_agent::DefaultAgent;
use tap_http::event::{EventLoggerConfig, LogDestination};
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_node::{NodeConfig, TapNode};
use tracing::{debug, error, info};

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
    println!("    --agent-did <DID>            DID for the TAP agent (optional, will create ephemeral if not provided)");
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
}

/// Create an agent for the server
///
/// This will either:
/// - Create an ephemeral agent with a new DID:key
/// - Use the provided agent DID and key
fn create_agent(
    agent_did: Option<String>,
    agent_key: Option<String>,
) -> Result<(DefaultAgent, String), Box<dyn Error>> {
    match (agent_did, agent_key) {
        (Some(_), None) => Err("Agent key must be provided when using a custom agent DID".into()),
        (None, Some(_)) => Err("Agent DID must be provided when using a custom agent key".into()),
        (Some(did), Some(key)) => {
            info!("Loading agent from provided DID and key");

            // First, validate the DID format
            if !did.starts_with("did:") {
                return Err("Invalid DID format. DID must start with 'did:'".into());
            }

            // Create a DID resolver
            let did_resolver = std::sync::Arc::new(tap_agent::did::MultiResolver::default());

            // Create a basic secret resolver for the key
            let mut secret_resolver = tap_agent::crypto::BasicSecretResolver::new();

            // Try to parse the key as a JWK first
            // This block is now handled directly in the add_secret call

            // Add the secret to the resolver
            secret_resolver.add_secret(
                &did,
                if key.trim().starts_with('{') {
                    // The key appears to be a JSON object, assume it's a JWK
                    info!("Using JWK format key");

                    // Parse JWK
                    let jwk: serde_json::Value = match serde_json::from_str(&key) {
                        Ok(jwk) => jwk,
                        Err(e) => return Err(format!("Failed to parse JWK: {}", e).into()),
                    };

                    // Create a secret from the JWK
                    let private_key_jwk = jwk.clone();

                    // Create a DIDComm secret
                    Secret {
                        type_: SecretType::JsonWebKey2020, // Use the correct variant for all key types
                        id: format!("{}#keys-1", did),
                        secret_material: SecretMaterial::JWK { private_key_jwk },
                    }
                } else if key.trim().contains(':') {
                    // The key might be a multibase encoded key
                    info!("Using multibase format key");

                    // Determine key type based on DID method
                    let key_type = if did.starts_with("did:key:") {
                        // did:key method, the key type is encoded in the key itself
                        tap_agent::did::KeyType::Ed25519 // Assume Ed25519 for now
                    } else {
                        // Determine from DID method or default to Ed25519
                        tap_agent::did::KeyType::Ed25519
                    };

                    // Create a private key from the multibase string
                    let (multicode_id, key_bytes) = match multibase::decode(key.trim()) {
                        Ok((id, bytes)) => (id, bytes),
                        Err(e) => {
                            return Err(format!("Failed to decode multibase key: {}", e).into())
                        }
                    };

                    // Convert to a secret format that DIDComm understands
                    // This will need to be customized based on the key format

                    // For Ed25519 keys
                    if key_type == tap_agent::did::KeyType::Ed25519 {
                        // Create a JWK from the key bytes
                        let private_key_jwk = serde_json::json!({
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "d": base64::engine::general_purpose::STANDARD.encode(&key_bytes),
                            "x": base64::engine::general_purpose::STANDARD.encode(&key_bytes[..32]), // First 32 bytes for Ed25519
                        });

                        // Create a DIDComm secret
                        Secret {
                            type_: SecretType::JsonWebKey2020,
                            id: format!("{}#keys-1", did),
                            secret_material: SecretMaterial::JWK { private_key_jwk },
                        }
                    } else {
                        return Err(format!(
                            "Unsupported key type for multibase key: {:?}",
                            multicode_id
                        )
                        .into());
                    }
                } else {
                    // Assume raw base64 format
                    info!("Using base64 format key");

                    // Determine key type based on DID method
                    let key_type = if did.starts_with("did:key:") {
                        // did:key method, the key type is encoded in the key itself
                        tap_agent::did::KeyType::Ed25519 // Assume Ed25519 for now
                    } else {
                        // Determine from DID method or default to Ed25519
                        tap_agent::did::KeyType::Ed25519
                    };

                    // Decode the base64 key
                    let key_bytes = match base64::engine::general_purpose::STANDARD
                        .decode(key.trim())
                    {
                        Ok(bytes) => bytes,
                        Err(e) => return Err(format!("Failed to decode base64 key: {}", e).into()),
                    };

                    // Create a JWK from the key bytes
                    let private_key_jwk = if key_type == tap_agent::did::KeyType::Ed25519 {
                        serde_json::json!({
                            "kty": "OKP",
                            "crv": "Ed25519",
                            "d": base64::engine::general_purpose::STANDARD.encode(&key_bytes),
                            "x": base64::engine::general_purpose::STANDARD.encode(&key_bytes[..32]), // Simplified, in reality Ed25519 public key is derived from private
                        })
                    } else {
                        return Err("Unsupported key type for base64 key".into());
                    };

                    // Create a DIDComm secret
                    Secret {
                        type_: SecretType::JsonWebKey2020,
                        id: format!("{}#keys-1", did),
                        secret_material: SecretMaterial::JWK { private_key_jwk },
                    }
                },
            );

            // Create a message packer
            let message_packer = tap_agent::crypto::DefaultMessagePacker::new(
                did_resolver,
                std::sync::Arc::new(secret_resolver),
                true, // debug mode
            );

            // Create agent configuration
            let config = tap_agent::config::AgentConfig {
                agent_did: did.clone(),
                parameters: std::collections::HashMap::new(),
                security_mode: Some("SIGNED".to_string()),
                debug: true,
                timeout_seconds: Some(30),
            };

            // Create the agent
            let agent = DefaultAgent::new(config, message_packer);

            Ok((agent, did))
        }
        (None, None) => {
            // Create an ephemeral agent
            info!("Creating ephemeral agent");
            let (agent, did) = tap_agent::agent::DefaultAgent::new_ephemeral()?;
            Ok((agent, did))
        }
    }
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
        let (_test_agent1, test_did1) = tap_agent::agent::DefaultAgent::new_ephemeral()?;
        let (_test_agent2, test_did2) = tap_agent::agent::DefaultAgent::new_ephemeral()?;
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
    let (agent, agent_did) = create_agent(args.agent_did.clone(), args.agent_key.clone())?;

    let agent_arc = Arc::new(agent);
    info!("Using agent with DID: {}", agent_did);

    // Print the DID to stdout for easy copying
    println!("TAP HTTP Server started with agent DID: {}", agent_did);

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
