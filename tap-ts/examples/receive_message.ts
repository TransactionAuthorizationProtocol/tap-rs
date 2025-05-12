/**
 * Example showing how to receive and process TAP messages
 * 
 * This example demonstrates:
 * 1. Receiving a TAP message as a JSON object
 * 2. Processing the message based on its type
 * 3. Creating appropriate responses
 */

import {
  TAPAgent,
  MessageWrapper,
  ensureInitialized,
  TapTypes
} from '../src';

async function main() {
  // Make sure the WASM module is initialized
  await ensureInitialized();
  
  // Create a TAP Agent
  const agent = await TAPAgent.create();
  console.log(`Created TAP Agent with DID: ${agent.getDID()}`);
  
  // Imagine this JSON was received from an HTTP API
  const receivedMessage = {
    "id": "msg_123456789",
    "type": "https://tap.rsvp/schema/1.0#Transfer",
    "from": "did:example:sender123",
    "to": ["did:example:agent123"],
    "created_time": 1713859200,
    "body": {
      "@context": "https://tap.rsvp/schema/1.0",
      "@type": "Transfer",
      "asset": "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "amount": "100.50",
      "originator": {
        "@id": "did:example:originator123",
        "@type": "Party",
        "role": "originator"
      },
      "beneficiary": {
        "@id": "did:example:beneficiary456",
        "@type": "Party",
        "role": "beneficiary"
      },
      "agents": [{
        "@id": "did:example:agent789",
        "@type": "Agent",
        "role": "agent"
      }],
      "memo": "Test transfer",
      "purpose": "CASH"
    }
  };
  
  // Process the received message
  async function processMessage(message: any) {
    // Verify the message signature
    try {
      // Determine message type
      if (message.type === "https://tap.rsvp/schema/1.0#Transfer") {
        // Create message wrapper
        const transfer = new MessageWrapper<TapTypes.Transfer>(
          message.type,
          message.body,
          {
            id: message.id,
            thid: message.thid
          }
        );
        
        // Set additional fields from the received message
        transfer.from = message.from;
        transfer.to = message.to;
        transfer.created_time = message.created_time;
        
        // Set the agent to enable reply methods
        transfer.setAgent(agent);
        
        // Verify the message (in a real app)
        // const isValid = await agent.verify(transfer);
        const isValid = true; // For example purposes
        
        if (isValid) {
          console.log('Received valid transfer message:', transfer.id);
          
          // Process the transfer (example: perform compliance checks)
          const passedCompliance = true; // This would be an actual compliance check
          
          if (passedCompliance) {
            // Create an authorize message
            const authorize = transfer.authorize(
              'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e',
              'Compliance checks passed',
              3600
            );
            
            // Sign the authorize message
            const signedAuthorize = await agent.sign(authorize);
            
            // In a real app, you would send this message to the recipient
            console.log('Created and signed authorize response:', signedAuthorize.id);
            return signedAuthorize;
          } else {
            // Create a reject message
            const reject = transfer.reject('Compliance checks failed');
            
            // Sign the reject message
            const signedReject = await agent.sign(reject);
            
            // In a real app, you would send this message to the recipient
            console.log('Created and signed reject response:', signedReject.id);
            return signedReject;
          }
        } else {
          console.error('Invalid signature on transfer message');
        }
      } else if (message.type === "https://tap.rsvp/schema/1.0#PaymentRequest") {
        // Handle payment request
        console.log('Received payment request');
        // Similar processing logic...
      } else {
        console.log('Unsupported message type:', message.type);
      }
    } catch (error) {
      console.error('Error processing message:', error);
    }
  }
  
  // Process the example message
  const response = await processMessage(receivedMessage);
  console.log('Response ready to be sent to recipient', response);
}

// Run the example
main().catch(error => {
  console.error('Error in example:', error);
});