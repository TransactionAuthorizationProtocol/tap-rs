import { describe, it, expect, vi } from 'vitest';

// Simple mock for testing
const mockWasmAgent = {
  free: vi.fn(),
  get_did: vi.fn(() => 'did:key:test123'),
  exportPrivateKey: vi.fn(() => 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234'),
  exportPublicKey: vi.fn(() => '1234567890abcd1234567890abcd1234567890abcd1234567890abcd12345678'),
  packMessage: vi.fn().mockResolvedValue({
    message: JSON.stringify({
      payload: 'eyJpZCI6InRlc3QtdXVpZC0xMjMifQ',
      signatures: [{
        protected: 'eyJhbGciOiJFZERTQSIsImtpZCI6ImRpZDprZXk6dGVzdDEyMyJ9',
        signature: 'test-signature'
      }]
    }),
    metadata: { type: 'jws' }
  }),
  unpackMessage: vi.fn().mockResolvedValue({
    id: 'test-msg',
    type: 'https://tap.rsvp/schema/1.0#Transfer',
    from: 'did:key:sender',
    to: ['did:key:receiver'],
    created_time: Date.now(),
    body: { 
      '@context': 'https://tap.rsvp/schema/1.0',
      '@type': 'Transfer',
      amount: '100.0',
      asset: 'USD',
      originator: { '@id': 'did:key:sender' },
      beneficiary: { '@id': 'did:key:receiver' }
    }
  }),
};

const mockWasmModule = {
  WasmTapAgent: vi.fn(() => mockWasmAgent),
  generateUUID: vi.fn(() => 'test-uuid-123'),
  generatePrivateKey: vi.fn(() => 'generated-key'),
  WasmKeyType: { Ed25519: 0, P256: 1, Secp256k1: 2 },
  default: vi.fn().mockResolvedValue({}),
};

vi.mock('tap-wasm', () => mockWasmModule);

const { TapAgent, createTransferMessage } = await import('../src/index.js');

describe('Simple TapAgent Test', () => {
  it('should create an agent and access its DID', async () => {
    const agent = await TapAgent.create();
    expect(agent.did).toBe('did:key:test123');
    expect(mockWasmAgent.get_did).toHaveBeenCalled();
  });

  it('should create a transfer message using helper', async () => {
    const message = await createTransferMessage({
      from: 'did:key:sender',
      to: ['did:key:receiver'],
      amount: '100.0',
      asset: 'eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
      originator: {
        '@id': 'did:key:sender',
        '@type': 'https://schema.org/Person',
        name: 'Alice'
      },
      beneficiary: {
        '@id': 'did:key:receiver',
        '@type': 'https://schema.org/Person',
        name: 'Bob'
      }
    });
    
    expect(message.id).toBe('test-uuid-123');
    expect(message.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(message.from).toBe('did:key:sender');
    expect(message.to).toEqual(['did:key:receiver']);
    expect(message.body.amount).toBe('100.0');
  });

  it('should pack a message', async () => {
    const agent = await TapAgent.create();
    const message = await createTransferMessage({
      from: agent.did,
      to: ['did:key:receiver'],
      amount: '100.0',
      asset: 'USD',
      originator: {
        '@id': agent.did,
        '@type': 'https://schema.org/Person',
        name: 'Alice'
      },
      beneficiary: {
        '@id': 'did:key:receiver',
        '@type': 'https://schema.org/Person',
        name: 'Bob'
      }
    });
    
    const packed = await agent.pack(message);
    
    // Now returns JWS object directly
    expect(packed).toHaveProperty('payload');
    expect(packed).toHaveProperty('signatures');
    expect(packed.payload).toBe('eyJpZCI6InRlc3QtdXVpZC0xMjMifQ');
    expect(packed.signatures[0].signature).toBe('test-signature');
    expect(mockWasmAgent.packMessage).toHaveBeenCalled();
    
    // Check that the message was converted to WASM format
    const callArgs = mockWasmAgent.packMessage.mock.calls[0][0];
    expect(callArgs.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(callArgs.id).toBe('test-uuid-123');
  });
});