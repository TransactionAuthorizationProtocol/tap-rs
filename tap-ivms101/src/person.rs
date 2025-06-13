//! Person data structures for IVMS 101.2023
//!
//! This module implements the natural person and legal person data structures
//! as defined in the IVMS 101.2023 specification.

use crate::error::{Error, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use tap_msg::utils::NameHashable;

/// Name identifier for natural person
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameIdentifier {
    /// Primary identifier (typically family name)
    pub primary_identifier: String,
    /// Secondary identifier (typically given names)
    pub secondary_identifier: String,
    /// Name identifier type
    pub name_identifier_type: NameIdentifierType,
}

impl NameIdentifier {
    /// Create a new name identifier
    pub fn new(
        primary_identifier: impl Into<String>,
        secondary_identifier: impl Into<String>,
        name_identifier_type: NameIdentifierType,
    ) -> Self {
        Self {
            primary_identifier: primary_identifier.into(),
            secondary_identifier: secondary_identifier.into(),
            name_identifier_type,
        }
    }

    /// Validate the name identifier
    pub fn validate(&self) -> Result<()> {
        if self.primary_identifier.is_empty() {
            return Err(Error::InvalidName(
                "Primary identifier cannot be empty".to_string(),
            ));
        }
        if self.primary_identifier.len() > 150 {
            return Err(Error::InvalidName(
                "Primary identifier exceeds 150 characters".to_string(),
            ));
        }
        if self.secondary_identifier.len() > 150 {
            return Err(Error::InvalidName(
                "Secondary identifier exceeds 150 characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// Local name identifier for natural person (in local script)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalNameIdentifier {
    /// Primary identifier in local script
    pub primary_identifier: String,
    /// Secondary identifier in local script
    pub secondary_identifier: String,
    /// Name identifier type
    pub name_identifier_type: NameIdentifierType,
}

/// Phonetic name identifier for natural person
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneticNameIdentifier {
    /// Primary identifier phonetic representation
    pub primary_identifier: String,
    /// Secondary identifier phonetic representation
    pub secondary_identifier: String,
    /// Name identifier type
    pub name_identifier_type: NameIdentifierType,
}

/// Natural person name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalPersonName {
    /// Name identifiers (at least one required)
    pub name_identifiers: Vec<NameIdentifier>,
    /// Local name identifiers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name_identifiers: Option<Vec<LocalNameIdentifier>>,
    /// Phonetic name identifiers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phonetic_name_identifiers: Option<Vec<PhoneticNameIdentifier>>,
}

impl NaturalPersonName {
    /// Create a new natural person name
    pub fn new(name_identifiers: Vec<NameIdentifier>) -> Self {
        Self {
            name_identifiers,
            local_name_identifiers: None,
            phonetic_name_identifiers: None,
        }
    }

    /// Validate the natural person name
    pub fn validate(&self) -> Result<()> {
        if self.name_identifiers.is_empty() {
            return Err(Error::InvalidName(
                "At least one name identifier is required".to_string(),
            ));
        }
        for identifier in &self.name_identifiers {
            identifier.validate()?;
        }
        Ok(())
    }

    /// Get the full name as a single string for TAIP-12 hashing
    /// Concatenates secondary identifier (given names) and primary identifier (family name)
    pub fn get_full_name(&self) -> Option<String> {
        self.name_identifiers.first().map(|name| {
            format!(
                "{} {}",
                name.secondary_identifier.trim(),
                name.primary_identifier.trim()
            )
            .trim()
            .to_string()
        })
    }
}

// Implement NameHashable for NaturalPersonName
impl NameHashable for NaturalPersonName {}

/// National identification for natural person
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NationalIdentification {
    /// National identifier
    pub national_identifier: String,
    /// National identifier type
    pub national_identifier_type: NationalIdentifierType,
    /// Country of issue (ISO 3166-1 alpha-2)
    pub country_of_issue: CountryCode,
    /// Registration authority (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_authority: Option<String>,
}

impl NationalIdentification {
    /// Validate the national identification
    pub fn validate(&self) -> Result<()> {
        if self.national_identifier.is_empty() {
            return Err(Error::InvalidNationalId(
                "National identifier cannot be empty".to_string(),
            ));
        }
        if self.national_identifier.len() > 35 {
            return Err(Error::InvalidNationalId(
                "National identifier exceeds 35 characters".to_string(),
            ));
        }
        if self.country_of_issue.len() != 2 {
            return Err(Error::InvalidCountryCode(format!(
                "Country code must be 2 characters: {}",
                self.country_of_issue
            )));
        }
        Ok(())
    }
}

/// Customer identification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerIdentification {
    /// Customer identifier
    pub customer_identifier: String,
    /// Customer identification type
    pub customer_identification_type: CustomerIdentificationType,
}

impl CustomerIdentification {
    /// Validate the customer identification
    pub fn validate(&self) -> Result<()> {
        if self.customer_identifier.is_empty() {
            return Err(Error::InvalidCustomerId(
                "Customer identifier cannot be empty".to_string(),
            ));
        }
        if self.customer_identifier.len() > 50 {
            return Err(Error::InvalidCustomerId(
                "Customer identifier exceeds 50 characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// Natural person data structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalPerson {
    /// Natural person name
    pub name: NaturalPersonName,
    /// Geographic addresses (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographic_addresses: Option<Vec<GeographicAddress>>,
    /// National identification (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub national_identification: Option<NationalIdentification>,
    /// Customer identification (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_identification: Option<CustomerIdentification>,
    /// Date and place of birth (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_and_place_of_birth: Option<DateAndPlaceOfBirth>,
    /// Country of residence (optional, ISO 3166-1 alpha-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_of_residence: Option<CountryCode>,
}

impl NaturalPerson {
    /// Create a new natural person
    pub fn new(name: NaturalPersonName) -> Self {
        Self {
            name,
            geographic_addresses: None,
            national_identification: None,
            customer_identification: None,
            date_and_place_of_birth: None,
            country_of_residence: None,
        }
    }

    /// Validate the natural person data
    pub fn validate(&self) -> Result<()> {
        self.name.validate()?;

        if let Some(ref id) = self.national_identification {
            id.validate()?;
        }

        if let Some(ref cust_id) = self.customer_identification {
            cust_id.validate()?;
        }

        if let Some(ref country) = self.country_of_residence {
            if country.len() != 2 {
                return Err(Error::InvalidCountryCode(format!(
                    "Country code must be 2 characters: {}",
                    country
                )));
            }
        }

        Ok(())
    }
}

/// Legal person name identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalPersonNameIdentifier {
    /// Legal person name
    pub legal_person_name: String,
    /// Legal person name identifier type
    pub legal_person_name_identifier_type: LegalPersonNameIdentifierType,
}

impl LegalPersonNameIdentifier {
    /// Create a new legal person name identifier
    pub fn new(
        legal_person_name: impl Into<String>,
        legal_person_name_identifier_type: LegalPersonNameIdentifierType,
    ) -> Self {
        Self {
            legal_person_name: legal_person_name.into(),
            legal_person_name_identifier_type,
        }
    }

    /// Validate the legal person name identifier
    pub fn validate(&self) -> Result<()> {
        if self.legal_person_name.is_empty() {
            return Err(Error::InvalidName(
                "Legal person name cannot be empty".to_string(),
            ));
        }
        if self.legal_person_name.len() > 150 {
            return Err(Error::InvalidName(
                "Legal person name exceeds 150 characters".to_string(),
            ));
        }
        Ok(())
    }
}

/// Legal person name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalPersonName {
    /// Name identifiers (at least one required)
    pub name_identifiers: Vec<LegalPersonNameIdentifier>,
    /// Local name identifiers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name_identifiers: Option<Vec<LegalPersonNameIdentifier>>,
    /// Phonetic name identifiers (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phonetic_name_identifiers: Option<Vec<LegalPersonNameIdentifier>>,
}

impl LegalPersonName {
    /// Create a new legal person name
    pub fn new(name_identifiers: Vec<LegalPersonNameIdentifier>) -> Self {
        Self {
            name_identifiers,
            local_name_identifiers: None,
            phonetic_name_identifiers: None,
        }
    }

    /// Validate the legal person name
    pub fn validate(&self) -> Result<()> {
        if self.name_identifiers.is_empty() {
            return Err(Error::InvalidName(
                "At least one name identifier is required".to_string(),
            ));
        }
        for identifier in &self.name_identifiers {
            identifier.validate()?;
        }
        Ok(())
    }

    /// Get the full legal name as a single string for TAIP-12 hashing
    pub fn get_full_name(&self) -> Option<String> {
        self.name_identifiers
            .first()
            .map(|name| name.legal_person_name.trim().to_string())
    }
}

// Implement NameHashable for LegalPersonName
impl NameHashable for LegalPersonName {}

/// Registration authority
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationAuthority {
    /// Registration authority code
    pub registration_authority_code: String,
    /// Registration authority type
    pub registration_authority_type: RegistrationAuthorityType,
}

/// Legal person national identification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalPersonNationalIdentification {
    /// National identifier
    pub national_identifier: String,
    /// National identifier type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub national_identifier_type: Option<String>,
    /// Country of issue (optional, ISO 3166-1 alpha-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_of_issue: Option<CountryCode>,
    /// Registration authority (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_authority: Option<RegistrationAuthority>,
    /// LEI code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lei_code: Option<LeiCode>,
}

impl LegalPersonNationalIdentification {
    /// Validate the legal person national identification
    pub fn validate(&self) -> Result<()> {
        if self.national_identifier.is_empty() {
            return Err(Error::InvalidNationalId(
                "National identifier cannot be empty".to_string(),
            ));
        }

        if let Some(ref lei) = self.lei_code {
            if lei.len() != 20 {
                return Err(Error::InvalidLei(format!(
                    "LEI must be 20 characters: {}",
                    lei
                )));
            }
        }

        Ok(())
    }
}

/// Legal person data structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalPerson {
    /// Legal person name
    pub name: LegalPersonName,
    /// Geographic addresses (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographic_addresses: Option<Vec<GeographicAddress>>,
    /// National identification (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub national_identification: Option<LegalPersonNationalIdentification>,
    /// Customer identification (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_identification: Option<CustomerIdentification>,
    /// Country of registration (optional, ISO 3166-1 alpha-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_of_registration: Option<CountryCode>,
}

impl LegalPerson {
    /// Create a new legal person
    pub fn new(name: LegalPersonName) -> Self {
        Self {
            name,
            geographic_addresses: None,
            national_identification: None,
            customer_identification: None,
            country_of_registration: None,
        }
    }

    /// Validate the legal person data
    pub fn validate(&self) -> Result<()> {
        self.name.validate()?;

        if let Some(ref id) = self.national_identification {
            id.validate()?;
        }

        if let Some(ref cust_id) = self.customer_identification {
            cust_id.validate()?;
        }

        if let Some(ref country) = self.country_of_registration {
            if country.len() != 2 {
                return Err(Error::InvalidCountryCode(format!(
                    "Country code must be 2 characters: {}",
                    country
                )));
            }
        }

        Ok(())
    }
}
