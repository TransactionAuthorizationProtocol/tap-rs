//! Participant types for TAP messages.
//!
//! This module defines the structure of participant information used in TAP messages.

use serde::{Deserialize, Serialize};

use crate::message::policy::Policy;

/// Participant in a transfer (TAIP-3, TAIP-11).
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
    pub fn with_role(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: Some(role.to_string()),

            policies: None,
            leiCode: None,
            name: None,
        }
    }
}
