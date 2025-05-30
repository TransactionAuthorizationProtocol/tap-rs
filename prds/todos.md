# Todos
- [X] Make sure Authorizable trait implements only the required messages from TAIP-4 except for settle.
- [X] Create a Transaction trait containing all required functions for transaction from TAIP-5, TAIP-6, TAIP-7, TAIP-8, TAIP-9 processing besides Authorizable, including agent handling, policies, settling, and managing parties.
- Message review
- - agent-management.rs remove boiler plate. Why rename transaction id to transfer_id
remove TapMessageObject boilerplate
- - Connect, UpdateParty and UpdatePolicies should remove TapMessageObject boilerplate
- - There is both a did_presentation.rs and presentation.rs



# Changes to tap-node
- [ ] Create new NodeEvent types:
- - [ ] RejectedMessage to be sent when a message is rejected - Create handler that updates the status of the message in database to Rejected.
- - [ ] AcceptedMessage to be sent when a message is accepted - Create handler that updates the status of the message in database to Accepted.
- - [ ] ReplyReceived to be sent when a reply is received. It should include the original message and the reply message
- [ ] Create as series of validation checks for Rejecting transactions.
- - [ ] Any message that is a response to a transaction should only be accepted and processed if the sender is one of the agents in the transaction_agents table
- - [ ] The id of the message should be unique
- - [ ] The timestamp of the message should not be more than 1 minute in the future and the message should not have expired
- [ ] Always make sure that agents within the same node send messages directly to each other through the node
- [ ] Create a new table `transaction_agents` for storing agents and their status for a transaction.
- [ ] Update the contents of the of the transaction message based on accepted messages
- - [ ] Update agents based on @taip-5 messages
- - [ ] Update policies based on @taip-7 messages
- [ ] Implement a simple state machine for processing accepted messages
- - [ ] If incoming message it always Authorizes a transaction_agents
- - [ ] Update status of agent sending 'Authorize', 'Cancel', 'Reject'
- - [ ] If all agents have Authorized update status of transaction to Authorized
- - [ ] If status is Authorized and we are the sending agent, send a Settle message and update status of transaction to Settled
- - [ ] If Settle message is received, update status of transaction to Settled
- - [ ] If cancel or reject messages are received from anyone update status of transaction to Cancelled or Rejected
- [ ] Store the raw signed or encrypted message in the messages table next to the plain messages


## Future
- [ ] Implement a MCP server as `tap-mcp` which creates a mcp server wrapping the tap-agent.
- [ ] Implement `return_path` in PlainMessage which can be used to open a websocket connection between the sender and receiver for direct communication. The open connections should be managed by the node and message routing should be handled automatically when sending a message to a DID for an open connection
- [ ] Implement [Basic Message](https://didcomm.org/basicmessage/2.0/)
- [ ] Implement [Trust Ping](https://identity.foundation/didcomm-messaging/spec/#trust-ping-protocol-20)
- [ ] Implement [Routing](https://identity.foundation/didcomm-messaging/spec/#routing-protocol-20)
- [ ] Implement [Message Pickup](https://didcomm.org/messagepickup/4.0/)
