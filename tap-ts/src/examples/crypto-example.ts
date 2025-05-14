import { TAPAgent } from '../agent';
import { DIDKeyType, createDIDKey } from '../wasm-loader';

/**
 * Example demonstrating the cryptographic functionality
 */
async function cryptoExample() {
  console.log('TAP Cryptographic Example');
  console.log('------------------------');
  
  // Create a DID key with real cryptographic functionality
  console.log('Creating a DID key with Ed25519...');
  const didKey = await createDIDKey(DIDKeyType.Ed25519);
  console.log(`Generated DID: ${didKey.did}`);
  console.log(`Key type: ${didKey.getKeyType()}`);
  console.log(`Public key (hex): ${didKey.getPublicKeyHex()}`);
  
  // Create an agent with the key
  const agent = new TAPAgent({
    did: didKey.did,
    nickname: 'Crypto Agent',
    debug: true
  });
  
  // Generate a simple message
  const message = agent.transfer({
    asset: {
      id: 'eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f',
      quantity: '100.0'
    },
    initiator: {
      '@id': agent.did,
      role: 'originator',
      name: 'Crypto Test'
    },
    memo: 'Testing cryptographic signing',
    agents: []
  });
  
  // Sign the message
  console.log('\nSigning a message...');
  const signedMessage = await agent.signMessage(message.getMessage());
  console.log('Message signed successfully');
  
  // Verify the message
  console.log('\nVerifying the signed message...');
  const verified = await agent.verifyMessage(signedMessage);
  console.log(`Message verification result: ${verified ? 'Success' : 'Failed'}`);
  
  // Demonstrate direct key signing
  console.log('\nDirect key signing example:');
  const dataToSign = 'Hello, TAP cryptography!';
  console.log(`Data to sign: "${dataToSign}"`);
  
  // Sign with the key directly
  const signature = didKey.signData(dataToSign);
  console.log(`Signature: ${signature}`);
  
  // Verify with the key directly
  const directVerification = didKey.verifySignature(dataToSign, signature);
  console.log(`Direct verification result: ${directVerification ? 'Success' : 'Failed'}`);
  
  console.log('\nCryptographic functionality test completed');
}

// Run the example if directly executed
if (require.main === module) {
  cryptoExample().catch(console.error);
}

export { cryptoExample };