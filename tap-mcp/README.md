# TAP-MCP: Model Context Protocol Server for TAP

A Model Context Protocol (MCP) server that provides AI applications with standardized access to Transaction Authorization Protocol (TAP) functionality. This enables LLMs and AI agents to create, manage, and monitor TAP transactions through a well-defined interface.

## Overview

TAP-MCP is a thin wrapper around TAP Node that exposes transaction authorization functionality through the Model Context Protocol standard. This enables AI applications to:

- **Agent Management**: Create TAP agents with auto-generated DIDs and manage cryptographic identities
- **Transaction Creation**: Initiate transfers, payments, and other TAP operations
- **Message Monitoring**: Access transaction history and message details
- **Delivery Tracking**: Monitor message delivery status, retry counts, and error details
- **Schema Access**: Get JSON schemas for TAP message types
- **Per-Agent Storage**: Each agent has isolated SQLite storage at `~/.tap/{sanitized_did}/transactions.db`

Key design principles:
- Agents are cryptographic identities (DIDs) without predefined roles
- Roles (SettlementAddress, Exchange, Compliance, etc.) are specified per transaction
- Party associations are transaction-specific, not stored with agents
- Automatic DID generation ensures globally unique identifiers
- All transaction and message operations require an `agent_did` parameter to specify which agent signs the message
- **Agent-Specific Storage**: Each agent has its own isolated SQLite database for transactions and messages

## Installation

### Prerequisites

- Rust 1.70+ with Cargo
- SQLite 3.0+
- TAP ecosystem components (tap-node, tap-agent, tap-msg)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/notabene-id/tap-rs.git
cd tap-rs/tap-mcp

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Quick Start

### 1. Basic Usage

Run the MCP server with default settings:

```bash
cargo run
```

The server will:
- Use `~/.tap` as the TAP root directory
- Create DID-based storage (if --agent-did is provided)
- Listen for MCP requests on stdin/stdout

### 2. Custom Configuration

```bash
# Use specific agent DID for organized storage
cargo run -- --agent-did "did:web:example.com"

# This creates database at ~/.tap/did_web_example.com/transactions.db

# Use custom TAP root directory
cargo run -- --tap-root /path/to/custom/tap

# Enable debug logging
cargo run -- --debug

# Show all options
cargo run -- --help
```

### 3. Integration with AI Applications

TAP-MCP uses stdio transport, making it compatible with MCP clients like Claude Desktop:

```json
{
  "mcpServers": {
    "tap": {
      "command": "/path/to/tap-mcp",
      "args": ["--tap-root", "/your/tap/directory"]
    }
  }
}
```

## Available Tools

TAP-MCP provides 36 comprehensive tools covering the complete TAP transaction lifecycle:

### Agent Management

#### `tap_create_agent`
Create a new TAP agent with auto-generated DID. Agents are cryptographic identities; roles and party associations are specified per transaction.

```json
{
  "label": "My Settlement Agent"
}
```

Returns:
```json
{
  "@id": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "label": "My Settlement Agent",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### `tap_list_agents`
List all configured agents from ~/.tap/keys.json.

```json
{
  "filter": {
    "has_label": true
  },
  "limit": 50,
  "offset": 0
}
```

### Agent Management for Transactions

#### `tap_add_agents`
Add agents to an existing TAP transaction (TAIP-5). Allows adding settlement addresses, compliance officers, or other agents to handle the transaction.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "agents": [
    {
      "@id": "did:example:new-settlement-agent",
      "role": "SettlementAddress",
      "for": "did:example:alice"
    },
    {
      "@id": "did:example:compliance-officer",
      "role": "ComplianceOfficer",
      "for": "did:example:bob"
    }
  ]
}
```

#### `tap_remove_agent`
Remove an agent from a TAP transaction (TAIP-5). Removes an agent's authorization to act on behalf of a party in the transaction.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "agent_to_remove": "did:example:old-settlement-agent"
}
```

#### `tap_replace_agent`
Replace an agent in a TAP transaction (TAIP-5). Atomically replaces one agent with another while maintaining continuity.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "original_agent": "did:example:old-agent",
  "new_agent": {
    "@id": "did:example:new-agent",
    "role": "SettlementAddress",
    "for": "did:example:alice"
  }
}
```

