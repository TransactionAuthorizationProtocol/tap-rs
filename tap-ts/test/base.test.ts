/**
 * Tests for the base DIDCommMessage class
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { DIDCommMessageBase } from '../src/api/messages/base';
import { ValidationError } from '../src/utils/errors';

// Mock the UUID generation to make tests deterministic
vi.mock('../src/utils/uuid', () => ({
  generateMessageId: vi.fn().mockResolvedValue('msg_test-uuid')
}));

// Simple test implementation of DIDCommMessageBase
class TestMessage extends DIDCommMessageBase<{ test: string }> {
  constructor(body: { test: string } = { test: 'value' }, options = {}) {
    super('https://test.com/schema#Test', body, options);
  }
}

describe('DIDCommMessageBase', () => {
  let message: TestMessage;

  beforeEach(() => {
    message = new TestMessage();
  });

  it('should create a message with default values', () => {
    expect(message.type).toBe('https://test.com/schema#Test');
    expect(message.body).toEqual({ test: 'value' });
    expect(message.to).toEqual([]);
    expect(message.created_time).toBeGreaterThan(0);
  });

  it('should set ID from options', () => {
    const msgWithId = new TestMessage({ test: 'value' }, { id: 'msg_custom-id' });
    expect(msgWithId.id).toBe('msg_custom-id');
  });

  it('should set thread ID from options', () => {
    const msgWithThread = new TestMessage({ test: 'value' }, { thid: 'thread-123' });
    expect(msgWithThread.thid).toBe('thread-123');
  });

  it('should set expiration time when provided', () => {
    const now = Math.floor(Date.now() / 1000);
    const msgWithExpiry = new TestMessage({ test: 'value' }, { expiresInSeconds: 3600 });
    
    expect(msgWithExpiry.expires_time).toBeGreaterThanOrEqual(now + 3600 - 5); // Allow 5 sec margin
    expect(msgWithExpiry.expires_time).toBeLessThanOrEqual(now + 3600 + 5);
  });

  it('should prepare an envelope with sender DID', () => {
    message._prepareEnvelope('did:example:123');
    
    expect(message.from).toBe('did:example:123');
    expect(message.created_time).toBeGreaterThan(0);
  });

  it('should validate required fields', () => {
    // Setup a valid message
    message._prepareEnvelope('did:example:123');
    
    // Should not throw
    expect(() => message._validate()).not.toThrow();
    
    // Test missing id
    const temp = message.id;
    (message as any).id = '';
    expect(() => message._validate()).toThrow(ValidationError);
    (message as any).id = temp;
    
    // Test missing type
    const tempType = message.type;
    (message as any).type = '';
    expect(() => message._validate()).toThrow(ValidationError);
    (message as any).type = tempType;
    
    // Test missing from
    const tempFrom = message.from;
    (message as any).from = undefined;
    expect(() => message._validate()).toThrow(ValidationError);
    (message as any).from = tempFrom;
  });

  it('should validate field formats', () => {
    message._prepareEnvelope('did:example:123');
    
    // Test invalid type format
    const tempType = message.type;
    (message as any).type = 'invalid-type';
    expect(() => message._validate()).toThrow(ValidationError);
    (message as any).type = tempType;
    
    // Test invalid from format
    const tempFrom = message.from;
    (message as any).from = 'not-a-did';
    expect(() => message._validate()).toThrow(ValidationError);
    (message as any).from = tempFrom;
    
    // Test invalid to format
    message.to.push('not-a-did');
    expect(() => message._validate()).toThrow(ValidationError);
    message.to = [];
  });

  it('should add recipients', () => {
    message.addRecipient('did:example:456');
    expect(message.to).toContain('did:example:456');
    
    // Adding the same recipient again should not duplicate
    message.addRecipient('did:example:456');
    expect(message.to.length).toBe(1);
    
    // Should error on invalid DID
    expect(() => message.addRecipient('not-a-did')).toThrow(ValidationError);
  });

  it('should set expiry correctly', () => {
    const now = Math.floor(Date.now() / 1000);
    message.setExpiry(3600);
    
    expect(message.expires_time).toBeGreaterThanOrEqual(now + 3600 - 5); // Allow 5 sec margin
    expect(message.expires_time).toBeLessThanOrEqual(now + 3600 + 5);
  });
});