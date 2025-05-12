/**
 * Example showing how to use the TAP Agent with native TAP types
 * 
 * This example demonstrates:
 * 1. Creating a TAP Agent
 * 2. Creating a Transfer message using the TapTypes directly
 * 3. Signing and sending the message
 * 4. Creating reply messages (Authorize, Reject, etc.)
 */

import {
  TAPAgent,
  TransferWrapper,
  PaymentRequestWrapper,
  MessageWrapper,
  ensureInitialized
} from '../src';

// Import the TAP types directly from the standard package
import {
  Transfer,
  Participant,
  CAIP19,
  Amount,
  DID,
  Authorize, 
  Reject,
  Settle,
  Cancel
} from '@taprsvp/types';

async function main() {
  // Make sure the WASM module is initialized
  await ensureInitialized();
  
  // Create a TAP Agent
  const agent = await TAPAgent.create();
  console.log(`Created TAP Agent with DID: ${agent.getDID()}`);
  
  // Create a sample transfer
  const transfer = agent.transfer({
    asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC on Ethereum
    amount: '100.50',
    originator: {
      '@id': 'did:example:originator123',
      '@type': 'Party',
      role: 'originator'
    },
    beneficiary: {
      '@id': 'did:example:beneficiary456',
      '@type': 'Party',
      role: 'beneficiary'
    },
    agents: [{
      '@id': 'did:example:agent789',
      '@type': 'Agent',
      role: 'agent'
    }],
    memo: 'Test transfer',
    purpose: 'CASH',
    messageOptions: {
      expiresInSeconds: 3600 // 1 hour
    }
  });
  
  console.log('Created transfer:', transfer);
  console.log('Transfer body:', transfer.body);
  
  // Sign the transfer
  const signedTransfer = await agent.sign(transfer);
  console.log('Signed transfer ID:', signedTransfer.id);
  
  // Create an authorize message in response to the transfer
  const authorize = transfer.authorize(
    'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e', // Settlement address
    'Compliance checks passed',
    3600 // Expires in 1 hour
  );
  
  console.log('Created authorize message:', authorize);
  
  // Sign the authorize message
  const signedAuthorize = await agent.sign(authorize);
  console.log('Signed authorize ID:', signedAuthorize.id);
  
  // Alternatively, create a rejection
  const reject = transfer.reject('Compliance failure');
  console.log('Created reject message:', reject);
  
  // Create a settlement message
  const settle = transfer.settle('eip155:1/tx/0x4a563af33c4871b51a8b108aa2fe1dd5280a30dfb7236170ae5e5e7957eb6392');
  console.log('Created settle message:', settle);
  
  // Create a payment request
  const payment = agent.paymentRequest({
    amount: '50.75',
    merchant: {
      '@id': 'did:example:merchant123',
      '@type': 'Party',
      role: 'merchant'
    },
    agents: [{
      '@id': 'did:example:agent789',
      '@type': 'Agent',
      role: 'agent'
    }],
    asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC on Ethereum
    customer: {
      '@id': 'did:example:customer456',
      '@type': 'Party',
      role: 'customer'
    }
  });
  
  console.log('Created payment request:', payment);
  
  // Create a complete message for the payment
  const complete = payment.complete(
    'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e' // Settlement address
  );
  
  console.log('Created complete message:', complete);
}

// Run the example
main().catch(error => {
  console.error('Error in example:', error);
});