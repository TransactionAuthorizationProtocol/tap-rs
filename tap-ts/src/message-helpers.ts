/**
 * Helper functions for creating TAP-compliant messages using @taprsvp/types
 */

import type {
  TransferMessage,
  PaymentMessage,
  AuthorizeMessage,
  RejectMessage,
  SettleMessage,
  ConnectMessage,
  Transfer,
  Payment,
  Authorize,
  Reject,
  Settle,
  Connect,
  Cancel,
  DIDCommMessage,
  Agent,
  Party,
  DID,
} from '@taprsvp/types';
import { generateUUID } from './utils.js';

/**
 * Create a TAP Transfer message
 * @param params Transfer parameters
 * @returns TAP-compliant Transfer message
 */
export async function createTransferMessage(params: {
  from: string;
  to: string[];
  amount: string;
  asset: string;
  originator: Party;
  beneficiary: Party;
  memo?: string;
  agents?: Agent[];
  thid?: string;
  pthid?: string;
  expires_time?: number;
}): Promise<TransferMessage> {
  const transferBody: Transfer = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Transfer',
    amount: params.amount as any,
    asset: params.asset as any,
    originator: params.originator,
    beneficiary: params.beneficiary,
    ...(params.memo !== undefined && { memo: params.memo }),
    agents: params.agents || [],
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Transfer',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    body: transferBody,
  };
  
  if (params.expires_time !== undefined) message.expires_time = params.expires_time;
  if (params.thid !== undefined) message.thid = params.thid;
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Payment message
 * @param params Payment parameters
 * @returns TAP-compliant Payment message
 */
export async function createPaymentMessage(params: {
  from: string;
  to: string[];
  amount: string;
  currency?: string;
  asset?: string;
  merchant: Party;
  settlement_address?: string;
  invoice?: any;
  memo?: string;
  agents?: Agent[];
  thid?: string;
  pthid?: string;
  expires_time?: number;
}): Promise<PaymentMessage> {
  const paymentBody: Payment = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Payment',
    amount: params.amount as any,
    ...(params.currency !== undefined && { currency: params.currency as any }),
    ...(params.asset !== undefined && { asset: params.asset as any }),
    merchant: params.merchant,
    ...(params.settlement_address !== undefined && { settlementAddress: params.settlement_address as any }),
    ...(params.invoice !== undefined && { invoice: params.invoice }),
    ...(params.memo !== undefined && { memo: params.memo }),
    agents: params.agents || [],
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Payment',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    body: paymentBody,
  };
  
  if (params.expires_time !== undefined) message.expires_time = params.expires_time;
  if (params.thid !== undefined) message.thid = params.thid;
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Authorize message
 * @param params Authorize parameters
 * @returns TAP-compliant Authorize message
 */
export async function createAuthorizeMessage(params: {
  from: string;
  to: string[];
  transaction_id: string;
  settlement_address?: string;
  settlement_asset?: string;
  expiry?: string;
  thid?: string;
  pthid?: string;
}): Promise<AuthorizeMessage> {
  const authorizeBody: Authorize = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Authorize',
    ...(params.settlement_address !== undefined && { settlementAddress: params.settlement_address as any }),
    ...(params.settlement_asset !== undefined && { settlementAsset: params.settlement_asset as any }),
    ...(params.expiry !== undefined && { expiry: params.expiry }),
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Authorize',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    thid: params.thid || params.transaction_id,
    body: authorizeBody,
  };
  
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Reject message
 * @param params Reject parameters
 * @returns TAP-compliant Reject message
 */
export async function createRejectMessage(params: {
  from: string;
  to: string[];
  transaction_id: string;
  reason: string;
  thid?: string;
  pthid?: string;
}): Promise<RejectMessage> {
  const rejectBody: Reject = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Reject',
    ...(params.reason !== undefined && { reason: params.reason }),
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Reject',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    thid: params.thid || params.transaction_id,
    body: rejectBody,
  };
  
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Cancel message
 * @param params Cancel parameters
 * @returns TAP-compliant Cancel message
 */
