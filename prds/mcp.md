# TAP-MCP: Model Context Protocol Server for TAP Node

## Executive Summary

TAP-MCP is a Model Context Protocol (MCP) server that wraps the TAP Node functionality, providing LLMs and AI applications with direct access to Transaction Authorization Protocol (TAP) capabilities. This server enables AI agents to create and manage blockchain transaction authorizations, handle payments, and interact with TAP agents through a standardized interface.

## Problem Statement

Currently, integrating TAP functionality into AI applications requires custom implementations for each use case. LLMs and AI agents need a standardized way to:
- Create and manage TAP agents
- Initiate transfers and payments
- List and monitor transactions
- Authorize, reject, or cancel transactions
- Access TAP's compliance and authorization workflows

Without a standardized interface, each AI application must implement its own TAP integration, leading to duplicated effort and inconsistent implementations.

## Solution Overview

TAP-MCP provides a Model Context Protocol server that exposes TAP Node functionality through standardized MCP tools and resources. This allows any MCP-compatible client (such as Claude Desktop, AI IDEs, or custom applications) to interact with TAP's transaction authorization system.

### Key Benefits

1. **Standardized Access**: Any MCP client can interact with TAP without custom integration
2. **AI-Native Interface**: Designed for LLMs to easily understand and use TAP functionality
3. **Secure by Default**: Leverages TAP's existing security model with MCP's transport security
4. **Extensible**: Easy to add new TAP capabilities as they become available
5. **Local-First**: Runs alongside tap-node, keeping sensitive data within user control

## Technical Architecture

### System Components

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   MCP Client    │────▶│    TAP-MCP      │────▶│    TAP Node     │
│ (Claude, IDE)   │◀────│     Server      │◀────│   (tap-node)    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                        │                        │
        │   stdio/SSE/WS        │      Rust API         │
        └────────────────────────┴────────────────────────┘
                                         │
                                         ▼
                                 ┌─────────────────┐
                                 │   Database      │
                                 │   (SQLite)      │
                                 └─────────────────┘
