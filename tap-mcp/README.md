# TAP-MCP: Model Context Protocol Server for TAP

A Model Context Protocol (MCP) server that provides AI applications with standardized access to Transaction Authorization Protocol (TAP) functionality. This enables LLMs and AI agents to create, manage, and monitor TAP transactions through a well-defined interface.

## Overview

TAP-MCP is a thin wrapper around TAP Node that exposes transaction authorization functionality through the Model Context Protocol standard. This enables AI applications to:

- **Agent Management**: Create TAP agents with auto-generated DIDs and manage cryptographic identities
- **Transaction Creation**: Initiate transfers, payments, and other TAP operations  
- **Agent-Specific Message Monitoring**: Access transaction history and message details from individual agent storage
- **Multi-Recipient Message Delivery**: Full DIDComm compliance with delivery to all recipients in the `to` field
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

TAP-MCP provides 8 comprehensive tools covering the complete TAP transaction lifecycle:

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

## Available Resources

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

**Important**: The `agent_did` parameter is required when accessing messages to specify which agent's storage to query. If not provided, you'll get an error message asking you to specify the agent. By default, both incoming and outgoing messages are shown unless you filter by `direction`.

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

# Get specific transaction details via resources (requires agent_did)
tap-mcp-client resource "tap://messages?agent_did=did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc&thread_id=tx-abc123"

# List recent messages for an agent (shows both incoming and outgoing by default)
tap-mcp-client resource "tap://messages?agent_did=did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"

# List only incoming messages for an agent
tap-mcp-client resource "tap://messages?agent_did=did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc&direction=incoming"
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
