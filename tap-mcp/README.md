# TAP-MCP: Model Context Protocol Server for TAP

A Model Context Protocol (MCP) server that provides AI applications with standardized access to Transaction Authorization Protocol (TAP) functionality. This enables LLMs and AI agents to create, manage, and monitor TAP transactions through a well-defined interface.

## Overview

TAP-MCP is a thin wrapper around TAP Node that exposes transaction authorization functionality through the Model Context Protocol standard. This enables AI applications to:

- **Agent Management**: Create and list TAP agents with policies
- **Transaction Creation**: Initiate transfers, payments, and other TAP operations  
- **Message Monitoring**: Access transaction history and message details
- **Schema Access**: Get JSON schemas for TAP message types
- **DID-based Storage**: Uses TAP Node's DID-organized database structure (~/.tap/{did}/)

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

TAP-MCP provides 8 comprehensive tools covering the complete TAP transaction lifecycle:

### Agent Management

#### `tap.create_agent`
Create a new TAP agent with specified role and policies.

```json
{
  "@id": "did:example:agent123",
  "role": "SettlementAddress", 
  "for": "did:example:party456",
  "policies": [
    {
      "type": "AmountLimit",
      "max_amount": "1000.00"
    }
  ],
  "metadata": {
    "description": "Settlement agent for party 456"
  }
}
```

#### `tap.list_agents`
List agents with optional filtering and pagination.

```json
{
  "filter": {
    "role": "Exchange",
    "for_party": "did:example:party123"
  },
  "limit": 50,
  "offset": 0
}
```

### Transaction Creation

#### `tap.create_transfer`
Initiate a new TAP transfer transaction (TAIP-3).

