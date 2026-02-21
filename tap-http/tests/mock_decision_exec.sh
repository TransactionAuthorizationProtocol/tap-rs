#!/bin/bash
# Mock external decision executable for integration testing.
# Reads JSON-RPC messages from stdin and auto-approves decisions.

while IFS= read -r line; do
    # Skip empty lines
    [ -z "$line" ] && continue

    # Parse the method field
    method=$(echo "$line" | grep -o '"method":"[^"]*"' | head -1 | cut -d'"' -f4)
    id=$(echo "$line" | grep -o '"id":[0-9]*' | head -1 | cut -d':' -f2)

    case "$method" in
        "tap/initialize")
            # Respond with ready
            echo '{"jsonrpc":"2.0","method":"tap/ready","params":{"name":"mock-auto-approve","version":"1.0.0"}}'
            ;;
        "tap/decision")
            # Auto-approve: respond with authorize action
            if [ -n "$id" ]; then
                echo "{\"jsonrpc\":\"2.0\",\"id\":$id,\"result\":{\"action\":\"authorize\",\"detail\":{\"reason\":\"auto-approved by mock\"}}}"
            fi
            ;;
        "tap/event")
            # Events are notifications, no response needed
            ;;
    esac
done
