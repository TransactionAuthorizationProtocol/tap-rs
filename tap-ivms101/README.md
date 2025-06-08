# TAP IVMS 101

[![Crates.io](https://img.shields.io/crates/v/tap-ivms101.svg)](https://crates.io/crates/tap-ivms101)
[![Documentation](https://docs.rs/tap-ivms101/badge.svg)](https://docs.rs/tap-ivms101)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A complete implementation of the IVMS 101.2023 data model for the Travel Asset Protocol (TAP).

## Overview

This crate provides a comprehensive implementation of the interVASP Messaging Standard (IVMS) 101.2023, which is used for Travel Rule compliance in virtual asset transfers. IVMS 101 is the global standard for exchanging required originator and beneficiary information between Virtual Asset Service Providers (VASPs) to comply with FATF Recommendation 16.

### Key Components

- **Person Entities**: Natural Person and Legal Person data structures with comprehensive identity information
- **Transaction Participants**: Originator, Beneficiary, Originating VASP, and Beneficiary VASP
- **Transaction Data**: Amount, currency, identifiers, payment types, and network information
- **Geographic Addresses**: Full postal address support with multiple address types
- **Identification**: National IDs, customer numbers, LEI, BIC, and other identifiers
- **Validation**: Built-in validation for all fields including ISO standards
- **Builder Patterns**: Ergonomic construction of complex data structures

## Features

- **Complete Data Model**: Implements all IVMS 101.2023 data structures
- **Validation**: Built-in validation for all fields including country codes, currency codes, LEI, BIC
- **Builder Patterns**: Ergonomic builders for constructing complex data structures
- **Serialization**: Full serde support for JSON serialization/deserialization
- **Type Safety**: Type-safe enumerations prevent invalid values
- **Documentation**: Comprehensive documentation with examples

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
tap-ivms101 = "0.1.0"
```

### Basic Example

```rust
use tap_ivms101::builder::*;
use tap_ivms101::types::*;
use tap_ivms101::message::*;

// Create a natural person
let person_name = NaturalPersonNameBuilder::new()
    .legal_name("Smith", "John")
    .build()?;

let person = NaturalPersonBuilder::new()
    .name(person_name)
    .country_of_residence("US")
    .national_id(
        "123456789",
        NationalIdentifierType::NationalIdentityNumber,
        "US"
    )
    .build()?;

// Create a legal person (VASP)
let vasp_name = LegalPersonNameBuilder::new()
    .legal_name("Example Exchange Ltd.")
    .build()?;

let vasp = LegalPersonBuilder::new()
    .name(vasp_name)
    .lei("529900HNOAA1KXQJUQ27")?
    .country_of_registration("US")
    .build()?;

// Create an IVMS message
let message = IvmsMessageBuilder::new()
    .originator(vec![Person::NaturalPerson(person)])
    .beneficiary(vec![Person::NaturalPerson(person.clone())])
    .originating_vasp(Person::LegalPerson(vasp))
    .transaction(
        "1000.00",
        "USD",
        TransactionDirection::Outgoing,
        "tx-123456",
        "2024-01-15T10:30:00Z"
    )?
    .build()?;

// Serialize to JSON
let json = message.to_json_pretty()?;
```

### Working with Geographic Addresses

```rust
use tap_ivms101::builder::*;
use tap_ivms101::types::*;

let address = GeographicAddressBuilder::new()
    .address_type(AddressType::Home)
    .street_name("123 Main Street")
    .building_number("123")
    .post_code("12345")
    .town_name("New York")
    .country("US")
    .build()?;

let person = NaturalPersonBuilder::new()
    .name(/* ... */)
    .add_address(address)
    .build()?;
```

### Validation

All data structures include comprehensive validation to ensure compliance:

```rust
use tap_ivms101::validation::*;

// Validate country codes (ISO 3166)
validate_country_code("US")?;  // OK
validate_country_code("USA")?; // Error: must be 2 characters
validate_country_code("XX")?; // Error: invalid country code

// Validate currency codes (ISO 4217)
validate_currency_code("USD")?;  // OK
validate_currency_code("EUR")?;  // OK
validate_currency_code("USDT")?; // Error: not ISO 4217

// Validate LEI (Legal Entity Identifier)
validate_lei("529900HNOAA1KXQJUQ27")?; // OK
validate_lei("INVALID")?; // Error: must be 20 characters

// Validate BIC/SWIFT codes
validate_bic("DEUTDEFF")?;    // OK (8 chars)
validate_bic("DEUTDEFFXXX")?; // OK (11 chars)
validate_bic("INVALID")?;     // Error: wrong length

// Validate required fields
let person = NaturalPersonBuilder::new()
    .build(); // Error: missing required name field
```

## IVMS 101 Compliance

This implementation follows the IVMS 101.2023 specification, ensuring full compliance with Travel Rule requirements:

### Data Structures
- **Natural Person**: Individual persons with names, addresses, national IDs, date/place of birth
- **Legal Person**: Organizations/VASPs with legal names, LEI, BIC, registration details
- **Name Types**: Support for legal, trading, shortened, and phonetic names
- **Address Types**: Home, business, geographic, and other address classifications
- **ID Types**: National ID, passport, driver's license, customer ID, and more

### Transaction Information
- Amount and currency (ISO 4217)
- Transaction direction (incoming/outgoing)
- Transaction identifiers and blockchain hashes
- Payment type codes
- Network information

### Validation Rules
- ISO 3166 country codes (2-letter)
- ISO 4217 currency codes (3-letter)
- LEI format validation (20 characters)
- BIC/SWIFT code validation (8 or 11 characters)
- Required field enforcement
- String length constraints

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Integration with TAP

This crate is designed to work seamlessly with the Travel Asset Protocol (TAP) ecosystem:

```rust
use tap_ivms101::message::IvmsMessage;
use tap_msg::message::transfer::Transfer;

// Create IVMS data for a TAP transfer
let ivms_data = IvmsMessageBuilder::new()
    // ... build IVMS message
    .build()?;

// Attach to TAP transfer message
let transfer = Transfer {
    // ... other fields
    ivms101: Some(ivms_data.to_json()?),
};
```

## Error Handling

The crate provides detailed error messages for validation failures:

```rust
use tap_ivms101::error::IvmsError;

match result {
    Err(IvmsError::ValidationError(msg)) => {
        // Handle validation error
        eprintln!("Validation failed: {}", msg);
    }
    Err(IvmsError::SerializationError(e)) => {
        // Handle JSON error
        eprintln!("Serialization failed: {}", e);
    }
    _ => {}
}
```

## References

- [IVMS 101.2023 Specification](https://www.intervasp.org/)
- [FATF Travel Rule Requirements](https://www.fatf-gafi.org/)
- [TAP Protocol Documentation](https://github.com/notabene/tap-rs)