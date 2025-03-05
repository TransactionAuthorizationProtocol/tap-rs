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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequireAuthorization {
    /// Type identifier for JSON-LD
    #[serde(rename = "@type")]
    pub type_: String,

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirePresentation {
    /// Type identifier for JSON-LD
    #[serde(rename = "@type")]
    pub type_: String,

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequireProofOfControl {
    /// Type identifier for JSON-LD
    #[serde(rename = "@type")]
    pub type_: String,

    /// Optional list of DIDs this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<String>>,

    /// Optional list of roles this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_role: Option<Vec<String>>,

    /// Optional list of agent types this policy applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent: Option<Vec<String>>,

    /// Randomized token to prevent replay attacks
    pub nonce: u64,

    /// Optional human-readable purpose for this requirement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
}

/// Enum representing the different types of policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@type")]
pub enum Policy {
    /// Require authorization from specific parties
    #[serde(rename = "RequireAuthorization")]
    RequireAuthorization(RequireAuthorization),

    /// Require verifiable credential presentation
    #[serde(rename = "RequirePresentation")]
    RequirePresentation(RequirePresentation),

    /// Require proof of control of an account or address
    #[serde(rename = "RequireProofOfControl")]
    RequireProofOfControl(RequireProofOfControl),
}

/// Create default implementations for the various policy types
impl Default for RequireAuthorization {
    fn default() -> Self {
        RequireAuthorization {
            type_: "RequireAuthorization".to_string(),
            from: None,
            from_role: None,
            from_agent: None,
            purpose: None,
        }
    }
}

impl Default for RequirePresentation {
    fn default() -> Self {
        RequirePresentation {
            type_: "RequirePresentation".to_string(),
            context: None,
            from: None,
            from_role: None,
            from_agent: None,
            about_party: None,
            about_agent: None,
            purpose: None,
            presentation_definition: None,
            credentials: None,
        }
    }
}

impl Default for RequireProofOfControl {
    fn default() -> Self {
        RequireProofOfControl {
            type_: "RequireProofOfControl".to_string(),
            from: None,
            from_role: None,
            from_agent: None,
            nonce: rand::random::<u64>(),
            purpose: None,
        }
    }
}