```

TAP-MCP acts as a thin wrapper around tap-node, which handles:
- **Database Management**: All transaction and message persistence
- **Message Processing**: Validation, routing, and state management
- **Event Handling**: Transaction lifecycle events and notifications
- **Agent Communication**: Sending and receiving TAP messages

### Transport Methods

The primary focus will be on **stdio** transport for:
- Local execution security
- Simple integration with desktop clients
- No network configuration required
- Direct process communication

Future support may include:
- Server-Sent Events (SSE) for web-based clients
- WebSocket for bidirectional streaming

### Core Dependencies

- `tap-node`: Core TAP engine for:
  - Database management (SQLite storage)
  - Message processing and validation
  - Transaction state management
  - Event subscription and handling
  - Message transport (HTTP/WebSocket)
- `tap-agent`: Agent management (loads from `~/.tap` directory)
- `tap-msg`: Message type definitions and serialization
- MCP SDK (Rust implementation to be created or TypeScript with Rust bindings)

### Configuration and Storage

TAP-MCP will leverage tap-agent's existing functionality to:
- Load agent configurations from the `~/.tap` directory
- Use existing DID documents and key material stored in `~/.tap/keys`
- Access saved agent profiles and credentials from `~/.tap/agents`
- Maintain compatibility with other TAP tools using the same directory structure

This ensures:
- Consistent agent identity across all TAP tools
- No duplication of configuration or keys
- Seamless integration with existing TAP installations
- Proper security for stored credentials

## Functional Requirements

TAP-MCP leverages tap-node's existing functionality for all database operations and message handling. Instead of reimplementing these features, TAP-MCP translates MCP tool calls into tap-node API calls.

### 1. Agent Management

TAP-MCP uses tap-agent's functionality to manage agents stored in the `~/.tap` directory. This ensures consistency with other TAP tools and maintains a single source of truth for agent configurations.

#### Tool: `tap.create_agent`
Creates a new TAP agent with specified configuration and stores it in `~/.tap/agents`.

**Parameters:**
```json
{
  "@id": "string",        // Agent DID
  "role": "string",       // Agent role (e.g., "SettlementAddress", "Exchange")
  "for": "string",        // DID of party agent acts for
  "policies": [],         // Optional: agent policies (TAIP-7)
  "metadata": {}          // Optional: additional metadata
}
```

**Returns:**
```json
{
  "agent": {
    "@id": "string",
    "role": "string",
    "for": "string"
  },
  "created_at": "timestamp"
}
```

#### Tool: `tap.list_agents`
Lists all configured agents from the `~/.tap/agents` directory.

**Parameters:**
```json
{
  "filter": {
    "role": "string",     // Optional: Filter by agent role
    "for_party": "string" // Optional: Filter by party DID
  },
  "limit": "number",      // Default: 50
  "offset": "number"      // Default: 0
}
```

**Returns:**
```json
{
  "agents": [
    {
      "@id": "string",
      "role": "string",
      "for": "string",
      "policies": [],
      "metadata": {}
    }
  ],
  "total": "number"
}
```

#### Tool: `tap.add_agents`
Adds agents to an existing transaction (TAIP-5).

**Parameters:**
```json
{
  "transaction_id": "string",
  "agents": [{
    "@id": "string",
    "role": "string",
    "for": "string",
    "policies": []
  }]
}
```

#### Tool: `tap.replace_agent`
Replaces an agent in a transaction (TAIP-5).

**Parameters:**
```json
{
  "transaction_id": "string",
  "original": "string",      // DID of agent to replace
  "replacement": {
    "@id": "string",
    "role": "string",
    "for": "string"
  }
}
```

#### Tool: `tap.remove_agent`
Removes an agent from a transaction (TAIP-5).

**Parameters:**
```json
{
  "transaction_id": "string",
  "agent": "string"          // DID of agent to remove
}
```

### 2. Transaction Creation

#### Tool: `tap.create_transfer`
Initiates a new transfer between parties using the TAP Transfer message (TAIP-3). This tool:
1. Creates a Transfer message using tap-msg types
2. Submits it to tap-node for processing
3. tap-node handles database storage and message routing

**Parameters:**
```json
{
  "asset": "string",           // CAIP-19 asset identifier
  "amount": "string",          // Decimal amount as string
  "originator": {
    "@id": "string",           // DID of the originator
    "metadata": {}             // Optional: additional party metadata
  },
  "beneficiary": {
    "@id": "string",           // DID of the beneficiary
    "metadata": {}             // Optional: additional party metadata
  },
  "agents": [{                 // List of agents involved
    "@id": "string",           // Agent DID
    "role": "string",          // Agent role (e.g., "SettlementAddress")
    "for": "string"            // DID of party agent acts for
  }],
  "memo": "string",            // Optional: transaction memo
  "metadata": {}               // Optional: additional metadata
}
```

**Returns:**
```json
{
  "transaction_id": "string",
  "message_id": "string",
  "status": "created",
  "created_at": "timestamp"
}
```

#### Tool: `tap.create_payment`
Creates a payment request using the TAP Payment message (TAIP-14).

**Parameters:**
```json
{
  "asset": "string",           // Optional: CAIP-19 asset identifier
  "currency_code": "string",   // Optional: ISO currency code
  "amount": "string",          // Payment amount
  "supported_assets": ["string"], // Optional: Array of CAIP-19 identifiers
  "merchant": {
    "@id": "string",           // Merchant DID
    "metadata": {}             // Optional: additional party metadata
  },
  "customer": {                // Optional: customer details
    "@id": "string",
    "metadata": {}
  },
  "invoice": {                 // Optional: TAIP-16 invoice
    "id": "string",
    "issueDate": "string",
    "currencyCode": "string",
    "lineItems": [],
    "total": "number"
  },
  "expiry": "string",          // Optional: ISO 8601 expiration time
  "agents": []                 // List of agents involved
}
```

**Returns:**
```json
{
  "transaction_id": "string",
  "message_id": "string",
  "status": "created",
  "created_at": "timestamp"
}
```

### 3. Transaction Management

All transaction queries are performed through tap-node's database layer, which maintains the complete message history and transaction state.

#### Tool: `tap.list_transactions`
Queries tap-node's database for stored messages with filtering and pagination.

**Parameters:**
```json
{
  "filter": {
    "message_type": "string",  // Filter by TAP message type
    "thread_id": "string",     // Filter by thread ID
    "from_did": "string",      // Filter by sender DID
    "to_did": "string",        // Filter by recipient DID
    "date_from": "timestamp",
    "date_to": "timestamp"
  },
  "sort": {
    "field": "string",         // "created_time", "id"
    "order": "string"          // "asc", "desc"
  },
  "limit": "number",
  "offset": "number"
}
```

**Returns:**
```json
{
  "messages": [
    {
      "id": "string",
      "type": "string",        // TAP message type
      "thread_id": "string",
      "from": "string",
      "to": ["string"],
      "created_time": "number",
      "body": {}               // Message body
    }
  ],
  "total": "number"
}
```

#### Tool: `tap.get_transaction`
Retrieves a specific transaction thread with all related messages.

**Parameters:**
```json
{
  "thread_id": "string",
  "include_attachments": "boolean"  // Include message attachments
}
```

**Returns:**
```json
{
  "thread": {
    "thread_id": "string",
    "messages": [{
      "id": "string",
      "type": "string",
      "from": "string",
      "to": ["string"],
      "body": {},
      "created_time": "number"
    }],
    "participants": ["string"]
  }
}
```

#### Tool: `tap.update_party`
Updates party information in a transaction (TAIP-6).

**Parameters:**
```json
{
  "transaction_id": "string",
  "party_type": "string",      // "originator" or "beneficiary"
  "party": {
    "@id": "string",
    "metadata": {}             // Additional party metadata
  }
}
```

#### Tool: `tap.update_policies`
Updates policies for a transaction (TAIP-7).

**Parameters:**
```json
{
  "transaction_id": "string",
  "policies": [{
    "@type": "string",         // Policy type
    "from": ["string"],        // Optional: DIDs policy applies to
    "purpose": "string"        // Optional: human-readable purpose
  }]
}
```

#### Tool: `tap.confirm_relationship`
Confirms a relationship between agents (TAIP-9).

**Parameters:**
```json
{
  "transaction_id": "string",
  "agent_id": "string",
  "relationship_type": "string"
}
```

### 4. Transaction Actions

#### Tool: `tap.authorize`
Creates and sends an Authorize message through tap-node (TAIP-4). tap-node handles:
- Message validation
- Database persistence
- State transitions
- Event notifications
- Message routing to other agents

**Parameters:**
```json
{
  "transaction_id": "string",
  "settlement_address": "string",  // Optional: CAIP-10 format address
  "expiry": "string"               // Optional: ISO 8601 expiry time
}
```

**Returns:**
```json
{
  "success": "boolean",
  "transaction_id": "string",
  "message_id": "string"
}
```

#### Tool: `tap.reject`
Sends a Reject message for a transaction (TAIP-4).

**Parameters:**
```json
{
  "transaction_id": "string",
  "reason": "string"               // Required: reason for rejection
}
```

**Returns:**
```json
{
  "success": "boolean",
  "transaction_id": "string",
  "message_id": "string"
}
```

#### Tool: `tap.cancel`
Sends a Cancel message for a transaction (TAIP-4).

**Parameters:**
```json
{
  "transaction_id": "string",
  "by": "string",                  // Party requesting cancellation
  "reason": "string"               // Optional: reason for cancellation
}
```

**Returns:**
```json
{
  "success": "boolean",
  "transaction_id": "string",
  "message_id": "string"
}
```

#### Tool: `tap.settle`
Sends a Settle message for a transaction (TAIP-4).

**Parameters:**
```json
{
  "transaction_id": "string",
  "settlement_id": "string",       // CAIP-220 identifier
  "amount": "string"               // Optional: amount settled
}
```

**Returns:**
```json
{
  "success": "boolean",
  "transaction_id": "string",
  "message_id": "string"
}
```

#### Tool: `tap.revert`
Sends a Revert message to request transaction reversal (TAIP-4).

**Parameters:**
```json
{
  "transaction_id": "string",
  "settlement_address": "string",  // CAIP-10 return address
  "reason": "string"               // Reason for reversal
}
```

**Returns:**
```json
{
  "success": "boolean",
  "transaction_id": "string",
  "message_id": "string"
}
```

### 5. Resources (Read-Only Data)

Resources provide read-only access to data managed by tap-node and tap-agent.

#### Resource: `tap://agents`
Provides read-only access to agent configurations from `~/.tap/agents`.

