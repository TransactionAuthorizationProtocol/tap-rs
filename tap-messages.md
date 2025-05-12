# TAP Message Types

This document provides an overview of all message types in the Transaction Authorization Protocol (TAP) implementation.

Each message type is documented with its corresponding TAIP specification reference and implementation status across the codebase.

## Message Types Table

| Message Type | TAIP Reference | Rust Implementation<br>`tap-msg` | WASM Implementation<br>`tap-wasm` | TypeScript Implementation<br>`tap-ts` |
|--------------|----------------|----------------------------------|-----------------------------------|--------------------------------------|
| Transfer | [TAIP-3](prds/taips/TAIPs/taip-3.md) | ✅ | ✅ | ✅ |
| Authorize | [TAIP-4](prds/taips/TAIPs/taip-4.md) | ✅ | ✅ | ✅ |
| Reject | [TAIP-4](prds/taips/TAIPs/taip-4.md) | ✅ | ✅ | ✅ |
| Settle | [TAIP-4](prds/taips/TAIPs/taip-4.md) | ✅ | ✅ | ✅ |
| Cancel | [TAIP-4](prds/taips/TAIPs/taip-4.md) | ✅ | ✅ | ✅ |
| Presentation | [TAIP-8](prds/taips/TAIPs/taip-8.md) | ✅ | ✅ | ✅ |
| PaymentRequest | [TAIP-14](prds/taips/TAIPs/taip-14.md) | ✅ | ✅ | ✅ |
| Invoice | [TAIP-16](prds/taips/TAIPs/taip-16.md) | ✅ | ✅ | ✅ |
| Connect | [TAIP-11](prds/taips/TAIPs/taip-11.md) | ✅ | ✅ | ✅ |
| Revert | [TAIP-12](prds/taips/TAIPs/taip-12.md) | ✅ | ✅ | ✅ |
| AddAgents | [TAIP-5](prds/taips/TAIPs/taip-5.md) | ✅ | ✅ | ✅ |
| ReplaceAgent | [TAIP-5](prds/taips/TAIPs/taip-5.md) | ✅ | ✅ | ✅ |
| RemoveAgent | [TAIP-5](prds/taips/TAIPs/taip-5.md) | ✅ | ✅ | ✅ |
| UpdateParty | [TAIP-6](prds/taips/TAIPs/taip-6.md) | ✅ | ❌ | ❌ |
| UpdatePolicies | [TAIP-7](prds/taips/TAIPs/taip-7.md) | ✅ | ✅ | ✅ |
| ConfirmRelationship | [TAIP-9](prds/taips/TAIPs/taip-9.md) | ✅ | ✅ | ✅ |
| Error | - | ✅ | ✅ | ✅ |

## Message Flow

TAP messages typically flow in the following sequence:

1. **Initiation**: Transfer or PaymentRequest initiates a new transaction flow
2. **Response**: Authorize, Reject, or Cancel responds to the initiation
3. **Completion**: Settle confirms on-chain settlement
4. **Exception**: Revert can be used after Settle if needed to reverse a transaction

Additional messages like AddAgents, ReplaceAgent, UpdatePolicies, etc. can be sent during the transaction flow to modify the transaction parameters.

## Core Message Types

### Transfer (TAIP-3)

The primary message type for initiating a virtual asset transfer between parties.

**Structure:**
- `asset`: CAIP-19 asset identifier
- `originator`: Participant who originates the transfer
- `beneficiary`: (Optional) Participant who benefits from the transfer
- `amount`: Amount as a decimal string
- `agents`: Array of participants involved in the transfer
- `settlement_id`: (Optional) Identifier of the settlement transaction
- `memo`: (Optional) Note for the transfer
- `metadata`: Additional metadata

### Authorize (TAIP-4)

Message for authorizing a transfer.

**Structure:**
- `transaction_id`: ID of the transfer being authorized
- `note`: (Optional) Additional note
- `metadata`: Additional metadata

### Reject (TAIP-4)

Message for rejecting a transfer.