### Transaction Creation

#### `tap_create_transfer`
Initiate a new TAP transfer transaction (TAIP-3). Requires specifying which agent will sign the message.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
  "amount": "100.50",
  "originator": {
    "@id": "did:example:alice"
  },
  "beneficiary": {
    "@id": "did:example:bob"
  },
  "agents": [
    {
      "@id": "did:example:settlement-agent",
      "role": "SettlementAddress",
      "for": "did:example:alice"
    }
  ],
  "memo": "Payment for services"
}
```

### Transaction Actions

#### `tap_authorize`
Authorize a TAP transaction (TAIP-4). The agent_did specifies which agent signs the authorization.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87",
  "expiry": "2024-12-31T23:59:59Z"
}
```

#### `tap_reject`
Reject a TAP transaction (TAIP-4). The agent_did specifies which agent signs the rejection.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "reason": "Insufficient compliance verification"
}
```

#### `tap_cancel`
Cancel a TAP transaction (TAIP-5). The agent_did specifies which agent signs the cancellation.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "by": "did:example:alice",
  "reason": "Change of plans"
}
```

#### `tap_settle`
Settle a TAP transaction (TAIP-6). The agent_did specifies which agent signs the settlement.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "settlement_id": "eip155:1:0xabcd1234567890abcdef1234567890abcdef1234",
  "amount": "100.50"
}
```

### Transaction Management

#### `tap_list_transactions`
List transactions with filtering and pagination support. Shows only transactions from the specified agent's storage.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "filter": {
    "message_type": "Transfer",
    "thread_id": "thread-abc123",
    "from_did": "did:example:alice",
    "to_did": "did:example:bob",
    "date_from": "2024-01-01T00:00:00Z",
    "date_to": "2024-12-31T23:59:59Z"
  },
  "sort": {
    "field": "created_time",
    "order": "desc"
  },
  "limit": 100,
  "offset": 0
}
```

### Policy Management

#### `tap_update_policies`
Update policies for a TAP transaction (TAIP-7). Allows agents to update transaction policies such as authorization requirements, compliance rules, or operational constraints.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "policies": [
    {
      "@type": "RequireAuthorization",
      "authorizer": "did:example:compliance-officer",
      "threshold": "100.00",
      "currency": "USD"
    },
    {
      "@type": "RequirePresentation",
      "requested_attributes": ["name", "address", "dateOfBirth"],
      "purpose": "KYC compliance"
    },
    {
      "@type": "TransactionLimit",
      "max_amount": "10000.00",
      "period": "daily"
    }
  ]
}
```

### Customer and Connection Management

#### `tap_list_customers`
Lists all customers (parties) that a specific agent acts on behalf of. Analyzes transaction history to identify parties represented by the agent and includes metadata about each party.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "limit": 50,
  "offset": 0
}
```

Returns:
```json
{
  "customers": [
    {
      "@id": "did:example:alice",
      "metadata": {
        "name": "Alice Smith",
        "company": "ACME Corp"
      },
      "transaction_count": 5,
      "transaction_ids": ["tx-123", "tx-456", "tx-789"]
    }
  ],
  "total": 1
}
```

#### `tap_list_connections`
Lists all counterparties (connections) that a specific party has transacted with. Searches across all agent databases to find transaction relationships and includes role information.

```json
{
  "party_id": "did:example:alice",
  "limit": 50,
  "offset": 0
}
```

Returns:
```json
{
  "connections": [
    {
      "@id": "did:example:bob",
      "metadata": {
        "name": "Bob Johnson",
        "company": "Widget Inc"
      },
      "transaction_count": 3,
      "transaction_ids": ["tx-123", "tx-456", "tx-789"],
      "roles": ["beneficiary", "originator"]
    }
  ],
  "total": 1
}
```

