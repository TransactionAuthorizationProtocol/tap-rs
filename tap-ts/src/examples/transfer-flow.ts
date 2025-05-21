/**
 * TAP Transfer Flow Example
 * 
 * This example demonstrates how to use the TAP Agent to create, pack, and unpack
 * transfer messages in a complete flow.
 */

import { TAPAgent, TransferMessage } from '../index';

/**
 * Main function
 */
async function main() {
  try {
    console.log('Initializing TAP Agents...');
    
    // Create originator agent
    const originatorAgent = await TAPAgent.create({
      nickname: 'Originator Agent',
      debug: true
    });
    
    console.log(`Created originator agent with DID: ${originatorAgent.did}`);
    
    // Create beneficiary agent
    const beneficiaryAgent = await TAPAgent.create({
      nickname: 'Beneficiary Agent',
      debug: true
    });
    
    console.log(`Created beneficiary agent with DID: ${beneficiaryAgent.did}`);
    
    // Create a transfer message from the originator
    console.log('\nCreating transfer message...');
    
    const transfer = originatorAgent.transfer({
      asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
      amount: '100.0',
      originator: {
        '@id': originatorAgent.did,
        role: 'originator',
        name: 'Originator Inc.'
      },
      beneficiary: {
        '@id': beneficiaryAgent.did,
        role: 'beneficiary',
        name: 'Beneficiary Corp.'
      },
      agents: [],
      memo: 'Test transfer'
    });
    
    console.log('Created transfer message with ID:', transfer.id);
    console.log('Message content:', JSON.stringify(transfer.toJSON(), null, 2));
    
    // Pack the message for sending
    console.log('\nPacking transfer message...');
    
    const packedResult = await transfer.pack();
    console.log('Message packed successfully!');
    
    // Just show the first 100 characters of the packed message 
    // (it can be quite long with all the signatures)
    console.log(`Packed message (first 100 chars): ${packedResult.message.substring(0, 100)}...`);
    
    // In a real app, you would send this packed message to the beneficiary
    // Here we'll just unpack it directly
    console.log('\nUnpacking transfer message...');
    
    // Unpack the message as the beneficiary
    const unpackedMessage = await beneficiaryAgent.unpackMessage(packedResult.message);
    console.log('Message unpacked successfully!');
    console.log('Unpacked message:', JSON.stringify(unpackedMessage, null, 2));
    
    // Create a response (authorize message)
    console.log('\nCreating authorize response...');
    
    const authorizeResponse = beneficiaryAgent.authorize({
      reason: 'Transfer approved',
      settlementAddress: '0x123456789abcdef0123456789abcdef012345678',
      expiry: new Date(Date.now() + 3600000).toISOString() // 1 hour from now
    });
    
    // Link the response to the original message through the thread ID
    authorizeResponse.setThreadId(unpackedMessage.id);
    
    // Set the recipient to the originator
    if (unpackedMessage.from) {
      authorizeResponse.setTo(unpackedMessage.from);
    }
    
    console.log('Created authorize response with ID:', authorizeResponse.id);
    console.log('Response content:', JSON.stringify(authorizeResponse.toJSON(), null, 2));
    
    // Pack the response for sending back
    console.log('\nPacking authorize response...');
    
    const packedResponse = await authorizeResponse.pack();
    console.log('Response packed successfully!');
    
    // In a real app, you would send this packed response to the originator
    // Here we'll just unpack it directly
    console.log('\nUnpacking authorize response...');
    
    // Unpack the response as the originator
    const unpackedResponse = await originatorAgent.unpackMessage(packedResponse.message);
    console.log('Response unpacked successfully!');
    console.log('Unpacked response:', JSON.stringify(unpackedResponse, null, 2));
    
    // Complete the flow with a settle message
    console.log('\nCreating settle message...');
    
    const settleMessage = originatorAgent.settle({
      settlementId: '0x123456789abcdef0123456789abcdef012345678',
      amount: '100.0'
    });
    
    // Link the settle message to the original thread
    settleMessage.setThreadId(transfer.id);
    
    // Set the recipient to the beneficiary
    settleMessage.setTo(beneficiaryAgent.did);
    
    console.log('Created settle message with ID:', settleMessage.id);
    console.log('Settle message content:', JSON.stringify(settleMessage.toJSON(), null, 2));
    
    // Pack the settle message for sending
    console.log('\nPacking settle message...');
    
    const packedSettle = await settleMessage.pack();
    console.log('Settle message packed successfully!');
    
    // In a real app, you would send this packed message to the beneficiary
    // Here we'll just unpack it directly
    console.log('\nUnpacking settle message...');
    
    // Unpack the settle message as the beneficiary
    const unpackedSettle = await beneficiaryAgent.unpackMessage(packedSettle.message);
    console.log('Settle message unpacked successfully!');
    console.log('Unpacked settle message:', JSON.stringify(unpackedSettle, null, 2));
    
    console.log('\nTransfer flow completed successfully!');
  } catch (error) {
    console.error('Error in transfer flow:', error);
    process.exit(1);
  }
}

// Run the main function
main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error('Unhandled error:', error);
    process.exit(1);
  });