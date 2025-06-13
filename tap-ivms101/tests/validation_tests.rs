//! Additional validation tests for IVMS 101 data structures

use tap_ivms101::builder::*;
use tap_ivms101::error::Error;
use tap_ivms101::types::*;
use tap_ivms101::validation::*;

#[test]
fn test_lei_validation_edge_cases() {
    // Valid LEI
    assert!(validate_lei("529900HNOAA1KXQJUQ27").is_ok());

    // Invalid: too short
    assert!(validate_lei("529900HNOAA1KXQJUQ2").is_err());

    // Invalid: too long
    assert!(validate_lei("529900HNOAA1KXQJUQ277X").is_err());

    // Invalid: lowercase letters
    assert!(validate_lei("529900hnoaa1kxqjuq27").is_err());

    // Invalid: special characters
    assert!(validate_lei("529900-NOAA1KXQJUQ27").is_err());
}

#[test]
fn test_bic_validation_edge_cases() {
    // Valid 8-character BIC
    assert!(validate_bic("DEUTDEFF").is_ok());

    // Valid 11-character BIC
    assert!(validate_bic("DEUTDEFFXXX").is_ok());

    // Invalid: wrong length (10 chars)
    assert!(validate_bic("DEUTDEFFXX").is_err());

    // Invalid: lowercase
    assert!(validate_bic("deutdeff").is_err());

    // Invalid: numbers in first 4 positions
    assert!(validate_bic("1EUTDEFF").is_err());

    // Invalid: special characters
    assert!(validate_bic("DEUT-EFF").is_err());
}

#[test]
fn test_natural_person_validation() {
    // Test missing name
    let result = NaturalPersonBuilder::new().build();
    assert!(matches!(result, Err(Error::MissingRequiredField(_))));

    // Test invalid country code
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Test", "User")
        .build()
        .unwrap();

    let result = NaturalPersonBuilder::new()
        .name(name)
        .country_of_residence("USA") // Should be "US"
        .build();

    assert!(matches!(result, Err(Error::InvalidCountryCode(_))));
}

#[test]
fn test_legal_person_validation() {
    // Test missing name in builder
    let result = LegalPersonBuilder::new().build();
    assert!(matches!(result, Err(Error::MissingRequiredField(_))));

    // Test invalid country code
    let name = LegalPersonNameBuilder::new()
        .legal_name("Test Company")
        .build()
        .unwrap();

    let result = LegalPersonBuilder::new()
        .name(name)
        .country_of_registration("USA") // Should be "US"
        .build();

    assert!(matches!(result, Err(Error::InvalidCountryCode(_))));
}

#[test]
fn test_transaction_validation() {
    use tap_ivms101::message::TransactionData;

    // Test invalid datetime format
    let tx = TransactionData {
        amount: "100.00".to_string(),
        currency: "USD".to_string(),
        direction: TransactionDirection::Outgoing,
        payment_type: None,
        transaction_identifier: "TX-001".to_string(),
        transaction_datetime: "2024-01-15 10:30:00".to_string(), // Invalid format
        transaction_network: None,
        transaction_hash: None,
    };

    assert!(matches!(tx.validate(), Err(Error::InvalidDate(_))));

    // Test empty amount
    let tx = TransactionData {
        amount: "".to_string(),
        currency: "USD".to_string(),
        direction: TransactionDirection::Outgoing,
        payment_type: None,
        transaction_identifier: "TX-001".to_string(),
        transaction_datetime: "2024-01-15T10:30:00Z".to_string(),
        transaction_network: None,
        transaction_hash: None,
    };

    assert!(matches!(tx.validate(), Err(Error::MissingRequiredField(_))));
}

#[test]
fn test_address_validation() {
    // Test address with very long address lines
    let builder = GeographicAddressBuilder::new()
        .street_name("Main Street")
        .post_code("12345")
        .town_name("Test Town")
        .country("US");

    // The builder should accept the address
    let address = builder.build().unwrap();

    // Verify required fields are present
    assert_eq!(address.street_name, "Main Street");
    assert_eq!(address.post_code, "12345");
    assert_eq!(address.town_name, "Test Town");
    assert_eq!(address.country, "US");
}

#[test]
fn test_name_identifier_validation() {
    use tap_ivms101::person::NameIdentifier;

    // Test empty primary identifier
    let name = NameIdentifier::new("", "John", NameIdentifierType::LegalName);
    assert!(matches!(name.validate(), Err(Error::InvalidName(_))));

    // Test very long primary identifier (>150 chars)
    let long_name = "A".repeat(151);
    let name = NameIdentifier::new(long_name, "John", NameIdentifierType::LegalName);
    assert!(matches!(name.validate(), Err(Error::InvalidName(_))));

    // Test very long secondary identifier (>150 chars)
    let long_name = "A".repeat(151);
    let name = NameIdentifier::new("Smith", long_name, NameIdentifierType::LegalName);
    assert!(matches!(name.validate(), Err(Error::InvalidName(_))));
}

#[test]
fn test_customer_identification_validation() {
    use tap_ivms101::person::CustomerIdentification;

    // Test empty customer identifier
    let id = CustomerIdentification {
        customer_identifier: "".to_string(),
        customer_identification_type: CustomerIdentificationType::CustomerIdentificationNumber,
    };
    assert!(matches!(id.validate(), Err(Error::InvalidCustomerId(_))));

    // Test very long customer identifier (>50 chars)
    let id = CustomerIdentification {
        customer_identifier: "X".repeat(51),
        customer_identification_type: CustomerIdentificationType::CustomerIdentificationNumber,
    };
    assert!(matches!(id.validate(), Err(Error::InvalidCustomerId(_))));
}

#[test]
fn test_national_identification_validation() {
    use tap_ivms101::person::NationalIdentification;

    // Test empty national identifier
    let id = NationalIdentification {
        national_identifier: "".to_string(),
        national_identifier_type: NationalIdentifierType::PassportNumber,
        country_of_issue: "US".to_string(),
        registration_authority: None,
    };
    assert!(matches!(id.validate(), Err(Error::InvalidNationalId(_))));

    // Test very long national identifier (>35 chars)
    let id = NationalIdentification {
        national_identifier: "X".repeat(36),
        national_identifier_type: NationalIdentifierType::PassportNumber,
        country_of_issue: "US".to_string(),
        registration_authority: None,
    };
    assert!(matches!(id.validate(), Err(Error::InvalidNationalId(_))));

    // Test invalid country code
    let id = NationalIdentification {
        national_identifier: "123456789".to_string(),
        national_identifier_type: NationalIdentifierType::PassportNumber,
        country_of_issue: "USA".to_string(), // Should be 2 chars
        registration_authority: None,
    };
    assert!(matches!(id.validate(), Err(Error::InvalidCountryCode(_))));
}