**URI Patterns:**
- `tap://agents` - List all agents
- `tap://agents?role={role}` - Filter by role
- `tap://agents?for={did}` - Filter by party DID

#### Resource: `tap://messages`
Provides read-only access to TAP messages stored in tap-node's SQLite database.

**URI Patterns:**
- `tap://messages` - Recent messages
- `tap://messages/{message_id}` - Specific message
- `tap://messages?thread_id={id}` - Messages in a thread
- `tap://messages?type={type}` - Filter by message type

#### Resource: `tap://schemas`
Provides access to TAP message schemas and types.

**URI Patterns:**
- `tap://schemas` - All message schemas
- `tap://schemas/{message_type}` - Specific message schema

### 6. Additional Message Types

#### Tool: `tap.connect`
Sends a Connect message to establish agent connections (TAIP-2).

**Parameters:**
```json
{
  "transaction_id": "string",
  "agent_id": "string",
  "for": "string",
  "role": "string",
  "constraints": {}            // Optional: connection constraints
}
```

#### Tool: `tap.request_presentation`
Requests verifiable presentations (TAIP-10).

**Parameters:**
```json
{
  "transaction_id": "string",
  "presentation_definition": "string",
  "challenge": "string",
  "description": "string",
  "for_originator": "boolean",
  "for_beneficiary": "boolean"
}
```

