# TAP-RS MCP Agent Instructions

This document provides instructions for Claude Desktop agents to interact with the TAP-RS (Transaction Authorization Protocol - Rust) system through the Model Context Protocol (MCP).

## Overview

TAP-RS is a Rust implementation of the Transaction Authorization Protocol that enables secure, compliant cryptocurrency transactions. Through MCP, you can:
- Create and manage TAP agents
- Send and receive TAP messages
- Track message deliveries
- Monitor transaction status

## Getting Started

### 1. Check Available Agents

First, list all registered TAP agents:

```
Resource: tap://agents
```

This will show you all agents with their DIDs, roles, and metadata.

### 2. Create a New Agent

To create a new TAP agent:

```
Tool: create_agent
Parameters:
{
  "label": "My Trading Desk"  // Optional human-readable name
}
```

This creates an ephemeral agent with a new DID. The response includes:
- `agent_did`: The agent's decentralized identifier
- `label`: The human-readable label

### 3. Send Messages

#### Send a Transfer Message

To initiate a cryptocurrency transfer:

```
Tool: send_transfer
Parameters:
{
  "agent_did": "did:key:z6Mk...",  // Your agent's DID
  "to_did": "did:key:z6Mk...",     // Recipient's DID
  "asset": "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",  // USDC on Ethereum
  "amount": "1000.00",
  "originator_name": "John Doe",
  "beneficiary_name": "Jane Smith"
}
```

The response includes:
- `message_id`: Unique identifier for tracking
- `delivery_results`: Status of delivery to each recipient

#### Send an Authorization Message

To authorize a transfer:

```
Tool: send_authorize
Parameters:
{
  "agent_did": "did:key:z6Mk...",
  "to_did": "did:key:z6Mk...",
  "transaction_id": "tx-123",
  "settlement_address": "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f2bD6e"
}
```

## Monitoring Deliveries

### Check Delivery Status

To see all message deliveries for an agent:

```
Resource: tap://deliveries?agent_did=did:key:z6Mk...
```

### Filter Deliveries

You can filter deliveries by various criteria:

#### Failed Deliveries
```
Resource: tap://deliveries?agent_did=did:key:z6Mk...&status=failed
```

#### External HTTP Deliveries
```
Resource: tap://deliveries?agent_did=did:key:z6Mk...&delivery_type=https
```

#### Deliveries for a Specific Message
```
Resource: tap://deliveries?agent_did=did:key:z6Mk...&message_id=msg-123
```

#### Get Specific Delivery Details
```
Resource: tap://deliveries/42
```

### Understanding Delivery Records

Each delivery record contains:
- `id`: Unique delivery ID
- `message_id`: ID of the message being delivered
- `message_text`: The full signed/encrypted message
- `recipient_did`: DID of the intended recipient
- `delivery_url`: HTTP endpoint (for external deliveries)
- `delivery_type`: Type of delivery (https, internal, return_path, pickup)
- `status`: Current status (pending, success, failed)
- `retry_count`: Number of delivery attempts
- `last_http_status_code`: HTTP response code (for external deliveries)
- `error_message`: Error details if failed
- `created_at`, `updated_at`, `delivered_at`: Timestamps

## Viewing Messages

### List All Messages for an Agent

```
Resource: tap://messages?agent_did=did:key:z6Mk...
```

### Filter Messages

#### Incoming Messages Only
```
Resource: tap://messages?agent_did=did:key:z6Mk...&direction=incoming
```

#### Messages by Type
```
Resource: tap://messages?agent_did=did:key:z6Mk...&type=https://tap.rsvp/schema/1.0#Transfer
```

#### Messages in a Thread
```
Resource: tap://messages?agent_did=did:key:z6Mk...&thread_id=tx-123
```

### Get Specific Message
```
Resource: tap://messages/msg-123
```

## Transaction Workflows

### Complete Transfer Flow

1. **Originator creates transfer**:
```
Tool: send_transfer
Parameters: {
  "agent_did": "did:key:originator",
  "to_did": "did:key:beneficiary_vasp",
  "asset": "eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
  "amount": "1000.00",
  "originator_name": "John Doe",
  "beneficiary_name": "Jane Smith"
}
```

2. **Check delivery status**:
```
Resource: tap://deliveries?agent_did=did:key:originator&message_id=<message_id_from_step_1>
```

3. **Beneficiary VASP views received transfer**:
```
Resource: tap://messages?agent_did=did:key:beneficiary_vasp&type=https://tap.rsvp/schema/1.0#Transfer
```

4. **Beneficiary VASP authorizes**:
```
Tool: send_authorize
Parameters: {
  "agent_did": "did:key:beneficiary_vasp",
  "to_did": "did:key:originator",
  "transaction_id": "<transaction_id_from_transfer>",
  "settlement_address": "eip155:1:0x742d35Cc6634C0532925a3b844Bc9e7595f2bD6e"
}
```

## Delivery Types Explained

- **`internal`**: Message delivered to another agent on the same TAP node
- **`https`**: Message delivered via HTTP to an external TAP node
- **`return_path`**: Response delivered back through the original connection
- **`pickup`**: Message stored for later retrieval by recipient

## Status Values

- **`pending`**: Delivery attempt in progress
- **`success`**: Successfully delivered (HTTP 2xx response or internal delivery)
- **`failed`**: Delivery failed (network error, HTTP error, or recipient not found)

## Best Practices

1. **Always Check Delivery Status**: After sending a message, check the delivery status to ensure it was received
2. **Monitor Failed Deliveries**: Regularly check for failed deliveries that may need manual intervention
3. **Use Thread IDs**: Keep track of transaction threads to follow the complete message flow
4. **Handle Errors Gracefully**: Check error messages in failed deliveries for troubleshooting

## Common Issues

### Delivery Failed with 404
The recipient's DID could not be resolved to an endpoint. Verify the recipient DID is correct.

### Delivery Failed with 403
The recipient's server rejected the message. This could be due to authorization issues or policy violations.

### No Service Endpoint Found
The recipient's DID document doesn't contain a service endpoint. They may not be set up to receive TAP messages.

## Example Monitoring Script

To monitor all your agent's activities:

```python
# Pseudo-code for monitoring
agent_did = "did:key:z6Mk..."

# Check recent messages
messages = fetch("tap://messages?agent_did={agent_did}&limit=10")

# Check delivery status
deliveries = fetch("tap://deliveries?agent_did={agent_did}&limit=10")

# Alert on failures
failed = fetch("tap://deliveries?agent_did={agent_did}&status=failed")
if failed.total > 0:
    alert("Failed deliveries detected!")
```

## Available Tools Summary

- `create_agent`: Create a new TAP agent
- `send_transfer`: Send a transfer message
- `send_authorize`: Send an authorization message
- `send_reject`: Reject a transaction
- `send_cancel`: Cancel a transaction
- `send_settle`: Mark a transaction as settled

## Available Resources Summary

- `tap://agents`: List all agents
- `tap://messages`: View messages with filtering
- `tap://deliveries`: Track message deliveries
- `tap://schemas`: View TAP message schemas

Remember: All message delivery is automatically tracked. You don't need to manually create delivery records - they're created automatically whenever you send a message.