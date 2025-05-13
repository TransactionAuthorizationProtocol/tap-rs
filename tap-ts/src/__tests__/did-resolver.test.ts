import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest';
import { StandardDIDResolver, createResolver, ResolverOptions } from '../did-resolver';
import { setupWasmTests } from './wasm-test-helper';

// Mock the did-resolver modules
// We need to keep these mocks because we can't make actual network calls in tests
vi.mock('did-resolver', () => ({
  Resolver: vi.fn().mockImplementation(() => ({
    resolve: vi.fn().mockImplementation(async (did) => {
      if (did.startsWith('did:key:')) {
        return { didDocument: { id: did, method: 'key' } };
      } else if (did.startsWith('did:ethr:')) {
        return { didDocument: { id: did, method: 'ethr' } };
      } else if (did.startsWith('did:pkh:')) {
        return { didDocument: { id: did, method: 'pkh' } };
      } else if (did.startsWith('did:web:')) {
        return { didDocument: { id: did, method: 'web' } };
      } else {
        throw new Error(`Cannot resolve DID: ${did}`);
      }
    })
  }))
}));

vi.mock('key-did-resolver', () => ({
  getResolver: vi.fn().mockReturnValue({
    key: async () => ({ id: 'did:key:resolved' })
  })
}));

vi.mock('ethr-did-resolver', () => ({
  getResolver: vi.fn().mockReturnValue({
    ethr: async () => ({ id: 'did:ethr:resolved' })
  })
}));

vi.mock('pkh-did-resolver', () => ({
  getResolver: vi.fn().mockReturnValue({
    pkh: async () => ({ id: 'did:pkh:resolved' })
  })
}));

vi.mock('web-did-resolver', () => ({
  getResolver: vi.fn().mockReturnValue({
    web: async () => ({ id: 'did:web:resolved' })
  })
}));

// Initialize WASM before all tests
beforeAll(async () => {
  await setupWasmTests();
});

describe('DID Resolver', () => {
  describe('createResolver', () => {
    it('should create a resolver with default options', () => {
      const resolver = createResolver();
      expect(resolver).toBeDefined();
    });

    it('should create a resolver with custom options', () => {
      const options: ResolverOptions = {
        resolvers: {
          key: true,
          ethr: false,
          pkh: true,
          web: false
        },
        ethrOptions: {
          networks: [
            {
              name: 'sepolia',
              rpcUrl: 'https://sepolia.infura.io/v3/abc123'
            }
          ]
        }
      };
      
      const resolver = createResolver(options);
      expect(resolver).toBeDefined();
    });
    
    it('should create a resolver with custom resolvers', () => {
      const customResolvers = {
        custom: async () => ({ id: 'did:custom:123' })
      };
      
      const options: ResolverOptions = {
        customResolvers
      };
      
      const resolver = createResolver(options);
      expect(resolver).toBeDefined();
    });
  });
  
  describe('StandardDIDResolver', () => {
    let resolver: StandardDIDResolver;
    
    beforeEach(() => {
      resolver = new StandardDIDResolver();
    });
    
    it('should resolve a key DID', async () => {
      const did = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
      const result = await resolver.resolve(did);
      expect(result).toBeDefined();
      expect(result.id).toBe(did);
      expect(result.method).toBe('key');
    });
    
    it('should resolve an ethr DID', async () => {
      const did = 'did:ethr:0x123456789abcdef';
      const result = await resolver.resolve(did);
      expect(result).toBeDefined();
      expect(result.id).toBe(did);
      expect(result.method).toBe('ethr');
    });
    
    it('should handle errors when resolving invalid DIDs', async () => {
      const did = 'did:invalid:123';
      await expect(resolver.resolve(did)).rejects.toThrow();
    });
  });
});