# Todos
- [ ] Make sure Authorizable trait implements all required methods
- [ ] Create a new table `transaction_agents` for storing agents and their status for a transaction.
- [ ] Update the contents of the of the transaction message based on messages
- [ ] Implement a simple state machine for transaction processing
- [ ] Store the raw signed or encrypted message in the messages table next to the plain messages


## Future
- [ ] Implement a MCP server as `tap-mcp` which creates a mcp server wrapping the tap-agent.
- [ ] Implement `return_path` in PlainMessage which can be used to open a websocket connection between the sender and receiver for direct communication. The open connections should be managed by the node and message routing should be handled automatically when sending a message to a DID for an open connection
- [ ] Implement [Basic Message](https://didcomm.org/basicmessage/2.0/)
- [ ] Implement [Trust Ping](https://identity.foundation/didcomm-messaging/spec/#trust-ping-protocol-20)
- [ ] Implement [Routing](https://identity.foundation/didcomm-messaging/spec/#routing-protocol-20)
- [ ] Implement [Message Pickup](https://didcomm.org/messagepickup/4.0/)
