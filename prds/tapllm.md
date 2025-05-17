# Transaction Authorization Protocol (TAP) - Condensed Reference

## Core Concepts

TAP creates a standardized authorization layer for blockchain transactions that:
- Enables multi-party coordination and compliance before settlement
- Works independently from blockchain settlement layers
- Supports agent-based architecture for flexible transaction flows
- Provides secure message exchange using DIDComm v2
- Ensures regulatory compliance while preserving blockchain benefits

## Transaction Participants

### Parties
- **Originator**: Entity sending assets
- **Beneficiary**: Entity receiving assets
- **Intermediaries**: Other parties involved in the transaction

### Agents
Agents act on behalf of parties or other agents:
- **VASP Agents**: Represent virtual asset service providers
- **Wallet Agents**: Interface with blockchain wallets
- **Custody Agents**: Manage secure storage
- **Compliance/Risk Agents**: Perform checks and verifications
- **AI/Application Agents**: Automated systems acting for a party

## Message Types

All TAP messages follow the DIDComm v2 message structure with these common attributes:
- `id`: Unique message identifier
- `type`: Message type URI in the `https://tap.rsvp/taips/N` namespace
- `from`: DID of the sender
- `to`: Array of recipient DIDs
- `thid`: Thread identifier (links related messages)
- `body`: Message payload as JSON-LD

### Transaction Messages

#### 1. Transfer
Initiates a virtual asset transfer between parties.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Transfer",
  "asset": "eip155:1/slip44:60",
  "amount": "1.23",
  "originator": {
    "@id": "did:example:sender"
  },
  "beneficiary": {
    "@id": "did:example:recipient"
  },
  "agents": [
    {
      "@id": "did:web:originator.vasp",
      "for": "did:example:sender"
    },
    {
      "@id": "did:web:beneficiary.vasp",
      "for": "did:example:recipient"
    }
  ],
  "purpose": "SUPP",
  "categoryPurpose": "CASH"
}
```

#### 2. Payment
Initiates a payment request from a merchant to a customer.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Payment",
  "currency": "USD",
  "amount": "50.00",
  "supportedAssets": [
    "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
  ],
  "merchant": {
    "@id": "did:web:merchant.vasp",
    "name": "Example Store"
  },
  "expiry": "2024-04-21T12:00:00Z"
}
```

### Authorization Flow Messages

#### 1. Authorize
Approves a transaction after completing compliance checks.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Authorize"
}
```

#### 2. Complete
Indicates that a transaction is ready for settlement in a Payment flow.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Complete",
  "settlementAddress": "eip155:1:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
  "amount": "95.50"
}
```

#### 3. Settle
Confirms the on-chain settlement of a transfer.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Settle",
  "settlementId": "eip155:1:tx/0x3edb98c24d46d148eb926c714f4fbaa117c47b0c0821f38bfce9763604457c33"
}
```

#### 4. Reject
Rejects a proposed transfer.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Reject",
  "reason": "Beneficiary account is not active"
}
```

#### 5. Cancel
Terminates an existing transaction or connection.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Cancel",
  "reason": "User requested cancellation"
}
```

#### 6. Revert
Requests a reversal of a settled transaction.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Revert",
  "settlementAddress": "eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
  "reason": "Insufficient Originator Information"
}
```

### Agent Management Messages

#### 1. UpdateAgent
Updates information about an agent.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#UpdateAgent",
  "agent": {
    "@id": "did:web:originator.vasp",
    "for": "did:eg:bob",
    "role": "SourceAddress"
  }
}
```

#### 2. UpdateParty
Updates information about a party.

```json
{
  "@context": {
    "@vocab": "https://tap.rsvp/schema/1.0",
    "lei": "https://schema.org/leiCode"
  },
  "@type": "https://tap.rsvp/schema/1.0#UpdateParty",
  "party": {
    "@id": "did:eg:alice",
    "lei:leiCode": "5493001KJTIIGC8Y1R12",
    "name": "Alice Corp Ltd"
  }
}
```

#### 3. AddAgents
Adds one or more additional agents to the transaction.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#AddAgents",
  "agents": [
    {
      "@id": "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
      "for": "did:web:beneficiary.vasp",
      "role": "settlementAddress"
    }
  ]
}
```

#### 4. ReplaceAgent
Replaces an agent with another agent in the transaction.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#ReplaceAgent",
  "original": "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
  "replacement": {
    "@id": "did:pkh:eip155:1:0x5678a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
    "for": "did:web:beneficiary.vasp",
    "role": "settlementAddress"
  }
}
```

#### 5. RemoveAgent
Removes an agent from the transaction.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#RemoveAgent",
  "agent": "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb"
}
```

### Relationship Proofs

