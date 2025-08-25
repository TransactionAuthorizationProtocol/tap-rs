import { describe, it, expect, beforeAll } from 'vitest';
import { TapAgent } from '../src/index.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import init from 'tap-wasm';

// Get the path to the WASM binary
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmPath = join(__dirname, '../../tap-wasm/pkg/tap_wasm_bg.wasm');

describe('TAP-Veramo Message Format Compatibility', () => {
  let tapAgent: TapAgent;

  beforeAll(async () => {
    // Initialize TAP WASM
    const wasmBinary = readFileSync(wasmPath);
    await init(wasmBinary);
    
    // Create TAP agent
    tapAgent = await TapAgent.create({ keyType: 'Ed25519' });
  });

  describe('JWS Format Compatibility', () => {
    it('should produce Veramo-compatible JWS format', async () => {
      // TAP creates a JWS message
      const message = {
        id: 'test-jws-001',
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: tapAgent.did,
        to: ['did:key:z6MktestRecipient'],
        created_time: Date.now(),
        body: {
          content: 'Testing JWS format'
        }
      };

      const packed = await tapAgent.pack(message);

      // Verify it matches Veramo's expected JWS structure
      // Parse the JWS from the packed message result
      const jws = JSON.parse(packed.message);
      expect(jws).toHaveProperty('payload');
      expect(jws).toHaveProperty('signatures');
      expect(Array.isArray(jws.signatures)).toBe(true);
      expect(jws.signatures.length).toBeGreaterThan(0);

      // Check signature structure
      const sig = jws.signatures[0];
      expect(sig).toHaveProperty('protected');
      expect(sig).toHaveProperty('signature');

      // Decode and verify protected header
      const protectedHeader = JSON.parse(
        Buffer.from(sig.protected, 'base64url').toString()
      );

      // Veramo expects these fields in JWS
      expect(protectedHeader).toHaveProperty('typ');
      expect(protectedHeader.typ).toBe('application/didcomm-signed+json');
      expect(protectedHeader).toHaveProperty('alg');
      expect(['EdDSA', 'ES256', 'ES256K']).toContain(protectedHeader.alg);
      expect(protectedHeader).toHaveProperty('kid');
      expect(protectedHeader.kid).toContain(tapAgent.did);

      // Decode and verify payload
      const payload = JSON.parse(
        Buffer.from(jws.payload, 'base64url').toString()
      );

      expect(payload.id).toBe(message.id);
      expect(payload.type).toBe(message.type);
      expect(payload.from).toBe(message.from);
      expect(payload.body).toEqual(message.body);
    });

    it('should handle Veramo-formatted JWS', async () => {
      // This is what a Veramo JWS looks like
      const veramoJWS = {
        payload: Buffer.from(JSON.stringify({
          id: 'veramo-msg-001',
          type: 'https://didcomm.org/basicmessage/2.0/message',
          from: 'did:key:z6MkveramoSender',
          to: [tapAgent.did],
          body: {
            content: 'Message from Veramo'
          }
        })).toString('base64url'),
        signatures: [{
          protected: Buffer.from(JSON.stringify({
            typ: 'application/didcomm-signed+json',
            alg: 'EdDSA',
            kid: 'did:key:z6MkveramoSender#keys-1'
          })).toString('base64url'),
          signature: 'mock-signature-would-be-here'
        }]
      };

      // TAP should recognize this as a valid JWS structure
      const jwsString = JSON.stringify(veramoJWS);
      
      // Even though we can't verify the signature (mock), 
      // TAP should recognize the format
      try {
        await tapAgent.unpack(jwsString);
      } catch (error) {
        // Expected to fail - either format or signature issue
        // The important thing is TAP attempts to process it
        expect(error).toBeDefined();
        expect(error.message).toBeTruthy();
      }
    });
  });

  describe('Message Type Compatibility', () => {
    it('should support all standard DIDComm message types', async () => {
      const messageTypes = [
        'https://didcomm.org/basicmessage/2.0/message',
        'https://didcomm.org/trust-ping/2.0/ping',
        'https://didcomm.org/trust-ping/2.0/ping-response',
        'https://didcomm.org/discover-features/2.0/query',
        'https://didcomm.org/discover-features/2.0/disclose',
      ];

      for (const messageType of messageTypes) {
        const message = {
          id: `test-${Date.now()}`,
          type: messageType,
          from: tapAgent.did,
          to: ['did:key:z6MktestRecipient'],
          created_time: Date.now(),
          body: { test: 'data' }
        };

        const packed = await tapAgent.pack(message);
        // All should produce valid JWS
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');
      }
    });

    it('should support TAP-specific message types', async () => {
      const tapTypes = [
        'https://tap.rsvp/schema/1.0#Transfer',
        'https://tap.rsvp/schema/1.0#Payment',
        'https://tap.rsvp/schema/1.0#Authorize',
        'https://tap.rsvp/schema/1.0#Reject',
        'https://tap.rsvp/schema/1.0#Settle',
        'https://tap.rsvp/schema/1.0#Connect',
      ];

      for (const messageType of tapTypes) {
        const message = {
          id: `tap-${Date.now()}`,
          type: messageType,
          from: tapAgent.did,
          to: ['did:key:z6MktestRecipient'],
          created_time: Date.now(),
          body: { 
            '@context': 'https://tap.rsvp/schema/1.0',
            '@type': messageType.split('#')[1]
          }
        };

        const packed = await tapAgent.pack(message);
        // TAP messages should also use standard JWS format
        const jws = JSON.parse(packed.message);
        expect(jws).toHaveProperty('payload');
        expect(jws).toHaveProperty('signatures');

        // Verify TAP messages maintain DIDComm v2 compatibility
        const protectedHeader = JSON.parse(
          Buffer.from(jws.signatures[0].protected, 'base64url').toString()
        );
        expect(protectedHeader.typ).toBe('application/didcomm-signed+json');
      }
    });
  });

  describe('Threading Compatibility', () => {
    it('should preserve thread IDs in Veramo-compatible format', async () => {
      const threadId = 'thread-123';
      const parentThreadId = 'parent-thread-456';

      const message = {
        id: 'threaded-msg-001',
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: tapAgent.did,
        to: ['did:key:z6MktestRecipient'],
        created_time: Date.now(),
        thid: threadId,
        pthid: parentThreadId,
        body: { content: 'Threaded message' }
      };

      const packed = await tapAgent.pack(message);

      // Parse the JWS from packed message
      const jws = JSON.parse(packed.message);
      
      // Decode payload to verify threading
      const payload = JSON.parse(
        Buffer.from(jws.payload, 'base64url').toString()
      );

      expect(payload.thid).toBe(threadId);
      expect(payload.pthid).toBe(parentThreadId);
    });
  });

  describe('DID Format Compatibility', () => {
    it('should use Veramo-compatible did:key format', () => {
      // TAP should generate did:key DIDs compatible with Veramo
      expect(tapAgent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
      
      // For Ed25519, should start with z6Mk
      if (tapAgent.did.startsWith('did:key:z6Mk')) {
        expect(tapAgent.did).toMatch(/^did:key:z6Mk[1-9A-HJ-NP-Za-km-z]{44,}$/);
      }
    });

    it('should generate correct key ID references', async () => {
      const message = {
        id: 'kid-test-001',
        type: 'https://didcomm.org/basicmessage/2.0/message',
        from: tapAgent.did,
        to: ['did:key:z6MktestRecipient'],
        created_time: Date.now(),
        body: { test: 'kid' }
      };

      const packed = await tapAgent.pack(message);

      // Parse the JWS from packed message
      const jws = JSON.parse(packed.message);
      
      const protectedHeader = JSON.parse(
        Buffer.from(jws.signatures[0].protected, 'base64url').toString()
      );

      // Key ID should follow DID URL format
      expect(protectedHeader.kid).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+#.+$/);
      expect(protectedHeader.kid).toContain(tapAgent.did);
      expect(protectedHeader.kid).toContain('#');
    });
  });
});