#### Tool: `tap.send_error`
Sends an error message for a transaction.

**Parameters:**
```json
{
  "code": "string",
  "description": "string",
  "original_message_id": "string"  // Optional: reference to original message
}
```

## Non-Functional Requirements

### Performance
- Sub-100ms response time for all read operations
- Sub-500ms response time for transaction creation
- Support for 1000+ concurrent transactions
- Efficient pagination for large result sets

### Security
- All communication via stdio by default (no network exposure)
- Leverage tap-node's existing authentication and encryption
- No storage of sensitive keys in MCP layer - all keys remain in `~/.tap/keys`
- Uses tap-agent's secure key loading from `~/.tap` directory
- Audit logging for all transaction operations
- Respects file permissions set on `~/.tap` directory structure

### Reliability
- Graceful handling of tap-node connection failures
- Transaction state consistency guarantees
- Automatic reconnection with exponential backoff
- Clear error messages for LLM interpretation

### Usability
- Clear, descriptive tool names and parameters
- Comprehensive error messages with suggested actions
- Self-documenting tool descriptions
- Example usage in all tool definitions

## Implementation Plan

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Set up Rust project structure for tap-mcp
- [ ] Implement MCP server basics with stdio transport
- [ ] Create tap-node integration layer:
  - [ ] Initialize tap-node instance
  - [ ] Set up database connection
  - [ ] Configure message processors
  - [ ] Set up event subscriptions
- [ ] Integrate tap-agent for `~/.tap` directory access
- [ ] Implement basic error handling and logging