**Structure:**
- `transaction_id`: ID of the transfer being rejected
- `reason`: Detailed description of the rejection
- `metadata`: Additional metadata

### Settle (TAIP-4)

Message indicating that a transfer has been settled on a blockchain.

**Structure:**
- `transaction_id`: ID of the transfer being settled
- `settlement_id`: Blockchain transaction ID or reference
- `amount`: (Optional) Final settled amount
- `metadata`: Additional metadata

### Cancel (TAIP-4)

Message for cancelling an in-progress transaction.

**Structure:**
- `transaction_id`: ID of the transfer being cancelled
- `reason`: (Optional) Reason for cancellation
- `note`: (Optional) Additional note
- `metadata`: Additional metadata

## Payment-Related Messages

### PaymentRequest (TAIP-14)

Message for requesting a payment with specific details.

**Structure:**
- `currency_code`: Currency code (e.g., "USD")
- `amount`: Amount as a decimal string
- `merchant`: Merchant participant information
- `payment_options`: (Optional) Available payment options
- `invoice`: (Optional) Detailed invoice information
- `metadata`: Additional metadata

### Invoice (TAIP-16)

Detailed invoice structure that can be included in payment requests.

**Structure:**
- `id`: Invoice identifier
- `issue_date`: ISO 8601 date
- `currency_code`: Currency code
- `line_items`: Array of line items
- `tax_total`: (Optional) Total tax amount
- `total`: Total invoice amount
- `sub_total`: (Optional) Subtotal before tax
- `due_date`: (Optional) ISO 8601 date for payment due
- `metadata`: Additional metadata

## Agent Management Messages

### AddAgents (TAIP-5)

Message for adding additional agents to a transaction.

**Structure:**
- `transaction_id`: ID of the related transfer
- `agents`: Array of agents to add
- `metadata`: Additional metadata

### ReplaceAgent (TAIP-5)

Message for replacing an agent with another in a transaction.

**Structure:**
- `transaction_id`: ID of the related transfer
- `original`: ID of the agent to replace
- `replacement`: New agent information
- `metadata`: Additional metadata

### RemoveAgent (TAIP-5)

Message for removing an agent from a transaction.

**Structure:**
- `transaction_id`: ID of the related transfer
- `agent`: ID of the agent to remove
- `metadata`: Additional metadata

## Other Protocol Messages

### Connect (TAIP-11)

Message to establish a connection between parties.

**Structure:**
- `for`: DID of the entity the connection is for
- `constraints`: Connection constraints
- `metadata`: Additional metadata

### Presentation (TAIP-8)

Message providing requested credentials or information.

**Structure:**
- `transaction_id`: ID of the related transfer
- `attributes`: Provided attributes
- `metadata`: Additional metadata

### Revert (TAIP-12)

Message for indicating a transaction should be reversed after settlement.

**Structure:**
- `transaction_id`: ID of the related transfer
- `settlement_id`: ID of the settlement being reverted
- `reason`: Reason for reversion
- `metadata`: Additional metadata

### UpdatePolicies (TAIP-7)

Message for updating policies for a transaction.

**Structure:**
- `transaction_id`: ID of the related transfer
- `policies`: Array of policies
- `metadata`: Additional metadata

### UpdateParty (TAIP-6)

Message for updating party information in a transaction.

**Structure:**
- `transaction_id`: ID of the related transfer
- `partyType`: Type of party being updated (e.g., "originator", "beneficiary")
- `party`: Party object with updated information
- `metadata`: Additional metadata

### ConfirmRelationship (TAIP-9)

Message for confirming a relationship between agents.

**Structure:**
- `transaction_id`: ID of the related transfer
- `agent_id`: DID of the agent
- `for`: DID of the entity the agent acts on behalf of
- `role`: (Optional) Role of the agent in the transaction
- `metadata`: Additional metadata
- `attachments`: (Optional) Array containing at most one CACAO message for cryptographic proof

### Error

General error message.

**Structure:**
- `code`: Error code
- `description`: Error description
- `original_message_id`: (Optional) ID of the message that caused the error
- `metadata`: Additional metadata
