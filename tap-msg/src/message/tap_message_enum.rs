//! Enum for all TAP message types
//!
//! This module provides an enum that encompasses all TAP message types
//! and functionality to convert from PlainMessage to the appropriate TAP message.

use crate::didcomm::PlainMessage;
use crate::error::{Error, Result};
use crate::message::{
    AddAgents, AuthorizationRequired, Authorize, BasicMessage, Cancel, ConfirmRelationship,
    Connect, DIDCommPresentation, ErrorBody, OutOfBand, Payment, Presentation, Reject, RemoveAgent,
    ReplaceAgent, RequestPresentation, Revert, Settle, Transfer, TrustPing, TrustPingResponse,
    UpdateParty, UpdatePolicies,
};
use serde::{Deserialize, Serialize};

/// Enum encompassing all TAP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum TapMessage {
    /// Add agents message (TAIP-5)
    AddAgents(AddAgents),
    /// Authorize message (TAIP-8)
    Authorize(Authorize),
    /// Authorization required message (TAIP-2)
    AuthorizationRequired(AuthorizationRequired),
    /// Basic message (DIDComm 2.0)
    BasicMessage(BasicMessage),
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
    /// Trust Ping message (DIDComm 2.0)
    TrustPing(TrustPing),
    /// Trust Ping Response message (DIDComm 2.0)
    TrustPingResponse(TrustPingResponse),
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
            "https://tap.rsvp/schema/1.0#AddAgents" => {
                let msg: AddAgents =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse AddAgents: {}", e))
                    })?;
                Ok(TapMessage::AddAgents(msg))
            }
            "https://tap.rsvp/schema/1.0#Authorize" => {
                let msg: Authorize =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Authorize: {}", e))
                    })?;
                Ok(TapMessage::Authorize(msg))
            }
            "https://tap.rsvp/schema/1.0#AuthorizationRequired" => {
                let msg: AuthorizationRequired = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse AuthorizationRequired: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::AuthorizationRequired(msg))
            }
            "https://didcomm.org/basicmessage/2.0/message" => {
                let msg: BasicMessage =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse BasicMessage: {}", e))
                    })?;
                Ok(TapMessage::BasicMessage(msg))
            }
            "https://tap.rsvp/schema/1.0#Cancel" => {
                let msg: Cancel = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Cancel: {}", e))
                })?;
                Ok(TapMessage::Cancel(msg))
            }
            "https://tap.rsvp/schema/1.0#ConfirmRelationship" => {
                let msg: ConfirmRelationship = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse ConfirmRelationship: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::ConfirmRelationship(msg))
            }
            "https://tap.rsvp/schema/1.0#Connect" => {
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
            "https://tap.rsvp/schema/1.0#Error" => {
                let msg: ErrorBody =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Error: {}", e))
                    })?;
                Ok(TapMessage::Error(msg))
            }
            "https://tap.rsvp/schema/1.0#OutOfBand" => {
                let msg: OutOfBand =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse OutOfBand: {}", e))
                    })?;
                Ok(TapMessage::OutOfBand(msg))
            }
            "https://tap.rsvp/schema/1.0#Payment" => {
                let msg: Payment = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Payment: {}", e))
                })?;
                Ok(TapMessage::Payment(msg))
            }
            "https://tap.rsvp/schema/1.0#Presentation" => {
                let msg: Presentation =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Presentation: {}", e))
                    })?;
                Ok(TapMessage::Presentation(msg))
            }
            "https://tap.rsvp/schema/1.0#Reject" => {
                let msg: Reject = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Reject: {}", e))
                })?;
                Ok(TapMessage::Reject(msg))
            }
            "https://tap.rsvp/schema/1.0#RemoveAgent" => {
                let msg: RemoveAgent =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse RemoveAgent: {}", e))
                    })?;
                Ok(TapMessage::RemoveAgent(msg))
            }
            "https://tap.rsvp/schema/1.0#ReplaceAgent" => {
                let msg: ReplaceAgent =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse ReplaceAgent: {}", e))
                    })?;
                Ok(TapMessage::ReplaceAgent(msg))
            }
            "https://tap.rsvp/schema/1.0#RequestPresentation" => {
                let msg: RequestPresentation = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse RequestPresentation: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::RequestPresentation(msg))
            }
            "https://tap.rsvp/schema/1.0#Revert" => {
                let msg: Revert = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Revert: {}", e))
                })?;
                Ok(TapMessage::Revert(msg))
            }
            "https://tap.rsvp/schema/1.0#Settle" => {
                let msg: Settle = serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                    Error::SerializationError(format!("Failed to parse Settle: {}", e))
                })?;
                Ok(TapMessage::Settle(msg))
            }
            "https://tap.rsvp/schema/1.0#Transfer" => {
                let msg: Transfer =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse Transfer: {}", e))
                    })?;
                Ok(TapMessage::Transfer(msg))
            }
            "https://tap.rsvp/schema/1.0#UpdateParty" => {
                let msg: UpdateParty =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse UpdateParty: {}", e))
                    })?;
                Ok(TapMessage::UpdateParty(msg))
            }
            "https://tap.rsvp/schema/1.0#UpdatePolicies" => {
                let msg: UpdatePolicies =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse UpdatePolicies: {}", e))
                    })?;
                Ok(TapMessage::UpdatePolicies(msg))
            }
            "https://didcomm.org/trust-ping/2.0/ping" => {
                let msg: TrustPing =
                    serde_json::from_value(plain_msg.body.clone()).map_err(|e| {
                        Error::SerializationError(format!("Failed to parse TrustPing: {}", e))
                    })?;
                Ok(TapMessage::TrustPing(msg))
            }
            "https://didcomm.org/trust-ping/2.0/ping-response" => {
                let msg: TrustPingResponse = serde_json::from_value(plain_msg.body.clone())
                    .map_err(|e| {
                        Error::SerializationError(format!(
                            "Failed to parse TrustPingResponse: {}",
                            e
                        ))
                    })?;
                Ok(TapMessage::TrustPingResponse(msg))
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
            TapMessage::AddAgents(_) => "https://tap.rsvp/schema/1.0#AddAgents",
            TapMessage::Authorize(_) => "https://tap.rsvp/schema/1.0#Authorize",
            TapMessage::AuthorizationRequired(_) => {
                "https://tap.rsvp/schema/1.0#AuthorizationRequired"
            }
            TapMessage::BasicMessage(_) => "https://didcomm.org/basicmessage/2.0/message",
            TapMessage::Cancel(_) => "https://tap.rsvp/schema/1.0#Cancel",
            TapMessage::ConfirmRelationship(_) => "https://tap.rsvp/schema/1.0#ConfirmRelationship",
            TapMessage::Connect(_) => "https://tap.rsvp/schema/1.0#Connect",
            TapMessage::DIDCommPresentation(_) => {
                "https://didcomm.org/present-proof/3.0/presentation"
            }
            TapMessage::Error(_) => "https://tap.rsvp/schema/1.0#Error",
            TapMessage::OutOfBand(_) => "https://tap.rsvp/schema/1.0#OutOfBand",
            TapMessage::Payment(_) => "https://tap.rsvp/schema/1.0#Payment",
            TapMessage::Presentation(_) => "https://tap.rsvp/schema/1.0#Presentation",
            TapMessage::Reject(_) => "https://tap.rsvp/schema/1.0#Reject",
            TapMessage::RemoveAgent(_) => "https://tap.rsvp/schema/1.0#RemoveAgent",
            TapMessage::ReplaceAgent(_) => "https://tap.rsvp/schema/1.0#ReplaceAgent",
            TapMessage::RequestPresentation(_) => "https://tap.rsvp/schema/1.0#RequestPresentation",
            TapMessage::Revert(_) => "https://tap.rsvp/schema/1.0#Revert",
            TapMessage::Settle(_) => "https://tap.rsvp/schema/1.0#Settle",
            TapMessage::Transfer(_) => "https://tap.rsvp/schema/1.0#Transfer",
            TapMessage::TrustPing(_) => "https://didcomm.org/trust-ping/2.0/ping",
            TapMessage::TrustPingResponse(_) => "https://didcomm.org/trust-ping/2.0/ping-response",
            TapMessage::UpdateParty(_) => "https://tap.rsvp/schema/1.0#UpdateParty",
            TapMessage::UpdatePolicies(_) => "https://tap.rsvp/schema/1.0#UpdatePolicies",
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
            "@type": "https://tap.rsvp/schema/1.0#Transfer",
            "transaction_id": "test-tx-123",
            "asset": "eip155:1/slip44:60",
            "originator": {
                "@id": "did:example:alice"
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
            type_: "https://tap.rsvp/schema/1.0#Transfer".to_string(),
            body: json!({
                "@type": "https://tap.rsvp/schema/1.0#Transfer",
                "transaction_id": "test-tx-456",
                "asset": "eip155:1/slip44:60",
                "originator": {
                    "@id": "did:example:alice"
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
                assert_eq!(
                    transfer.originator.as_ref().unwrap().id,
                    "did:example:alice"
                );
            }
            _ => panic!("Expected Transfer message"),
        }
    }

    #[test]
    fn test_message_type() {
        let transfer = Transfer {
            asset: "eip155:1/slip44:60".parse().unwrap(),
            originator: Some(crate::message::Party::new("did:example:alice")),
            beneficiary: None,
            amount: "100".to_string(),
            agents: vec![],
            memo: None,
            settlement_id: None,
            connection_id: None,
            transaction_id: "tx-123".to_string(),
            metadata: Default::default(),
        };

        let tap_msg = TapMessage::Transfer(transfer);
        assert_eq!(
            tap_msg.message_type(),
            "https://tap.rsvp/schema/1.0#Transfer"
        );
    }
}
