//! Integration layer with TAP ecosystem components

use crate::error::{Error, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tap_agent::TapAgent;
use tap_node::{TapNode, NodeConfig};
use tracing::{debug, info};

/// TAP ecosystem integration - thin wrapper around TapNode
pub struct TapIntegration {
    node: Arc<TapNode>,
}

impl TapIntegration {
    /// Create new TAP integration using TapNode
    pub async fn new(agent_did: Option<&str>) -> Result<Self> {
        // Create node configuration
        let mut config = NodeConfig::default();
        
        // Set agent DID for proper storage organization
        if let Some(did) = agent_did {
            config.agent_did = Some(did.to_string());
        }
        
        // Enable storage features
        config.enable_message_logging = true;
        config.log_message_content = true;
        
        // Create the node
        let mut node = TapNode::new(config);
        
        // Initialize storage with DID-based structure
        node.init_storage().await
            .map_err(|e| Error::configuration(format!("Failed to initialize TAP node storage: {}", e)))?;
        
        info!("Initialized TAP integration with DID-based storage");
        
        Ok(Self {
            node: Arc::new(node),
        })
    }
    
    /// Create new TAP integration for testing with custom paths
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
        node.init_storage().await
            .map_err(|e| Error::configuration(format!("Failed to initialize TAP node storage: {}", e)))?;
        
        debug!("Created TAP integration for testing with DID: {}", agent_did);
        
        Ok(Self {
            node: Arc::new(node),
        })
    }
    
    /// Get reference to underlying TapNode
    pub fn node(&self) -> &Arc<TapNode> {
        &self.node
    }
    
    /// Get storage reference (if available)
    pub fn storage(&self) -> Option<&Arc<tap_node::storage::Storage>> {
        self.node.storage()
    }
    
    /// List all registered agents
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let agent_dids = self.node.list_agents();
        let mut agents = Vec::new();
        
        for did in agent_dids {
            // For MCP, we'll return basic agent info
            // In a real implementation, we might want to store more metadata
            agents.push(AgentInfo {
                id: did.clone(),
                role: "Agent".to_string(), // Default role
                for_party: did, // Agent represents itself by default
                policies: vec![],
                metadata: std::collections::HashMap::new(),
            });
        }
        
        Ok(agents)
    }
    
    /// Create a new agent (register with the node)
    pub async fn create_agent(&self, agent_info: &AgentInfo) -> Result<()> {
        // Create a new TAP agent
        let (agent, _did) = TapAgent::from_ephemeral_key().await
            .map_err(|e| Error::configuration(format!("Failed to create agent: {}", e)))?;
        
        // Register with the node
        self.node.register_agent(Arc::new(agent)).await
            .map_err(|e| Error::configuration(format!("Failed to register agent: {}", e)))?;
        
        info!("Created and registered agent: {}", agent_info.id);
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