#### 1. ConfirmRelationship
Confirms a relationship between an agent and the entity it acts on behalf of.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#ConfirmRelationship",
  "body": {
    "@context": "https://tap.rsvp/schema/1.0",
    "@type": "https://tap.rsvp/schema/1.0#Agent",
    "@id": "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
    "for": "did:web:beneficiary.vasp",
    "role": "settlementAddress"
  },
  "attachments": [
    {
      "id": "proof-1",
      "media_type": "application/json",
      "data": {
        "json": {
          "h": "eth-personal-sign",
          "s": "0x...", // CACAO signature
          "p": "I confirm that this wallet is controlled by did:web:beneficiary.vasp",
          "t": "2024-03-07T12:00:00Z"
        }
      }
    }
  ]
}
```

### Policy Messages

#### 1. UpdatePolicies
Updates policies for a transaction.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#UpdatePolicies",
  "policies": [
    {
      "@type": "RequirePresentation",
      "@context": ["https://schema.org/Person"],
      "fromAgent": "originator",
      "aboutParty": "originator",
      "presentationDefinition": "https://tap.rsvp/presentation-definitions/ivms-101/eu/tfr"
    },
    {
      "@type": "RequirePurpose",
      "fromAgent": "originator",
      "fields": ["purpose", "categoryPurpose"]
    }
  ]
}
```

### Connection Messages

#### 1. Connect
Requests a connection between agents with specified constraints.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#Connect",
  "agent": {
    "@id": "did:example:b2b-service",
    "name": "B2B Payment Service",
    "type": "ServiceAgent"
  },
  "for": "did:example:business-customer",
  "constraints": {
    "purposes": ["BEXP", "SUPP"],
    "categoryPurposes": ["CASH", "CCRD"],
    "limits": {
      "per_transaction": "10000.00",
      "daily": "50000.00",
      "currency": "USD"
    }
  }
}
```

#### 2. AuthorizationRequired
Provides an authorization URL for interactive connection approval.

```json
{
  "@context": "https://tap.rsvp/schema/1.0",
  "@type": "https://tap.rsvp/schema/1.0#AuthorizationRequired",
  "authorization_url": "https://vasp.com/authorize?request=abc123",
  "expires": "2024-03-22T15:00:00Z"
}
```

## Data Elements

### Party
Represents a participant in a transaction (originator or beneficiary).

```json
{
  "@id": "did:example:sender",
  "lei:leiCode": "3M5E1GQKGL17HI8CPN20",
  "name": "ACME Corporation"
}
```

### Agent
Represents a service involved in executing transactions.

```json
{
  "@id": "did:web:vasp.example",
  "role": "SettlementAddress",
  "for": "did:example:party"
}
```

### Policy
Defines requirements for transaction progress.

#### RequirePresentation
```json
{
  "@type": "RequirePresentation",
  "@context": ["https://schema.org/Person"],
  "fromAgent": "originator",
  "aboutParty": "originator",
  "presentationDefinition": "https://tap.rsvp/presentation-definitions/ivms-101/eu/tfr"
}
```

#### RequirePurpose
```json
{
  "@type": "RequirePurpose",
  "fields": ["purpose", "categoryPurpose"],
  "fromAgent": "originator"
}
```

#### RequireAuthorization
```json
{
  "@type": "RequireAuthorization",
  "fromAgent": "beneficiary",
  "reason": "Beneficiary approval required for transfers over 1000 USDC"
}
```

#### RequireRelationshipConfirmation
```json
{
  "@type": "RequireRelationshipConfirmation",
  "fromAgent": "originator",
  "aboutParty": "originator",
  "aboutAgent": "did:pkh:eip155:1:0x1234a96D359eC26a11e2C2b3d8f8B8942d5Bfcdb",
  "reason": "Please confirm control of the settlement address"
}
```

## Common Flow Patterns

### 1. Basic Transfer Flow
1. Originator sends **Transfer** message
2. Beneficiary responds with **UpdatePolicies** if needed
3. Parties exchange required information
4. Beneficiary sends **Authorize** message
5. Originator executes on-chain transaction
6. Originator sends **Settle** message with transaction ID

### 2. Payment Request Flow
1. Merchant sends **Payment** message with amount and invoice
2. Customer responds with **Authorize** (or **Reject**)
3. Merchant sends **Complete** with settlement address
4. Customer executes on-chain transaction
5. Customer sends **Settle** message with transaction ID

### 3. Connection Establishment
1. Service sends **Connect** message with constraints
2. VASP responds with **AuthorizationRequired** for user approval
3. After user approval, VASP sends **Authorize**
4. Connection is established for future transactions

### 4. Transaction Cancellation
- Any party can send **Cancel** message before settlement
- Refers to original transaction via thread ID

### 5. Transaction Reversion
1. Party sends **Revert** message with return address and reason
2. Recipient may respond with **Authorize** or **Reject**
3. If authorized, sender executes on-chain transaction
4. Sender sends **Settle** message for the reversion

## Implementation Considerations

### Security Best Practices
- Use DIDComm v2 encryption for all messages
- Implement proper key management for DIDs
- Validate all message signatures
- Follow agent relationship validation practices

### Compliance Integration
- Integrate IVMS-101 formatted data for Travel Rule
- Support ISO 20022 purpose codes for transaction classification
- Include LEIs where appropriate for legal entities
- Implement proper data minimization practices

## References
- Full TAP Specification: [TAIPs](https://github.com/notabene/taips)
- DIDComm v2 Specification: [DIDComm Messaging v2.0](https://identity.foundation/didcomm-messaging/spec/v2.0/)