#### `tap_get_customer_details`
Get detailed information about a specific customer, including all their profile data and transaction history.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "customer_id": "did:example:alice"
}
```

#### `tap_generate_ivms101`
Generate IVMS101 travel rule data from a customer's profile. Converts stored customer information into the IVMS101 format required for FATF travel rule compliance.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "customer_id": "did:example:alice"
}
```

Returns IVMS101-formatted data for the customer's natural or legal person information.

#### `tap_update_customer_profile`
Update customer profile data using Schema.org vocabulary. Stores structured customer information in the agent's database.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "customer_id": "did:example:alice",
  "profile_data": {
    "givenName": "Alice",
    "familyName": "Smith",
    "addressCountry": "US",
    "addressRegion": "CA",
    "addressLocality": "San Francisco",
    "postalCode": "94105",
    "streetAddress": "123 Main St"
  }
}
```

#### `tap_update_customer_from_ivms101`
Update customer profile from IVMS101 data. Converts IVMS101 travel rule data into Schema.org format and stores it in the customer profile.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "customer_id": "did:example:alice",
  "ivms101_data": {
    "naturalPerson": {
      "name": {
        "nameIdentifier": [
          {
            "primaryIdentifier": "Smith",
            "secondaryIdentifier": "Alice",
            "nameIdentifierType": "LEGL"
          }
        ]
      },
      "geographicAddress": [
        {
          "addressType": "HOME",
          "townLocationName": "San Francisco",
          "countrySubDivision": "CA",
          "country": "US",
          "postCode": "94105"
        }
      ]
    }
  }
}
```

### Communication Tools

#### `tap_trust_ping`
Send a trust ping message (DIDComm standard) to verify connectivity and agent availability.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "recipient_did": "did:example:bob",
  "comment": "Testing connection"
}
```

#### `tap_basic_message`
Send a basic text message (DIDComm standard) for human-readable communication between agents.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "recipient_did": "did:example:bob",
  "content": "Please review the pending transaction"
}
```

### Transaction Reversal

#### `tap_revert`
Request reversal of a settled transaction (TAIP-14). Used for compliance reversals or dispute resolutions.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-12345",
  "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87",
  "reason": "Compliance investigation required"
}
```

### Message Delivery Tracking

#### `tap_list_deliveries_by_recipient`
List message deliveries to a specific recipient, showing delivery status and retry information.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "recipient_did": "did:example:bob",
  "limit": 50,
  "offset": 0
}
```

#### `tap_list_deliveries_by_message`
List all delivery attempts for a specific message ID.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "message_id": "msg-12345",
  "limit": 50,
  "offset": 0
}
```

#### `tap_list_deliveries_by_thread`
List all message deliveries for a transaction thread.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "thread_id": "tx-12345",
  "limit": 50,
  "offset": 0
}
```

### Database Tools

#### `tap_query_database`
Execute direct SQL queries on the agent's database for advanced analysis. Read-only access.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "query": "SELECT COUNT(*) as total, message_type FROM messages WHERE created_time > datetime('now', '-7 days') GROUP BY message_type",
  "params": []
}
```

#### `tap_get_database_schema`
Get the complete database schema for understanding the data structure.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"
}
```

### Message Debugging Tools

#### `tap_list_received`
Lists raw received messages with filtering and pagination support. Shows all incoming messages (JWE, JWS, or plain) before processing.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "status": "pending",
  "source_type": "https",
  "limit": 50,
  "offset": 0
}
```

#### `tap_get_pending_received`
Gets pending received messages that haven't been processed yet. Useful for debugging message processing issues.

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "limit": 50
}
```

#### `tap_view_raw_received`
Views the raw content of a received message. Shows the complete raw message as received (JWE, JWS, or plain JSON).

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "received_id": 42
}
```

### Decision Management

#### `tap_list_pending_decisions`
Lists pending decisions from the `decision_log` table. Use this to poll for decisions that require action when tap-http is running in poll mode (`--decision-mode poll`).

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "status": "pending",
  "since_id": 0,
  "limit": 50
}
```

