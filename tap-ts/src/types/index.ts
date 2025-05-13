/**
 * Type definition for DID identifiers
 */
export type DID = `${string}:${string}` | `did:${string}:${string}`;

/**
 * Message type enum for TAP messages
 */
export type MessageTypeUri =
  | "https://tap.rsvp/schema/1.0#Transfer"
  | "https://tap.rsvp/schema/1.0#Payment"
  | "https://tap.rsvp/schema/1.0#Authorize"
  | "https://tap.rsvp/schema/1.0#Settle"
  | "https://tap.rsvp/schema/1.0#Reject"
  | "https://tap.rsvp/schema/1.0#Cancel"
  | "https://tap.rsvp/schema/1.0#Revert"
  | "https://tap.rsvp/schema/1.0#Connect"
  | "https://tap.rsvp/schema/1.0#Presentation"
  | "https://tap.rsvp/schema/1.0#ConfirmRelationship"
  | "https://tap.rsvp/schema/1.0#AddAgents"
  | "https://tap.rsvp/schema/1.0#ReplaceAgent"
  | "https://tap.rsvp/schema/1.0#RemoveAgent"
  | "https://tap.rsvp/schema/1.0#UpdatePolicies"
  | "https://tap.rsvp/schema/1.0#UpdateParty"
  | "https://tap.rsvp/schema/1.0#Error"
  | "https://tap.rsvp/schema/1.0#Complete";

/**
 * Basic interface for TAP messages
 */
export interface TAPMessage {
  id: string;
  type: MessageTypeUri;
  from?: DID;
  to?: DID[];
  created_time: number;
  body: any;
}

/**
 * DIDComm message interface
 */
export interface DIDCommMessage {
  id: string;
  type: string;
  from?: DID;
  to?: DID[];
  created_time: number;
  body: any;
}

/**
 * Entity reference object
 */
export interface EntityReference {
  '@id': DID;
  name?: string;
  role?: string;
}

/**
 * Participant interface - generic participant with a type parameter
 */
export interface Participant<T extends string> {
  '@type': T;
  '@id': DID;
  role?: string;
  name?: string;
}

/**
 * CAIP19 Asset Identifier (Chain Agnostic Improvement Proposal 19)
 */
export type CAIP19 = string;

/**
 * Amount type for asset transfers
 */
export type Amount = string;

/**
 * Asset interface for transfers
 */
export interface Asset {
  id: string;
  quantity: string;
}

/**
 * Transfer message interface (TAIP-3)
 */
export interface Transfer {
  '@type': "https://tap.rsvp/schema/1.0#Transfer";
  '@context': string;
  initiator?: EntityReference;
  beneficiary?: EntityReference;
  asset: {
    id: string;
    quantity: string;
  };
  memo?: string;
  expiry?: number;
  agents?: any[];
}

/**
 * Payment request message interface (TAIP-14)
 */
export interface Payment {
  '@type': "https://tap.rsvp/schema/1.0#Payment";
  '@context': string;
  merchant: EntityReference;
  customer?: EntityReference;
  asset: {
    id: string;
    quantity: string;
  };
  memo?: string;
  expiry?: number;
}

/**
 * Connection message interface
 */
export interface Connect {
  '@type': "https://tap.rsvp/schema/1.0#Connect";
  '@context': string;
  agent: EntityReference;
  for?: string;
  constraints?: Record<string, any>;
  expiry?: number;
}

/**
 * Authorization response interface (TAIP-4)
 */
export interface Authorize {
  '@type': "https://tap.rsvp/schema/1.0#Authorize";
  '@context': string;
  thread_id: string;
  from: EntityReference;
  approve: boolean;
  reason?: string;
}

/**
 * Reject response interface (TAIP-4)
 */
export interface Reject {
  '@type': "https://tap.rsvp/schema/1.0#Reject";
  '@context': string;
  thread_id: string;
  from: EntityReference;
  reason: string;
}

/**
 * Settlement notification interface (TAIP-4)
 */
export interface Settle {
  '@type': "https://tap.rsvp/schema/1.0#Settle";
  '@context': string;
  thread_id: string;
  from: EntityReference;
  settlement: {
    id: string;
    network?: string;
    timestamp: number;
  };
}

/**
 * Cancellation message interface
 */
export interface Cancel {
  '@type': "https://tap.rsvp/schema/1.0#Cancel";
  '@context': string;
  thread_id: string;
  from: EntityReference;
  reason?: string;
}