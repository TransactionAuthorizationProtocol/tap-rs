/**
 * Tests for the Transfer message class
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Transfer } from '../src/api/messages/Transfer';
import { ValidationError } from '../src/utils/errors';

// Mock the UUID generation to make tests deterministic
vi.mock('../src/utils/uuid', () => ({
  generateMessageId: vi.fn().mockResolvedValue('msg_test-transfer-uuid')
}));

describe('Transfer', () => {
  const mockOriginator = {
    '@id': 'did:example:originator123',
    '@type': 'Party' as const,
    role: 'originator'
  };
  
  const mockBeneficiary = {
    '@id': 'did:example:beneficiary456',
    '@type': 'Party' as const,
    role: 'beneficiary'
  };
  
  const mockAgent = {
    '@id': 'did:example:agent789',
    '@type': 'Agent' as const,
    role: 'agent'
  };
  
  let transfer: Transfer;
  
  beforeEach(() => {
    transfer = new Transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC on Ethereum
      amount: '100.50',
      originator: mockOriginator,
      beneficiary: mockBeneficiary,
      agents: [mockAgent]
    });
  });
  
  it('should create a transfer message with correct type', () => {
    expect(transfer.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
    expect(transfer.body['@type']).toBe('Transfer');
  });
  
  it('should set all required fields in the body', () => {
    expect(transfer.body.asset).toBe('eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48');
    expect(transfer.body.amount).toBe('100.50');
    expect(transfer.body.originator).toEqual(mockOriginator);
    expect(transfer.body.beneficiary).toEqual(mockBeneficiary);
    expect(transfer.body.agents).toContainEqual(mockAgent);
  });
  
  it('should set optional fields when provided', () => {
    const transferWithOptions = new Transfer({
      asset: 'eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      amount: '100.50',
      originator: mockOriginator,
      beneficiary: mockBeneficiary,
      agents: [mockAgent],
      settlementId: 'eip155:1/tx/0x123abc',
      memo: 'Test transfer',
      purpose: 'CASH',
      categoryPurpose: 'CASH',
      expiry: '2023-12-31T23:59:59Z'
    });
    
    expect(transferWithOptions.body.settlementId).toBe('eip155:1/tx/0x123abc');
    expect(transferWithOptions.body.memo).toBe('Test transfer');
    expect(transferWithOptions.body.purpose).toBe('CASH');
    expect(transferWithOptions.body.categoryPurpose).toBe('CASH');
    expect(transferWithOptions.body.expiry).toBe('2023-12-31T23:59:59Z');
  });
  
  it('should validate required fields', () => {
    transfer._prepareEnvelope('did:example:sender123');
    
    // Should not throw with valid data
    expect(() => transfer._validate()).not.toThrow();
    
    // Test missing asset
    const tempAsset = transfer.body.asset;
    (transfer.body as any).asset = undefined;
    expect(() => transfer._validate()).toThrow(ValidationError);
    transfer.body.asset = tempAsset;
    
    // Test missing amount
    const tempAmount = transfer.body.amount;
    (transfer.body as any).amount = undefined;
    expect(() => transfer._validate()).toThrow(ValidationError);
    transfer.body.amount = tempAmount;
    
    // Test missing originator
    const tempOriginator = transfer.body.originator;
    (transfer.body as any).originator = undefined;
    expect(() => transfer._validate()).toThrow(ValidationError);
    transfer.body.originator = tempOriginator;
    
    // Test missing agents
    const tempAgents = transfer.body.agents;
    (transfer.body as any).agents = undefined;
    expect(() => transfer._validate()).toThrow(ValidationError);
    (transfer.body as any).agents = [];
    expect(() => transfer._validate()).toThrow(ValidationError);
    transfer.body.agents = tempAgents;
  });
  
  it('should validate amount format', () => {
    transfer._prepareEnvelope('did:example:sender123');
    
    // Valid formats
    const validAmounts = ['100', '100.50', '0.5', '1000000'];
    
    for (const amount of validAmounts) {
      transfer.body.amount = amount;
      expect(() => transfer._validate()).not.toThrow();
    }
    
    // Invalid formats
    const invalidAmounts = ['$100', 'one hundred', '100,50', '100-50'];
    
    for (const amount of invalidAmounts) {
      transfer.body.amount = amount;
      expect(() => transfer._validate()).toThrow(ValidationError);
    }
    
    // Restore original amount
    transfer.body.amount = '100.50';
  });
  
  it('should create authorize message with the correct thread ID', () => {
    const authorize = transfer.authorize();
    
    expect(authorize['@type']).toBe('Authorize');
    expect(authorize.thid).toBe(transfer.id);
  });
  
  it('should create authorize message with optional parameters', () => {
    const authorize = transfer.authorize(
      'eip155:1:0x123abc456def',
      'Compliance checks passed',
      3600
    );
    
    expect(authorize.settlementAddress).toBe('eip155:1:0x123abc456def');
    expect(authorize.reason).toBe('Compliance checks passed');
    expect(authorize.expiry).toBeDefined();
  });
  
  it('should create reject message with reason', () => {
    const reject = transfer.reject('Compliance failure');
    
    expect(reject['@type']).toBe('Reject');
    expect(reject.thid).toBe(transfer.id);
    expect(reject.reason).toBe('Compliance failure');
  });
  
  it('should create settle message with settlement ID', () => {
    const settle = transfer.settle('eip155:1/tx/0xabc123');
    
    expect(settle['@type']).toBe('Settle');
    expect(settle.thid).toBe(transfer.id);
    expect(settle.settlementId).toBe('eip155:1/tx/0xabc123');
  });
  
  it('should create settle message with optional amount', () => {
    const settle = transfer.settle('eip155:1/tx/0xabc123', '95.50');
    
    expect(settle.settlementId).toBe('eip155:1/tx/0xabc123');
    expect(settle.amount).toBe('95.50');
  });
  
  it('should create cancel message', () => {
    const cancel = transfer.cancel();
    
    expect(cancel['@type']).toBe('Cancel');
    expect(cancel.thid).toBe(transfer.id);
    expect(cancel.reason).toBeUndefined();
  });
  
  it('should create cancel message with optional reason', () => {
    const cancel = transfer.cancel('User requested cancellation');
    
    expect(cancel.reason).toBe('User requested cancellation');
  });
  
  it('should create revert message with required parameters', () => {
    const revert = transfer.revert({
      settlementAddress: 'eip155:1:0xdef456',
      reason: 'Compliance reversal required'
    });
    
    expect(revert['@type']).toBe('Revert');
    expect(revert.thid).toBe(transfer.id);
    expect(revert.settlementAddress).toBe('eip155:1:0xdef456');
    expect(revert.reason).toBe('Compliance reversal required');
  });
});