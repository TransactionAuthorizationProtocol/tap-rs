import { describe, it, expect } from 'vitest';
import type { DIDCommMessage } from '../src/types.js';

// Import the type mapping functions (to be implemented)
const { convertToWasmMessage, convertFromWasmMessage, validateTapMessageType } = await import('../src/type-mapping.js');

describe('Type Mapping', () => {
  describe('convertToWasmMessage', () => {
    it('should convert TypeScript DIDComm message to WASM format', () => {
      const message: DIDCommMessage<{ amount: string; asset: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        created_time: 1640995200000,
        body: {
          amount: '100.0',
          asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        },
      };

      const wasmMessage = convertToWasmMessage(message);

      expect(wasmMessage).toEqual({
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer', // WASM uses 'type' field
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        created_time: 1640995200000,
        body: {
          amount: '100.0',
          asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        },
      });
    });

    it('should handle messages with thread information', () => {
      const message: DIDCommMessage<{ transaction_id: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Authorize',
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        thid: 'thread-123',
        pthid: 'parent-thread-123',
        body: {
          transaction_id: 'tx-456',
        },
      };

      const wasmMessage = convertToWasmMessage(message);

      expect(wasmMessage.thid).toBe('thread-123');
      expect(wasmMessage.pthid).toBe('parent-thread-123');
    });

    it('should handle messages with attachments', () => {
      const message: DIDCommMessage<{ memo: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Payment',
        from: 'did:key:sender',
        body: { memo: 'Invoice payment' },
        attachments: [{
          id: 'att-1',
          description: 'Invoice PDF',
          filename: 'invoice.pdf',
          media_type: 'application/pdf',
          data: {
            encoding: 'base64',
            content: 'base64-encoded-pdf-data',
          },
        }],
      };

      const wasmMessage = convertToWasmMessage(message);

      expect(wasmMessage.attachments).toHaveLength(1);
      expect(wasmMessage.attachments![0]).toEqual({
        id: 'att-1',
        description: 'Invoice PDF',
        filename: 'invoice.pdf',
        media_type: 'application/pdf',
        data: {
          encoding: 'base64',
          content: 'base64-encoded-pdf-data',
        },
      });
    });

    it('should handle optional fields correctly', () => {
      const minimalMessage: DIDCommMessage<{ test: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: { test: 'value' },
      };

      const wasmMessage = convertToWasmMessage(minimalMessage);

      expect(wasmMessage.id).toBe('msg-123');
      expect(wasmMessage.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
      expect(wasmMessage.body).toEqual({ test: 'value' });
      expect(wasmMessage.from).toBeUndefined();
      expect(wasmMessage.to).toBeUndefined();
      expect(wasmMessage.created_time).toBeUndefined();
    });

    it('should throw error for invalid message structure', () => {
      const invalidMessage = {
        // Missing required 'id' field
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: { amount: '100.0' },
      };

      expect(() => convertToWasmMessage(invalidMessage as any)).toThrow('Invalid message structure');
    });
  });

  describe('convertFromWasmMessage', () => {
    it('should convert WASM message to TypeScript DIDComm format', () => {
      const wasmMessage = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        created_time: 1640995200000,
        body: {
          amount: '100.0',
          asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        },
      };

      const message = convertFromWasmMessage(wasmMessage);

      expect(message).toEqual({
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer', // TypeScript uses 'type'
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        created_time: 1640995200000,
        body: {
          amount: '100.0',
          asset: 'eip155:1/erc20:0xdac17f958d2ee523a2206206994597c13d831ec7',
        },
      });
    });

    it('should handle WASM messages with thread information', () => {
      const wasmMessage = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Authorize',
        from: 'did:key:sender',
        thid: 'thread-123',
        pthid: 'parent-thread-123',
        body: { transaction_id: 'tx-456' },
      };

      const message = convertFromWasmMessage(wasmMessage);

      expect(message.thid).toBe('thread-123');
      expect(message.pthid).toBe('parent-thread-123');
    });

    it('should preserve type safety with generic body types', () => {
      const wasmMessage = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: {
          amount: '100.0',
          asset: 'USD',
          originator: { '@id': 'did:key:originator' },
          beneficiary: { '@id': 'did:key:beneficiary' },
        },
      };

      const message = convertFromWasmMessage(wasmMessage);

      // TypeScript should enforce the body type
      expect((message.body as any).amount).toBe('100.0');
      expect((message.body as any).originator['@id']).toBe('did:key:originator');
    });

    it('should throw error for invalid WASM message', () => {
      const invalidWasmMessage = {
        id: 'test-msg',
        // Missing required 'type' field
        body: { amount: '100.0' },
      };

      expect(() => convertFromWasmMessage(invalidWasmMessage as any)).toThrow('Invalid WASM message structure');
    });
  });

  describe('validateTapMessageType', () => {
    it('should validate supported TAP message types', () => {
      const validTypes = [
        'Transfer',
        'Payment',
        'Authorize',
        'Reject',
        'Settle',
        'Cancel',
        'Revert',
        'Connect',
        'Escrow',
        'Capture',
        'AddAgents',
        'ReplaceAgent',
        'RemoveAgent',
        'UpdatePolicies',
        'UpdateParty',
        'ConfirmRelationship',
        'AuthorizationRequired',
        'Presentation',
        'TrustPing',
        'BasicMessage',
      ];

      validTypes.forEach(type => {
        expect(validateTapMessageType(type)).toBe(true);
      });
    });

    it('should validate TAP message type URIs', () => {
      const validUris = [
        'https://tap.rsvp/schema/1.0#Transfer',
        'https://tap.rsvp/schema/1.0#Payment',
        'https://tap.rsvp/schema/1.0#Authorize',
      ];

      validUris.forEach(uri => {
        expect(validateTapMessageType(uri)).toBe(true);
      });
    });

    it('should reject invalid message types', () => {
      const invalidTypes = [
        'InvalidType',
        'transfer', // case sensitive
        'PAYMENT', // case sensitive
        '',
        'https://invalid.com/schema#Transfer',
      ];

      invalidTypes.forEach(type => {
        expect(validateTapMessageType(type)).toBe(false);
      });
    });

    it('should handle null and undefined', () => {
      expect(validateTapMessageType(null as any)).toBe(false);
      expect(validateTapMessageType(undefined as any)).toBe(false);
    });
  });

  describe('Bidirectional Conversion', () => {
    it('should maintain data integrity through round-trip conversion', () => {
      const originalMessage: DIDCommMessage<{ amount: string; memo: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: 'did:key:sender',
        to: ['did:key:recipient1', 'did:key:recipient2'],
        created_time: 1640995200000,
        thid: 'thread-123',
        body: {
          amount: '100.0',
          memo: 'Test transfer',
        },
        attachments: [{
          id: 'att-1',
          data: {
            encoding: 'json',
            content: { metadata: 'test' },
          },
        }],
      };

      // Convert to WASM and back
      const wasmMessage = convertToWasmMessage(originalMessage);
      const convertedMessage = convertFromWasmMessage(wasmMessage);

      expect(convertedMessage).toEqual(originalMessage);
    });

    it('should handle edge cases in round-trip conversion', () => {
      const edgeCaseMessage: DIDCommMessage<{ empty?: string; null_field?: null }> = {
        id: 'msg-edge',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: {
          empty: '',
          null_field: null,
        },
      };

      const wasmMessage = convertToWasmMessage(edgeCaseMessage);
      const convertedMessage = convertFromWasmMessage(wasmMessage);

      expect((convertedMessage.body as any).empty).toBe('');
      expect((convertedMessage.body as any).null_field).toBe(null);
    });
  });

  describe('Performance', () => {
    it('should convert messages efficiently', () => {
      const message: DIDCommMessage<{ amount: string }> = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        from: 'did:key:sender',
        to: ['did:key:recipient'],
        body: { amount: '100.0' },
      };

      const start = performance.now();
      
      for (let i = 0; i < 1000; i++) {
        const wasmMessage = convertToWasmMessage(message);
        convertFromWasmMessage(wasmMessage);
      }
      
      const end = performance.now();
      const duration = end - start;
      
      // Should complete 1000 round-trip conversions in under 50ms
      expect(duration).toBeLessThan(50);
    });
  });

  describe('Error Handling', () => {
    it('should provide detailed error messages for conversion failures', () => {
      const malformedMessage = {
        id: 'msg-123',
        // Missing required 'type' field
        body: { amount: '100.0' },
      };

      expect(() => convertToWasmMessage(malformedMessage as any)).toThrow(
        'Invalid message structure: missing required field \'type\''
      );
    });

    it('should handle circular references gracefully', () => {
      const circularMessage: any = {
        id: 'msg-123',
        type: 'https://tap.rsvp/schema/1.0#Transfer',
        body: { amount: '100.0' },
      };
      
      // Create circular reference
      circularMessage.body.self = circularMessage;

      expect(() => convertToWasmMessage(circularMessage)).toThrow('Circular reference detected');
    });
  });
});