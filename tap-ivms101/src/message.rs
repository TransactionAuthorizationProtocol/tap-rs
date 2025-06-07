//! IVMS 101 message data structures
//!
//! This module implements the top-level IVMS message structures including
//! originator, beneficiary, and transaction data.

use crate::error::{Error, Result};
use crate::person::{LegalPerson, NaturalPerson};
use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Person type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Person {
    /// Natural person
    #[serde(rename = "naturalPerson")]
    Natural(NaturalPerson),
    /// Legal person
    #[serde(rename = "legalPerson")]
    Legal(LegalPerson),
}

impl Person {
    /// Validate the person data
    pub fn validate(&self) -> Result<()> {
        match self {
            Person::Natural(person) => person.validate(),
            Person::Legal(person) => person.validate(),
        }
    }

    /// Check if this is a natural person
    pub fn is_natural_person(&self) -> bool {
        matches!(self, Person::Natural(_))
    }

    /// Check if this is a legal person
    pub fn is_legal_person(&self) -> bool {
        matches!(self, Person::Legal(_))
    }
}

/// Originator information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Originator {
    /// Originator persons (at least one required)
    pub originator_persons: Vec<Person>,
    /// Account numbers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_numbers: Option<Vec<String>>,
    /// BIC (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<BicCode>,
}

impl Originator {
    /// Create a new originator
    pub fn new(originator_persons: Vec<Person>) -> Self {
        Self {
            originator_persons,
            account_numbers: None,
            bic: None,
        }
    }

    /// Validate the originator data
    pub fn validate(&self) -> Result<()> {
        if self.originator_persons.is_empty() {
            return Err(Error::MissingRequiredField(
                "At least one originator person is required".to_string(),
            ));
        }

        for person in &self.originator_persons {
            person.validate()?;
        }

        if let Some(ref bic) = self.bic {
            if bic.len() != 8 && bic.len() != 11 {
                return Err(Error::InvalidBic(format!(
                    "BIC must be 8 or 11 characters: {}",
                    bic
                )));
            }
        }

        Ok(())
    }
}

/// Beneficiary information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Beneficiary {
    /// Beneficiary persons (at least one required)
    pub beneficiary_persons: Vec<Person>,
    /// Account numbers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_numbers: Option<Vec<String>>,
}

impl Beneficiary {
    /// Create a new beneficiary
    pub fn new(beneficiary_persons: Vec<Person>) -> Self {
        Self {
            beneficiary_persons,
            account_numbers: None,
        }
    }

    /// Validate the beneficiary data
    pub fn validate(&self) -> Result<()> {
        if self.beneficiary_persons.is_empty() {
            return Err(Error::MissingRequiredField(
                "At least one beneficiary person is required".to_string(),
            ));
        }

        for person in &self.beneficiary_persons {
            person.validate()?;
        }

        Ok(())
    }
}

/// Originating VASP information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginatingVasp {
    /// Originating VASP person
    pub originating_vasp: Person,
    /// BIC (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bic: Option<BicCode>,
}

impl OriginatingVasp {
    /// Create a new originating VASP
    pub fn new(originating_vasp: Person) -> Self {
        Self {
            originating_vasp,
            bic: None,
        }
    }

    /// Validate the originating VASP data
    pub fn validate(&self) -> Result<()> {
        self.originating_vasp.validate()?;

        if let Some(ref bic) = self.bic {
            if bic.len() != 8 && bic.len() != 11 {
                return Err(Error::InvalidBic(format!(
                    "BIC must be 8 or 11 characters: {}",
                    bic
                )));
            }
        }

        Ok(())
    }
}

/// Beneficiary VASP information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeneficiaryVasp {
    /// Beneficiary VASP person
    pub beneficiary_vasp: Person,
}

impl BeneficiaryVasp {
    /// Create a new beneficiary VASP
    pub fn new(beneficiary_vasp: Person) -> Self {
        Self { beneficiary_vasp }
    }

    /// Validate the beneficiary VASP data
    pub fn validate(&self) -> Result<()> {
        self.beneficiary_vasp.validate()
    }
}

/// Transaction data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    /// Transaction amount
    pub amount: String,
    /// Currency code (ISO 4217)
    pub currency: CurrencyCode,
    /// Transaction direction
    pub direction: TransactionDirection,
    /// Payment type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_type: Option<PaymentType>,
    /// Transaction identifier
    pub transaction_identifier: String,
    /// Transaction datetime (ISO 8601)
    pub transaction_datetime: String,
    /// Transaction network (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_network: Option<TransactionNetworkType>,
    /// Transaction hash (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
}

impl TransactionData {
    /// Validate the transaction data
    pub fn validate(&self) -> Result<()> {
        if self.amount.is_empty() {
            return Err(Error::MissingRequiredField(
                "Transaction amount is required".to_string(),
            ));
        }

        if self.currency.len() != 3 {
            return Err(Error::InvalidCurrencyCode(format!(
                "Currency code must be 3 characters: {}",
                self.currency
            )));
        }

        if self.transaction_identifier.is_empty() {
            return Err(Error::MissingRequiredField(
                "Transaction identifier is required".to_string(),
            ));
        }

        // Validate datetime format
        if chrono::DateTime::parse_from_rfc3339(&self.transaction_datetime).is_err() {
            return Err(Error::InvalidDate(format!(
                "Invalid transaction datetime format: {}",
                self.transaction_datetime
            )));
        }

        Ok(())
    }
}

/// Main IVMS 101 message structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IvmsMessage {
    /// Originator information
    pub originator: Originator,
    /// Beneficiary information
    pub beneficiary: Beneficiary,
    /// Originating VASP information
    pub originating_vasp: OriginatingVasp,
    /// Beneficiary VASP information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beneficiary_vasp: Option<BeneficiaryVasp>,
    /// Transaction data
    pub transaction: TransactionData,
}

impl IvmsMessage {
    /// Create a new IVMS message
    pub fn new(
        originator: Originator,
        beneficiary: Beneficiary,
        originating_vasp: OriginatingVasp,
        transaction: TransactionData,
    ) -> Self {
        Self {
            originator,
            beneficiary,
            originating_vasp,
            beneficiary_vasp: None,
            transaction,
        }
    }

    /// Validate the entire IVMS message
    pub fn validate(&self) -> Result<()> {
        let mut issues = Vec::new();

        if let Err(e) = self.originator.validate() {
            issues.push(format!("Originator: {}", e));
        }

        if let Err(e) = self.beneficiary.validate() {
            issues.push(format!("Beneficiary: {}", e));
        }

        if let Err(e) = self.originating_vasp.validate() {
            issues.push(format!("Originating VASP: {}", e));
        }

        if let Some(ref beneficiary_vasp) = self.beneficiary_vasp {
            if let Err(e) = beneficiary_vasp.validate() {
                issues.push(format!("Beneficiary VASP: {}", e));
            }
        }

        if let Err(e) = self.transaction.validate() {
            issues.push(format!("Transaction: {}", e));
        }

        if !issues.is_empty() {
            return Err(Error::ValidationFailed { issues });
        }

        Ok(())
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::from)
    }

    /// Convert to pretty JSON
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Error::from)
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Error::from)
    }
}