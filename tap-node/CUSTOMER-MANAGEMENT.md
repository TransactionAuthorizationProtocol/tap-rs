# Customer Management in TAP Node

## Overview

The TAP Node Customer Management system provides automatic extraction, storage, and management of customer data from TAP messages. It supports multiple identifier types, Schema.org profiles, relationship tracking, and IVMS101 compliance data generation.

## Architecture

### Components

1. **CustomerManager**: Core service for customer operations
2. **CustomerEventHandler**: Automatic event-driven data extraction
3. **Storage Layer**: SQLite database with customer tables
4. **IVMS101 Integration**: Travel Rule compliance data generation

### Database Schema

```sql
-- Main customer table
CREATE TABLE customers (
    id TEXT PRIMARY KEY,
    agent_did TEXT NOT NULL,
    schema_type TEXT NOT NULL,  -- 'Person' or 'Organization'
    given_name TEXT,
    family_name TEXT,
    display_name TEXT,
    legal_name TEXT,             -- For organizations
    lei_code TEXT,               -- Legal Entity Identifier
    mcc_code TEXT,               -- Merchant Category Code
    address_country TEXT,
    address_locality TEXT,
    postal_code TEXT,
    street_address TEXT,
    profile TEXT NOT NULL,       -- JSON-LD Schema.org profile
    ivms101_data TEXT,          -- Cached IVMS101 data
    verified_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Customer identifiers (DIDs, emails, phones, etc.)
CREATE TABLE customer_identifiers (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    identifier_type TEXT NOT NULL,  -- 'did', 'email', 'phone', 'url', 'account', 'other'
    verified BOOLEAN DEFAULT FALSE,
    verification_method TEXT,
    verified_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);

-- Customer relationships (TAIP-9 compliance)
CREATE TABLE customer_relationships (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,  -- 'controls', 'owns', 'manages', etc.
    related_identifier TEXT NOT NULL,
    proof TEXT,                       -- JSON proof of relationship
    confirmed_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (customer_id) REFERENCES customers(id)
);
```

## Automatic Data Extraction

### How It Works

The CustomerEventHandler automatically processes TAP messages to extract customer data:

1. **Agent Registration**: When an agent is registered, a CustomerEventHandler is automatically created
2. **Message Monitoring**: The handler monitors all message events for the agent
3. **Data Extraction**: Party information is extracted from relevant messages
4. **Storage**: Customer records are created/updated in the agent's database

### Supported Message Types

#### Transfer Messages
```rust
Transfer {
    originator: Party {
        id: "did:key:alice",
        metadata: {
            "name": "Alice Smith",
            "givenName": "Alice",
            "familyName": "Smith",
            "addressCountry": "US"
        }
    },
    beneficiary: Party {
        id: "customer@bank.com",
        metadata: {
            "name": "Bob Jones"
        }
    }
}
```

#### UpdateParty Messages
```rust
UpdateParty {
    party: Party {
        id: "did:key:alice",
        metadata: {
            "@context": "https://schema.org",
            "@type": "Person",
            "name": "Alice Smith",
            "address": {
                "@type": "PostalAddress",
                "streetAddress": "123 Main St",
                "addressLocality": "New York",
                "addressRegion": "NY",
                "postalCode": "10001",
                "addressCountry": "US"
            }
        }
    }
}
```

#### ConfirmRelationship Messages
```rust
ConfirmRelationship {
    subject: "did:key:alice",
    relationship_type: "controls",
    object: "caip10:1:0x123...",
    proof: { /* relationship proof */ }
}
```

## Customer Identifiers

### Supported Types

1. **DID** (`did:*`)
   - Decentralized identifiers
   - Example: `did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK`

2. **Email** (`mailto:*`)
   - Email addresses
   - Example: `mailto:alice@example.com`

3. **Phone** (`tel:*` or `sms:*`)
   - Phone numbers
   - Example: `tel:+1-555-0123`

4. **URL** (`http://*` or `https://*`)
   - Web URLs, converted to did:web
   - Example: `https://example.com` → `did:web:example.com`

5. **Account** (CAIP-10 format)
   - Blockchain accounts
   - Example: `caip10:1:0x1234567890abcdef`

### Automatic Identifier Extraction

```rust
// Single identifier
Party::from("did:key:alice")

// Multiple identifiers (comma-separated)
Party::from("did:key:alice, mailto:alice@example.com, tel:+1-555-0123")

// All identifiers are extracted and stored
```

## Schema.org Profiles

Customer data is stored as Schema.org JSON-LD profiles for maximum interoperability:

### Person Profile
```json
{
  "@context": "https://schema.org",
  "@type": "Person",
  "identifier": "did:key:alice",
  "name": "Alice Smith",
  "givenName": "Alice",
  "familyName": "Smith",
  "email": "alice@example.com",
  "telephone": "+1-555-0123",
  "address": {
    "@type": "PostalAddress",
    "streetAddress": "123 Main St",
    "addressLocality": "New York",
    "addressRegion": "NY",
    "postalCode": "10001",
    "addressCountry": "US"
  }
}
```

### Organization Profile
```json
{
  "@context": "https://schema.org",
  "@type": "Organization",
  "identifier": "did:web:example.com",
  "name": "Example Corp",
  "legalName": "Example Corporation Ltd.",
  "leiCode": "529900HNOAA1KXQJUQ27",
  "address": {
    "@type": "PostalAddress",
    "streetAddress": "456 Business Ave",
    "addressLocality": "San Francisco",
    "addressRegion": "CA",
    "postalCode": "94105",
    "addressCountry": "US"
  }
}
```

## API Usage

### Creating/Updating Customers

