//! Integration layer with TAP ecosystem components

use crate::error::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tap_agent::TapAgent;
use tap_node::{NodeConfig, TapNode};
use tracing::{debug, error, info};

/// TAP ecosystem integration - thin wrapper around TapNode
pub struct TapIntegration {
    node: Arc<TapNode>,
    /// Custom storage path for testing (if set, overrides default ~/.tap)
    storage_path: Option<PathBuf>,
}

impl TapIntegration {
    /// Create new TAP integration using TapNode with agent registration
    pub async fn new(
        agent_did: Option<&str>,
        tap_root: Option<&str>,
        agent: Option<Arc<TapAgent>>,
    ) -> Result<Self> {
        // Create node configuration
        let mut config = NodeConfig::default();

        // Set agent DID for proper storage organization
        if let Some(did) = agent_did {
            config.agent_did = Some(did.to_string());
        }

        // Set custom TAP root if provided
        if let Some(root) = tap_root {
            config.tap_root = Some(PathBuf::from(root));
        }

        // Enable storage features
        config.enable_message_logging = true;
        config.log_message_content = true;

        // Create the node
        let mut node = TapNode::new(config);

        // Initialize storage with DID-based structure
        node.init_storage().await.map_err(|e| {
            Error::configuration(format!("Failed to initialize TAP node storage: {}", e))
        })?;

        info!("Initialized TAP integration with DID-based storage");

        let node_arc = Arc::new(node);

        // Register the primary agent if provided
        if let Some(agent) = agent {
            node_arc
                .register_agent(agent)
                .await
                .map_err(|e| Error::configuration(format!("Failed to register agent: {}", e)))?;
            info!("Registered primary agent with TAP Node");
        }

        // Load and register all additional agents from storage
        match tap_agent::storage::KeyStorage::load_default() {
            Ok(storage) => {
                let stored_dids: Vec<String> = storage.keys.keys().cloned().collect();
                info!("Found {} total keys in storage", stored_dids.len());

                for stored_did in &stored_dids {
                    // Skip the primary agent if it's already registered
                    if agent_did.map_or(false, |did| stored_did == did) {
                        continue;
                    }

                    info!("Registering additional agent: {}", stored_did);
                    match TapAgent::from_stored_keys(Some(stored_did.clone()), true).await {
                        Ok(additional_agent) => {
                            let additional_agent_arc = Arc::new(additional_agent);
                            if let Err(e) = node_arc.register_agent(additional_agent_arc).await
                            {
                                error!(
                                    "Failed to register additional agent {}: {}",
                                    stored_did, e
                                );
                            } else {
                                info!(
                                    "Successfully registered additional agent: {}",
                                    stored_did
                                );
                            }
                        }
                        Err(e) => {
                            error!("Failed to load additional agent {}: {}", stored_did, e);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Could not load additional keys from storage: {}", e);
            }
        }

        Ok(Self {
            node: node_arc,
            storage_path: None,
        })
    }

    /// Create new TAP integration for testing with custom paths
    #[allow(dead_code)]
    pub async fn new_for_testing(tap_root: Option<&str>, agent_did: &str) -> Result<Self> {
        let mut config = NodeConfig::default();

        // Set custom TAP root for testing
        if let Some(root) = tap_root {
            config.tap_root = Some(PathBuf::from(root));
        }

        // Set agent DID
        config.agent_did = Some(agent_did.to_string());
        config.enable_message_logging = true;
        config.log_message_content = true;

        let mut node = TapNode::new(config);
        node.init_storage().await.map_err(|e| {
            Error::configuration(format!("Failed to initialize TAP node storage: {}", e))
        })?;

        debug!(
            "Created TAP integration for testing with DID: {}",
            agent_did
        );

        // For testing, use the keys.json file in the TAP root
        let storage_path = if let Some(root) = tap_root {
            Some(PathBuf::from(root).join("keys.json"))
        } else {
            None
        };

        // Create a test agent for testing
        let (test_agent, _) = TapAgent::from_ephemeral_key()
            .await
            .map_err(|e| Error::configuration(format!("Failed to create test agent: {}", e)))?;

        let node_arc = Arc::new(node);
        node_arc
            .register_agent(Arc::new(test_agent))
            .await
            .map_err(|e| Error::configuration(format!("Failed to register test agent: {}", e)))?;

        Ok(Self {
            node: node_arc,
            storage_path,
        })
    }

    /// Get reference to underlying TapNode
    #[allow(dead_code)]
    pub fn node(&self) -> &Arc<TapNode> {
        &self.node
    }

    /// Get storage reference (if available)
    pub fn storage(&self) -> Option<&Arc<tap_node::storage::Storage>> {
        self.node.storage()
    }

    /// List all registered agents (from storage and in-memory registry)
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let mut agents = Vec::new();

        // Get agents from tap-agent storage with policies and metadata
        let enhanced_agents = if let Some(ref storage_path) = self.storage_path {
            TapAgent::list_enhanced_agents_with_path(Some(storage_path.clone()))
        } else {
            TapAgent::list_enhanced_agents()
        }
        .map_err(|e| Error::configuration(format!("Failed to list enhanced agents: {}", e)))?;

        for (did, policies, metadata) in enhanced_agents {
            // Role and for_party are not stored in metadata anymore
            // They will be determined per transaction
            agents.push(AgentInfo {
                id: did.clone(),
                role: "Agent".to_string(), // Default role, will be determined per transaction
                for_party: did.clone(),    // Default to self, will be determined per transaction
                policies,
                metadata,
            });
        }

        // Also include any agents only registered in TapNode (for backward compatibility)
        let node_agent_dids = self.node.list_agents();
        for did in node_agent_dids {
            // Check if we already have this agent from enhanced storage
            if !agents.iter().any(|a| a.id == did) {
                agents.push(AgentInfo {
                    id: did.clone(),
                    role: "Agent".to_string(),
                    for_party: did,
                    policies: vec![],
                    metadata: std::collections::HashMap::new(),
                });
            }
        }

        Ok(agents)
    }

    /// Create a new agent with enhanced storage and auto-registration
    pub async fn create_agent(&self, agent_info: &AgentInfo) -> Result<()> {
        // Create enhanced agent with policies and metadata using tap-agent
        // Note: role and for_party are not stored, they are determined per transaction
        let (agent, _did) = if let Some(ref storage_path) = self.storage_path {
            TapAgent::create_enhanced_agent_with_path(
                agent_info.id.clone(),
                agent_info.policies.clone(),
                agent_info.metadata.clone(),
                true, // Save to storage
                Some(storage_path.clone()),
            )
            .await
        } else {
            TapAgent::create_enhanced_agent(
                agent_info.id.clone(),
                agent_info.policies.clone(),
                agent_info.metadata.clone(),
                true, // Save to storage
            )
            .await
        }
        .map_err(|e| Error::configuration(format!("Failed to create enhanced agent: {}", e)))?;

        // Register with the TAP Node for message processing
        self.node
            .register_agent(Arc::new(agent))
            .await
            .map_err(|e| {
                Error::configuration(format!("Failed to register agent with TAP Node: {}", e))
            })?;

        info!(
            "Created enhanced agent with DID {} and registered with TAP Node",
            agent_info.id
        );
        debug!(
            "Agent directory created at ~/.tap/{}",
            agent_info.id.replace(':', "_")
        );
        debug!("Policies: {:?}", agent_info.policies);
        debug!("Metadata: {:?}", agent_info.metadata);
        debug!("Note: role '{}' and for_party '{}' are not stored, they are determined per transaction", agent_info.role, agent_info.for_party);

        Ok(())
    }
}

/// Agent information for MCP interface
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub role: String,
    pub for_party: String,
    pub policies: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}
