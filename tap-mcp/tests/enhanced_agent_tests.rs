//! Enhanced agent management tests
//!
//! These tests verify the enhanced agent creation and management functionality
//! that integrates tap-agent storage with TAP-MCP.

use std::collections::HashMap;
use tap_agent::{Agent, TapAgent};
use tap_mcp::error::Result;
use tap_mcp::tap_integration::{AgentInfo, TapIntegration};
use tempfile::TempDir;

/// Test helper to create a temporary TAP environment for enhanced agent testing
struct EnhancedTestEnvironment {
    _temp_dir: TempDir,
    original_home: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

// Global mutex to serialize test execution
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl EnhancedTestEnvironment {
    fn new() -> Result<Self> {
        // Acquire the global lock to serialize tests
        let lock = TEST_MUTEX.lock().unwrap();

        let temp_dir = tempfile::tempdir()?;

        // Store original HOME value
        let original_home = std::env::var("HOME").ok();

        // Set HOME to the temp directory so tap-agent uses it for storage
        std::env::set_var("HOME", temp_dir.path());

        Ok(Self {
            _temp_dir: temp_dir,
            original_home,
            _lock: lock,
        })
    }
}

impl Drop for EnhancedTestEnvironment {
    fn drop(&mut self) {
        // Restore original HOME environment variable
        if let Some(original_home) = &self.original_home {
            std::env::set_var("HOME", original_home);
        } else {
            std::env::remove_var("HOME");
        }
        // Lock is automatically released when dropped
    }
}

#[tokio::test]
async fn test_enhanced_agent_creation() -> Result<()> {
    let _env = EnhancedTestEnvironment::new()?;

    // Create agent info with policies and JSON-LD metadata
    let agent_info = AgentInfo {
        id: "did:example:enhanced-agent".to_string(),
        role: "SettlementAddress".to_string(), // Role not stored, used only for transaction context
        for_party: "did:example:bank".to_string(), // For party not stored, used only for transaction context
        policies: vec!["KYC_REQUIRED".to_string(), "AML_CHECK".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("name".to_string(), "Bank ABC Settlement Agent".to_string());
            meta.insert("url".to_string(), "https://bank-abc.com".to_string());
            meta.insert("lei".to_string(), "5493001RKR55V4X61F71".to_string());
            meta.insert("region".to_string(), "US".to_string());
            meta
        },
    };

    // Create enhanced agent using tap-agent functionality (role/for_party not stored)
    let (agent, created_did) = TapAgent::create_enhanced_agent(
        agent_info.id.clone(),
        agent_info.policies.clone(),
        agent_info.metadata.clone(),
        true, // Save to storage
    )
    .await?;

    // Verify the agent was created correctly
    assert_eq!(created_did, agent_info.id);
    assert_eq!(agent.get_agent_did(), &agent_info.id);

    // Load the agent back from storage
    let (loaded_agent, loaded_policies, loaded_metadata) =
        TapAgent::load_enhanced_agent(&agent_info.id).await?;

    // Verify the loaded agent has the correct properties
    assert_eq!(loaded_agent.get_agent_did(), &agent_info.id);
    assert_eq!(loaded_policies, agent_info.policies);
    assert_eq!(
        loaded_metadata.get("name"),
        Some(&"Bank ABC Settlement Agent".to_string())
    );
    assert_eq!(
        loaded_metadata.get("url"),
        Some(&"https://bank-abc.com".to_string())
    );
    assert_eq!(
        loaded_metadata.get("lei"),
        Some(&"5493001RKR55V4X61F71".to_string())
    );
    assert_eq!(loaded_metadata.get("region"), Some(&"US".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_enhanced_agent_listing() -> Result<()> {
    let _env = EnhancedTestEnvironment::new()?;

    // Create multiple enhanced agents with JSON-LD metadata
    let agents_to_create = vec![
        ("did:example:agent1", "First Bank Settlement"),
        ("did:example:agent2", "Second Bank Compliance"),
        ("did:example:agent3", "Exchange Platform"),
    ];

    for (id, name) in &agents_to_create {
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), name.to_string());
        metadata.insert("type".to_string(), "financial_institution".to_string());

        TapAgent::create_enhanced_agent(
            id.to_string(),
            vec!["POLICY1".to_string()],
            metadata,
            true,
        )
        .await?;
    }

    // List all enhanced agents
    let enhanced_agents = TapAgent::list_enhanced_agents()?;

    // Verify we have the expected number of agents
    assert_eq!(enhanced_agents.len(), 3);

    // Verify each agent has the correct properties
    for (id, name) in &agents_to_create {
        let agent = enhanced_agents
            .iter()
            .find(|(did, _, _)| did == id)
            .expect(&format!("Agent {} not found", id));

        assert_eq!(agent.1, vec!["POLICY1".to_string()]);
        assert_eq!(agent.2.get("name"), Some(&name.to_string()));
        assert_eq!(
            agent.2.get("type"),
            Some(&"financial_institution".to_string())
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_tap_mcp_enhanced_integration() -> Result<()> {
    let _env = EnhancedTestEnvironment::new()?;

    // Create TAP integration
    let integration =
        TapIntegration::new_for_testing(Some("/tmp/test-tap"), "did:example:test-node").await?;

    // Create an agent through TAP-MCP with JSON-LD metadata
    let agent_info = AgentInfo {
        id: "did:example:mcp-agent".to_string(),
        role: "Custodian".to_string(), // Role used for transaction context only
        for_party: "did:example:custodian".to_string(), // For party used for transaction context only
        policies: vec![
            "CUSTODY_REQUIRED".to_string(),
            "INSURANCE_REQUIRED".to_string(),
        ],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert(
                "name".to_string(),
                "SecureVault Custodian Services".to_string(),
            );
            meta.insert("url".to_string(), "https://securevault.com".to_string());
            meta.insert("lei".to_string(), "1234567890ABCDEF1234".to_string());
            meta.insert("vault_type".to_string(), "cold_storage".to_string());
            meta.insert("insurance_amount".to_string(), "10000000".to_string());
            meta
        },
    };

    // Create agent through TAP-MCP (should use enhanced creation)
    integration.create_agent(&agent_info).await?;

    // List agents through TAP-MCP (should include enhanced agents)
    let listed_agents = integration.list_agents().await?;

    // Verify our agent is in the list with correct properties
    let found_agent = listed_agents
        .iter()
        .find(|a| a.id == agent_info.id)
        .expect("Created agent not found in list");

    // Note: role and for_party are not stored, so they default to "Agent" and DID respectively
    assert_eq!(found_agent.role, "Agent"); // Default since role is not stored
    assert_eq!(found_agent.for_party, agent_info.id); // Default to self since for_party is not stored
    assert_eq!(found_agent.policies, agent_info.policies);
    assert_eq!(
        found_agent.metadata.get("name"),
        Some(&"SecureVault Custodian Services".to_string())
    );
    assert_eq!(
        found_agent.metadata.get("url"),
        Some(&"https://securevault.com".to_string())
    );
    assert_eq!(
        found_agent.metadata.get("lei"),
        Some(&"1234567890ABCDEF1234".to_string())
    );
    assert_eq!(
        found_agent.metadata.get("vault_type"),
        Some(&"cold_storage".to_string())
    );
    assert_eq!(
        found_agent.metadata.get("insurance_amount"),
        Some(&"10000000".to_string())
    );

    Ok(())
}