### Phase 2: Agent Management (Week 3)
- [ ] Implement `create_agent` tool using tap-agent's `~/.tap` storage
- [ ] Implement `list_agents` tool to read from `~/.tap/agents`
- [ ] Add agent resource provider with `~/.tap` directory monitoring
- [ ] Ensure proper permissions handling for `~/.tap` access
- [ ] Write comprehensive tests

### Phase 3: Transaction Creation (Week 4)
- [ ] Implement `create_transfer` tool using tap-node's send_message API
- [ ] Implement `create_payment` tool using tap-node's send_message API
- [ ] Add input validation before submitting to tap-node
- [ ] Handle tap-node responses and errors
- [ ] Integration tests with tap-node database

### Phase 4: Transaction Management (Week 5)
- [ ] Implement `list_transactions` tool using tap-node's query APIs
- [ ] Implement `get_transaction` tool using tap-node's database
- [ ] Add transaction resource provider connecting to tap-node
- [ ] Leverage tap-node's built-in pagination support

### Phase 5: Transaction Actions (Week 6)
- [ ] Implement `authorize` tool using tap-node's message handling
- [ ] Implement `reject` tool using tap-node's message handling
- [ ] Implement `cancel` tool using tap-node's message handling
- [ ] Implement `settle` and `revert` tools
- [ ] End-to-end transaction flow tests through tap-node

### Phase 6: Polish and Documentation (Week 7)
- [ ] Comprehensive documentation
- [ ] Example client implementations
- [ ] Performance optimization
- [ ] Security audit

### Phase 7: Release Preparation (Week 8)
- [ ] Package as cask for easy installation
- [ ] Create demo videos and tutorials
- [ ] Set up CI/CD pipeline
- [ ] Initial release

## Success Metrics

1. **Adoption Metrics**
   - Number of MCP client integrations
   - Daily active transactions through MCP
   - Developer feedback score > 4.5/5

2. **Performance Metrics**
   - 99.9% uptime for stdio transport
   - < 100ms average response time
   - Zero data loss incidents

3. **Usability Metrics**
   - < 5 minutes to first successful transaction
   - > 90% successful LLM interactions
   - < 10% error rate in production

## Risks and Mitigations

### Technical Risks

1. **MCP Rust SDK Availability**
   - Risk: No official Rust MCP SDK available
   - Mitigation: Create minimal Rust implementation or use TypeScript with FFI

2. **Stdio Transport Limitations**
   - Risk: Limited to local execution only
   - Mitigation: Design for future transport methods from the start

3. **~/.tap Directory Access**
   - Risk: Permission issues or missing directory
   - Mitigation: Graceful fallback and clear error messages, automatic directory creation with proper permissions

3. **tap-node API Changes**
   - Risk: Breaking changes in tap-node APIs
   - Mitigation: Version pinning and comprehensive integration tests

### Operational Risks

1. **Complex Error Scenarios**
   - Risk: LLMs struggle with complex TAP error messages
   - Mitigation: Implement clear error taxonomy and examples

2. **Security Concerns**
   - Risk: Exposing sensitive operations through MCP
   - Mitigation: Implement granular permissions and audit logging

## Future Enhancements

1. **Advanced Transports**
   - WebSocket support for real-time updates
   - SSE for web-based integrations
   - HTTP for cloud deployments

2. **Extended Functionality**
   - Bulk transaction operations
   - Transaction templates
   - Automated compliance workflows
   - Real-time transaction monitoring

3. **AI-Native Features**
   - Pre-trained prompts for common workflows
   - Intelligent error recovery suggestions
   - Transaction pattern analysis
   - Natural language transaction queries

4. **Ecosystem Integration**
   - Direct integration with popular wallets
   - Bridge to other MCP servers
   - Webhook support for external systems

## Conclusion

TAP-MCP will provide a critical bridge between the TAP ecosystem and the growing world of AI applications. By implementing the Model Context Protocol, we enable any AI system to interact with blockchain transactions in a secure, standardized way. The focus on stdio transport ensures security and simplicity for initial adoption, while the architecture supports future expansion to other transport methods and advanced features.