Returns:
```json
{
  "decisions": [
    {
      "id": 1,
      "transaction_id": "tx-12345",
      "agent_did": "did:key:z6Mk...",
      "decision_type": "authorization_required",
      "status": "pending",
      "context": { "transaction_state": "received", "pending_agents": ["did:key:z6Mk..."] },
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### `tap_resolve_decision`
Resolves a pending decision by marking it as resolved in the database. Note: this only updates the decision log — to actually send the TAP message, call the corresponding action tool (e.g., `tap_authorize`).

```json
{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "decision_id": 1,
  "action": "authorize",
  "detail": "Approved by compliance team"
}
```

**Auto-resolution**: When action tools (`tap_authorize`, `tap_reject`, `tap_settle`, `tap_cancel`, `tap_revert`) succeed, matching pending decisions are automatically resolved. This means you typically only need to call the action tool — you don't need to explicitly call `tap_resolve_decision` afterwards.

## Available Resources

TAP-MCP provides 6 read-only resources for accessing TAP data without requiring tool calls:

### `tap://agents`
Read-only access to agent information.

```
tap://agents                           # All agents with their DIDs and labels
```

### `tap://messages`
Access to transaction messages and history from agent-specific storage.

```
tap://messages?agent_did=did:key:z6Mk...           # Messages for specific agent (required)
tap://messages?agent_did=did:key:z6Mk...&direction=incoming   # Filter by direction
tap://messages?agent_did=did:key:z6Mk...&direction=outgoing   # Outgoing messages only
tap://messages?agent_did=did:key:z6Mk...&thread_id=abc123     # Filter by thread
tap://messages?agent_did=did:key:z6Mk...&type=Transfer        # Filter by message type
tap://messages?agent_did=did:key:z6Mk...&limit=100&offset=50  # Pagination
tap://messages/msg-id-123                          # Specific message
```

### `tap://deliveries`
Access to message delivery tracking information.

```
tap://deliveries?agent_did=did:key:z6Mk... # Delivery records for specific agent
tap://deliveries?message_id=msg-123        # Deliveries for specific message
tap://deliveries?recipient_did=did:example:bob # Deliveries to specific recipient
tap://deliveries?delivery_type=https       # Filter by delivery type (https/internal/return_path/pickup)
tap://deliveries?status=failed             # Filter by status (pending/success/failed)
tap://deliveries?limit=50&offset=100       # Pagination
tap://deliveries/123                        # Specific delivery record by ID
```

### `tap://database-schema`
**New in v0.5.0** - Access to database schema information for agent storage.

```
tap://database-schema?agent_did=did:key:z6Mk...    # Complete schema for agent's database (required)
tap://database-schema?agent_did=did:key:z6Mk...&table_name=messages # Specific table schema
```

Returns comprehensive database schema information including:
- Table structures and column definitions
- Index information and constraints  
- Row counts for each table
- Database path and metadata

This resource provides the same information as the `tap_get_database_schema` tool but through the MCP resource interface, making it more appropriate for read-only data access.

### `tap://schemas`
JSON schemas for TAP message types with enhanced lookup capabilities.

```
tap://schemas                          # All TAP message schemas with version info
tap://schemas/Transfer                 # Specific schema for Transfer message type
tap://schemas/Authorize                # Specific schema for Authorize message type
tap://schemas/Reject                   # Specific schema for Reject message type
tap://schemas/Settle                   # Specific schema for Settle message type
tap://schemas/Cancel                   # Specific schema for Cancel message type
```

**Enhanced in v0.5.0** with individual schema lookup:
- Access specific message schemas by name (e.g., `Transfer`, `Authorize`)
- Search by message type URL (e.g., `https://tap.rsvp/schema/1.0#Transfer`)
- Includes comprehensive JSON schemas for all TAIP message types
- Version information and TAIP specification references

### `tap://received`
Access to raw received messages before processing.

