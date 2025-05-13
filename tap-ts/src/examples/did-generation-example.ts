/**
 * Example of DID generation using TAP-TS
 * 
 * This example demonstrates how to:
 * 1. Generate DIDs with different key types
 * 2. Generate web DIDs for domains
 * 3. Access DID document information
 * 4. Save DID documents to files
 * 5. Create an agent with an automatically generated DID
 */

import { TAPAgent, DIDKeyType, createDIDKey, createDIDWeb } from '../index';
import * as fs from 'fs';

async function main() {
  console.log('TAP-TS DID Generation Example');
  console.log('-----------------------------');

  // PART 1: Automatic DID Generation
  console.log('\nPART 1: Automatic DID Generation');
  console.log('--------------------------------');
  
  // Create a new agent - it will automatically generate an Ed25519 DID key
  console.log('\nCreating agent with automatically generated DID...');
  const autoAgent = new TAPAgent({
    nickname: "Auto DID Agent",
    debug: true
  });

  // Give time for initialization to complete
  await new Promise(resolve => setTimeout(resolve, 500));
  
  console.log(`Agent created with auto-generated DID: ${autoAgent.did}`);
  console.log('This DID was automatically created when the agent was initialized.');
  console.log('By default, an Ed25519 did:key is used, but this can be customized.');
  
  // Get key manager info
  const keyManagerInfo = autoAgent.getKeyManagerInfo();
  console.log('Key manager info:', JSON.stringify(keyManagerInfo, null, 2));

  // PART 2: Manual DID Generation
  console.log('\nPART 2: Manual DID Generation');
  console.log('----------------------------');
  
  // Create a new agent
  const agent = new TAPAgent({
    nickname: "DID Generator Agent",
    debug: true
  });

  console.log(`\nAgent created with DID: ${agent.did}`);

  // 1. Generate DIDs with different key types
  console.log('\n1. Generating DIDs with different key types...');
  
  console.log('\n1.1 Generating Ed25519 key DID...');
  const ed25519Did = await agent.generateDID(DIDKeyType.Ed25519);
  console.log(`DID: ${ed25519Did.did}`);
  console.log(`Key Type: ${ed25519Did.getKeyType()}`);
  console.log(`Public Key (hex): ${ed25519Did.getPublicKeyHex()}`);
  console.log(`Private Key (hex): ${ed25519Did.getPrivateKeyHex()}`);
  
  console.log('\n1.2 Generating P-256 key DID...');
  const p256Did = await agent.generateDID(DIDKeyType.P256);
  console.log(`DID: ${p256Did.did}`);
  console.log(`Key Type: ${p256Did.getKeyType()}`);
  console.log(`Public Key (hex): ${p256Did.getPublicKeyHex()}`);
  
  console.log('\n1.3 Generating Secp256k1 key DID...');
  const secp256k1Did = await agent.generateDID(DIDKeyType.Secp256k1);
  console.log(`DID: ${secp256k1Did.did}`);
  console.log(`Key Type: ${secp256k1Did.getKeyType()}`);
  console.log(`Public Key (hex): ${secp256k1Did.getPublicKeyHex()}`);

  // 2. Generate web DIDs for domains
  console.log('\n2. Generating web DIDs for domains...');
  
  console.log('\n2.1 Generating web DID for example.com with Ed25519 key...');
  const webDid = await agent.generateWebDID('example.com', DIDKeyType.Ed25519);
  console.log(`DID: ${webDid.did}`);
  console.log(`Key Type: ${webDid.getKeyType()}`);
  console.log(`Public Key (hex): ${webDid.getPublicKeyHex()}`);

  // 3. List all DIDs managed by the agent
  console.log('\n3. Listing all DIDs managed by the agent...');
  const dids = await agent.listDIDs();
  console.log(`Managed DIDs: ${dids.join(', ')}`);

  // 4. Get keys information
  console.log('\n4. Getting key information...');
  const keysInfo = agent.getKeysInfo();
  console.log('Keys info:', JSON.stringify(keysInfo, null, 2));

  // 5. Save DID documents to files
  console.log('\n5. Saving DID documents to files...');
  
  // Create output directory if it doesn't exist
  const outputDir = './did-examples';
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir);
  }
  
  // Save Ed25519 DID document
  fs.writeFileSync(
    `${outputDir}/ed25519-did.json`, 
    ed25519Did.didDocument
  );
  console.log(`Ed25519 DID document saved to ${outputDir}/ed25519-did.json`);
  
  // Save P-256 DID document
  fs.writeFileSync(
    `${outputDir}/p256-did.json`, 
    p256Did.didDocument
  );
  console.log(`P-256 DID document saved to ${outputDir}/p256-did.json`);
  
  // Save Secp256k1 DID document
  fs.writeFileSync(
    `${outputDir}/secp256k1-did.json`, 
    secp256k1Did.didDocument
  );
  console.log(`Secp256k1 DID document saved to ${outputDir}/secp256k1-did.json`);
  
  // Save web DID document
  fs.writeFileSync(
    `${outputDir}/web-did.json`, 
    webDid.didDocument
  );
  console.log(`Web DID document saved to ${outputDir}/web-did.json`);

  // 6. Use standalone functions
  console.log('\n6. Using standalone functions...');
  
  console.log('\n6.1 Creating DID using createDIDKey...');
  const standaloneDid = await createDIDKey(DIDKeyType.Ed25519);
  console.log(`DID: ${standaloneDid.did}`);
  console.log(`Key Type: ${standaloneDid.getKeyType()}`);
  
  console.log('\n6.2 Creating web DID using createDIDWeb...');
  const standaloneWebDid = await createDIDWeb('another-example.com', DIDKeyType.P256);
  console.log(`DID: ${standaloneWebDid.did}`);
  console.log(`Key Type: ${standaloneWebDid.getKeyType()}`);
  
  // 7. Exploring Key Agreement
  console.log('\n7. Exploring Key Agreement Information');
  
  const ed25519DidDoc = JSON.parse(ed25519Did.didDocument);
  if (ed25519DidDoc.keyAgreement && ed25519DidDoc.keyAgreement.length > 0) {
    console.log('\nEd25519 key agreement verification method:');
    console.log(`ID: ${ed25519DidDoc.keyAgreement[0]}`);
    
    // Find the referenced verification method
    const keyAgreementId = ed25519DidDoc.keyAgreement[0];
    const keyAgreementMethod = ed25519DidDoc.verificationMethod.find(
      (vm: any) => vm.id === keyAgreementId
    );
    
    if (keyAgreementMethod) {
      console.log('Key agreement method:');
      console.log(JSON.stringify(keyAgreementMethod, null, 2));
      console.log('\nFor Ed25519 keys, an X25519 key is automatically derived for key agreement (encryption).');
    }
  }
  
  console.log('\nExample completed successfully!');
}

// Run the example
main().catch(error => {
  console.error('Error in example:', error);
});