export async function createCancelMessage(params: {
  from: string;
  to: string[];
  transaction_id: string;
  by: string;
  reason?: string;
  thid?: string;
  pthid?: string;
}): Promise<DIDCommMessage<Cancel>> {
  const cancelBody: Cancel = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Cancel',
    by: params.by as any,
    ...(params.reason !== undefined && { reason: params.reason }),
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Cancel',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    thid: params.thid || params.transaction_id,
    body: cancelBody,
  };
  
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Settle message
 * @param params Settle parameters
 * @returns TAP-compliant Settle message
 */
export async function createSettleMessage(params: {
  from: string;
  to: string[];
  transaction_id: string;
  settlement_address: string;
  settlement_id?: string;
  amount?: string;
  thid?: string;
  pthid?: string;
}): Promise<SettleMessage> {
  const settleBody: Settle = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Settle',
    settlementAddress: params.settlement_address as any,
    ...(params.settlement_id !== undefined && { settlementId: params.settlement_id as any }),
    ...(params.amount !== undefined && { amount: params.amount as any }),
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Settle',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    thid: params.thid || params.transaction_id,
    body: settleBody,
  };
  
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a TAP Connect message
 * @param params Connect parameters
 * @returns TAP-compliant Connect message
 */
export async function createConnectMessage(params: {
  from: string;
  to: string[];
  requester: Party;
  principal: Party;
  agents?: Agent[];
  constraints: any;
  thid?: string;
  pthid?: string;
  expires_time?: number;
}): Promise<ConnectMessage> {
  const connectBody: Connect = {
    '@context': 'https://tap.rsvp/schema/1.0',
    '@type': 'Connect',
    requester: params.requester,
    principal: params.principal,
    agents: params.agents || [],
    constraints: params.constraints,
  };

  const message: any = {
    id: await generateUUID(),
    type: 'https://tap.rsvp/schema/1.0#Connect',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    body: connectBody,
  };
  
  if (params.expires_time !== undefined) message.expires_time = params.expires_time;
  if (params.thid !== undefined) message.thid = params.thid;
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a DIDComm BasicMessage
 * @param params Message parameters
 * @returns DIDComm-compliant BasicMessage
 */
export async function createBasicMessage(params: {
  from: string;
  to: string[];
  content: string;
  locale?: string;
  thid?: string;
  pthid?: string;
  expires_time?: number;
}): Promise<DIDCommMessage<{ content: string; locale?: string }>> {
  const message: any = {
    id: await generateUUID(),
    type: 'https://didcomm.org/basicmessage/2.0/message',
    from: params.from as DID,
    to: params.to as DID[],
    created_time: Date.now(),
    body: {
      content: params.content,
      ...(params.locale !== undefined && { locale: params.locale }),
    },
  };
  
  if (params.expires_time !== undefined) message.expires_time = params.expires_time;
  if (params.thid !== undefined) message.thid = params.thid;
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}

/**
 * Create a generic DIDComm message
 * @param params Message parameters
 * @returns DIDComm-compliant message
 */
export async function createDIDCommMessage<T = any>(params: {
  type: string;
  from: string;
  to?: string[];
  body: T;
  id?: string;
  thid?: string;
  pthid?: string;
  created_time?: number;
  expires_time?: number;
}): Promise<DIDCommMessage<T>> {
  const message: any = {
    id: params.id || await generateUUID(),
    type: params.type,
    from: params.from as DID,
    to: params.to as DID[],
    created_time: params.created_time || Date.now(),
    body: params.body,
  };
  
  if (params.expires_time !== undefined) message.expires_time = params.expires_time;
  if (params.thid !== undefined) message.thid = params.thid;
  if (params.pthid !== undefined) message.pthid = params.pthid;
  
  return message;
}