//! Enum for all TAP message types
//!
//! This module provides an enum that encompasses all TAP message types
//! and functionality to convert from PlainMessage to the appropriate TAP message.

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::message::{
    AddAgents, AuthorizationRequired, Authorize, Cancel, ConfirmRelationship, Connect,
    DIDCommPresentation, ErrorBody, OutOfBand, Payment, Presentation, Reject, RemoveAgent,
    ReplaceAgent, RequestPresentation, Revert, Settle, Transfer, UpdateParty, UpdatePolicies,
};
use serde::{Deserialize, Serialize};

/// Enum encompassing all TAP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TapMessage {
    /// Add agents message (TAIP-5)
    AddAgents(AddAgents),
    /// Authorize message (TAIP-8)
    Authorize(Authorize),
    /// Authorization required message (TAIP-2)
    AuthorizationRequired(AuthorizationRequired),
    /// Cancel message (TAIP-11)
    Cancel(Cancel),
    /// Confirm relationship message (TAIP-14)
    ConfirmRelationship(ConfirmRelationship),
    /// Connect message (TAIP-2)
    Connect(Connect),
    /// DIDComm presentation message
    DIDCommPresentation(DIDCommPresentation),
    /// Error message
    Error(ErrorBody),
    /// Out of band message (TAIP-2)
    OutOfBand(OutOfBand),
    /// Payment message (TAIP-13)
    Payment(Payment),
    /// Presentation message (TAIP-6)
    Presentation(Presentation),
    /// Reject message (TAIP-10)
    Reject(Reject),
    /// Remove agent message (TAIP-5)
    RemoveAgent(RemoveAgent),
    /// Replace agent message (TAIP-5)
    ReplaceAgent(ReplaceAgent),
    /// Request presentation message (TAIP-6)
    RequestPresentation(RequestPresentation),
    /// Revert message (TAIP-12)
    Revert(Revert),
    /// Settle message (TAIP-9)
    Settle(Settle),
    /// Transfer message (TAIP-3)
    Transfer(Transfer),
    /// Update party message (TAIP-4)
    UpdateParty(UpdateParty),
    /// Update policies message (TAIP-7)
    UpdatePolicies(UpdatePolicies),
}

