/**
 * Type definitions for TAP messages and entities
 */

/**
 * DID type (a string that conforms to the DID specification)
 */
export type DID = string;

/**
 * Message type URI
 */
export type MessageTypeUri = string;

/**
 * Asset identifier type
 */
export type Asset = string;

/**
 * Base interface for TAP participants
 */
export interface TapParticipant {
  /**
   * The ID of the participant
   */
  '@id': string;
}

/**
 * Agent in a transaction (TAIP-5)
 * Agents are services involved in executing transactions
 */
export interface Agent extends TapParticipant {
  /**
   * Role of the agent in this transaction (REQUIRED per TAIP-5)
   */
  role: string;

  /**
   * DID or IRI of another Agent or Party that this agent acts on behalf of (REQUIRED per TAIP-5)
   * Can be a single party or multiple parties
   */
  for: string | string[];

  /**
   * Policies of the agent according to TAIP-7 (optional)
   */
  policies?: any[];

  /**
   * Additional JSON-LD metadata for the agent
   */
  [key: string]: any;
}

/**
 * Party in a transaction (TAIP-6)
 * Parties are real-world entities (legal or natural persons)
 */
export interface Party extends TapParticipant {
  /**
   * Additional JSON-LD metadata for the party
   * This allows for extensible metadata like country codes, LEI codes, MCC codes, etc.
   */
  [key: string]: any;
}


/**
 * Generic TAP message structure
 */
export interface TAPMessage {
  /**
   * Unique identifier for the message
   */
  id: string;

  /**
   * The type URI for the message
   */
  type: MessageTypeUri;

  /**
   * The DID of the sender
   */
  from?: DID;

  /**
   * The DIDs of the recipients
   */
  to?: DID[];

  /**
   * The message creation timestamp
   */
  created_time?: number;

  /**
   * The message expiry timestamp
   */
  expires_time?: number;

  /**
   * The thread ID (for message threading)
   */
  thid?: string;

  /**
   * The parent thread ID (for nested threading)
   */
  pthid?: string;

  /**
   * The message body containing type-specific data
   */
  body: any;
}

/**
 * Transfer message body
 */
export interface Transfer {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The asset being transferred
   */
  asset: Asset;

  /**
   * The amount being transferred
   */
  amount: string;

  /**
   * The originator of the transfer
   */
  originator: Party;

  /**
   * The beneficiary of the transfer
   */
  beneficiary?: Party;

  /**
   * The agents involved in the transfer
   */
  agents?: Agent[];

  /**
   * A memo for the transfer
   */
  memo?: string;

  /**
   * The settlement ID for the transfer
   */
  settlementId?: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Payment message body
 */
export interface Payment {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The asset for the payment
   */
  asset?: string;

  /**
   * The currency for the payment
   */
  currency?: string;

  /**
   * The amount of the payment
   */
  amount: string;

  /**
   * The merchant for the payment
   */
  merchant: Party;

  /**
   * The customer for the payment
   */
  customer?: Party;

  /**
   * The invoice ID for the payment
   */
  invoice?: string;

  /**
   * The expiry timestamp for the payment
   */
  expiry?: string;

  /**
   * The supported assets for the payment
   */
  supportedAssets?: string[];

  /**
   * The agents involved in the payment
   */
  agents?: Agent[];

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Connect message body
 */
export interface Connect {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The agent making the connection
   */
  agent?: Agent;

  /**
   * What the connection is for
   */
  for: string;

  /**
   * The constraints for the connection
   */
  constraints: any;

  /**
   * The expiry timestamp for the connection
   */
  expiry?: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Authorize message body
 */
export interface Authorize {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The reason for the authorization
   */
  reason?: string;

  /**
   * The settlement address for the authorization
   */
  settlementAddress?: string;

  /**
   * The expiry timestamp for the authorization
   */
  expiry?: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Reject message body
 */
export interface Reject {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The reason for the rejection
   */
  reason: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Settle message body
 */
export interface Settle {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The settlement ID
   */
  settlementId: string;

  /**
   * The amount that was settled
   */
  amount?: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Cancel message body
 */
export interface Cancel {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The reason for the cancellation
   */
  reason?: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

/**
 * Revert message body
 */
export interface Revert {
  /**
   * The type of the message (as a TAP URI)
   */
  '@type': MessageTypeUri;

  /**
   * The JSON-LD context
   */
  '@context'?: string;

  /**
   * The settlement address to revert
   */
  settlementAddress: string;

  /**
   * The reason for the reversion
   */
  reason: string;

  /**
   * Additional properties
   */
  [key: string]: any;
}

// Export other type definitions from the wasm.ts file
export * from './wasm';