//! Agent policy types and structures.
//!
//! This module defines policies that agents can use according to TAIP-7.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FromType specifies who the policy applies to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FromType {
    /// Specific DIDs
    #[serde(rename = "from")]
    From(Vec<String>),

    /// Specific transaction roles
    #[serde(rename = "fromRole")]
    FromRole(Vec<String>),

    /// Specific agent types
    #[serde(rename = "fromAgent")]
    FromAgent(Vec<String>),
}

/// RequireAuthorization policy requires authorization from specific parties
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequireAuthorization {
    /// Optional list of DIDs this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<String>>,

    /// Optional list of roles this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<Vec<String>>,

    /// Optional list of agent types this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent: Option<Vec<String>>,

    /// Optional human-readable purpose for this requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

/// RequirePresentation policy requires verifiable credential presentation
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RequirePresentation {
    /// JSON-LD context for additional schemas
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<String>>,

    /// Optional list of DIDs this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<String>>,

    /// Optional list of roles this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<Vec<String>>,

    /// Optional list of agent types this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent: Option<Vec<String>>,

    /// Party the presentation should be about
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about_party: Option<String>,

    /// Agent the presentation should be about
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about_agent: Option<String>,

    /// Optional human-readable purpose for this requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// URL to the presentation definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_definition: Option<String>,

    /// Specific credentials required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<HashMap<String, Vec<String>>>,
}

/// RequireProofOfControl policy requires proving control of an account or address
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RequireProofOfControl {
    /// Optional list of DIDs this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<String>>,

    /// Optional list of roles this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<Vec<String>>,

    /// Optional list of agent types this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent: Option<Vec<String>>,

    /// ID of the account or address that needs to be proven
    #[serde(default)]
    pub address_id: String,

    /// Optional human-readable purpose for this requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

/// RequireRelationshipConfirmation policy requires confirming a relationship
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RequireRelationshipConfirmation {
    /// Optional list of roles this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<String>,

    /// Optional human-readable purpose for this requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,

    /// Optional nonce for security
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
}

/// Enum representing the different types of policies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@type")]
pub enum Policy {
    /// Require authorization from specified agents
    RequireAuthorization(RequireAuthorization),

    /// Require verifiable credential presentation
    RequirePresentation(RequirePresentation),

    /// Require proof of control of an account or address
    RequireProofOfControl(RequireProofOfControl),

    /// Require confirmation of a relationship
    RequireRelationshipConfirmation(RequireRelationshipConfirmation),
}

impl Policy {
    /// Validates the policy based on its specific type
    pub fn validate(&self) -> crate::error::Result<()> {
        // Basic validation logic for policies
        match self {
            Policy::RequireAuthorization(_) => Ok(()),
            Policy::RequirePresentation(_) => Ok(()),
            Policy::RequireProofOfControl(_) => Ok(()),
            Policy::RequireRelationshipConfirmation(_) => Ok(()),
        }
    }
}

/// Create default implementations for the various policy types
impl Default for RequireAuthorization {
    fn default() -> Self {
        RequireAuthorization {
            from: None,
            from_role: None,
            from_agent: None,
            purpose: None,
        }
    }
}
