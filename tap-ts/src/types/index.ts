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
 * Reference to an entity in a TAP message
 */
export interface EntityReference {
  /**
   * The ID of the entity
   */
  '@id': string;

  /**
   * The role of the entity
   */
  role?: string;

  /**
   * The name of the entity
   */
  name?: string;

  /**
   * The LEI code of the entity
   */
  leiCode?: string;

  /**
   * A hashed representation of the entity's name (for privacy)
   */
  nameHash?: string;

  /**
   * A reference to another entity this entity acts for
   */
  for?: string;

  /**
   * Additional properties
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
  originator: EntityReference;

  /**
   * The beneficiary of the transfer
   */
  beneficiary?: EntityReference;

  /**
   * The agents involved in the transfer
   */
  agents?: EntityReference[];

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
  merchant: EntityReference;

  /**
   * The customer for the payment
   */
  customer?: EntityReference;

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
  agents?: EntityReference[];

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
  agent?: EntityReference;

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