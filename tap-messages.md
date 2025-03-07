# TAP Message Types

This document provides an overview of all message types in the Transaction Authorization Protocol (TAP) implementation.

Each message type is documented with its corresponding TAIP specification reference and implementation status across the codebase.

## Message Types Table

| Message Type | TAIP Reference | Rust Implementation<br>`tap-msg` | WASM Implementation<br>`tap-wasm` | TypeScript Implementation<br>`tap-ts` |
|--------------|----------------|----------------------------------|-----------------------------------|--------------------------------------|
| Transfer | [TAIP-3](prds/taips/TAIPs/taip-3.md) | - [x] | - [x] | - [x] |
| Authorize | [TAIP-4](prds/taips/TAIPs/taip-4.md) | - [x] | - [x] | - [x] |
| Reject | [TAIP-4](prds/taips/TAIPs/taip-4.md) | - [x] | - [x] | - [x] |
| Settle | [TAIP-4](prds/taips/TAIPs/taip-4.md) | - [x] | - [x] | - [x] |
| Presentation | [TAIP-8](prds/taips/TAIPs/taip-8.md) | - [x] | - [x] | - [x] |
| AddAgents | [TAIP-5](prds/taips/TAIPs/taip-5.md) | - [x] | - [x] | - [x] |
| ReplaceAgent | [TAIP-5](prds/taips/TAIPs/taip-5.md) | - [x] | - [x] | - [x] |
| RemoveAgent | [TAIP-5](prds/taips/TAIPs/taip-5.md) | - [x] | - [x] | - [x] |
| UpdatePolicies | [TAIP-7](prds/taips/TAIPs/taip-7.md) | - [x] | - [x] | - [x] |
| Error | - | - [x] | - [x] | - [x] |

## Message Type Details

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
- `transfer_id`: ID of the transfer being authorized
- `note`: (Optional) Additional note
- `timestamp`: ISO 8601 timestamp
- `metadata`: Additional metadata

### Reject (TAIP-4)

Message for rejecting a transfer.

**Structure:**
- `transfer_id`: ID of the transfer being rejected
- `code`: Rejection code
- `description`: Detailed description of the rejection
- `note`: (Optional) Additional note
- `timestamp`: ISO 8601 timestamp
- `metadata`: Additional metadata

### Settle (TAIP-4)

Message indicating that a transfer has been settled on a blockchain.

**Structure:**
- `transfer_id`: ID of the transfer being settled
- `transaction_id`: Blockchain transaction ID
- `transaction_hash`: (Optional) Transaction hash
- `block_height`: (Optional) Block height
- `note`: (Optional) Additional note
- `timestamp`: ISO 8601 timestamp
- `metadata`: Additional metadata

### Presentation (TAIP-8)

Message providing requested credentials or information.

**Structure:**
- `transfer_id`: ID of the related transfer
- `attributes`: Provided attributes
- `metadata`: Additional metadata

### AddAgents (TAIP-5)

Message for adding additional agents to a transaction.

**Structure:**
- `transfer_id`: ID of the related transfer
- `agents`: Array of agents to add
- `metadata`: Additional metadata

### ReplaceAgent (TAIP-5)

Message for replacing an agent with another in a transaction.

**Structure:**
- `transfer_id`: ID of the related transfer
- `original`: ID of the agent to replace
- `replacement`: New agent information
- `metadata`: Additional metadata

### RemoveAgent (TAIP-5)

Message for removing an agent from a transaction.

**Structure:**
- `transfer_id`: ID of the related transfer
- `agent`: ID of the agent to remove
- `metadata`: Additional metadata

### UpdatePolicies (TAIP-7)

Message for updating policies for a transaction.

**Structure:**
- `transfer_id`: ID of the related transfer
- `policies`: Array of policies
- `metadata`: Additional metadata

### Error

General error message.

**Structure:**
- `code`: Error code
- `description`: Error description
- `original_message_id`: (Optional) ID of the message that caused the error
- `metadata`: Additional metadata