```json
{
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

#### `tap.authorize`
Authorize a TAP transaction (TAIP-4).

```json
{
  "transaction_id": "tx-12345",
  "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87",
  "expiry": "2024-12-31T23:59:59Z"
}
```

#### `tap.reject`
Reject a TAP transaction (TAIP-4).

```json
{
  "transaction_id": "tx-12345",
  "reason": "Insufficient compliance verification"
}
```

#### `tap.cancel`
Cancel a TAP transaction (TAIP-5).

```json
{
  "transaction_id": "tx-12345",
  "by": "did:example:alice",
  "reason": "Change of plans"
}
```

#### `tap.settle`
Settle a TAP transaction (TAIP-6).

```json
{
  "transaction_id": "tx-12345",
  "settlement_id": "eip155:1:0xabcd1234567890abcdef1234567890abcdef1234",
  "amount": "100.50"
}
```

### Transaction Management

#### `tap.list_transactions`
List transactions with filtering and pagination support.

```json
{
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

## Available Resources

### `tap://agents`
Read-only access to agent information with filtering support.

```
tap://agents                           # All agents
tap://agents?role=Exchange             # Filter by role
tap://agents?for=did:example:party123  # Filter by party
```

### `tap://messages`
Access to transaction messages and history.

```
tap://messages                         # Recent messages
tap://messages?thread_id=abc123        # Filter by thread
tap://messages?type=Transfer           # Filter by message type
tap://messages?limit=100&offset=50     # Pagination
tap://messages/msg-id-123              # Specific message
```

### `tap://schemas`
JSON schemas for TAP message types.

```
tap://schemas                          # All schemas
```

## Configuration

### Environment Variables

- `TAP_ROOT`: Default TAP root directory (default: `~/.tap`)
- `TAP_DB_PATH`: Database file path (default: `$TAP_ROOT/tap-node.db`)
- `RUST_LOG`: Logging level (debug, info, warn, error)

### Directory Structure

```
~/.tap/                    # TAP root directory
├── agents/               # Agent storage
├── keys/                 # Cryptographic keys
├── tap-node.db          # SQLite database
└── config.toml          # Configuration file
```

## Examples

### Creating a Complete Transfer Flow

1. **Create agents for both parties:**

```bash
# Create settlement agent for originator
echo '{"@id": "did:example:alice-settlement", "role": "SettlementAddress", "for": "did:example:alice"}' | \
  tap-mcp-client call tap.create_agent

# Create compliance agent for beneficiary  
echo '{"@id": "did:example:bob-compliance", "role": "Compliance", "for": "did:example:bob"}' | \
  tap-mcp-client call tap.create_agent
```

2. **Initiate transfer:**

```bash
echo '{
  "asset": "eip155:1/erc20:0xa0b86a33e6a4a3c3fcb4b0f0b2a4b6e1c9f8d5c4",
  "amount": "250.00",
  "originator": {"@id": "did:example:alice"},
  "beneficiary": {"@id": "did:example:bob"},
  "agents": [
    {"@id": "did:example:alice-settlement", "role": "SettlementAddress", "for": "did:example:alice"},
    {"@id": "did:example:bob-compliance", "role": "Compliance", "for": "did:example:bob"}
  ]
}' | tap-mcp-client call tap.create_transfer
```

3. **Authorize the transfer:**

```bash
echo '{
  "transaction_id": "tx-abc123",
  "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87",
  "expiry": "2024-12-31T23:59:59Z"
}' | tap-mcp-client call tap.authorize
```

4. **Settle the transaction:**

```bash
echo '{
  "transaction_id": "tx-abc123",
  "settlement_id": "eip155:1:0xabcd1234567890abcdef1234567890abcdef1234",
  "amount": "250.00"
}' | tap-mcp-client call tap.settle
```

5. **Monitor transactions:**

```bash
# List all transactions
tap-mcp-client call tap.list_transactions

# List recent transfers
echo '{"filter": {"message_type": "Transfer"}, "limit": 10}' | \
  tap-mcp-client call tap.list_transactions

# Get specific transaction details via resources
tap-mcp-client resource tap://messages?thread_id=tx-abc123

# List recent messages
tap-mcp-client resource tap://messages
```

### Alternative Workflow: Rejecting a Transaction

If a transaction needs to be rejected instead of authorized:

```bash
# Reject with reason
echo '{
  "transaction_id": "tx-abc123",
  "reason": "Insufficient compliance verification"
}' | tap-mcp-client call tap.reject
```

### Canceling a Transaction

Either party can cancel a transaction before settlement:

```bash
# Cancel transaction
echo '{
  "transaction_id": "tx-abc123",
  "by": "did:example:alice",
  "reason": "Change of plans"
}' | tap-mcp-client call tap.cancel
```

## Integration Examples

### Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "tap": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "/path/to/tap-rs/tap-mcp/Cargo.toml", "--"],
      "env": {
        "TAP_ROOT": "/your/tap/directory"
      }
    }
  }
}
```

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
                "tap.create_agent",
                {
                    "@id": "did:example:test-agent",
                    "role": "Exchange", 
                    "for": "did:example:test-party"
                }
            )
            print(f"Agent created: {agent_result}")
            
            # Create a transfer transaction
            transfer_result = await session.call_tool(
                "tap.create_transfer",
                {
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
                "tap.authorize",
                {
                    "transaction_id": "tx-12345",
                    "settlement_address": "eip155:1:0x742d35cc6bbf4c04623b5daa50a09de81bc4ff87"
                }
            )
            print(f"Transaction authorized: {auth_result}")
            
            # List recent transactions
            list_result = await session.call_tool(
                "tap.list_transactions",
                {
                    "limit": 10,
                    "filter": {"message_type": "Transfer"}
                }
            )
            print(f"Recent transfers: {list_result}")

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
- [Model Context Protocol](https://github.com/anthropics/mcp) - MCP specification and tools
- [Claude Desktop](https://claude.ai) - AI assistant with MCP support

## Support

- GitHub Issues: [Report bugs and request features](https://github.com/notabene-id/tap-rs/issues)
- Documentation: [TAP Protocol Documentation](https://tap.rsvp)
- Community: [TAP Discord Server](https://discord.gg/tap)