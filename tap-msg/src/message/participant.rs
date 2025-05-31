//! Participant types for TAP messages (DEPRECATED).
//!
//! This module defines the legacy structure of participant information used in TAP messages.
//! 
//! **DEPRECATED**: Use `Agent` and `Party` types instead for better TAIP compliance.
//! - For TAIP-5 Agents: Use `crate::message::Agent` 
//! - For TAIP-6 Parties: Use `crate::message::Party`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::message::{Agent, Party};
use crate::message::policy::Policy;

/// Participant in a transfer (TAIP-3, TAIP-11).
/// 
/// **DEPRECATED**: This type combines both Agent and Party concepts from TAIP-5 and TAIP-6.
/// Use `Agent` for transaction agents and `Party` for real-world entities instead.
#[deprecated(since = "0.1.0", note = "Use Agent and Party types instead for better TAIP compliance")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct Participant {
    /// DID of the participant.
    #[serde(default)]
    pub id: String,

    /// Role of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub role: Option<String>,

    // Name of the participant
    pub name: Option<String>,

    /// Policies of the participant according to TAIP-7 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub policies: Option<Vec<Policy>>,

    /// Legal Entity Identifier (LEI) code of the participant (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leiCode: Option<String>,
}

impl Participant {
    /// Create a new participant with the given DID.
    #[deprecated(since = "0.1.0", note = "Use Party::new() or Agent::new() instead")]
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            role: None,
            policies: None,
            leiCode: None,
            name: None,
        }
    }

    /// Create a new participant with the given DID and role.
    #[deprecated(since = "0.1.0", note = "Use Agent::new() for agents with roles")]
    pub fn with_role(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),
            policies: None,
            leiCode: None,
            name: None,
        }
    }

    /// Convert this Participant to a Party (for real-world entities).
    /// Loses role and policies information.
    pub fn to_party(&self) -> Party {
        let mut metadata = HashMap::new();
        
        if let Some(name) = &self.name {
            metadata.insert("name".to_string(), serde_json::Value::String(name.clone()));
        }
        
        if let Some(lei_code) = &self.leiCode {
            metadata.insert("https://schema.org/leiCode".to_string(), 
                           serde_json::Value::String(lei_code.clone()));
        }
        
        Party {
            id: self.id.clone(),
            metadata,
        }
    }

    /// Convert this Participant to an Agent (for transaction agents).
    /// Requires specifying for_party since it's required for agents.
    pub fn to_agent(&self, for_party: &str) -> Option<Agent> {
        // Agent requires both role and for_party
        if let Some(role) = &self.role {
            let mut metadata = HashMap::new();
            
            if let Some(name) = &self.name {
                metadata.insert("name".to_string(), serde_json::Value::String(name.clone()));
            }
            
            if let Some(lei_code) = &self.leiCode {
                metadata.insert("https://schema.org/leiCode".to_string(), 
                               serde_json::Value::String(lei_code.clone()));
            }
            
            Some(Agent {
                id: self.id.clone(),
                role: role.clone(),
                for_parties: crate::message::agent::ForParties(vec![for_party.to_string()]),
                policies: self.policies.clone(),
                metadata,
            })
        } else {
            None // Cannot convert to Agent without a role
        }
    }
}

impl From<Party> for Participant {
    /// Convert a Party to a Participant for backward compatibility.
    fn from(party: Party) -> Self {
        let mut participant = Self {
            id: party.id.clone(),
            role: None,
            policies: None,
            leiCode: None,
            name: None,
        };

        // Extract known metadata fields
        if let Some(name) = party.get_metadata("name") {
            if let Some(name_str) = name.as_str() {
                participant.name = Some(name_str.to_string());
            }
        }

        if let Some(lei) = party.get_metadata("https://schema.org/leiCode") {
            if let Some(lei_str) = lei.as_str() {
                participant.leiCode = Some(lei_str.to_string());
            }
        }

        participant
    }
}

impl From<Agent> for Participant {
    /// Convert an Agent to a Participant for backward compatibility.
    /// Note: Loses the for_parties information (only keeps the first party if multiple).
    fn from(agent: Agent) -> Self {
        let mut participant = Self {
            id: agent.id.clone(),
            role: Some(agent.role.clone()),
            policies: agent.policies.clone(),
            leiCode: None,
            name: None,
        };

        // Extract known metadata fields
        if let Some(name) = agent.get_metadata("name") {
            if let Some(name_str) = name.as_str() {
                participant.name = Some(name_str.to_string());
            }
        }

        if let Some(lei) = agent.get_metadata("https://schema.org/leiCode") {
            if let Some(lei_str) = lei.as_str() {
                participant.leiCode = Some(lei_str.to_string());
            }
        }

        participant
    }
}
