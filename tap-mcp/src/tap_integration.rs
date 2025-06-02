//! Integration layer with TAP ecosystem components

use crate::error::{Error, Result};
use std::path::PathBuf;
use tap_node::storage::Storage;
use tracing::{debug, info, warn};

/// TAP ecosystem integration
pub struct TapIntegration {
    storage: Storage,
    agent_storage_path: PathBuf,
}

impl TapIntegration {
    /// Create new TAP integration
    pub async fn new(tap_root: Option<&str>, db_path: Option<&str>) -> Result<Self> {
        // Determine TAP root directory
        let tap_root = if let Some(root) = tap_root {
            PathBuf::from(root)
        } else {
            // Default to ~/.tap
            dirs::home_dir()
                .ok_or_else(|| Error::configuration("Could not find home directory"))?
                .join(".tap")
        };

        // Ensure TAP directory exists
        if !tap_root.exists() {
            info!("Creating TAP directory at: {:?}", tap_root);
            std::fs::create_dir_all(&tap_root)?;
        }

        // Initialize storage
        let storage = if let Some(db_path) = db_path {
            Storage::new(Some(PathBuf::from(db_path))).await?
        } else {
            Storage::new_with_did("tap-mcp", Some(tap_root.clone())).await?
        };

        // Agent storage path
        let agent_storage_path = tap_root.join("agents");
        if !agent_storage_path.exists() {
            info!("Creating agents directory at: {:?}", agent_storage_path);
            std::fs::create_dir_all(&agent_storage_path)?;
        }

        info!("TAP integration initialized with root: {:?}", tap_root);

        Ok(Self {
            storage,
            agent_storage_path,
        })
    }

    /// Get storage reference
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// List all agents in the agent storage directory
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let mut agents = Vec::new();

        if !self.agent_storage_path.exists() {
            return Ok(agents);
        }

        let mut dir = tokio::fs::read_dir(&self.agent_storage_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                match self.load_agent_from_file(&path).await {
                    Ok(agent_info) => agents.push(agent_info),
                    Err(e) => {
                        warn!("Failed to load agent from {:?}: {}", path, e);
                    }
                }
            }
        }

        debug!("Loaded {} agents from storage", agents.len());
        Ok(agents)
    }

    /// Load agent information from file
    async fn load_agent_from_file(&self, path: &PathBuf) -> Result<AgentInfo> {
        let content = tokio::fs::read_to_string(path).await?;
        let agent_data: serde_json::Value = serde_json::from_str(&content)?;

        Ok(AgentInfo {
            id: agent_data
                .get("@id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            role: agent_data
                .get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            for_party: agent_data
                .get("for")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            policies: agent_data
                .get("policies")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            metadata: agent_data
                .get("metadata")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        })
    }

    /// Save agent to storage
    pub async fn save_agent(&self, agent_info: &AgentInfo) -> Result<()> {
        let agent_data = serde_json::json!({
            "@id": agent_info.id,
            "role": agent_info.role,
            "for": agent_info.for_party,
            "policies": agent_info.policies,
            "metadata": agent_info.metadata
        });

        // Use agent ID as filename (sanitized)
        let filename = format!("{}.json", sanitize_filename(&agent_info.id));
        let file_path = self.agent_storage_path.join(filename);

        let content = serde_json::to_string_pretty(&agent_data)?;
        tokio::fs::write(&file_path, content).await?;

        info!("Saved agent {} to {:?}", agent_info.id, file_path);
        Ok(())
    }

    /// Create a TAP agent and save to storage
    pub async fn create_agent(
        &self,
        id: String,
        role: String,
        for_party: String,
        policies: Option<Vec<serde_json::Value>>,
        metadata: Option<serde_json::Value>,
    ) -> Result<AgentInfo> {
        let agent_info = AgentInfo {
            id,
            role,
            for_party,
            policies: policies.unwrap_or_default(),
            metadata: metadata.unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        };

        self.save_agent(&agent_info).await?;
        Ok(agent_info)
    }
}

/// Agent information structure
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub role: String,
    pub for_party: String,
    pub policies: Vec<serde_json::Value>,
    pub metadata: serde_json::Value,
}

/// Sanitize filename by replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}
