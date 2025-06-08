# Travel Rule Implementation in TAP Node

## Overview

The TAP Node implements TAIP-10 Travel Rule compliance through automated IVMS101 data exchange. This document describes how the Travel Rule processor works and how to integrate it into your TAP implementation.

## Architecture

The Travel Rule implementation consists of three main components:

1. **IVMS101 Data Model** (`tap-ivms101`): Complete implementation of the IVMS 101.2023 standard
2. **Travel Rule Processor** (`tap-node`): Handles Travel Rule message flows and attachments
3. **Customer Manager** (`tap-node`): Manages customer data and IVMS101 generation

## How It Works

### 1. Automatic IVMS101 Attachment

When sending a Transfer message, the Travel Rule processor automatically:

- Checks if IVMS101 data should be attached (based on amount, jurisdiction, policies)
- Generates IVMS101 data from customer records
- Attaches the data as a Verifiable Presentation
- Sends the enhanced Transfer message

```rust
// Original Transfer message
let transfer = Transfer {
    originator: Party::from("did:key:alice"),
    beneficiary: Party::from("customer@example.com"),
    amount: "1000.00",
    currency: "USD",
    // ... other fields
};

// Travel Rule processor automatically adds:
// - IVMS101 Verifiable Presentation attachment
// - Proper DIDComm attachment formatting
// - Compliance metadata
```

### 2. Policy-Based Requests

When a counterparty requires IVMS101 data, they send an UpdatePolicies message:

```json
{
  "@type": "https://tap.rsvp/schema/1.0/UpdatePolicies",
  "policies": [{
    "@type": "RequirePresentation",
    "@context": ["https://intervasp.org/ivms101"],
    "credential_types": ["TravelRuleCredential"],
    "purpose": "Travel Rule Compliance"
  }]
}
```

The processor:
- Detects IVMS101 requirements
- Stores the policy for the transaction
- Ensures subsequent messages include required data

### 3. Presentation Processing

When receiving IVMS101 presentations:

```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://intervasp.org/ivms101"
  ],
  "type": ["VerifiablePresentation", "PresentationSubmission"],
  "verifiableCredential": [{
    "type": ["VerifiableCredential", "TravelRuleCredential"],
    "credentialSubject": {
      "originator": {
        "naturalPerson": {
          "name": {
            "nameIdentifiers": [{
              "primaryIdentifier": "Smith",
              "secondaryIdentifier": "Alice"
            }]
          },
          "geographicAddress": [{
            "addressType": "HOME",
            "streetName": "123 Main Street",
            "townName": "New York",
            "country": "US"
          }]
        }
      }
    }
  }]
}
```

The processor:
- Validates the presentation format
- Extracts IVMS101 data
- Updates customer records
- Stores compliance data for reporting

## Customer Data Management

### Automatic Extraction

The Customer Manager automatically extracts party information from TAP messages:

```rust
// From Transfer messages
Transfer {
    originator: Party {
        id: "did:key:alice",
        metadata: {
            "name": "Alice Smith",
            "addressCountry": "US"
        }
    }
}
// Creates/updates customer record with extracted data
```

### Schema.org Profiles

Customer data is stored as Schema.org JSON-LD profiles:

```json
{
  "@context": "https://schema.org",
  "@type": "Person",
  "identifier": "did:key:alice",
  "name": "Alice Smith",
  "givenName": "Alice",
  "familyName": "Smith",
  "addressCountry": "US"
}
```

### IVMS101 Generation

When needed, customer profiles are converted to IVMS101 format:

```rust
// Customer profile → IVMS101 Natural Person
let ivms_data = customer_manager
    .generate_ivms101_data(&customer_id)
    .await?;
```

## Configuration

### Basic Setup

```rust
use tap_node::{TapNode, NodeConfig};

// Create node with default Travel Rule support
let config = NodeConfig::default();
let mut node = TapNode::new(config);
node.init_storage().await?;

// Travel Rule processor is automatically enabled
```

### Custom Policies

```rust
// Configure when to attach IVMS101 data
impl TravelRuleProcessor {
    async fn should_attach_ivms101(&self, message: &PlainMessage) -> bool {
        // Check transaction amount
        if let Some(amount) = message.body.get("amount") {
            if amount.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0) > 1000.0 {
                return true;
            }
        }
        
        // Check counterparty jurisdiction
        // Check internal policies
        // etc.
        
        false
    }
}
```

## Integration Examples

### 1. Sending Compliant Transfers

```rust
// Customer data is automatically extracted from parties
let transfer = Transfer {
    originator: Party::with_metadata(
        "did:key:alice",
        hashmap! {
            "name" => json!("Alice Smith"),
            "addressCountry" => json!("US")
        }
    ),
    beneficiary: Party::from("customer@bank.com"),
    amount: "5000.00",
    currency: "USD",
};

// Send - IVMS101 data automatically attached if required
node.send_message(
    "did:key:alice",
    "did:web:bank.com",
    transfer.into()
).await?;
```

### 2. Handling Presentation Requests

The processor automatically handles UpdatePolicies messages:

```rust
// Incoming UpdatePolicies with IVMS101 requirement
// → Processor detects requirement
// → Next Transfer includes IVMS101 presentation
// → Counterparty receives compliance data
```

### 3. Customer Data Updates

```rust
// Update customer with additional data
let profile = json!({
    "@context": "https://schema.org",
    "@type": "Person",
    "address": {
        "@type": "PostalAddress",
        "streetAddress": "123 Main St",
        "addressLocality": "New York",
        "addressRegion": "NY",
        "postalCode": "10001",
        "addressCountry": "US"
    }
});

customer_manager.update_customer_profile(&customer_id, profile).await?;
```

## Compliance Considerations

### Data Privacy

- Customer data is stored per-agent (data isolation)
- IVMS101 data is only shared when required
- Sensitive fields can be masked or excluded
- All data access is logged for audit

### Regulatory Alignment

The implementation supports:
- FATF Recommendation 16 (Travel Rule)
- US FinCEN Travel Rule requirements
- EU 5AMLD/6AMLD requirements
- IVMS 101.2023 standard compliance

### Audit Trail

All Travel Rule activities are logged:
- IVMS101 data generation
- Presentation requests/responses
- Customer data updates
- Compliance policy changes

## Testing

### Unit Tests

```bash
# Test IVMS101 data structures
cargo test --package tap-ivms101

# Test Travel Rule processor
cargo test --package tap-node travel_rule

# Test customer management
cargo test --package tap-node customer
```

### Integration Tests

```bash
# Full Travel Rule flow test
cargo test --package tap-node travel_rule_integration_test
```

### Compliance Testing

See `tap-node/tests/travel_rule_integration_test.rs` for examples of:
- IVMS101 attachment to transfers
- Presentation request handling
- Customer data extraction
- Cross-VASP communication

## Troubleshooting

### Common Issues

1. **Missing IVMS101 Data**
   - Check customer records exist
   - Verify required fields (name, address)
   - Check policy configuration

2. **Validation Failures**
   - Ensure country codes are ISO 3166 (2-letter)
   - Ensure currency codes are ISO 4217 (3-letter)
   - Check required fields are present

3. **Presentation Not Attached**
   - Verify amount thresholds
   - Check counterparty policies
   - Review processor configuration

### Debug Logging

Enable debug logging for Travel Rule components:

```bash
RUST_LOG=tap_node::message::travel_rule_processor=debug cargo run
```

## Future Enhancements

- Support for Legal Person (organization) IVMS101 data
- Selective disclosure of IVMS fields
- Integration with identity verification services
- Support for multiple IVMS versions
- Enhanced privacy features (zero-knowledge proofs)