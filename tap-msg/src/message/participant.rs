//! Temporary compatibility layer for the removed Participant type.
//!
//! This module provides backward compatibility during the transition from
//! unified Participant to separate Agent and Party types.

use serde::{Deserialize, Serialize};
use crate::message::{Agent, Party, Policy};
use crate::message::agent::TapParticipant;

/// Temporary compatibility struct for Participant
/// This maintains the old API structure while internally wrapping Agent/Party
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct Participant {
    /// Identifier for this participant
    pub id: String,
    
    /// Role of the participant (optional for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    
    /// Policies of the participant (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<Vec<Policy>>,
    
    /// LEI code (optional, for backward compatibility)
    #[serde(rename = "leiCode", skip_serializing_if = "Option::is_none")]
    pub leiCode: Option<String>,
    
    /// Name (optional, for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl TapParticipant for Participant {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Participant {
    /// Create a new participant with just an ID
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            policies: None,
            leiCode: None,
            name: None,
        }
    }
    
    /// Create a new participant from an agent
    pub fn from_agent(agent: Agent) -> Self {
        Self {
            id: agent.id.clone(),
            role: Some(agent.role.clone()),
            policies: agent.policies.clone(),
            leiCode: None,
            name: None,
        }
    }
    
    /// Create a new participant from a party
    pub fn from_party(party: Party) -> Self {
        Self {
            id: party.id.clone(),
            role: None,
            policies: None,
            leiCode: party.lei_code(),
            name: party.get_metadata("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }
    
    /// Convert this participant to an Agent (if it has a role)
    pub fn to_agent(&self, for_party: &str) -> Option<Agent> {
        if let Some(ref role) = self.role {
            let mut agent = Agent::new(&self.id, role, for_party);
            if let Some(ref policies) = self.policies {
                agent = agent.with_policies(policies.clone());
            }
            Some(agent)
        } else {
            None
        }
    }
    
    /// Convert this participant to a Party
    pub fn to_party(&self) -> Party {
        let mut party = Party::new(&self.id);
        
        if let Some(ref lei) = self.leiCode {
            party = party.with_lei(lei);
        }
        
        if let Some(ref name) = self.name {
            party.add_metadata("name".to_string(), serde_json::Value::String(name.clone()));
        }
        
        party
    }
    
    /// Get the ID of this participant
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Check if this participant has a role (likely an agent)
    pub fn has_role(&self) -> bool {
        self.role.is_some()
    }
    
    /// Get the role if present
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}

impl From<Agent> for Participant {
    fn from(agent: Agent) -> Self {
        Self::from_agent(agent)
    }
}

impl From<Party> for Participant {
    fn from(party: Party) -> Self {
        Self::from_party(party)
    }
}