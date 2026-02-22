//! Binary executable for the TAP HTTP server.

use env_logger::Env;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use tap_agent::agent_key_manager::AgentKeyManagerBuilder;
use tap_agent::config::AgentConfig;
use tap_agent::did::{DIDGenerationOptions, KeyType};
use tap_agent::key_manager::KeyManager;
use tap_agent::storage::KeyStorage;
use tap_agent::Agent;
use tap_agent::TapAgent;
use tap_http::event::{EventLoggerConfig, LogDestination};
use tap_http::external_decision::{ExternalDecisionConfig, ExternalDecisionManager, SubscribeMode};
use tap_http::{TapHttpConfig, TapHttpServer};
use tap_mcp::tap_integration::TapIntegration;
use tap_mcp::tools::ToolRegistry;
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
    db_path: Option<String>,
    tap_root: Option<String>,
    enable_web_did: bool,
    decision_mode: String,
    decision_exec: Option<String>,
    decision_exec_args: Vec<String>,
    decision_subscribe: String,
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
            db_path: args
                .opt_value_from_str("--db-path")?
                .or_else(|| env::var("TAP_NODE_DB_PATH").ok()),
            tap_root: args
                .opt_value_from_str("--tap-root")?
                .or_else(|| env::var("TAP_ROOT").ok()),
            enable_web_did: args.contains("--enable-web-did")
                || env::var("TAP_ENABLE_WEB_DID").is_ok(),
            decision_mode: {
                let raw: Option<String> = args.opt_value_from_str(["-M", "--decision-mode"])?;
                raw.unwrap_or_else(|| {
                    env::var("TAP_DECISION_MODE").unwrap_or_else(|_| "auto".to_string())
                })
            },
            decision_exec: args
                .opt_value_from_str(["-D", "--decision-exec"])?
                .or_else(|| env::var("TAP_DECISION_EXEC").ok()),
            decision_exec_args: {
                let raw: Option<String> =
                    args.opt_value_from_str(["-A", "--decision-exec-args"])?;
                raw.or_else(|| env::var("TAP_DECISION_EXEC_ARGS").ok())
                    .map(|s| s.split(',').map(|a| a.trim().to_string()).collect())
                    .unwrap_or_default()
            },
            decision_subscribe: {
                let raw: Option<String> =
                    args.opt_value_from_str(["-S", "--decision-subscribe"])?;
                raw.unwrap_or_else(|| {
                    env::var("TAP_DECISION_SUBSCRIBE").unwrap_or_else(|_| "decisions".to_string())
                })
            },
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
    println!(
        "\
TAP HTTP Server v{}
A HTTP server for the Transaction Authorization Protocol (TAP).

Receives DIDComm v2 messages over HTTP, processes them through the TAP
transaction state machine, and routes decisions to the configured handler.
All agent data (transactions, messages, decisions) is stored in per-agent
SQLite databases under the TAP root directory.

USAGE:
    tap-http [OPTIONS]

SERVER OPTIONS:
    -h, --host <HOST>              Host to bind to [default: 127.0.0.1]
    -p, --port <PORT>              Port to listen on [default: 8000]
    -e, --endpoint <ENDPOINT>      DIDComm endpoint path [default: /didcomm]
    -t, --timeout <SECONDS>        Request timeout in seconds [default: 30]
    -v, --verbose                  Enable verbose logging
    --structured-logs              Use structured JSON logging
    --enable-web-did               Serve /.well-known/did.json for did:web hosting

AGENT OPTIONS:
    --agent-did <DID>              DID for the TAP agent (auto-generated if omitted)
    --agent-key <KEY>              Private key for the TAP agent
    --tap-root <DIR>               TAP root directory [default: ~/.tap]
    --db-path <PATH>               Database file path (overrides per-agent default)
    --logs-dir <DIR>               Event log directory [default: ~/.tap/logs]

DECISION OPTIONS:
    -M, --decision-mode <MODE>     Decision handling mode [default: auto]
                                   Modes:
                                     auto  - Automatically approve all decisions
                                     poll  - Log decisions to DB, wait for external resolution
                                     exec  - Spawn external process (implied by -D)
    -D, --decision-exec <PATH>     Path to external decision executable
    -A, --decision-exec-args <ARGS>  Comma-separated arguments for the executable
    -S, --decision-subscribe <MODE>  What to forward to the executable [default: decisions]
                                     decisions - Only decision points
                                     all       - All events + decision points

    --help                         Print this help information
    --version                      Print version information

ENVIRONMENT VARIABLES:
    TAP_HTTP_HOST                  Host to bind to
    TAP_HTTP_PORT                  Port to listen on
    TAP_HTTP_DIDCOMM_ENDPOINT      DIDComm endpoint path
    TAP_HTTP_TIMEOUT               Request timeout in seconds
    TAP_AGENT_DID                  DID for the TAP agent
    TAP_AGENT_KEY                  Private key for the TAP agent
    TAP_ROOT                       TAP root directory
    TAP_NODE_DB_PATH               Database file path
    TAP_LOGS_DIR                   Event log directory
    TAP_STRUCTURED_LOGS            Enable structured JSON logging (set to any value)
    TAP_ENABLE_WEB_DID             Enable did:web endpoint (set to any value)
    TAP_DECISION_MODE              Decision handling: auto, poll, or exec
    TAP_DECISION_EXEC              Path to external decision executable
    TAP_DECISION_EXEC_ARGS         Comma-separated arguments
    TAP_DECISION_SUBSCRIBE         Event forwarding: decisions or all

DECISION MODES:

  Auto Mode (default):
    All decisions are automatically approved. No external process needed.
      tap-http --decision-mode auto

  Poll Mode:
    Decisions are logged to the decision_log table in the per-agent SQLite DB.
    An external process (e.g., tap-mcp connected to an AI agent, or tap-cli)
    polls the DB and resolves decisions using MCP tools or CLI commands.
      tap-http --decision-mode poll

    Workflow:
      1. tap-http receives a Transfer and hits a decision point
      2. Decision is written to decision_log (status: pending)
      3. External process calls tap_list_pending_decisions (via MCP or CLI)
      4. External process acts: tap_authorize / tap_reject / tap_settle
      5. Action tools auto-resolve matching decisions in the shared DB

  Exec Mode:
    tap-http spawns a child process and communicates over stdin/stdout using
    JSON-RPC 2.0. The process receives decision requests and can call back
    using MCP tool calls.
      tap-http --decision-exec ./my-compliance-engine
      tap-http -D ./my-engine -A \"--config,prod.yaml\" -S all

EXTERNAL DECISION EXECUTABLE PROTOCOL:

  The external executable communicates via newline-delimited JSON-RPC 2.0 on
  stdin (from tap-http) and stdout (to tap-http).

  Initialization (tap-http -> executable, notification):
    {{\"jsonrpc\":\"2.0\",\"method\":\"tap/initialize\",\"params\":{{
      \"version\":\"0.1.0\",\"agent_dids\":[\"did:key:z6Mk...\"],
      \"subscribe_mode\":\"decisions\",\"capabilities\":{{\"tools\":true,\"decisions\":true}}
    }}}}

  Decision request (tap-http -> executable, request with id):
    {{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tap/decision\",\"params\":{{
      \"decision_id\":42,\"transaction_id\":\"txn-123\",
      \"agent_did\":\"did:key:z6Mk...\",
      \"decision_type\":\"authorization_required\",
      \"context\":{{\"transaction_state\":\"Received\",
        \"transaction\":{{\"type\":\"transfer\",\"asset\":\"eip155:1/slip44:60\",
          \"amount\":\"100\",\"originator\":\"did:key:z1...\",\"beneficiary\":\"did:key:z2...\"}}
      }}
    }}}}

  Decision response (executable -> tap-http):
    {{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{
      \"action\":\"authorize\",\"detail\":{{\"settlement_address\":\"eip155:1:0xABC\"}}
    }}}}

  Valid actions per decision type:
    authorization_required:       authorize, reject, update_policies, defer
    policy_satisfaction_required:  present, reject, cancel, defer
    settlement_required:          settle, cancel, defer

  The executable can also call MCP tools via stdout:
    {{\"jsonrpc\":\"2.0\",\"id\":100,\"method\":\"tools/call\",\"params\":{{
      \"name\":\"tap_authorize\",\"arguments\":{{
        \"agent_did\":\"did:key:z6Mk...\",\"transaction_id\":\"txn-123\"}}
    }}}}

  Available tools: tap_authorize, tap_reject, tap_settle, tap_cancel,
  tap_revert, tap_list_pending_decisions, tap_resolve_decision, and all
  other MCP tools from tap-mcp.

  Process lifecycle:
    - On process exit: restart with exponential backoff (1s..30s)
    - Pending decisions replay on reconnect
    - Graceful shutdown: EOF on stdin, then SIGTERM, then SIGKILL

