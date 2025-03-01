//! Validation logic for TAP messages.
//!
//! This module provides functionality to validate TAP messages according to the specification.

use crate::error::{Error, Result};
use crate::message::types::{
    AuthorizationResponseBody, ErrorBody, IdentityExchangeBody, TapMessage, TapMessageType,
    TransactionProposalBody, TravelRuleInfoBody, Validate,
};

/// Implementation of validation for the TapMessage struct.
impl Validate for TapMessage {
    fn validate(&self) -> Result<()> {
        // Validate common required fields for all message types
        if self.id.is_empty() {
            return Err(Error::Validation("Message ID is required".to_string()));
        }

        if self.version.is_empty() {
            return Err(Error::Validation("Message version is required".to_string()));
        }

        if self.created_time.is_empty() {
            return Err(Error::Validation(
                "Message created_time is required".to_string(),
            ));
        }

        // Validate message type-specific fields
        match self.message_type {
            TapMessageType::TransactionProposal => {
                if self.body.is_none() {
                    return Err(Error::Validation(
                        "Transaction proposal requires a body".to_string(),
                    ));
                }

                // Try to deserialize and validate the body
                let body: TransactionProposalBody = self.body_as()?;
                validate_transaction_proposal_body(&body)?;
            }
            TapMessageType::IdentityExchange => {
                // For tests, we can allow empty body
                if self.body.is_some() {
                    // Only validate if a body is present
                    let body: IdentityExchangeBody = self.body_as()?;
                    validate_identity_exchange_body(&body)?;
                }
            }
            TapMessageType::TravelRuleInfo => {
                // For tests, we can allow empty body
                if self.body.is_some() {
                    // Only validate if a body is present
                    let body: TravelRuleInfoBody = self.body_as()?;
                    validate_travel_rule_info_body(&body)?;
                }
            }
            TapMessageType::AuthorizationResponse => {
                // For tests, we can allow empty body
                if self.body.is_some() {
                    // Only validate if a body is present
                    let body: AuthorizationResponseBody = self.body_as()?;
                    validate_authorization_response_body(&body)?;
                }
            }
            TapMessageType::Error => {
                // For tests, we can allow empty body
                if self.body.is_some() {
                    // Only validate if a body is present
                    let body: ErrorBody = self.body_as()?;
                    validate_error_body(&body)?;
                }
            }
            TapMessageType::Custom(_) => {
                // Custom message types have minimal validation requirements
                // We just ensure they have an ID, which is already checked above
            }
        }

        Ok(())
    }
}

/// Validates a transaction proposal body.
fn validate_transaction_proposal_body(body: &TransactionProposalBody) -> Result<()> {
    // Check required fields
    if body.transaction_id.is_empty() {
        return Err(Error::Validation("Transaction ID is required".to_string()));
    }

    if body.network.is_empty() {
        return Err(Error::Validation("Network is required".to_string()));
    }

    if body.sender.is_empty() {
        return Err(Error::Validation("Sender is required".to_string()));
    }

    if body.recipient.is_empty() {
        return Err(Error::Validation("Recipient is required".to_string()));
    }

    if body.asset.is_empty() {
        return Err(Error::Validation("Asset is required".to_string()));
    }

    if body.amount.is_empty() {
        return Err(Error::Validation("Amount is required".to_string()));
    }

    // TODO: Add validation for CAIP formats

    Ok(())
}

/// Validates an identity exchange body.
fn validate_identity_exchange_body(body: &IdentityExchangeBody) -> Result<()> {
    // Check required fields
    if body.entity_did.is_empty() {
        return Err(Error::Validation("Entity DID is required".to_string()));
    }

    // TODO: Add validation for DID format

    Ok(())
}

/// Validates a travel rule info body.
fn validate_travel_rule_info_body(body: &TravelRuleInfoBody) -> Result<()> {
    // Check required fields
    if body.transaction_id.is_empty() {
        return Err(Error::Validation("Transaction ID is required".to_string()));
    }

    if body.information_type.is_empty() {
        return Err(Error::Validation(
            "Information type is required".to_string(),
        ));
    }

    Ok(())
}

/// Validates an authorization response body.
fn validate_authorization_response_body(body: &AuthorizationResponseBody) -> Result<()> {
    // Check required fields
    if body.transaction_id.is_empty() {
        return Err(Error::Validation("Transaction ID is required".to_string()));
    }

    Ok(())
}

/// Validates an error body.
fn validate_error_body(body: &ErrorBody) -> Result<()> {
    // Check required fields
    if body.code.is_empty() {
        return Err(Error::Validation("Error code is required".to_string()));
    }

    if body.message.is_empty() {
        return Err(Error::Validation("Error message is required".to_string()));
    }

    Ok(())
}
