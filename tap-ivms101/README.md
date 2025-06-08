# TAP IVMS 101

[![Crates.io](https://img.shields.io/crates/v/tap-ivms101.svg)](https://crates.io/crates/tap-ivms101)
[![Documentation](https://docs.rs/tap-ivms101/badge.svg)](https://docs.rs/tap-ivms101)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A complete implementation of the IVMS 101.2023 data model for the Travel Asset Protocol (TAP).

## Overview

This crate provides a comprehensive implementation of the interVASP Messaging Standard (IVMS) 101.2023, which is used for Travel Rule compliance in virtual asset transfers. It includes:

- All IVMS 101 data structures (Natural Person, Legal Person, Originator, Beneficiary, VASP, Transaction)
- Comprehensive validation for all fields
- Type-safe enumerations for all code lists
- Serde JSON serialization/deserialization
- Builder patterns for easy construction
- ISO country code and currency code validation

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

All data structures include comprehensive validation:

```rust
use tap_ivms101::validation::*;

// Validate country codes
validate_country_code("US")?;  // OK
validate_country_code("USA")?; // Error: must be 2 characters

// Validate currency codes
validate_currency_code("USD")?;  // OK
validate_currency_code("USDT")?; // Error: not ISO 4217

// Validate LEI
validate_lei("529900HNOAA1KXQJUQ27")?; // OK

// Validate BIC
validate_bic("DEUTDEFF")?;    // OK (8 chars)
validate_bic("DEUTDEFFXXX")?; // OK (11 chars)
```

## IVMS 101 Compliance

This implementation follows the IVMS 101.2023 specification, including:

- Natural Person and Legal Person data structures
- Name identifiers with support for legal, trading, and other name types
- Geographic addresses with full postal details
- National identification with multiple ID types
- Customer identification
- Transaction data with payment types and network information
- Full validation of all required fields

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## References

- [IVMS 101.2023 Specification](https://www.intervasp.org/)
- [Travel Rule Information](https://www.fatf-gafi.org/)