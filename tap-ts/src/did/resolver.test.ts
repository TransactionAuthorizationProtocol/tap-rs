/**
 * Tests for DID resolvers
 * 
 * This module contains tests for the DID resolver implementations.
 */

import { assertEquals, assertStringIncludes } from '@std/assert/mod.ts';
import resolverRegistry from './resolver.ts';
import { keyResolver } from './key.ts';
import { webResolver } from './web.ts';
import { pkhResolver } from './pkh.ts';

Deno.test('DID Resolver Registry', async (t) => {
  // Register resolvers
  resolverRegistry.register(keyResolver);
  resolverRegistry.register(webResolver);
  resolverRegistry.register(pkhResolver);
  
  await t.step('getResolver should return the correct resolver', () => {
    const resolver = resolverRegistry.getResolver('key');
    assertEquals(resolver, keyResolver);
  });
  
  await t.step('getResolver should return undefined for unknown method', () => {
    const resolver = resolverRegistry.getResolver('unknown');
    assertEquals(resolver, undefined);
  });
  
  await t.step('unregister should remove a resolver', () => {
    // First, register a test resolver
    const testResolver = {
      getMethod: () => 'test',
      canResolve: () => true,
      resolve: async () => ({
        didDocument: { id: 'did:test:123' },
        didResolutionMetadata: {},
        didDocumentMetadata: {},
      }),
    };
    
    resolverRegistry.register(testResolver);
    assertEquals(resolverRegistry.getResolver('test'), testResolver);
    
    // Now unregister it
    const result = resolverRegistry.unregister('test');
    assertEquals(result, true);
    assertEquals(resolverRegistry.getResolver('test'), undefined);
  });
});

Deno.test('KeyDIDResolver', async (t) => {
  await t.step('getMethod should return key', () => {
    assertEquals(keyResolver.getMethod(), 'key');
  });
  
  await t.step('canResolve should return true for did:key', () => {
    assertEquals(keyResolver.canResolve('did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH'), true);
  });
  
  await t.step('canResolve should return false for other DIDs', () => {
    assertEquals(keyResolver.canResolve('did:web:example.com'), false);
  });
  
  await t.step('resolve should create a valid DID document for did:key', async () => {
    const did = 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH';
    const result = await keyResolver.resolve(did);
    
    assertEquals(result.didDocument.id, did);
    assertEquals(result.didResolutionMetadata.contentType, 'application/did+json');
    assertEquals(result.didDocument.verificationMethod?.length, 1);
    assertEquals(result.didDocument.verificationMethod?.[0].controller, did);
    assertEquals(result.didDocument.verificationMethod?.[0].type, 'Ed25519VerificationKey2020');
    assertEquals(result.didDocument.authentication?.length, 1);
  });
});

Deno.test('WebDIDResolver', async (t) => {
  await t.step('getMethod should return web', () => {
    assertEquals(webResolver.getMethod(), 'web');
  });
  
  await t.step('canResolve should return true for did:web', () => {
    assertEquals(webResolver.canResolve('did:web:example.com'), true);
  });
  
  await t.step('canResolve should return false for other DIDs', () => {
    assertEquals(webResolver.canResolve('did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH'), false);
  });
  
  // Note: Full resolution test would require mocking HTTP requests
});

Deno.test('PkhDIDResolver', async (t) => {
  await t.step('getMethod should return pkh', () => {
    assertEquals(pkhResolver.getMethod(), 'pkh');
  });
  
  await t.step('canResolve should return true for did:pkh', () => {
    assertEquals(pkhResolver.canResolve('did:pkh:eip155:1:0x1234567890123456789012345678901234567890'), true);
  });
  
  await t.step('canResolve should return false for other DIDs', () => {
    assertEquals(pkhResolver.canResolve('did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH'), false);
  });
  
  await t.step('resolve should create a valid DID document for did:pkh (Ethereum)', async () => {
    const did = 'did:pkh:eip155:1:0x1234567890123456789012345678901234567890';
    const result = await pkhResolver.resolve(did);
    
    assertEquals(result.didDocument.id, did);
    assertEquals(result.didResolutionMetadata.contentType, 'application/did+json');
    assertEquals(result.didDocument.verificationMethod?.length, 1);
    assertEquals(result.didDocument.verificationMethod?.[0].controller, did);
    assertEquals(result.didDocument.verificationMethod?.[0].type, 'EcdsaSecp256k1RecoveryMethod2020');
    assertEquals(result.didDocument.authentication?.length, 1);
  });
  
  await t.step('resolve should return an error for invalid did:pkh', async () => {
    const did = 'did:pkh:invalid';
    const result = await pkhResolver.resolve(did);
    
    assertEquals(result.didResolutionMetadata.error, 'invalidDid');
    assertStringIncludes(result.didResolutionMetadata.message || '', 'Invalid did:pkh format');
  });
});
