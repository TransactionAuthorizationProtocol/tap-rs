/**
 * Example demonstrating how to use DID resolvers with the TAP Agent
 */
import { TAPAgent, StandardDIDResolver, ResolverOptions } from '../index';

async function main() {
  console.log('TAP Agent with DID Resolver Example');
  
  // Example 1: Using default resolver configuration
  console.log('\nExample 1: Default DID resolver configuration');
  const agent1 = await TAPAgent.create({
    nickname: 'Agent with Default Resolver'
  });
  console.log(`Agent 1 DID: ${agent1.did}`);
  
  // Example 2: Customizing resolver options
  console.log('\nExample 2: Custom DID resolver configuration');
  
  // Custom resolver options to only enable key and ethr resolvers
  const resolverOptions: ResolverOptions = {
    resolvers: {
      key: true,
      ethr: true,
      pkh: false,
      web: false
    },
    ethrOptions: {
      networks: [
        {
          name: 'mainnet',
          rpcUrl: 'https://mainnet.infura.io/v3/YOUR_INFURA_KEY'
        },
        {
          name: 'sepolia',
          rpcUrl: 'https://sepolia.infura.io/v3/YOUR_INFURA_KEY'
        }
      ]
    }
  };
  
  const agent2 = await TAPAgent.create({
    nickname: 'Agent with Custom Resolver',
    resolverOptions
  });
  console.log(`Agent 2 DID: ${agent2.did}`);
  
  // Example 3: Creating a custom resolver and passing it directly
  console.log('\nExample 3: Creating and passing a custom resolver');
  
  // Create a custom resolver with specific options
  const customResolver = new StandardDIDResolver({
    resolvers: {
      key: true,
      ethr: true,
      pkh: true,
      web: false
    },
    ethrOptions: {
      networks: [
        {
          name: 'goerli',
          rpcUrl: 'https://goerli.infura.io/v3/YOUR_INFURA_KEY'
        }
      ]
    }
  });
  
  const agent3 = await TAPAgent.create({
    nickname: 'Agent with Direct Resolver',
    didResolver: customResolver
  });
  console.log(`Agent 3 DID: ${agent3.did}`);
  
  // Example 4: Implementing a custom DID resolver
  console.log('\nExample 4: Implementing a custom DID resolver');
  
  class MyCustomResolver {
    // This method implements the required DID resolver interface
    async resolve(did: string): Promise<any> {
      console.log(`Resolving DID with custom resolver: ${did}`);
      
      // For did:key, we can use a mock implementation
      if (did.startsWith('did:key:')) {
        return {
          id: did,
          verificationMethod: [
            {
              id: `${did}#key-1`,
              type: 'Ed25519VerificationKey2020',
              controller: did,
              publicKeyMultibase: did.split(':')[2]
            }
          ]
        };
      }
      
      // For other DID methods, you could delegate to other resolvers
      // or implement custom logic
      return {
        id: did,
        '@context': 'https://www.w3.org/ns/did/v1'
      };
    }
  }
  
  const agent4 = await TAPAgent.create({
    nickname: 'Agent with Custom Resolver Implementation',
    didResolver: new MyCustomResolver()
  });
  console.log(`Agent 4 DID: ${agent4.did}`);
  
  // Example 5: Using the resolver to resolve DIDs
  console.log('\nExample 5: Resolving DIDs with the resolver');
  
  // Create a standalone resolver for demonstration
  const resolver = new StandardDIDResolver();
  
  try {
    // Resolve a did:key
    const keyDid = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
    console.log(`Resolving: ${keyDid}`);
    const keyDoc = await resolver.resolve(keyDid);
    console.log('DID Document:', JSON.stringify(keyDoc, null, 2).substring(0, 100) + '...');
    
    // Resolve a did:ethr (this would require a running Ethereum node or provider)
    const ethrDid = 'did:ethr:0x9E63B020ae098E73cF201EE1357EDc72DFEaA518';
    console.log(`Resolving: ${ethrDid}`);
    try {
      const ethrDoc = await resolver.resolve(ethrDid);
      console.log('DID Document:', JSON.stringify(ethrDoc, null, 2).substring(0, 100) + '...');
    } catch (error) {
      console.log('Error resolving did:ethr (this is expected without a provider):', 
        error instanceof Error ? error.message : String(error));
    }
  } catch (error) {
    console.error('Error during DID resolution:', error);
  }
}

// Run the example
main().catch(console.error);