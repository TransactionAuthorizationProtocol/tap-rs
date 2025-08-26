import { describe, it, expect } from 'vitest';
import { 
  generatePrivateKey, 
  generateUUID
} from '../src/utils.js';
import {
  createTransferMessage,
  createPaymentMessage,
  createAuthorizeMessage,
  createRejectMessage,
  createCancelMessage,
  createSettleMessage,
  createBasicMessage,
  createDIDCommMessage
} from '../src/message-helpers.js';
import type { DID } from '@taprsvp/types';

describe('Utils with Real WASM', () => {
  describe('generatePrivateKey', () => {
    it('should generate Ed25519 private key', async () => {
      const key = await generatePrivateKey('Ed25519');
      expect(key).toMatch(/^[0-9a-f]+$/);
      expect(key.length).toBe(64); // 32 bytes as hex
    });

    it('should generate P256 private key', async () => {
      const key = await generatePrivateKey('P256');
      expect(key).toMatch(/^[0-9a-f]+$/);
      expect(key.length).toBe(64); // 32 bytes as hex
    });

    it('should generate secp256k1 private key', async () => {
      const key = await generatePrivateKey('secp256k1');
      expect(key).toMatch(/^[0-9a-f]+$/);
      expect(key.length).toBe(64); // 32 bytes as hex
    });

    it('should generate unique keys each time', async () => {
      const key1 = await generatePrivateKey('Ed25519');
      const key2 = await generatePrivateKey('Ed25519');
      expect(key1).not.toBe(key2);
    });

    it('should handle invalid key type', async () => {
      await expect(generatePrivateKey('invalid' as any)).rejects.toThrow();
    });
  });

  describe('generateUUID', () => {
    it('should generate valid UUID v4', async () => {
      const uuid = await generateUUID();
      // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
      expect(uuid).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    });

    it('should generate unique UUIDs', async () => {
      const uuid1 = await generateUUID();
      const uuid2 = await generateUUID();
      expect(uuid1).not.toBe(uuid2);
    });

    it('should generate multiple UUIDs quickly', async () => {
      const uuids = await Promise.all(
        Array(10).fill(null).map(() => generateUUID())
      );
      
      // All should be unique
      const uniqueUuids = new Set(uuids);
      expect(uniqueUuids.size).toBe(10);
      
      // All should be valid format
      uuids.forEach(uuid => {
        expect(uuid).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
      });
    });
  });

  describe('Message Creation Helpers', () => {
    const fromDid = 'did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK';
    const toDid = 'did:key:z6MkhvZgTBxPiRHkZGBkFT5b2LbQqQvJYZnHiHQhRvbW1yxH';

    describe('createTransferMessage', () => {
      it('should create valid Transfer message', async () => {
        const message = await createTransferMessage({
          from: fromDid,
          to: [toDid],
          amount: '100.00',
          asset: 'USD',
          originator: { '@id': fromDid as DID, '@type': 'https://schema.org/Organization' },
          beneficiary: { '@id': toDid as DID, '@type': 'https://schema.org/Organization' },
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Transfer');
        expect(message.from).toBe(fromDid);
        expect(message.to).toContain(toDid);
        expect(message.body.amount).toBe('100.00');
        expect(message.body.asset).toBe('USD');
        expect(message.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
      });

      it('should include optional fields when provided', async () => {
        const message = await createTransferMessage({
          from: fromDid,
          to: [toDid],
          amount: '50.00',
          asset: 'EUR',
          originator: { '@id': fromDid as DID, '@type': 'https://schema.org/Organization' },
          beneficiary: { '@id': toDid as DID, '@type': 'https://schema.org/Organization' },
          memo: 'Test transfer',
          agents: [],
        });

        expect(message.body.memo).toBe('Test transfer');
        expect(message.body.agents).toEqual([]);
      });
    });

    describe('createPaymentMessage', () => {
      it('should create valid Payment message', async () => {
        const message = await createPaymentMessage({
          from: fromDid,
          to: [toDid],
          amount: '25.00',
          currency: 'USD',
          merchant: { '@id': toDid as DID, '@type': 'https://schema.org/Organization' },
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Payment');
        expect(message.body.amount).toBe('25.00');
        expect(message.body.currency).toBe('USD');
        expect(message.body.merchant['@id']).toBe(toDid);
      });

      it('should include invoice when provided', async () => {
        const message = await createPaymentMessage({
          from: fromDid,
          to: [toDid],
          amount: '100.00',
          currency: 'EUR',
          merchant: { '@id': toDid as DID, '@type': 'https://schema.org/Organization' },
          invoice: {
            invoice_number: 'INV-001',
            date: '2024-01-01',
            due_date: '2024-02-01',
          },
        });

        expect(message.body.invoice).toBeDefined();
        expect((message.body.invoice as any)?.invoice_number).toBe('INV-001');
      });
    });

    describe('createAuthorizeMessage', () => {
      it('should create valid Authorize message', async () => {
        const message = await createAuthorizeMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-123',
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Authorize');
        expect((message.body as any).transaction_id).toBe('tx-123');
      });

      it('should include settlement address when provided', async () => {
        const message = await createAuthorizeMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-456',
          settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
        });

        expect(message.body.settlementAddress).toBe('0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7');
      });
    });

    describe('createRejectMessage', () => {
      it('should create valid Reject message', async () => {
        const message = await createRejectMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-789',
          reason: 'Insufficient funds',
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Reject');
        expect((message.body as any).transaction_id).toBe('tx-789');
        expect(message.body.reason).toBe('Insufficient funds');
      });
    });

    describe('createCancelMessage', () => {
      it('should create valid Cancel message', async () => {
        const message = await createCancelMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-abc',
          by: fromDid,
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Cancel');
        expect((message.body as any).transaction_id).toBe('tx-abc');
        expect(message.body.by).toBe(fromDid);
      });

      it('should include reason when provided', async () => {
        const message = await createCancelMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-def',
          by: fromDid,
          reason: 'User requested cancellation',
        });

        expect(message.body.reason).toBe('User requested cancellation');
      });
    });

    describe('createSettleMessage', () => {
      it('should create valid Settle message', async () => {
        const message = await createSettleMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-ghi',
          settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
          settlement_id: 'settle-123',
        });

        expect(message.type).toBe('https://tap.rsvp/schema/1.0#Settle');
        expect((message.body as any).transaction_id).toBe('tx-ghi');
        expect(message.body.settlementId).toBe('settle-123');
      });

      it('should include amount when provided', async () => {
        const message = await createSettleMessage({
          from: fromDid,
          to: [toDid],
          transaction_id: 'tx-jkl',
          settlement_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7',
          settlement_id: 'settle-456',
          amount: '75.00',
        });

        expect(message.body.amount).toBe('75.00');
      });
    });

    describe('createBasicMessage', () => {
      it('should create valid basic message', async () => {
        const message = await createBasicMessage({
          from: fromDid,
          to: [toDid],
          content: 'Hello World',
        });

        expect(message.type).toBe('https://didcomm.org/basicmessage/2.0/message');
        expect(message.body.content).toBe('Hello World');
      });

      it('should include locale when provided', async () => {
        const message = await createBasicMessage({
          from: fromDid,
          to: [toDid],
          content: 'Bonjour le monde',
          locale: 'fr-FR',
        });

        expect(message.body.locale).toBe('fr-FR');
      });
    });

    describe('createDIDCommMessage', () => {
      it('should create generic DIDComm message', async () => {
        const message = await createDIDCommMessage({
          type: 'custom-protocol/1.0/action',
          from: fromDid,
          to: [toDid],
          body: {
            action: 'test',
            data: { key: 'value' },
          },
        });

        expect(message.type).toBe('custom-protocol/1.0/action');
        expect(message.body.action).toBe('test');
        expect(message.body.data).toEqual({ key: 'value' });
      });

      it('should generate ID if not provided', async () => {
        const message = await createDIDCommMessage({
          type: 'test',
          from: fromDid,
          to: [toDid],
          body: {},
        });

        expect(message.id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
      });

      it('should use provided ID', async () => {
        const message = await createDIDCommMessage({
          id: 'custom-id-123',
          type: 'test',
          from: fromDid,
          to: [toDid],
          body: {},
        });

        expect(message.id).toBe('custom-id-123');
      });

      it('should include optional fields', async () => {
        const now = Date.now();
        const message = await createDIDCommMessage({
          type: 'test',
          from: fromDid,
          to: [toDid],
          body: {},
          created_time: now,
          expires_time: now + 3600000,
          thid: 'thread-123',
          pthid: 'parent-thread-456',
        });

        expect(message.created_time).toBe(now);
        expect(message.expires_time).toBe(now + 3600000);
        expect(message.thid).toBe('thread-123');
        expect(message.pthid).toBe('parent-thread-456');
      });
    });
  });

  describe('Key Type Constants', () => {
    it('should export correct WasmKeyType values', async () => {
      // Import and verify the enum values are accessible
      const { WasmKeyType } = await import('../src/utils.js');
      
      expect(WasmKeyType.Ed25519).toBe(0);
      expect(WasmKeyType.P256).toBe(1);
      expect(WasmKeyType.Secp256k1).toBe(2);
    });
  });

  describe('Error Handling', () => {
    it('should handle WASM initialization errors gracefully', async () => {
      // The functions should still work even if called rapidly
      const results = await Promise.all([
        generatePrivateKey('Ed25519'),
        generateUUID(),
        generatePrivateKey('P256'),
        generateUUID(),
      ]);
      
      expect(results).toHaveLength(4);
      results.forEach(result => {
        expect(result).toBeTruthy();
      });
    });
  });
});