```
tap://received?agent_did=did:key:z6Mk... # Received messages for specific agent
tap://received?agent_did=did:key:z6Mk...&status=pending # Filter by status (pending/processed/failed)
tap://received?agent_did=did:key:z6Mk...&source_type=https # Filter by source type
tap://received?agent_did=did:key:z6Mk...&limit=50&offset=100 # Pagination
tap://received/123                      # Specific received message by ID
```

## Configuration

### Environment Variables

- `TAP_ROOT`: Default TAP root directory (default: `~/.tap`)
- `TAP_DB_PATH`: Database file path (default: `$TAP_ROOT/tap-node.db`)
- `RUST_LOG`: Logging level (debug, info, warn, error)

### Directory Structure

```
~/.tap/                               # TAP root directory
├── keys.json                        # Agent keys storage
├── did_key_z6MkpGuzuD38tpgZKPfm/     # Auto-generated agent directory
│   └── transactions.db              # SQLite database for this agent
├── did_web_example.com/             # Manual agent directory
│   └── transactions.db              # SQLite database for this agent
└── logs/                           # Log files directory
```

**Automatic Storage Initialization**: When you create a new agent using `tap_create_agent`, TAP-MCP automatically:
1. Generates a unique DID for the agent
2. Creates a sanitized directory name from the DID (replacing `:` with `_`)
3. Initializes a dedicated SQLite database for that agent's transactions
4. Registers the agent with the TAP Node for message processing

**Agent-Specific Storage**: Each transaction operation (create, authorize, reject, cancel, settle, list) uses the storage database specific to the `agent_did` parameter. This ensures:
- Complete transaction isolation between different agents
- Scalable storage architecture as each agent manages its own data
- Clear audit trails per agent identity
- No cross-contamination of transaction data between agents
- Multi-agent transaction storage: transactions involving multiple agents are stored in all participating agents' databases
- Multi-recipient message delivery: messages are delivered to ALL recipients specified in the `to` field following DIDComm specification

## Examples

### Creating a Complete Transfer Flow

1. **Create agents for both parties:**

```bash
# Create settlement agent for originator
echo '{"label": "Alice Settlement Agent"}' | \
  tap-mcp-client call tap_create_agent

# Create compliance agent for beneficiary
echo '{"label": "Bob Compliance Agent"}' | \
  tap-mcp-client call tap_create_agent
```

2. **Initiate transfer:** (Note: Use the DID from the created agent)

```bash
echo '{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
  "amount": "250.00",
  "originator": {"@id": "did:example:alice"},
  "beneficiary": {"@id": "did:example:bob"},
  "agents": [
    {"@id": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc", "role": "SettlementAddress", "for": "did:example:alice"},
    {"@id": "did:key:z6MkhVTKxFiPPR8RdLYgfJxL8jmCW6e4NPEFhR6BEhPo8Lyy", "role": "Compliance", "for": "did:example:bob"}
  ]
}' | tap-mcp-client call tap_create_transfer
```

3. **Authorize the transfer:**

```bash
echo '{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-abc123",
  "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87",
  "expiry": "2024-12-31T23:59:59Z"
}' | tap-mcp-client call tap_authorize
```

4. **Settle the transaction:**

```bash
echo '{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-abc123",
  "settlement_id": "eip155:1:0xabcd1234567890abcdef1234567890abcdef1234",
  "amount": "250.00"
}' | tap-mcp-client call tap_settle
```

5. **Monitor transactions:**

```bash
# List all transactions for a specific agent
echo '{"agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"}' | \
  tap-mcp-client call tap_list_transactions

# List recent transfers for an agent
echo '{"agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc", "filter": {"message_type": "Transfer"}, "limit": 10}' | \
  tap-mcp-client call tap_list_transactions

# List recent messages
tap-mcp-client resource tap://messages

# Check delivery status for the transaction
tap-mcp-client resource tap://deliveries?agent_did=did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc

# Check failed deliveries
tap-mcp-client resource tap://deliveries?status=failed

# Get database schema for the agent
tap-mcp-client resource tap://database-schema?agent_did=did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc

# Get schema for Transfer messages
tap-mcp-client resource tap://schemas/Transfer

# Get all message schemas
tap-mcp-client resource tap://schemas

# List customers that the agent represents
echo '{"agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"}' | \
  tap-mcp-client call tap_list_customers

# List connections for a specific party
echo '{"party_id": "did:example:alice"}' | \
  tap-mcp-client call tap_list_connections
```