```rust
use tap_node::customer::CustomerManager;
use tap_msg::message::Party;

let customer_manager = CustomerManager::new(storage);

// Extract from party
let party = Party::with_metadata(
    "did:key:alice",
    hashmap! {
        "name" => json!("Alice Smith"),
        "addressCountry" => json!("US")
    }
);

let customer_id = customer_manager
    .extract_customer_from_party(&party, "did:key:agent", "originator")
    .await?;
```

### Updating Customer Profiles

```rust
// Update with additional Schema.org data
let profile_update = json!({
    "@context": "https://schema.org",
    "@type": "Person",
    "birthDate": "1990-01-15",
    "jobTitle": "Software Engineer",
    "worksFor": {
        "@type": "Organization",
        "name": "Tech Corp"
    }
});

customer_manager
    .update_customer_profile(&customer_id, profile_update)
    .await?;
```

### Managing Relationships

```rust
// Add a verified relationship
customer_manager
    .add_relationship(
        &customer_id,
        "controls",  // Relationship type
        "caip10:1:0x1234...",  // Related identifier
        Some(json!({  // Proof
            "type": "SignedMessage",
            "signature": "0xabc..."
        }))
    )
    .await?;
```

### Generating IVMS101 Data

```rust
// Generate IVMS101-compliant data for Travel Rule
let ivms_data = customer_manager
    .generate_ivms101_data(&customer_id)
    .await?;

// Result:
{
  "naturalPerson": {
    "name": {
      "nameIdentifiers": [{
        "primaryIdentifier": "Smith",
        "secondaryIdentifier": "Alice",
        "nameIdentifierType": "LEGAL"
      }]
    },
    "geographicAddress": [{
      "addressType": "HOME",
      "streetName": "123 Main St",
      "townName": "New York",
      "countrySubDivision": "NY",
      "postCode": "10001",
      "country": "US"
    }]
  }
}
```

## Event-Driven Updates

The CustomerEventHandler automatically processes events:

### Message Events
```rust
// Automatic extraction from Transfer messages
NodeEvent::MessageReceived { message, source } => {
    if message.type_.contains("Transfer") {
        // Extract originator and beneficiary
        // Create/update customer records
    }
}
```

### Transaction Events
```rust
// Extract parties from new transactions
NodeEvent::TransactionCreated { transaction, agent_did } => {
    // Extract all parties involved
    // Create customer records
    // Establish relationships
}
```

## Privacy and Security

### Data Isolation
- Customer data is stored per-agent
- Each agent has its own database
- No cross-agent data access

### Data Minimization
- Only necessary fields are extracted
- Sensitive data can be excluded
- Configurable extraction rules

### Access Control
- Agent-specific access only
- Read/write permissions per agent
- Audit logging of all access

## Configuration

### Basic Setup
```rust
// Customer management is automatic with TapNode
let mut node = TapNode::new(config);
node.init_storage().await?;

// Register agent - CustomerEventHandler automatically created
let agent = Agent::new("did:key:agent", key_manager);
node.register_agent(agent).await?;
```

### Custom Configuration
```rust
// Access customer manager directly
let customer_manager = node
    .get_agent_storage_manager(&agent_did)?
    .get_customer_manager();

// Custom operations
customer_manager.update_customer_profile(...).await?;
```

## Best Practices

### 1. Data Quality
- Provide complete party metadata in messages
- Use standard Schema.org properties
- Include structured addresses when possible

### 2. Identifier Management
- Use persistent identifiers (DIDs)
- Include multiple identifiers for redundancy
- Verify identifiers when possible

### 3. Compliance
- Keep IVMS101 data up-to-date
- Store relationship proofs
- Maintain audit trails

### 4. Performance
- Customer data is cached in memory
- IVMS101 data is pre-generated and cached
- Indexes on identifiers for fast lookup

## Examples

### Complete Customer Flow
```rust
// 1. Customer sends Transfer
let transfer = Transfer {
    originator: Party::with_metadata(
        "did:key:alice",
        hashmap! {
            "name" => json!("Alice Smith"),
            "givenName" => json!("Alice"),
            "familyName" => json!("Smith"),
            "addressCountry" => json!("US"),
            "email" => json!("alice@example.com")
        }
    ),
    // ...
};

// 2. CustomerEventHandler automatically:
//    - Creates customer record for Alice
//    - Stores all metadata
//    - Creates identifiers (DID, email)

// 3. Later, update with more data
let update = UpdateParty {
    party: Party::with_metadata(
        "did:key:alice",
        hashmap! {
            "@context" => json!("https://schema.org"),
            "@type" => json!("Person"),
            "address" => json!({
                "@type": "PostalAddress",
                "streetAddress": "123 Main St",
                "addressLocality": "New York",
                "postalCode": "10001",
                "addressCountry": "US"
            })
        }
    )
};

// 4. Generate IVMS101 for compliance
let ivms = customer_manager
    .generate_ivms101_data("did:key:alice")
    .await?;
```

## Troubleshooting

### Common Issues

1. **Customer Not Found**
   - Check the customer ID format
   - Verify the agent DID is correct
   - Ensure the party was in a processed message

2. **IVMS101 Generation Fails**
   - Check required fields (name, country)
   - Verify address format
   - Review validation errors

3. **Duplicate Customers**
   - Use consistent identifiers
   - Check for comma-separated IDs
   - Review identifier extraction logic

### Debug Logging
```bash
RUST_LOG=tap_node::customer=debug cargo run
```

## Future Enhancements

- Enhanced deduplication algorithms
- Fuzzy matching for customer records
- Integration with KYC/AML services
- Advanced relationship verification
- Privacy-preserving customer analytics