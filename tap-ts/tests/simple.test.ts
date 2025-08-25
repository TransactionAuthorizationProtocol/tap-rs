import { describe, it, expect } from 'vitest';
import { TapAgent, createTransferMessage } from '../src/index.js';

describe('Simple Real WASM Tests', () => {
  it('should create and use a real WASM agent', async () => {
    const agent = await TapAgent.create();
    
    expect(agent.did).toMatch(/^did:key:z[1-9A-HJ-NP-Za-km-z]+$/);
    
    agent.dispose();
  });

  it('should create a Transfer message with real WASM', async () => {
    const message = await createTransferMessage({
      from: 'did:key:sender',
      to: ['did:key:receiver'],
      amount: '100.00',
      asset: 'USD',
      originator: { 
        '@id': 'did:key:sender' as `did:${string}:${string}`,
        '@type': 'https://schema.org/Person'
      },
      beneficiary: { 
        '@id': 'did:key:receiver' as `did:${string}:${string}`,
        '@type': 'https://schema.org/Person'
      }
    });

    expect(message.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(message.body.amount).toBe('100.00');
  });

  it('should pack and unpack a message with real WASM', async () => {
    const agent = await TapAgent.create();
    
    const message = await createTransferMessage({
      from: agent.did,
      to: [agent.did], // Send to self for testing
      amount: '100.0',
      asset: 'USD',
      originator: {
        '@id': agent.did as `did:${string}:${string}`,
        '@type': 'https://schema.org/Person',
        name: 'Alice'
      },
      beneficiary: {
        '@id': 'did:key:receiver' as `did:${string}:${string}`,
        '@type': 'https://schema.org/Person',
        name: 'Bob'
      }
    });
    
    const packed = await agent.pack(message);
    
    // Verify packed message structure
    expect(packed).toHaveProperty('message');
    expect(packed).toHaveProperty('metadata');
    
    // Parse and verify JWS
    const jws = JSON.parse(packed.message);
    expect(jws).toHaveProperty('payload');
    expect(jws).toHaveProperty('signatures');
    expect(jws.signatures.length).toBeGreaterThan(0);
    
    // Verify signature structure
    const signature = jws.signatures[0];
    expect(signature).toHaveProperty('protected');
    expect(signature).toHaveProperty('signature');
    
    // Decode and verify protected header
    const protectedHeader = JSON.parse(
      Buffer.from(signature.protected, 'base64url').toString()
    );
    expect(protectedHeader).toHaveProperty('alg');
    expect(protectedHeader).toHaveProperty('kid');
    
    // Unpack the message
    const unpacked = await agent.unpack(packed.message);
    
    expect(unpacked.id).toBe(message.id);
    expect(unpacked.type).toBe(message.type);
    expect(unpacked.from).toBe(message.from);
    expect((unpacked.body as any).amount).toBe('100.0');
    
    agent.dispose();
  });
});