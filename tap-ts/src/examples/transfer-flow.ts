import { TAPAgent } from '../agent';
import { DID, Transfer, Participant, CAIP19, Amount, Asset } from '../types';

/**
 * Example demonstrating a complete transfer flow
 */
async function transferFlowExample() {
  // Create originator agent
  const originatorAgent = new TAPAgent({
    nickname: 'Originator',
    debug: true
  });
  
  // Create beneficiary agent
  const beneficiaryAgent = new TAPAgent({
    nickname: 'Beneficiary',
    debug: true
  });
  
  // Log the DIDs
  console.log(`Originator DID: ${originatorAgent.did}`);
  console.log(`Beneficiary DID: ${beneficiaryAgent.did}`);
  
  // Set up message handler for the beneficiary
  beneficiaryAgent.onMessage('Transfer', async (message) => {
    console.log('Beneficiary received transfer:', message);
    
    // Process the message and decide to authorize it
    return beneficiaryAgent.processMessage(message);
  });
  
  // Set up message handler for the originator
  originatorAgent.onMessage('Authorize', async (message) => {
    console.log('Originator received authorization:', message);
    
    // Process the message and create a settlement
    return originatorAgent.processMessage(message);
  });
  
  // Create the initiator and beneficiary
  const initiator = {
    '@id': originatorAgent.did,
    role: 'originator',
    name: 'Alice'
  };
  
  // Create the beneficiary 
  const beneficiary = {
    '@id': beneficiaryAgent.did,
    role: 'beneficiary',
    name: 'Bob'
  };
  
  // Define the asset and amount
  const assetId: CAIP19 = 'eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f';
  const amount: Amount = '100.0';
  
  // Create asset object
  const asset: Asset = {
    id: assetId,
    quantity: amount
  };
  
  // Create a transfer from originator to beneficiary
  const transfer = originatorAgent.transfer({
    asset,
    initiator,
    beneficiary,
    memo: 'Payment for services',
    agents: []
  });
  
  // Send the transfer (in a real implementation, this would use a transport mechanism)
  console.log('Sending transfer...');
  await transfer.send();
  
  // Manually simulate receiving the transfer at the beneficiary
  const transferMessage = transfer.getMessage();
  await beneficiaryAgent.processMessage(transferMessage);
  
  // Beneficiary decides to authorize the transfer
  console.log('Beneficiary authorizing transfer...');
  const authorization = transfer.authorize({
    settlementAddress: 'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e',
    expiry: new Date(Date.now() + 86400000).toISOString() // 24 hours from now
  });
  
  // Send the authorization (in a real implementation, this would use a transport mechanism)
  await authorization.send();
  
  // Manually simulate receiving the authorization at the originator
  const authorizeMessage = authorization.getMessage();
  await originatorAgent.processMessage(authorizeMessage);
  
  // Originator completes the settlement
  console.log('Originator settling transfer...');
  const settlement = authorization.settle({
    settlementId: 'eip155:1/tx/0x123456789abcdef'
  });
  
  // Send the settlement (in a real implementation, this would use a transport mechanism)
  await settlement.send();
  
  console.log('Transfer flow completed successfully');
}

// Run the example if directly executed
if (require.main === module) {
  transferFlowExample().catch(console.error);
}

export { transferFlowExample };