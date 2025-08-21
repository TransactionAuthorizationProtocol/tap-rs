/**
 * Basic Transfer Example
 * 
 * This example demonstrates a simple transfer between two agents
 */

import { TapAgent } from '@taprsvp/agent';

async function main() {
  console.log('TAP Basic Transfer Example\n');
  
  // Create Alice's agent
  console.log('Creating Alice\'s agent...');
  const alice = await TapAgent.create({ keyType: 'Ed25519' });
  console.log('Alice DID:', alice.did);
  
  // Create Bob's agent
  console.log('\nCreating Bob\'s agent...');
  const bob = await TapAgent.create({ keyType: 'Ed25519' });
  console.log('Bob DID:', bob.did);
  
  // Alice creates a transfer to Bob
  console.log('\n--- Step 1: Alice creates transfer ---');
  const transfer = await alice.createMessage('Transfer', {
    amount: '1000.00',
    asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48', // USDC
    originator: {
      '@id': alice.did,
      '@type': 'https://schema.org/Person',
      name: 'Alice Smith',
      email: 'alice@example.com',
      customerIdentification: 'ACC-001',
      countryOfResidence: 'US'
    },
    beneficiary: {
      '@id': bob.did,
      '@type': 'https://schema.org/Person',
      name: 'Bob Jones',
      email: 'bob@example.com',
      customerIdentification: 'ACC-002',
      countryOfResidence: 'US'
    },
    memo: 'Invoice #12345 - Consulting services',
    agents: []  // Could include settlement agents, compliance officers, etc.
  });
  
  // Set recipient
  transfer.to = [bob.did];
  
  // Pack the message
  const packedTransfer = await alice.pack(transfer);
  console.log('Transfer packed, size:', packedTransfer.message.length, 'bytes');
  console.log('Transfer ID:', transfer.id);
  
  // Bob receives and unpacks the transfer
  console.log('\n--- Step 2: Bob receives transfer ---');
  const receivedTransfer = await bob.unpack(packedTransfer.message);
  console.log('Transfer received from:', receivedTransfer.from);
  console.log('Amount:', receivedTransfer.body.amount);
  console.log('Asset:', receivedTransfer.body.asset);
  console.log('Memo:', receivedTransfer.body.memo);
  
  // Bob authorizes the transfer
  console.log('\n--- Step 3: Bob authorizes transfer ---');
  const authorize = await bob.createMessage('Authorize', {
    transaction_id: receivedTransfer.id,
    settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
    expiry: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString()
  }, {
    thid: receivedTransfer.id, // Thread ID links to original transfer
    to: [alice.did]
  });
  
  const packedAuth = await bob.pack(authorize);
  console.log('Authorization sent, thread ID:', authorize.thid);
  
  // Alice receives the authorization
  console.log('\n--- Step 4: Alice receives authorization ---');
  const receivedAuth = await alice.unpack(packedAuth.message);
  console.log('Authorization received for transaction:', receivedAuth.body.transaction_id);
  console.log('Settlement address:', receivedAuth.body.settlement_address);
  
  // Alice confirms settlement
  console.log('\n--- Step 5: Alice confirms settlement ---');
  const settle = await alice.createMessage('Settle', {
    transaction_id: receivedTransfer.id,
    settlement_id: `eip155:1:0x${Math.random().toString(16).slice(2, 42)}`,
    amount: '1000.00'
  }, {
    thid: receivedTransfer.id,
    to: [bob.did]
  });
  
  const packedSettle = await alice.pack(settle);
  console.log('Settlement sent, ID:', settle.body.settlement_id);
  
  // Bob receives settlement confirmation
  console.log('\n--- Step 6: Bob receives settlement ---');
  const receivedSettle = await bob.unpack(packedSettle.message);
  console.log('Settlement confirmed!');
  console.log('Settlement ID:', receivedSettle.body.settlement_id);
  console.log('Amount settled:', receivedSettle.body.amount);
  
  console.log('\n✅ Transfer completed successfully!');
  
  // Export keys for future use
  console.log('\n--- Key Export (for future sessions) ---');
  console.log('Alice private key:', alice.exportPrivateKey());
  console.log('Bob private key:', bob.exportPrivateKey());
  
  // Clean up
  alice.dispose();
  bob.dispose();
}

// Run the example
main().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});