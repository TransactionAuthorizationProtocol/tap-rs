/**
 * Tests for the MessageWrapper and TAPAgent classes
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { TAPAgent } from '../src/agent/TAPAgent';
import { MessageWrapper, TransferWrapper } from '../src/agent/MessageWrapper';
import { ValidationError, CryptoError } from '../src/utils/errors';

// Mock the UUID generation to make tests deterministic
vi.mock('../src/utils/uuid', () => ({
  generateMessageId: vi.fn().mockResolvedValue('msg_test-transfer-uuid')
}));

// Mock the WASM module
vi.mock('../src/wasm/bridge', () => ({
  getWasmModule: vi.fn().mockResolvedValue({}),
  createAgent: vi.fn().mockResolvedValue({}),
  initialize: vi.fn().mockResolvedValue({})
}));

describe('TAPAgent with MessageWrapper', () => {
  let agent: TAPAgent;
  
  beforeEach(async () => {
    // Create a test agent with a mock signer
    agent = new TAPAgent({
      did: 'did:example:agent123',
      signer: {
        async sign(data: Uint8Array): Promise<Uint8Array> {
          return new Uint8Array(32); // Mock signature
        },
        getDID(): string {
          return 'did:example:agent123';
        }
      }
    });
  });
  
  it('should create a transfer message with correct type', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    expect(transfer).toBeInstanceOf(TransferWrapper);
    expect(transfer.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(transfer.body['@type']).toBe('Transfer');
  });
  
  it('should set all required fields in transfer body', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    expect(transfer.body.asset).toBe('eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48');
    expect(transfer.body.amount).toBe('100.50');
    expect(transfer.body.originator).toEqual({
      '@id': 'did:example:originator123',
      '@type': 'Party',
      role: 'originator'
    });
    expect(transfer.body.agents).toHaveLength(1);
    expect(transfer.body.agents[0]).toEqual({
      '@id': 'did:example:agent789',
      '@type': 'Agent',
      role: 'agent'
    });
  });
  
  it('should set optional fields when provided', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      beneficiary: {
        '@id': 'did:example:beneficiary456',
        '@type': 'Party',
        role: 'beneficiary'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }],
      settlementId: 'eip155:1/tx/0x123abc',
      memo: 'Test transfer',
      purpose: 'CASH',
      categoryPurpose: 'CASH',
      expiry: '2023-12-31T23:59:59Z'
    });
    
    expect(transfer.body.beneficiary).toEqual({
      '@id': 'did:example:beneficiary456',
      '@type': 'Party',
      role: 'beneficiary'
    });
    expect(transfer.body.settlementId).toBe('eip155:1/tx/0x123abc');
    expect(transfer.body.memo).toBe('Test transfer');
    expect(transfer.body.purpose).toBe('CASH');
    expect(transfer.body.categoryPurpose).toBe('CASH');
    expect(transfer.body.expiry).toBe('2023-12-31T23:59:59Z');
  });
  
  it('should validate message when signing', async () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    const signedTransfer = await agent.sign(transfer);
    expect(signedTransfer.from).toBe('did:example:agent123');
    expect(signedTransfer.created_time).toBeDefined();
  });

  it('should create an authorize message linked to the transfer', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    const authorize = transfer.authorize(
      'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e',
      'Compliance checks passed',
      3600
    );
    
    expect(authorize.body['@type']).toBe('Authorize');
    expect(authorize.thid).toBe(transfer.id);
    expect(authorize.body.transfer).toEqual({ '@id': transfer.id });
    expect(authorize.body.reason).toBe('Compliance checks passed');
    expect((authorize.body as any).settlementAddress).toBe('eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e');
    expect((authorize.body as any).expiry).toBeDefined();
  });
  
  it('should create a reject message linked to the transfer', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    const reject = transfer.reject('Compliance failure');
    
    expect(reject.body['@type']).toBe('Reject');
    expect(reject.thid).toBe(transfer.id);
    expect(reject.body.transfer).toEqual({ '@id': transfer.id });
    expect(reject.body.reason).toBe('Compliance failure');
  });
  
  it('should create a settle message linked to the transfer', () => {
    const transfer = agent.transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: {
        '@id': 'did:example:originator123',
        '@type': 'Party',
        role: 'originator'
      },
      agents: [{
        '@id': 'did:example:agent789',
        '@type': 'Agent',
        role: 'agent'
      }]
    });
    
    const settle = transfer.settle('eip155:1/tx/0xabc123', '95.50');
    
    expect(settle.body['@type']).toBe('Settle');
    expect(settle.thid).toBe(transfer.id);
    expect(settle.body.transfer).toEqual({ '@id': transfer.id });
    expect(settle.body.settlementId).toBe('eip155:1/tx/0xabc123');
    expect(settle.body.amount).toBe('95.50');
  });
  
  it('should create a payment request message', () => {
    const payment = agent.paymentRequest({
      amount: '50.75',
      merchant: {
        '@id': 'did:example:merchant123',
        '@type': 'Party',
        role: 'merchant'
      },
      agents: [{
        '@id': 'did:example:agent456',
        '@type': 'Agent',
        role: 'agent'
      }],
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
    });
    
    expect(payment.body['@type']).toBe('PaymentRequest');
    expect(payment.type).toBe('https://tap.rsvp/schema/1.0#PaymentRequest');
    expect(payment.body.amount).toBe('50.75');
    expect(payment.body.merchant).toEqual({
      '@id': 'did:example:merchant123',
      '@type': 'Party',
      role: 'merchant'
    });
    expect(payment.body.asset).toBe('eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48');
  });
  
  it('should create a complete message for a payment request', () => {
    const payment = agent.paymentRequest({
      amount: '50.75',
      merchant: {
        '@id': 'did:example:merchant123',
        '@type': 'Party',
        role: 'merchant'
      },
      agents: [{
        '@id': 'did:example:agent456',
        '@type': 'Agent',
        role: 'agent'
      }],
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
    });
    
    const complete = payment.complete(
      'eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e',
      '50.00'
    );
    
    expect(complete.body['@type']).toBe('Complete');
    expect(complete.thid).toBe(payment.id);
    expect(complete.body.settlementAddress).toBe('eip155:1:0x742d35Cc6634C0532925a3b844Bc454e4438f44e');
    expect(complete.body.amount).toBe('50.00');
  });
});