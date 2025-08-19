import { describe, it, expect, vi } from 'vitest';

// Simple mock for testing
const mockWasmAgent = {
  free: vi.fn(),
  get_did: vi.fn(() => 'did:key:test123'),
  exportPrivateKey: vi.fn(() => 'abcd1234567890abcd1234567890abcd1234567890abcd1234567890abcd1234'),
  exportPublicKey: vi.fn(() => '1234567890abcd1234567890abcd1234567890abcd1234567890abcd12345678'),
  packMessage: vi.fn().mockResolvedValue({
    message: 'packed-content',
    metadata: { type: 'encrypted' }
  }),
  unpackMessage: vi.fn().mockResolvedValue({
    id: 'test-msg',
    typ: 'https://tap.rsvp/schema/1.0#Transfer',
    body: { test: 'data' }
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

const { TapAgent } = await import('../src/tap-agent.js');

describe('Simple TapAgent Test', () => {
  it('should create an agent and access its DID', async () => {
    const agent = await TapAgent.create();
    expect(agent.did).toBe('did:key:test123');
    expect(mockWasmAgent.get_did).toHaveBeenCalled();
  });

  it('should create a message', async () => {
    const agent = await TapAgent.create();
    const message = agent.createMessage('Transfer', { amount: '100.0' });
    
    expect(message.id).toBe('test-uuid-123');
    expect(message.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(message.from).toBe('did:key:test123');
    expect(message.body).toEqual({ amount: '100.0' });
  });

  it('should pack a message', async () => {
    const agent = await TapAgent.create();
    const message = agent.createMessage('Transfer', { amount: '100.0' });
    
    const packed = await agent.pack(message);
    
    expect(packed.message).toBe('packed-content');
    expect(mockWasmAgent.packMessage).toHaveBeenCalled();
    
    // Check that the message was converted to WASM format
    const callArgs = mockWasmAgent.packMessage.mock.calls[0][0];
    expect(callArgs.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(callArgs.id).toBe('test-uuid-123');
    expect(callArgs.body).toEqual({ amount: '100.0' });
  });
});