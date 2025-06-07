#!/bin/bash

# Example script showing how to use the new received message debugging tools

# Set the agent DID (replace with your actual agent DID)
AGENT_DID="did:key:z6MkpGuzuD38tpgZKPfmLmmD8R6gihP9KJhuopMuVvfGzLmc"

echo "=== TAP MCP Received Message Debugging Example ==="
echo

# 1. List all received messages for an agent
echo "1. Listing all received messages for agent $AGENT_DID:"
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tap_list_received",
    "arguments": {
      "agent_did": "'$AGENT_DID'",
      "limit": 10
    }
  },
  "id": 1
}'
echo

# 2. Get only pending messages
echo "2. Getting pending messages that need processing:"
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tap_get_pending_received",
    "arguments": {
      "agent_did": "'$AGENT_DID'",
      "limit": 5
    }
  },
  "id": 2
}'
echo

# 3. View a specific raw message
echo "3. Viewing raw content of received message ID 1:"
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tap_view_raw_received",
    "arguments": {
      "agent_did": "'$AGENT_DID'",
      "received_id": 1
    }
  },
  "id": 3
}'
echo

# 4. Filter by source type
echo "4. Listing only HTTPS received messages:"
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tap_list_received",
    "arguments": {
      "agent_did": "'$AGENT_DID'",
      "source_type": "https",
      "limit": 10
    }
  },
  "id": 4
}'
echo

# 5. Filter by failed status
echo "5. Finding failed messages for debugging:"
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "tap_list_received",
    "arguments": {
      "agent_did": "'$AGENT_DID'",
      "status": "failed",
      "limit": 10
    }
  },
  "id": 5
}'
echo

# 6. Using the resource endpoint
echo "6. Accessing received messages via resource:"
echo '{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "tap://received?agent_did='$AGENT_DID'&status=pending&limit=5"
  },
  "id": 6
}'
echo

echo
echo "=== Usage Notes ==="
echo "- All tools require an agent_did parameter"
echo "- The received table stores raw messages before processing"
echo "- Status can be: pending, processed, or failed"
echo "- Source types include: https, internal, websocket, return_path, pickup"
echo "- Use received_id from list results to view specific messages"
echo
echo "To run these examples with tap-mcp:"
echo "cat example.json | cargo run --package tap-mcp"