NOTES:
    - If no agent DID and key are provided, the server will:
      1. Try to load keys from ~/.tap/keys.json
      2. Automatically create and save new keys if none exist
    - Data is stored under ~/.tap/<did-hash>/tap.db per agent
    - Use --tap-root to change the storage directory",
        env!("CARGO_PKG_VERSION")
    );
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

    // Create the actual agent - try to load from storage first, create if none exist
    let agent = match TapAgent::from_stored_keys(None, true).await {
        Ok(agent) => {
            info!("Loaded agent from stored keys");
            agent
        }
        Err(e) => {
            info!("No stored keys found ({}), creating new agent...", e);

            // Create a key manager with storage enabled and generate a new key
            let default_key_path = KeyStorage::default_key_path().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine default key path",
                )
            })?;
            let key_manager_builder =
                AgentKeyManagerBuilder::new().load_from_path(default_key_path);
            let key_manager = key_manager_builder.build()?;

            // Generate a new key
            let generated_key = key_manager.generate_key(DIDGenerationOptions {
                key_type: KeyType::Ed25519,
            })?;

            info!("Generated new agent with DID: {}", generated_key.did);

            // Create agent config and build agent
            let config = AgentConfig::new(generated_key.did.clone()).with_debug(true);

            #[cfg(all(not(target_arch = "wasm32"), test))]
            let agent = TapAgent::new(config, Arc::new(key_manager));

            #[cfg(all(not(target_arch = "wasm32"), not(test)))]
            let agent = TapAgent::new(config, Arc::new(key_manager));

            #[cfg(target_arch = "wasm32")]
            let agent = TapAgent::new(config, Arc::new(key_manager));

            info!("New key saved to storage successfully");
            agent
        }
    };
    let agent_did = agent.get_agent_did().to_string(); // Clone to a String to avoid borrowing issues

    let agent_arc = Arc::new(agent);
    info!("Using agent with DID: {}", agent_did);

    // Print the DID to stdout for easy copying
    println!("TAP HTTP Server started with agent DID: {}", agent_did);

    // Check if we're using command-line provided keys vs. stored keys
    if args.agent_did.is_some() && args.agent_key.is_some() {
        println!("Note: Using command-line provided DID and key.");
    } else {
        match KeyStorage::default_key_path() {
            Some(path) if path.exists() => {
                println!("Note: Using persistent agent keys from storage.");
            }
            _ => {
                println!("Note: Created new persistent agent keys in storage for future use.");
            }
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
        enable_web_did: args.enable_web_did,
        max_agents: 100,
    };

    // Configure event logging - use TAP root-based default if not specified
    let tap_root_path = args.tap_root.as_ref().map(PathBuf::from);
    let logs_dir = args
        .logs_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| tap_node::storage::Storage::default_logs_dir(tap_root_path.clone()));
    let log_path = logs_dir.join("tap-http.log");

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
    info!("  Web DID hosting: {}", config.enable_web_did);
    info!("  Agent DID: {}", agent_did);
    debug!("  Event logging: {}", log_path.to_string_lossy());
    debug!("  Structured logs: {}", args.structured_logs);

    // Create node configuration with the agent and storage
    let mut node_config = NodeConfig::default();

    // Configure storage
    if let Some(db_path) = args.db_path {
        // Use explicit database path
        node_config.storage_path = Some(PathBuf::from(db_path));
        info!(
            "Using database at: {:?}",
            node_config.storage_path.as_ref().unwrap()
        );
    } else {
        // Use DID-based storage path
        node_config.agent_did = Some(agent_did.clone());
        node_config.tap_root = tap_root_path.clone();
        let expected_path = tap_root_path
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".tap")
            })
            .join(agent_did.replace(':', "_"))
            .join("transactions.db");
        info!("Using database at: {:?}", expected_path);
    }

    // Create TAP Node
    let mut node = TapNode::new(node_config);

    // Initialize storage (tap-node has storage feature enabled by default)
    if let Err(e) = node.init_storage().await {
        error!("Failed to initialize storage: {}", e);
        return Err(e.into());
    }

    // Register the primary agent with the node
    if let Err(e) = node.register_agent(agent_arc.clone()).await {
        error!("Failed to register agent: {}", e);
        return Err(e.into());
    }

    // Register all additional agents from storage
    match KeyStorage::load_default() {
        Ok(storage) => {
            let stored_dids: Vec<String> = storage.keys.keys().cloned().collect();
            info!("Found {} total keys in storage", stored_dids.len());

            for stored_did in &stored_dids {
                // Skip the primary agent as it's already registered
                if stored_did == &agent_did {
                    continue;
                }

                info!("Registering additional agent: {}", stored_did);
                match TapAgent::from_stored_keys(Some(stored_did.clone()), true).await {
                    Ok(additional_agent) => {
                        let additional_agent_arc = Arc::new(additional_agent);
                        if let Err(e) = node.register_agent(additional_agent_arc).await {
                            error!("Failed to register additional agent {}: {}", stored_did, e);
                        } else {
                            info!("Successfully registered additional agent: {}", stored_did);
                        }
                    }
                    Err(e) => {
                        error!("Failed to load additional agent {}: {}", stored_did, e);
                    }
                }
            }

            if stored_dids.len() > 1 {
                info!(
                    "Registered {} agents total (1 primary + {} additional)",
                    stored_dids.len(),
                    stored_dids.len() - 1
                );
            } else {
                info!("Registered 1 agent (primary only)");
            }
        }
        Err(e) => {
            info!("Could not load additional keys from storage: {}", e);
            info!("Only the primary agent is registered");
        }
    }

    // Determine effective decision mode
    let effective_decision_mode = if args.decision_exec.is_some() {
        "exec".to_string()
    } else {
        args.decision_mode.clone()
    };

    // Set up external decision manager if configured
    let _decision_manager: Option<Arc<ExternalDecisionManager>> = if effective_decision_mode
        == "exec"
    {
        let exec_path = args
            .decision_exec
            .as_ref()
            .expect("--decision-exec required for exec mode");
        info!("Configuring external decision executable: {}", exec_path);

        let subscribe_mode: SubscribeMode = args.decision_subscribe.parse().unwrap_or_else(|e| {
            error!("Invalid decision subscribe mode: {}. Using 'decisions'", e);
            SubscribeMode::Decisions
        });

        let decision_config = ExternalDecisionConfig {
            exec_path: exec_path.clone(),
            exec_args: args.decision_exec_args.clone(),
            subscribe_mode,
        };

        // Get storage from the node
        let storage = node
            .storage()
            .expect("Storage must be initialized for external decision support")
            .clone();

        // Build the ToolRegistry via a TapIntegration instance
        let tool_registry = {
            let ti = match TapIntegration::new(
                Some(&agent_did),
                args.tap_root.as_deref(),
                Some(agent_arc.clone()),
            )
            .await
            {
                Ok(ti) => Arc::new(ti),
                Err(e) => {
                    error!("Failed to create TapIntegration for decision tools: {}", e);
                    process::exit(1);
                }
            };
            Arc::new(ToolRegistry::new(ti))
        };

        let manager = Arc::new(ExternalDecisionManager::new(
            decision_config,
            vec![agent_did.clone()],
            tool_registry,
            storage,
        ));

        // Set decision mode to Custom with the manager as handler
        node.set_decision_mode(tap_node::state_machine::fsm::DecisionMode::Custom(
            manager.clone(),
        ));

        // Subscribe to events for expiration and forwarding
        node.event_bus().subscribe(manager.clone()).await;

        // Start the external process
        manager.start().await;

        info!("External decision manager started");
        Some(manager)
    } else {
        None
    };

    // Set up poll mode if configured
    if effective_decision_mode == "poll" {
        info!("Configuring poll decision mode (external process polls decision_log)");

        let storage = node
            .storage()
            .expect("Storage must be initialized for poll decision mode")
            .clone();

        // Create DecisionLogHandler that writes decisions to the database
        let decision_handler = Arc::new(
            tap_node::event::decision_log_handler::DecisionLogHandler::new(
                storage.clone(),
                vec![agent_did.clone()],
            ),
        );

        // Set decision mode to Custom with the log handler
        node.set_decision_mode(tap_node::state_machine::fsm::DecisionMode::Custom(
            decision_handler,
        ));

        // Subscribe DecisionStateHandler for auto-resolution on state changes
        let state_handler =
            Arc::new(tap_node::event::decision_state_handler::DecisionStateHandler::new(storage));
        node.event_bus().subscribe(state_handler).await;

        info!("Poll decision mode configured — decisions logged to decision_log table");
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

    // Stop external decision manager if running
    if let Some(manager) = _decision_manager {
        manager.shutdown().await;
    }

    // Stop the server
    if let Err(e) = server.stop().await {
        error!("Error during shutdown: {}", e);
    }

    info!("Server shutdown complete");
    Ok(())
}
