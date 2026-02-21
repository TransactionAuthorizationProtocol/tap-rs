# tap-mcp Crate

Model Context Protocol (MCP) server implementation for TAP Node functionality, enabling AI assistants to interact with the TAP ecosystem.

## Purpose

The `tap-mcp` crate provides:
- MCP server for TAP Node operations
- AI-friendly tools for transaction management
- Agent and customer management interfaces
- Database querying capabilities
- Schema and resource access
- Integration with Claude Code and other AI assistants

## Key Components

- `mcp/` - MCP protocol implementation
  - `server.rs` - MCP server with stdio transport
  - `protocol.rs` - MCP message types and handlers
  - `transport.rs` - Transport layer abstraction
- `tools/` - MCP tool implementations
  - `agent_tools.rs` - Agent creation and management
  - `transaction_tools.rs` - Transaction operations
  - `customer_tools.rs` - Customer data management
  - `database_tools.rs` - Direct database access
  - `delivery_tools.rs` - Message delivery tracking
- `resources.rs` - MCP resource providers
- `tap_integration.rs` - TAP ecosystem integration

## Build Commands

```bash
# Build the crate
cargo build -p tap-mcp

# Run tests
cargo test -p tap-mcp

# Run specific test
cargo test -p tap-mcp test_name

# Run the MCP server
cargo run -p tap-mcp

# Run the MCP server with specific config
cargo run -p tap-mcp -- --config path/to/config.json

# Build release version
cargo build -p tap-mcp --release
```

## Development Guidelines

### MCP Tool Implementation
- Each tool should be atomic and focused on a single operation
- Include comprehensive input validation
- Provide clear error messages for AI assistants
- Return structured data that's easy to parse
- Support both individual and bulk operations

### Resource Management
- Implement resources for static data access (schemas, documentation)
- Support dynamic resources for agent and transaction data
- Include proper caching where appropriate
- Handle resource updates and notifications

### Database Integration
- Use prepared statements for security
- Support read-only queries for safety
- Include proper error handling and logging
- Provide both high-level and low-level access patterns

### AI Assistant Integration
- Design tools with natural language interfaces in mind
- Include helpful descriptions and examples
- Support common workflows and use cases
- Provide clear feedback and status information

## MCP Server Features

### Agent Management Tools
- `tap_create_agent` - Create new TAP agents
- `tap_list_agents` - List all configured agents
- Agent configuration and key management
- DID generation and management

### Transaction Tools
- `tap_create_transfer` - Initiate transfer transactions
- `tap_payment` - Create payment requests
- `tap_authorize` - Authorize transactions
- `tap_settle` - Settle transactions
- `tap_cancel` - Cancel transactions
- `tap_list_transactions` - Query transaction history

### Customer Management
- `tap_create_customer` - Create customer profiles
- `tap_update_customer_profile` - Update customer data
- `tap_generate_ivms101` - Generate IVMS101 data
- `tap_list_customers` - List customer records

### Communication Tools
- `tap_basic_message` - Send basic text messages
- `tap_trust_ping` - Test connectivity between agents
- Message delivery tracking and status

### Database Tools
- `tap_query_database` - Execute read-only SQL queries
- `tap_get_database_schema` - Retrieve database schema
- Direct access to TAP Node storage

## Resource Providers

### Schema Access
- TAP message schemas
- Database table schemas
- CAIP identifier formats
- API documentation

### Documentation Resources
- TAIP specifications
- Implementation guides
- Best practices
- Example workflows

## Configuration

The MCP server supports various configuration options:
- Database connection strings
- Agent storage paths
- Logging levels and formats
- Resource access permissions

## Examples

The crate includes practical examples:
- `pii_hashing_demo.rs` - Customer data hashing
- Integration with Claude Code workflows
- Multi-agent transaction scenarios

Run examples with:
```bash
cargo run --example pii_hashing_demo -p tap-mcp
```

## AI Assistant Integration

### Claude Code Integration
The MCP server is designed to work seamlessly with Claude Code:
- Provides tools for TAP development workflows
- Supports code generation and testing
- Enables transaction debugging and monitoring
- Facilitates compliance reporting

### Natural Language Interface
Tools are designed to be AI-friendly:
- Clear parameter descriptions
- Helpful error messages
- Structured output formats
- Common workflow support

## Security Features

### Read-Only Database Access
- SQL queries are restricted to SELECT statements
- No data modification through MCP tools
- Proper query validation and sanitization
- Transaction isolation for data integrity

### Agent Isolation
- Each agent maintains separate data
- Proper authorization checks
- Secure key management
- Audit logging for all operations

## Testing

Comprehensive test coverage including:
- Unit tests for individual tools
- Integration tests with TAP Node
- MCP protocol compliance tests
- End-to-end workflow tests
- Performance and reliability tests

Run the full test suite:
```bash
cargo test -p tap-mcp
```

## Deployment

The MCP server can be deployed as:
- Standalone binary for development
- System service for production
- Docker container
- Part of larger TAP Node deployment

Build for deployment:
```bash
cargo build -p tap-mcp --release
```

## Usage with AI Assistants

To use with Claude Code or other MCP-compatible AI assistants:

1. Build and run the MCP server:
   ```bash
   cargo run -p tap-mcp
   ```

2. Configure your AI assistant to connect to the MCP server via stdio

3. Use natural language to interact with TAP functionality:
   - "Create a new TAP agent"
   - "List all transactions for agent X"
   - "Generate IVMS101 data for customer Y"

The MCP server translates natural language requests into proper TAP operations and returns structured results.

## Related Tools

- [tap-cli](../tap-cli/README.md) — Terminal CLI alternative; use when scripting or working outside an AI assistant
- [tap-http](../tap-http/README.md) — HTTP server for DIDComm message transport