impl TapMessage {
    /// Convert a PlainMessage into the appropriate TapMessage variant
    /// based on the message type field
    pub fn from_plain_message(plain_msg: &PlainMessage) -> Result<Self> {
        // Extract the type from either the type_ field or from the body's @type field
        let message_type =
            if !plain_msg.type_.is_empty() && plain_msg.type_ != "application/didcomm-plain+json" {
                &plain_msg.type_
            } else if let Some(body_obj) = plain_msg.body.as_object() {
                if let Some(type_val) = body_obj.get("@type") {
                    type_val.as_str().unwrap_or("")
                } else {
                    ""
                }
            } else {
                ""
            };

        if message_type.is_empty() {
            return Err(Error::Validation(
                "Message type not found in PlainMessage".to_string(),
            ));
        }

        // Parse the message body based on the type
        match message_type {
            "https://tap.rsvp/schema/1.0#add-agents" => {
                let msg: AddAgents =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse AddAgents: {}", e))
                    })?;
                Ok(TapMessage::AddAgents(msg))
            }
            "https://tap.rsvp/schema/1.0#authorize" => {
                let msg: Authorize =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Authorize: {}", e))
                    })?;
                Ok(TapMessage::Authorize(msg))
            }
            "https://tap.rsvp/schema/1.0#authorizationrequired" => {
                let msg: AuthorizationRequired = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse AuthorizationRequired: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::AuthorizationRequired(msg))
            }
            "https://tap.rsvp/schema/1.0#cancel" => {
                let msg: Cancel = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Cancel: {}", e))
                })?;
                Ok(TapMessage::Cancel(msg))
            }
            "https://tap.rsvp/schema/1.0#confirmrelationship" => {
                let msg: ConfirmRelationship = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse ConfirmRelationship: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::ConfirmRelationship(msg))
            }
            "https://tap.rsvp/schema/1.0#connect" => {
                let msg: Connect = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Connect: {}", e))
                })?;
                Ok(TapMessage::Connect(msg))
            }
            "https://didcomm.org/present-proof/3.0/presentation" => {
                let msg: DIDCommPresentation = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse DIDCommPresentation: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::DIDCommPresentation(msg))
            }
            "https://tap.rsvp/schema/1.0#error" => {
                let msg: ErrorBody =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Error: {}", e))
                    })?;
                Ok(TapMessage::Error(msg))
            }
            "https://tap.rsvp/schema/1.0#outofband" => {
                let msg: OutOfBand =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse OutOfBand: {}", e))
                    })?;
                Ok(TapMessage::OutOfBand(msg))
            }
            "https://tap.rsvp/schema/1.0#payment" => {
                let msg: Payment = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Payment: {}", e))
                })?;
                Ok(TapMessage::Payment(msg))
            }
            "https://tap.rsvp/schema/1.0#presentation" => {
                let msg: Presentation =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Presentation: {}", e))
                    })?;
                Ok(TapMessage::Presentation(msg))
            }
            "https://tap.rsvp/schema/1.0#reject" => {
                let msg: Reject = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Reject: {}", e))
                })?;
                Ok(TapMessage::Reject(msg))
            }
            "https://tap.rsvp/schema/1.0#remove-agent" => {
                let msg: RemoveAgent =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse RemoveAgent: {}", e))
                    })?;
                Ok(TapMessage::RemoveAgent(msg))
            }
            "https://tap.rsvp/schema/1.0#replace-agent" => {
                let msg: ReplaceAgent =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse ReplaceAgent: {}", e))
                    })?;
                Ok(TapMessage::ReplaceAgent(msg))
            }
            "https://tap.rsvp/schema/1.0#request-presentation" => {
                let msg: RequestPresentation = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse RequestPresentation: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::RequestPresentation(msg))
            }
            "https://tap.rsvp/schema/1.0#revert" => {
                let msg: Revert = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Revert: {}", e))
                })?;
                Ok(TapMessage::Revert(msg))
            }
            "https://tap.rsvp/schema/1.0#settle" => {
                let msg: Settle = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Settle: {}", e))
                })?;
                Ok(TapMessage::Settle(msg))
            }
            "https://tap.rsvp/schema/1.0#transfer" => {
                let msg: Transfer =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Transfer: {}", e))
                    })?;
                Ok(TapMessage::Transfer(msg))
            }
            "https://tap.rsvp/schema/1.0#update-party" => {
                let msg: UpdateParty =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse UpdateParty: {}", e))
                    })?;
                Ok(TapMessage::UpdateParty(msg))
            }
            "https://tap.rsvp/schema/1.0#update-policies" => {
                let msg: UpdatePolicies =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse UpdatePolicies: {}", e))
                    })?;
                Ok(TapMessage::UpdatePolicies(msg))
            }
            _ => Err(Error::Validation(format!(
                "Unknown message type: {}",
                message_type
            ))),
        }
    }

    /// Get the message type string for this TapMessage
    pub fn message_type(&self) -> &'static str {
        match self {
            TapMessage::AddAgents(_) => "https://tap.rsvp/schema/1.0#add-agents",
            TapMessage::Authorize(_) => "https://tap.rsvp/schema/1.0#authorize",
            TapMessage::AuthorizationRequired(_) => {
                "https://tap.rsvp/schema/1.0#authorizationrequired"
            }
            TapMessage::Cancel(_) => "https://tap.rsvp/schema/1.0#cancel",
            TapMessage::ConfirmRelationship(_) => "https://tap.rsvp/schema/1.0#confirmrelationship",
            TapMessage::Connect(_) => "https://tap.rsvp/schema/1.0#connect",
            TapMessage::DIDCommPresentation(_) => {
                "https://didcomm.org/present-proof/3.0/presentation"
            }
            TapMessage::Error(_) => "https://tap.rsvp/schema/1.0#error",
            TapMessage::OutOfBand(_) => "https://tap.rsvp/schema/1.0#outofband",
            TapMessage::Payment(_) => "https://tap.rsvp/schema/1.0#payment",
            TapMessage::Presentation(_) => "https://tap.rsvp/schema/1.0#presentation",
            TapMessage::Reject(_) => "https://tap.rsvp/schema/1.0#reject",
            TapMessage::RemoveAgent(_) => "https://tap.rsvp/schema/1.0#remove-agent",
            TapMessage::ReplaceAgent(_) => "https://tap.rsvp/schema/1.0#replace-agent",
            TapMessage::RequestPresentation(_) => {
                "https://tap.rsvp/schema/1.0#request-presentation"
            }
            TapMessage::Revert(_) => "https://tap.rsvp/schema/1.0#revert",
            TapMessage::Settle(_) => "https://tap.rsvp/schema/1.0#settle",
            TapMessage::Transfer(_) => "https://tap.rsvp/schema/1.0#transfer",
            TapMessage::UpdateParty(_) => "https://tap.rsvp/schema/1.0#update-party",
            TapMessage::UpdatePolicies(_) => "https://tap.rsvp/schema/1.0#update-policies",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_transfer_body() {
        let body = json!({
            "@type": "https://tap.rsvp/schema/1.0#transfer",
            "asset": {
                "chain_id": {
                    "namespace": "eip155",
                    "reference": "1"
                },
                "namespace": "slip44",
                "reference": "60"
            },
            "originator": {
                "id": "did:example:alice",
                "name": "Test Originator"
            },
            "amount": "100",
            "agents": [],
            "metadata": {}
        });

        match serde_json::from_value::<Transfer>(body.clone()) {
            Ok(transfer) => {
                println!("Successfully parsed Transfer: {:?}", transfer);
                assert_eq!(transfer.amount, "100");
            }
            Err(e) => {
                panic!("Failed to parse Transfer: {}", e);
            }
        }
    }

    #[test]
    fn test_from_plain_message_transfer() {
        let plain_msg = PlainMessage {
            id: "test-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://tap.rsvp/schema/1.0#transfer".to_string(),
            body: json!({
                "@type": "https://tap.rsvp/schema/1.0#transfer",
                "asset": {
                    "chain_id": {
                        "namespace": "eip155",
                        "reference": "1"
                    },
                    "namespace": "slip44",
                    "reference": "60"
                },
                "originator": {
                    "id": "did:example:alice",
                    "name": "Alice"
                },
                "amount": "100",
                "agents": [],
                "metadata": {}
            }),
            from: "did:example:alice".to_string(),
            to: vec!["did:example:bob".to_string()],
            thid: None,
            pthid: None,
            created_time: Some(1234567890),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: Default::default(),
        };

        let tap_msg = TapMessage::from_plain_message(&plain_msg).unwrap();

        match tap_msg {
            TapMessage::Transfer(transfer) => {
                assert_eq!(transfer.amount, "100");
                assert_eq!(transfer.originator.id, "did:example:alice");
            }
            _ => panic!("Expected Transfer message"),
        }
    }

    #[test]
    fn test_message_type() {
        let transfer = Transfer {
            asset: "eip155:1/slip44:60".parse().unwrap(),
            originator: crate::message::Participant {
                id: "did:example:alice".to_string(),
                name: Some("Alice".to_string()),
                role: None,
                policies: None,
                leiCode: None,
            },
            beneficiary: None,
            amount: "100".to_string(),
            agents: vec![],
            memo: None,
            settlement_id: None,
            transaction_id: "tx-123".to_string(),
            metadata: Default::default(),
        };

        let tap_msg = TapMessage::Transfer(transfer);
        assert_eq!(
            tap_msg.message_type(),
            "https://tap.rsvp/schema/1.0#transfer"
        );
    }
}
