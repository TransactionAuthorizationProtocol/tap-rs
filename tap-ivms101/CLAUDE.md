# tap-ivms101 Crate

IVMS 101.2023 data model implementation for travel rule compliance in the Transaction Authorization Protocol.

## Purpose

The `tap-ivms101` crate provides:
- IVMS 101.2023 compliant data structures
- Travel rule compliance data modeling
- VASP (Virtual Asset Service Provider) information handling
- Customer identification and verification data
- Integration with TAP messaging system
- Validation and serialization support

## Key Components

- `message.rs` - IVMS101 message wrapper and handling
- `person.rs` - Natural person data structures
- `types.rs` - IVMS101 core types and enums
- `builder.rs` - Builder pattern for IVMS101 data construction
- `validation.rs` - Data validation and compliance checks
- `error.rs` - IVMS101-specific error types

## Build Commands

```bash
# Build the crate
cargo build -p tap-ivms101

# Run tests
cargo test -p tap-ivms101

# Run specific test
cargo test -p tap-ivms101 test_name

# Test with pretty assertions
cargo test -p tap-ivms101 --features pretty_assertions

# Build documentation
cargo doc -p tap-ivms101 --open
```

## Development Guidelines

### IVMS101 Compliance
- Follow IVMS 101.2023 specification exactly
- Implement all required and optional fields
- Support proper data validation
- Include comprehensive error handling
- Maintain compatibility with updates

### Travel Rule Implementation
- Support jurisdictional requirements
- Handle threshold-based reporting
- Implement proper data minimization
- Include privacy protection measures
- Support cross-border compliance

### Data Validation
- Validate all structured data fields
- Enforce business rules and constraints
- Support multiple validation levels
- Provide clear error messages
- Include field-level validation

### Builder Pattern Usage
- Use builders for complex data construction
- Provide fluent API interfaces
- Include validation at build time
- Support incremental data building
- Handle optional vs required fields

## IVMS101 Data Structures

### Natural Person
```rust
use tap_ivms101::{NaturalPerson, NaturalPersonBuilder};

let person = NaturalPersonBuilder::new()
    .name_identifier("John", "Doe")
    .birth_date("1990-01-01")
    .country_of_residence("US")
    .national_identification("123-45-6789", "SSN", "US")
    .customer_identification("CUST001")
    .address("123 Main St", "New York", "US", "10001")
    .build()?;
```

### Legal Person
```rust
use tap_ivms101::{LegalPerson, LegalPersonBuilder};

let entity = LegalPersonBuilder::new()
    .legal_name("Example Corp")
    .legal_entity_identifier("254900ABCD1234567890")
    .country_of_registration("US")
    .address("456 Corporate Blvd", "New York", "US", "10002")
    .customer_identification("CORP001")
    .build()?;
```

### Transaction Information
```rust
use tap_ivms101::{Transaction, TransactionBuilder};

let transaction = TransactionBuilder::new()
    .originator_person(originator)
    .beneficiary_person(beneficiary)
    .asset("BTC")
    .amount("1.5")
    .transaction_date("2024-01-01T12:00:00Z")
    .build()?;
```

## Travel Rule Compliance

### Threshold Management
- Support different jurisdictional thresholds
- Automatic threshold detection
- Compliance level determination
- Risk assessment integration

### Data Requirements
- Required vs optional field handling
- Jurisdiction-specific requirements
- Cross-border compliance rules
- Data retention policies

### Privacy Protection
- Data minimization principles
- Purpose limitation
- Storage limitation
- Data quality requirements

## Validation Features

### Field-Level Validation
- Country code validation (ISO 3166)
- Currency code validation (ISO 4217)
- Date format validation (ISO 8601)
- Identifier format validation
- Name format validation

### Business Rule Validation
- Transaction amount validation
- Customer type validation
- Address format validation
- Required field enforcement
- Cross-field consistency checks

### Compliance Validation
- Travel rule threshold checks
- Jurisdiction requirement validation
- VASP registration validation
- Sanctions screening support
- PEP (Politically Exposed Person) checks

## Examples

### Complete IVMS101 Message
```rust
use tap_ivms101::{IvmsMessage, MessageBuilder};

let message = MessageBuilder::new()
    .originating_vasp("VASP001", "Origin Bank")
    .beneficiary_vasp("VASP002", "Destination Bank")
    .originator_person(originator_person)
    .beneficiary_person(beneficiary_person)
    .transaction_amount("1000.00", "USD")
    .build()?;

let json = serde_json::to_string_pretty(&message)?;
```

### Travel Rule Assessment
```rust
use tap_ivms101::TravelRuleAssessment;

let assessment = TravelRuleAssessment::new()
    .amount("1000.00")
    .currency("USD")
    .originator_jurisdiction("US")
    .beneficiary_jurisdiction("EU")
    .evaluate()?;

if assessment.requires_ivms101() {
    // Collect and validate IVMS101 data
}
```

## Integration with TAP

### Message Integration
- IVMS101 data embedded in TAP messages
- Travel rule metadata handling
- Compliance flag support
- Automatic data extraction

### Workflow Integration
- Pre-transaction compliance checks
- Real-time validation
- Post-transaction reporting
- Audit trail generation

## Error Handling

Comprehensive error types for compliance failures:
- `ValidationError` - Field validation failures
- `ComplianceError` - Travel rule compliance issues
- `FormatError` - Data format problems
- `RequiredFieldError` - Missing required data
- `BusinessRuleError` - Business logic violations

## Testing

The crate includes extensive testing:
- Unit tests for all data structures
- Validation rule tests
- Compliance scenario tests
- Integration tests with TAP messages
- Real-world data format tests

Run comprehensive tests:
```bash
cargo test -p tap-ivms101
```

## Serialization Support

Full serialization support for:
- JSON (primary format)
- XML (IVMS101 standard)
- Custom binary formats
- Database storage formats

## Standards Compliance

The implementation follows these standards:
- **IVMS 101.2023** - InterVASP Messaging Standard
- **ISO 3166** - Country codes
- **ISO 4217** - Currency codes
- **ISO 8601** - Date/time formats
- **FATF Travel Rule** - International guidelines

## Privacy Features

### Data Protection
- Field-level encryption support
- Hashing for sensitive data
- Data anonymization utilities
- GDPR compliance helpers

### Access Controls
- Role-based data access
- Field-level permissions
- Audit logging
- Data retention policies

## Jurisdictional Support

Support for major jurisdictions:
- **United States** - FinCEN requirements
- **European Union** - 5AMLD/6AMLD
- **Singapore** - MAS requirements
- **Switzerland** - FINMA requirements
- **Canada** - FINTRAC requirements
- **Japan** - FSA requirements

Each jurisdiction includes:
- Specific threshold amounts
- Required data fields
- Reporting requirements
- Compliance timelines