//! # TAP IVMS 101 Implementation
//!
//! This crate provides a complete implementation of the IVMS 101.2023 data model
//! for the Travel Asset Protocol (TAP). It includes all data structures, validation,
//! and serialization support as defined in the interVASP Messaging Standard.
//!
//! ## Features
//!
//! - Complete IVMS 101.2023 data model implementation
//! - Comprehensive validation for all data types
//! - Serde JSON serialization/deserialization support
//! - Builder patterns for easy construction
//! - Type-safe enumerations for all IVMS code lists
//! - ISO country code and currency code validation
//!
//! ## Example Usage
//!
//! ```rust
//! use tap_ivms101::builder::*;
//! use tap_ivms101::types::*;
//! use tap_ivms101::message::*;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a natural person
//! let natural_person_name = NaturalPersonNameBuilder::new()
//!     .legal_name("Smith", "John")
//!     .build()?;
//!
//! let natural_person = NaturalPersonBuilder::new()
//!     .name(natural_person_name)
//!     .country_of_residence("US")
//!     .build()?;
//!
//! // Create a legal person (VASP)
//! let legal_person_name = LegalPersonNameBuilder::new()
//!     .legal_name("Example VASP Inc.")
//!     .build()?;
//!
//! let legal_person = LegalPersonBuilder::new()
//!     .name(legal_person_name)
//!     .lei("529900HNOAA1KXQJUQ27")
//!     .country_of_registration("US")
//!     .build()?;
//!
//! // Create an IVMS message
//! let ivms_message = IvmsMessageBuilder::new()
//!     .originator(vec![Person::Natural(natural_person)])
//!     .beneficiary(vec![Person::Natural(natural_person.clone())])
//!     .originating_vasp(Person::Legal(legal_person))
//!     .transaction(
//!         "100.00",
//!         "USD",
//!         TransactionDirection::Outgoing,
//!         "tx-123",
//!         "2024-01-15T10:30:00Z"
//!     )?
//!     .build()?;
//!
//! // Serialize to JSON
//! let json = ivms_message.to_json_pretty()?;
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod error;
pub mod message;
pub mod person;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use error::{Error, Result};
pub use message::{IvmsMessage, Person};
pub use person::{LegalPerson, NaturalPerson};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::*;
    use crate::types::*;

    #[test]
    fn test_create_natural_person() {
        let name = NaturalPersonNameBuilder::new()
            .legal_name("Doe", "John")
            .build()
            .unwrap();

        let person = NaturalPersonBuilder::new()
            .name(name)
            .country_of_residence("US")
            .build()
            .unwrap();

        assert!(person.validate().is_ok());
    }

    #[test]
    fn test_create_legal_person() {
        let name = LegalPersonNameBuilder::new()
            .legal_name("Test Company Ltd.")
            .build()
            .unwrap();

        let person = LegalPersonBuilder::new()
            .name(name)
            .lei("529900HNOAA1KXQJUQ27")
            .country_of_registration("GB")
            .build()
            .unwrap();

        assert!(person.validate().is_ok());
    }

    #[test]
    fn test_create_ivms_message() {
        // Create originator
        let originator_name = NaturalPersonNameBuilder::new()
            .legal_name("Smith", "Alice")
            .build()
            .unwrap();

        let originator_person = NaturalPersonBuilder::new()
            .name(originator_name)
            .build()
            .unwrap();

        // Create beneficiary
        let beneficiary_name = NaturalPersonNameBuilder::new()
            .legal_name("Jones", "Bob")
            .build()
            .unwrap();

        let beneficiary_person = NaturalPersonBuilder::new()
            .name(beneficiary_name)
            .build()
            .unwrap();

        // Create VASP
        let vasp_name = LegalPersonNameBuilder::new()
            .legal_name("Example Exchange")
            .build()
            .unwrap();

        let vasp = LegalPersonBuilder::new()
            .name(vasp_name)
            .build()
            .unwrap();

        // Create message
        let message = IvmsMessageBuilder::new()
            .originator(vec![Person::Natural(originator_person)])
            .beneficiary(vec![Person::Natural(beneficiary_person)])
            .originating_vasp(Person::Legal(vasp))
            .transaction(
                "1000.50",
                "USD",
                TransactionDirection::Outgoing,
                "tx-12345",
                "2024-01-15T10:30:00Z",
            )
            .unwrap()
            .build();

        assert!(message.is_ok());
        let message = message.unwrap();
        assert!(message.validate().is_ok());

        // Test serialization
        let json = message.to_json();
        assert!(json.is_ok());

        // Test deserialization
        let parsed = IvmsMessage::from_json(&json.unwrap());
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_validation_errors() {
        // Test empty name
        let result = NaturalPersonNameBuilder::new().build();
        assert!(result.is_err());

        // Test invalid country code
        let name = NaturalPersonNameBuilder::new()
            .legal_name("Test", "User")
            .build()
            .unwrap();

        let result = NaturalPersonBuilder::new()
            .name(name)
            .country_of_residence("USA") // Should be "US"
            .build();

        assert!(result.is_err());
    }
}