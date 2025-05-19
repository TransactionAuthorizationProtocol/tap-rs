//! Event Logger Demo
//!
//! This example demonstrates how to set up and use the TAP Node event logger
//! to monitor and log all events occurring within a TAP Node.

use std::sync::Arc;
use std::time::Duration;

use tap_agent::{AgentConfig, DefaultAgent};
use tap_msg::didcomm::PlainMessage;
use tap_node::event::logger::{EventLoggerConfig, LogDestination};
use tap_node::{NodeConfig, TapNode};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    env_logger::init();

    // Create a temporary directory for logs
    let temp_dir = tempfile::tempdir()?;
    let log_path = temp_dir.path().join("tap-events.log");
    let log_path_str = log_path.to_str().unwrap().to_string();
    println!("Logging events to: {}", log_path_str);

    // Configure the event logger
    let event_logger_config = EventLoggerConfig {
        destination: LogDestination::File {
            path: log_path_str,
            max_size: Some(10 * 1024 * 1024), // 10 MB
            rotate: true,
        },
        structured: true, // Use JSON format
        log_level: log::Level::Info,
    };

    // Create node configuration with event logger
    let node_config = NodeConfig {
        debug: true,
        enable_message_logging: true,
        log_message_content: true,
        event_logger: Some(event_logger_config),
        ..Default::default()
    };

    // Create a new TAP Node
    let node = TapNode::new(node_config);

    // Create a test agent for demonstration
    let agent_did = "did:example:agent1".to_string();
    let agent_config = AgentConfig::new(agent_did.clone());

    // In a real scenario, you'd set up crypto properly
    // This is simplified for the example
    let did_resolver = Arc::new(tap_agent::did::MultiResolver::default());

    // Create a test secrets resolver for the example
    #[derive(Debug)]
    struct TestSecretsResolver {
        secrets: std::collections::HashMap<String, didcomm::secrets::Secret>,
    }

    impl TestSecretsResolver {
        pub fn new() -> Self {
            Self {
                secrets: std::collections::HashMap::new(),
            }
        }
    }

    impl tap_agent::crypto::DebugSecretsResolver for TestSecretsResolver {
        fn get_secrets_map(&self) -> &std::collections::HashMap<String, didcomm::secrets::Secret> {
            &self.secrets
        }
    }

    let secrets_resolver = Arc::new(TestSecretsResolver::new());
    let message_packer = Arc::new(tap_agent::crypto::DefaultMessagePacker::new(
        did_resolver,
        secrets_resolver,
    ));

    let agent = DefaultAgent::new(agent_config, message_packer);

    // Register the agent with the node
    println!("Registering agent: {}", agent_did);
    node.register_agent(Arc::new(agent)).await?;

    // Generate some events for demonstration

    // 1. Create a test message
    let message = Message {
        id: "msg-demo-1".to_string(),
        typ: "application/didcomm-plain+json".to_string(),
        type_: "test-message".to_string(),
        body: serde_json::json!({
            "greeting": "Hello, TAP!",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
        from: None,
        to: None,
        thid: None,
        pthid: None,
        created_time: Some(chrono::Utc::now().timestamp() as u64),
        expires_time: None,
        from_prior: None,
        attachments: None,
        extra_headers: std::collections::HashMap::new(),
    };

    // 2. Send the message to demonstrate MessageSent event
    println!("Sending a test message");
    let second_did = "did:example:agent2".to_string();
    let _ = node.send_message(&agent_did, &second_did, message.clone());

    // 3. Receive the message to demonstrate MessageReceived event
    println!("Receiving a test message");
    let _ = node.receive_message(message);

    // 4. Unregister the agent to demonstrate AgentUnregistered event
    println!("Unregistering agent");
    node.unregister_agent(&agent_did).await?;

    // Wait to ensure all events are processed
    println!("Waiting for events to be processed...");
    sleep(Duration::from_secs(1)).await;

    println!("Demo completed successfully!");
    println!("The events have been logged to the file specified above.");
    println!("You can examine the log file to see the structured event logs.");

    Ok(())
}
