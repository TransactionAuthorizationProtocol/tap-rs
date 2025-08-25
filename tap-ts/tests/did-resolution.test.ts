import { describe, it, expect } from 'vitest';
import { TapAgent } from '../src/index.js';
import type { DIDResolver, DIDResolutionResult } from '../src/types.js';

describe('DID Resolution with Real WASM', () => {
  describe('Built-in did:key resolver', () => {
    it('should resolve did:key DIDs', async () => {
      const agent = await TapAgent.create();
      const result = await agent.resolveDID(agent.did);
      
      expect(result.didDocument).toBeDefined();
      expect(result.didDocument?.id).toBe(agent.did);
      expect(result.didDocument?.verificationMethod).toBeDefined();
      expect(result.didDocument?.verificationMethod?.length).toBeGreaterThan(0);
      
      // Check verification method structure
      const vm = result.didDocument?.verificationMethod?.[0];
      expect(vm).toBeDefined();
      expect(vm?.id).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+#/);
      expect(vm?.type).toBeDefined();
      expect(vm?.controller).toBe(agent.did);
      
      agent.dispose();
    });

    it('should resolve different key types', async () => {
      const keyTypes: Array<'Ed25519' | 'P256' | 'secp256k1'> = ['Ed25519', 'P256', 'secp256k1'];
      
      for (const keyType of keyTypes) {
        const agent = await TapAgent.create({ keyType });
        const result = await agent.resolveDID(agent.did);
        
        expect(result.didDocument).toBeDefined();
        expect(result.didDocument?.id).toBe(agent.did);
        expect(result.didDocumentMetadata).toBeDefined();
        expect(result.didResolutionMetadata).toBeDefined();
        
        agent.dispose();
      }
    });

    it('should handle non-did:key DIDs appropriately', async () => {
      const agent = await TapAgent.create();
      
      // Should return null or error for unsupported DID methods
      const result = await agent.resolveDID('did:web:example.com');
      
      // Built-in resolver only supports did:key
      expect(result.didDocument).toBeUndefined();
      expect(result.didResolutionMetadata.error).toBeDefined();
      
      agent.dispose();
    });
  });

  describe('Custom DID resolver', () => {
    it('should use custom resolver when provided', async () => {
      const customResolver: DIDResolver = {
        resolve: async (did: string): Promise<DIDResolutionResult> => {
          return {
            didDocument: {
              '@context': ['https://www.w3.org/ns/did/v1'],
              id: did,
              verificationMethod: [{
                id: `${did}#custom-key`,
                type: 'CustomKeyType',
                controller: did,
                publicKeyJwk: {
                  kty: 'OKP',
                  crv: 'Ed25519',
                  x: 'test-key',
                },
              }],
              authentication: [`${did}#custom-key`],
              assertionMethod: [`${did}#custom-key`],
            },
            didDocumentMetadata: {
              created: new Date().toISOString(),
            },
            didResolutionMetadata: {
              contentType: 'application/did+json',
            },
          };
        },
      };

      const agent = await TapAgent.create({ didResolver: customResolver });
      
      // Should use custom resolver for any DID
      const result = await agent.resolveDID('did:custom:12345');
      
      expect(result.didDocument).toBeDefined();
      expect(result.didDocument?.id).toBe('did:custom:12345');
      expect(result.didDocument?.verificationMethod?.[0].id).toBe('did:custom:12345#custom-key');
      expect(result.didDocument?.verificationMethod?.[0].type).toBe('CustomKeyType');
      
      agent.dispose();
    });

    it('should handle resolver errors gracefully', async () => {
      const errorResolver: DIDResolver = {
        resolve: async (did: string): Promise<DIDResolutionResult> => {
          if (did === 'did:error:test') {
            throw new Error('Resolution failed');
          }
          return {
            didDocument: undefined as any,
            didDocumentMetadata: {},
            didResolutionMetadata: {
              error: 'notFound',
              message: 'DID not found',
            },
          };
        },
      };

      const agent = await TapAgent.create({ didResolver: errorResolver });
      
      // Should handle thrown errors
      await expect(agent.resolveDID('did:error:test')).rejects.toThrow('Failed to resolve DID');
      
      // Should handle error responses
      const result = await agent.resolveDID('did:unknown:123');
      expect(result.didDocument).toBeUndefined();
      expect(result.didResolutionMetadata.error).toBe('notFound');
      
      agent.dispose();
    });

    it('should support resolver with caching', async () => {
      let resolutionCount = 0;
      const cachingResolver: DIDResolver = {
        resolve: async (did: string): Promise<DIDResolutionResult> => {
          resolutionCount++;
          return {
            didDocument: {
              '@context': ['https://www.w3.org/ns/did/v1'],
              id: did,
              verificationMethod: [{
                id: `${did}#key-${resolutionCount}`,
                type: 'JsonWebKey2020',
                controller: did,
                publicKeyJwk: {},
              }],
            },
            didDocumentMetadata: {},
            didResolutionMetadata: {},
          };
        },
      };

      const agent = await TapAgent.create({ didResolver: cachingResolver });
      
      // First resolution
      const result1 = await agent.resolveDID('did:test:123');
      expect(result1.didDocument?.verificationMethod?.[0].id).toBe('did:test:123#key-1');
      
      // Second resolution of same DID (resolver might cache internally)
      const result2 = await agent.resolveDID('did:test:123');
      expect(result2.didDocument?.verificationMethod?.[0].id).toBe('did:test:123#key-2');
      
      // Resolution count should be 2 (no caching in this simple example)
      expect(resolutionCount).toBe(2);
      
      agent.dispose();
    });
  });

  describe('DID Document validation', () => {
    it('should validate DID document structure', async () => {
      const agent = await TapAgent.create();
      const result = await agent.resolveDID(agent.did);
      
      const doc = result.didDocument;
      expect(doc).toBeDefined();
      
      // Check required fields
      expect(doc?.['@context']).toBeDefined();
      expect(Array.isArray(doc?.['@context'])).toBe(true);
      expect(doc?.id).toBe(agent.did);
      
      // Check verification methods
      expect(doc?.verificationMethod).toBeDefined();
      expect(Array.isArray(doc?.verificationMethod)).toBe(true);
      doc?.verificationMethod?.forEach(vm => {
        expect(vm.id).toBeDefined();
        expect(vm.type).toBeDefined();
        expect(vm.controller).toBeDefined();
        expect(vm.publicKeyJwk || vm.publicKeyMultibase).toBeDefined();
      });
      
      // Check verification relationships
      if (doc?.authentication) {
        expect(Array.isArray(doc.authentication)).toBe(true);
      }
      if (doc?.assertionMethod) {
        expect(Array.isArray(doc.assertionMethod)).toBe(true);
      }
      if (doc?.keyAgreement) {
        expect(Array.isArray(doc.keyAgreement)).toBe(true);
      }
      if (doc?.capabilityInvocation) {
        expect(Array.isArray(doc.capabilityInvocation)).toBe(true);
      }
      if (doc?.capabilityDelegation) {
        expect(Array.isArray(doc.capabilityDelegation)).toBe(true);
      }
      
      agent.dispose();
    });
  });

  describe('Resolution metadata', () => {
    it('should include proper resolution metadata', async () => {
      const agent = await TapAgent.create();
      const result = await agent.resolveDID(agent.did);
      
      // Check resolution metadata
      expect(result.didResolutionMetadata).toBeDefined();
      if (result.didDocument) {
        // Successful resolution should have minimal metadata
        expect(result.didResolutionMetadata.error).toBeUndefined();
      }
      
      // Check document metadata
      expect(result.didDocumentMetadata).toBeDefined();
      
      agent.dispose();
    });

    it('should handle resolution errors with proper metadata', async () => {
      const agent = await TapAgent.create();
      
      // Try to resolve an invalid DID
      const result = await agent.resolveDID('invalid-did');
      
      expect(result.didDocument).toBeUndefined();
      expect(result.didResolutionMetadata.error).toBeDefined();
      expect(result.didResolutionMetadata.error).toMatch(/invalid|unsupported|notFound/i);
      
      agent.dispose();
    });
  });
});