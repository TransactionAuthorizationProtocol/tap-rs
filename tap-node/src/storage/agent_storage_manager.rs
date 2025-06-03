//! Agent-specific storage management
//!
//! This module provides the AgentStorageManager that handles per-agent storage instances,
//! ensuring that each agent's data is isolated in its own SQLite database.

use crate::error::Result as NodeResult;
use crate::storage::Storage;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Manages storage instances for multiple agents
///
/// Each agent gets its own isolated SQLite database located at:
/// `{tap_root}/{sanitized_did}/transactions.db`
#[derive(Clone)]
pub struct AgentStorageManager {
    /// Cache of agent storage instances (DID -> Storage)
    agent_storages: DashMap<String, Arc<Storage>>,
    /// TAP root directory for storage
    tap_root: Option<PathBuf>,
}

impl AgentStorageManager {
    /// Create a new agent storage manager
    pub fn new(tap_root: Option<PathBuf>) -> Self {
        info!("Creating AgentStorageManager with TAP root: {:?}", tap_root);
        Self {
            agent_storages: DashMap::new(),
            tap_root,
        }
    }

    /// Get or create storage for an agent
    ///
    /// This method maintains a cache of storage instances to avoid recreating
    /// databases for the same agent. If the storage doesn't exist, it creates
    /// a new one using the agent's DID for the database path.
    pub async fn get_agent_storage(&self, agent_did: &str) -> NodeResult<Arc<Storage>> {
        // Check cache first
        if let Some(storage) = self.agent_storages.get(agent_did) {
            debug!("Using cached storage for agent: {}", agent_did);
            return Ok(storage.clone());
        }

        // Create new storage for this agent
        debug!("Creating new storage for agent: {}", agent_did);
        let storage = Storage::new_with_did(agent_did, self.tap_root.clone())
            .await
            .map_err(|e| {
                crate::Error::Storage(format!(
                    "Failed to create storage for agent {}: {}",
                    agent_did, e
                ))
            })?;

        let storage_arc = Arc::new(storage);

        // Cache it
        self.agent_storages
            .insert(agent_did.to_string(), storage_arc.clone());
        info!("Created and cached storage for agent: {}", agent_did);

        Ok(storage_arc)
    }

    /// Get storage for an agent if it exists in cache (doesn't create new one)
    pub fn get_cached_agent_storage(&self, agent_did: &str) -> Option<Arc<Storage>> {
        self.agent_storages.get(agent_did).map(|s| s.clone())
    }

    /// Remove an agent's storage from the cache
    ///
    /// This doesn't delete the database files, just removes the instance from memory.
    /// Useful when an agent is unregistered.
    pub fn remove_agent_storage(&self, agent_did: &str) -> Option<Arc<Storage>> {
        debug!("Removing storage cache for agent: {}", agent_did);
        self.agent_storages
            .remove(agent_did)
            .map(|(_, storage)| storage)
    }

    /// Get count of cached storage instances
    pub fn cached_storage_count(&self) -> usize {
        self.agent_storages.len()
    }

    /// List all agent DIDs that have cached storage
    pub fn cached_agent_dids(&self) -> Vec<String> {
        self.agent_storages
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Clear all cached storage instances
    ///
    /// This forces recreation of storage instances on next access.
    /// Useful for testing or when storage configuration changes.
    pub fn clear_cache(&self) {
        info!("Clearing all cached agent storage instances");
        self.agent_storages.clear();
    }

    /// Ensure storage exists for an agent (creates if needed but doesn't cache)
    ///
    /// This is useful during agent registration to ensure the storage directory
    /// and database are properly initialized.
    pub async fn ensure_agent_storage(&self, agent_did: &str) -> NodeResult<()> {
        match Storage::new_with_did(agent_did, self.tap_root.clone()).await {
            Ok(_) => {
                info!("Ensured storage exists for agent: {}", agent_did);
                Ok(())
            }
            Err(e) => {
                error!("Failed to ensure storage for agent {}: {}", agent_did, e);
                Err(crate::Error::Storage(format!(
                    "Failed to ensure storage for agent {}: {}",
                    agent_did, e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_agent_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AgentStorageManager::new(Some(temp_dir.path().to_path_buf()));

        assert_eq!(manager.cached_storage_count(), 0);
        assert!(manager.cached_agent_dids().is_empty());
    }

    #[tokio::test]
    async fn test_get_agent_storage() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AgentStorageManager::new(Some(temp_dir.path().to_path_buf()));

        let agent_did = "did:example:test-agent";

        // First call should create storage
        let storage1 = manager.get_agent_storage(agent_did).await.unwrap();
        assert_eq!(manager.cached_storage_count(), 1);

        // Second call should return cached storage
        let storage2 = manager.get_agent_storage(agent_did).await.unwrap();
        assert_eq!(manager.cached_storage_count(), 1);

        // Should be the same instance
        assert!(Arc::ptr_eq(&storage1, &storage2));
    }

    #[tokio::test]
    async fn test_remove_agent_storage() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AgentStorageManager::new(Some(temp_dir.path().to_path_buf()));

        let agent_did = "did:example:test-agent";

        // Create storage
        let _storage = manager.get_agent_storage(agent_did).await.unwrap();
        assert_eq!(manager.cached_storage_count(), 1);

        // Remove from cache
        let removed = manager.remove_agent_storage(agent_did);
        assert!(removed.is_some());
        assert_eq!(manager.cached_storage_count(), 0);
    }

    #[tokio::test]
    async fn test_multiple_agents() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AgentStorageManager::new(Some(temp_dir.path().to_path_buf()));

        let agent1 = "did:example:agent1";
        let agent2 = "did:example:agent2";

        // Create storage for both agents
        let _storage1 = manager.get_agent_storage(agent1).await.unwrap();
        let _storage2 = manager.get_agent_storage(agent2).await.unwrap();

        assert_eq!(manager.cached_storage_count(), 2);

        let cached_dids = manager.cached_agent_dids();
        assert!(cached_dids.contains(&agent1.to_string()));
        assert!(cached_dids.contains(&agent2.to_string()));
    }
}
