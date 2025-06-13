//! Tests for builder pattern functionality

use tap_ivms101::builder::*;
use tap_ivms101::error::Error;
use tap_ivms101::types::*;
use tap_ivms101::Person;

#[test]
fn test_lei_validation_in_builder() {
    let name = LegalPersonNameBuilder::new()
        .legal_name("Test Company")
        .build()
        .unwrap();

    // Test invalid LEI
    let result = LegalPersonBuilder::new()
        .name(name.clone())
        .lei("INVALID_LEI");

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::InvalidLei(_))));

    // Test valid LEI
    let result = LegalPersonBuilder::new()
        .name(name)
        .lei("529900HNOAA1KXQJUQ27");

    assert!(result.is_ok());
}

#[test]
fn test_country_validation_in_builders() {
    // Test NaturalPersonBuilder with invalid country
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Test", "User")
        .build()
        .unwrap();

    let result = NaturalPersonBuilder::new()
        .name(name)
        .country_of_residence("USA") // Should be 2 chars
        .build();

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::InvalidCountryCode(_))));

    // Test GeographicAddressBuilder with invalid country
    let result = GeographicAddressBuilder::new()
        .street_name("Main Street")
        .post_code("12345")
        .town_name("Test Town")
        .country("USA") // Should be 2 chars
        .build();

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::InvalidCountryCode(_))));
}

#[test]
fn test_transaction_builder_currency_validation() {
    let originator_name = NaturalPersonNameBuilder::new()
        .legal_name("Doe", "John")
        .build()
        .unwrap();

    let originator = NaturalPersonBuilder::new()
        .name(originator_name)
        .build()
        .unwrap();

    let beneficiary_name = NaturalPersonNameBuilder::new()
        .legal_name("Smith", "Jane")
        .build()
        .unwrap();

    let beneficiary = NaturalPersonBuilder::new()
        .name(beneficiary_name)
        .build()
        .unwrap();

    let vasp_name = LegalPersonNameBuilder::new()
        .legal_name("VASP Inc.")
        .build()
        .unwrap();

    let vasp = LegalPersonBuilder::new().name(vasp_name).build().unwrap();

    // Test invalid currency code
    let result = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(originator)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary)])
        .originating_vasp(Person::LegalPerson(vasp))
        .transaction(
            "100.00",
            "INVALID", // Invalid currency
            TransactionDirection::Outgoing,
            "TX-001",
            "2024-01-01T00:00:00Z",
        );

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::InvalidCurrencyCode(_))));
}

#[test]
fn test_builder_method_chaining() {
    // Test that all builder methods can be chained properly
    let address = GeographicAddressBuilder::new()
        .address_type(AddressType::Business)
        .street_name("Wall Street")
        .building_number("100")
        .post_code("10005")
        .town_name("New York")
        .country("US")
        .build()
        .unwrap();

    assert_eq!(address.address_type, Some(AddressType::Business));
    assert_eq!(address.street_name, "Wall Street");
    assert_eq!(address.building_number, Some("100".to_string()));
    assert_eq!(address.post_code, "10005");
    assert_eq!(address.town_name, "New York");
    assert_eq!(address.country, "US");
}

#[test]
fn test_optional_fields_in_builders() {
    // Test that optional fields are properly handled
    let person = NaturalPersonBuilder::new()
        .name(
            NaturalPersonNameBuilder::new()
                .legal_name("Minimal", "Person")
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    assert!(person.geographic_addresses.is_none());
    assert!(person.national_identification.is_none());
    assert!(person.customer_identification.is_none());
    assert!(person.date_and_place_of_birth.is_none());
    assert!(person.country_of_residence.is_none());
}

#[test]
fn test_multiple_name_identifiers() {
    let name = NaturalPersonNameBuilder::new()
        .add_name_identifier("Smith", "John", NameIdentifierType::LegalName)
        .add_name_identifier("Smythe", "Johnny", NameIdentifierType::OtherName)
        .build()
        .unwrap();

    assert_eq!(name.name_identifiers.len(), 2);
    assert_eq!(name.name_identifiers[0].primary_identifier, "Smith");
    assert_eq!(name.name_identifiers[0].secondary_identifier, "John");
    assert_eq!(
        name.name_identifiers[0].name_identifier_type,
        NameIdentifierType::LegalName
    );
    assert_eq!(name.name_identifiers[1].primary_identifier, "Smythe");
    assert_eq!(name.name_identifiers[1].secondary_identifier, "Johnny");
    assert_eq!(
        name.name_identifiers[1].name_identifier_type,
        NameIdentifierType::OtherName
    );
}

#[test]
fn test_legal_person_name_builder_types() {
    let name = LegalPersonNameBuilder::new()
        .legal_name("Example Company Ltd.")
        .trading_name("Example Co")
        .add_name_identifier(
            "Example Corporation",
            LegalPersonNameIdentifierType::ShortName,
        )
        .build()
        .unwrap();

    assert_eq!(name.name_identifiers.len(), 3);
    assert_eq!(
        name.name_identifiers[0].legal_person_name,
        "Example Company Ltd."
    );
    assert_eq!(
        name.name_identifiers[0].legal_person_name_identifier_type,
        LegalPersonNameIdentifierType::LegalName
    );
    assert_eq!(name.name_identifiers[1].legal_person_name, "Example Co");
    assert_eq!(
        name.name_identifiers[1].legal_person_name_identifier_type,
        LegalPersonNameIdentifierType::TradingName
    );
    assert_eq!(
        name.name_identifiers[2].legal_person_name,
        "Example Corporation"
    );
    assert_eq!(
        name.name_identifiers[2].legal_person_name_identifier_type,
        LegalPersonNameIdentifierType::ShortName
    );
}