### Alternative Workflow: Rejecting a Transaction

If a transaction needs to be rejected instead of authorized:

```bash
# Reject with reason
echo '{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-abc123",
  "reason": "Insufficient compliance verification"
}' | tap-mcp-client call tap_reject
```

### Canceling a Transaction

Either party can cancel a transaction before settlement:

```bash
# Cancel transaction
echo '{
  "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
  "transaction_id": "tx-abc123",
  "by": "did:example:alice",
  "reason": "Change of plans"
}' | tap-mcp-client call tap_cancel
```

## Integration Examples

### Claude Desktop

To configure Claude Desktop to use TAP-MCP, you need to add the server configuration to your Claude Desktop settings. The configuration file location depends on your operating system:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

#### Option 1: Using the built binary (recommended for production)

First, build and install TAP-MCP:

```bash
cd /path/to/tap-rs/tap-mcp
cargo build --release
sudo cp target/release/tap-mcp /usr/local/bin/
```

Then add this configuration to your Claude Desktop config file:

```json
{
  "mcpServers": {
    "tap": {
      "command": "tap-mcp",
      "env": {
        "TAP_ROOT": "/your/tap/directory",
        "RUST_LOG": "info"
      }
    }
  }
}
```

#### Option 2: Using cargo run (for development)

Add this configuration to your Claude Desktop config file:

```json
{
  "mcpServers": {
    "tap": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "/path/to/tap-rs/tap-mcp/Cargo.toml",
      ],
      "env": {
        "TAP_ROOT": "/your/tap/directory",
        "RUST_LOG": "info"
      }
    }
  }
}
```

#### Configuration Options

- `--agent-did`: (Optional) Specify an agent DID for organized storage. Creates database at `~/.tap/{sanitized-did}/`
- `--tap-root`: (Optional) Custom TAP root directory (default: `~/.tap`)
- `--debug`: Enable debug logging
- `TAP_ROOT`: Environment variable alternative to `--tap-root`
- `RUST_LOG`: Control log level (debug, info, warn, error)

#### Complete Example Configuration

```json
{
  "mcpServers": {
    "tap-production": {
      "command": "tap-mcp",
      "args": [
        "--agent-did", "did:web:mycompany.com:agents:settlement",
        "--tap-root", "/home/user/.tap-production"
      ],
      "env": {
        "RUST_LOG": "info"
      }
    },
    "tap-development": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path", "/home/user/code/tap-rs/tap-mcp/Cargo.toml",
        "--",
        "--debug",
        "--agent-did", "did:web:localhost:8080:dev-agent"
      ],
      "env": {
        "TAP_ROOT": "/tmp/tap-dev",
        "RUST_LOG": "debug"
      }
    }
  }
}
```

#### Verifying the Configuration

After updating your Claude Desktop configuration:

1. **Restart Claude Desktop** for the changes to take effect
2. **Check the Claude Desktop logs** (usually in the app's menu under "View" → "Developer" → "Developer Tools")
3. **Test the connection** by asking Claude: "List the available TAP tools"

You should see tools like `tap_create_agent`, `tap_create_transfer`, `tap_authorize`, etc.

#### Troubleshooting Claude Desktop Integration

If TAP-MCP doesn't appear in Claude Desktop:

1. **Check the config file syntax** - ensure valid JSON formatting
2. **Verify file paths** - make sure the `command` and `manifest-path` are correct
3. **Check permissions** - ensure Claude Desktop can execute the command
4. **Review logs** - check Claude Desktop's developer console for error messages
5. **Test manually** - try running the exact command from terminal first:

```bash
# Test the command manually
cargo run --manifest-path /path/to/tap-rs/tap-mcp/Cargo.toml -- --help

# Or if using the binary
tap-mcp --help
```

Once configured, you can interact with TAP through Claude Desktop by asking questions like:
- "Create a new TAP agent for settlement services"
- "Show me recent TAP transactions"
- "Help me authorize a transfer transaction"
- "List all customers that my agent represents"
- "Show me connections for a specific party"

### Python MCP Client

```python
import asyncio
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

async def main():
    async with stdio_client(
        StdioServerParameters(
            command="tap-mcp",
            args=["--tap-root", "/your/tap/directory"]
        )
    ) as (read, write):
        async with ClientSession(read, write) as session:
            # Initialize the session
            await session.initialize()

            # List available tools
            tools = await session.list_tools()
            print(f"Available tools: {[tool.name for tool in tools.tools]}")

            # Create an agent
            agent_result = await session.call_tool(
                "tap_create_agent",
                {
                    "label": "Test Exchange Agent"
                }
            )
            print(f"Agent created: {agent_result}")

            # Create a transfer transaction
            transfer_result = await session.call_tool(
                "tap_create_transfer",
                {
                    "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
                    "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
                    "amount": "100.00",
                    "originator": {"@id": "did:example:alice"},
                    "beneficiary": {"@id": "did:example:bob"},
                    "agents": [
                        {"@id": "did:example:test-agent", "role": "Exchange", "for": "did:example:alice"}
                    ]
                }
            )
            print(f"Transfer created: {transfer_result}")

            # Authorize the transaction
            auth_result = await session.call_tool(
                "tap_authorize",
                {
                    "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
                    "transaction_id": "tx-12345",
                    "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87"
                }
            )
            print(f"Transaction authorized: {auth_result}")

            # List recent transactions
            list_result = await session.call_tool(
                "tap_list_transactions",
                {
                    "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc",
                    "limit": 10,
                    "filter": {"message_type": "Transfer"}
                }
            )
            print(f"Recent transfers: {list_result}")

            # List customers of the agent
            customers_result = await session.call_tool(
                "tap_list_customers",
                {
                    "agent_did": "did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"
                }
            )
            print(f"Agent customers: {customers_result}")

            # List connections for a specific party
            connections_result = await session.call_tool(
                "tap_list_connections",
                {
                    "party_id": "did:example:alice"
                }
            )
            print(f"Party connections: {connections_result}")

if __name__ == "__main__":
    asyncio.run(main())
```

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   ```bash
   # Ensure TAP Node has initialized the database
   tap-node init --db-path ~/.tap/tap-node.db
   ```

2. **Permission Errors**
   ```bash
   # Check file permissions
   chmod 755 ~/.tap
   chmod 644 ~/.tap/tap-node.db
   ```

3. **Missing Dependencies**
   ```bash
   # Rebuild with all features
   cargo clean
   cargo build --features full
   ```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
RUST_LOG=debug cargo run -- --debug
```

### Testing the Connection

Test basic MCP connectivity:

```bash
# Send a simple initialize request
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test"}}}' | cargo run
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_agent
```

### Adding New Tools

1. Create tool implementation in `src/tools/`
2. Add to tool registry in `src/tools/mod.rs`
3. Add JSON schema in `src/tools/schema.rs`
4. Update documentation

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [TAP-RS](https://github.com/notabene-id/tap-rs) - Core TAP implementation in Rust
- [tap-cli](../tap-cli/README.md) - Command-line interface for TAP Agent operations (terminal alternative to tap-mcp)
- [Model Context Protocol](https://github.com/anthropics/mcp) - MCP specification and tools
- [Claude Desktop](https://claude.ai) - AI assistant with MCP support

## Support

- GitHub Issues: [Report bugs and request features](https://github.com/notabene-id/tap-rs/issues)
- Documentation: [TAP Protocol Documentation](https://tap.rsvp)
- Community: [TAP Discord Server](https://discord.gg/tap)
