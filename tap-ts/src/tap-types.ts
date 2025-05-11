/**
 * Transaction Authorization Protocol (TAP) Type Definitions
 * 
 * This module provides TypeScript type definitions for the TAP protocol
 * based on the TAP specifications from prds/taips/packages/typescript/src/tap.ts.
 * 
 * These types provide a strongly-typed interface for working with TAP messages
 * and enable better IntelliSense and compile-time checking.
 */

// Import and re-export all types from tap.ts
export type {
  // Common Types
  DID,
  IRI,
  ISO8601DateTime,
  CAIP2,
  CAIP10,
  CAIP19,
  CAIP220,
  DTI,
  Asset,
  Amount,
  LEICode,
  ISO20022PurposeCode,
  ISO20022CategoryPurposeCode,

  // Message Structure
  DIDCommMessage,
  DIDCommReply,
  Participant,
  Policy,
  RequireAuthorization,
  RequirePresentation,
  RequireRelationshipConfirmation,
  RequirePurpose,

  // Core TAP types
  Transfer,
  Payment,
  Transactions,
  Authorize,
  Settle,
  Reject,
  Cancel,
  Revert,
  UpdateAgent,
  UpdateParty,
  AddAgents,
  ReplaceAgent,
  RemoveAgent,
  ConfirmRelationship,
  UpdatePolicies,
  TransactionConstraints,
  Connect,
  AuthorizationRequired,
  Complete,

  // DIDComm Message types
  TransferMessage,
  PaymentMessage,
  AuthorizeMessage,
  SettleMessage,
  RejectMessage,
  CancelMessage,
  RevertMessage,
  UpdateAgentMessage,
  UpdatePartyMessage,
  AddAgentsMessage,
  ReplaceAgentMessage,
  RemoveAgentMessage,
  CACAOAttachment,
  ConfirmRelationshipMessage,
  UpdatePoliciesMessage,
  ConnectMessage,
  AuthorizationRequiredMessage,
  CompleteMessage,
  TAPMessage,
} from "../../prds/taips/packages/typescript/src/tap";

// Type aliases for convenience and backward compatibility
import { 
  DID,
  Participant as TAPParticipant,
  TAPMessage,
  DIDCommMessage
} from "../../prds/taips/packages/typescript/src/tap";

/**
 * Legacy participant type for backward compatibility
 * This maps to the Participant interface in the current implementation
 * but with added compatibility for the new TAP specification.
 */
export interface LegacyParticipant {
  /** DID of the participant */
  "@id": string;
  
  /** Optional role of the participant in the transaction */
  role?: string;

  /** Optional Legal Entity Identifier */
  lei?: string;
  
  /** Optional name of the participant */
  name?: string;
  
  /** Optional hash of the participant's name */
  nameHash?: string;
  
  /** Optional reference to the party this agent is acting for */
  for?: string;
  
  /** Additional fields may be present */
  [key: string]: unknown;
}

/**
 * Maps legacy MessageType enum values to the new TAP message type strings
 * This enables backward compatibility while using the new type system.
 */
export const MessageTypeMap = {
  TRANSFER: "https://tap.rsvp/schema/1.0#Transfer",
  PAYMENT: "https://tap.rsvp/schema/1.0#Payment",
  PRESENTATION: "https://tap.rsvp/schema/1.0#Presentation",
  AUTHORIZE: "https://tap.rsvp/schema/1.0#Authorize",
  REJECT: "https://tap.rsvp/schema/1.0#Reject",
  SETTLE: "https://tap.rsvp/schema/1.0#Settle",
  CANCEL: "https://tap.rsvp/schema/1.0#Cancel",
  REVERT: "https://tap.rsvp/schema/1.0#Revert",
  ADD_AGENTS: "https://tap.rsvp/schema/1.0#AddAgents",
  REPLACE_AGENT: "https://tap.rsvp/schema/1.0#ReplaceAgent",
  REMOVE_AGENT: "https://tap.rsvp/schema/1.0#RemoveAgent",
  UPDATE_POLICIES: "https://tap.rsvp/schema/1.0#UpdatePolicies",
  UPDATE_PARTY: "https://tap.rsvp/schema/1.0#UpdateParty",
  CONFIRM_RELATIONSHIP: "https://tap.rsvp/schema/1.0#ConfirmRelationship",
  CONNECT: "https://tap.rsvp/schema/1.0#Connect",
  AUTHORIZATION_REQUIRED: "https://tap.rsvp/schema/1.0#AuthorizationRequired",
  COMPLETE: "https://tap.rsvp/schema/1.0#Complete",
  ERROR: "https://tap.rsvp/schema/1.0#Error",
} as const;

/**
 * TAP Message Type string literal type
 * This represents all valid TAP message types as string literals.
 */
export type TAPMessageTypeString = typeof MessageTypeMap[keyof typeof MessageTypeMap];

/**
 * Helper function to validate if a string is a valid TAP message type
 * @param type String to check
 * @returns true if the string is a valid TAP message type
 */
export function isValidTAPMessageType(type: string): type is TAPMessageTypeString {
  return Object.values(MessageTypeMap).includes(type as TAPMessageTypeString);
}

/**
 * Legacy LegacyParticipant to TAP Participant converter
 * @param participant Legacy LegacyParticipant object
 * @returns TAP Participant object
 */
export function convertToTAPParticipant(participant: LegacyParticipant): TAPParticipant<"Party" | "Agent"> {
  // Determine if this is a Party or Agent based on role
  const type = participant.role?.toLowerCase().includes("agent") ? "Agent" : "Party";
  
  const tapParticipant: TAPParticipant<typeof type> = {
    "@context": "https://tap.rsvp/schema/1.0",
    "@type": type,
    "@id": participant["@id"] as DID,
  };
  
  // Copy over properties that exist in both
  if (participant.role) tapParticipant.role = participant.role;
  if (participant.name) tapParticipant.name = participant.name;
  if (participant.nameHash) tapParticipant.nameHash = participant.nameHash;
  if (participant.for) tapParticipant.for = participant.for as DID;
  if (participant.lei) tapParticipant["lei:leiCode"] = participant.lei;
  
  return tapParticipant;
}

/**
 * TAP Participant to Legacy LegacyParticipant converter
 * @param participant TAP Participant object
 * @returns Legacy LegacyParticipant object
 */
export function convertToLegacyParticipant(participant: TAPParticipant<any>): LegacyParticipant {
  const legacyParticipant: LegacyParticipant = {
    "@id": participant["@id"],
  };
  
  // Copy over properties that exist in both
  if (participant.role) legacyParticipant.role = participant.role;
  if (participant.name) legacyParticipant.name = participant.name;
  if (participant.nameHash) legacyParticipant.nameHash = participant.nameHash;
  if (participant.for) legacyParticipant.for = participant.for;
  if (participant["lei:leiCode"]) legacyParticipant.lei = participant["lei:leiCode"];
  
  return legacyParticipant;
}

/**
 * Options for message creation
 * Extends the existing MessageOptions interface with TAP-specific options
 */
export interface TAPMessageOptions {
  /** Message type as a TAP message type string */
  type: TAPMessageTypeString;
  
  /** Optional message ID (auto-generated if not provided) */
  id?: string;
  
  /** Thread ID for tracking message threads */
  thid?: string;
  
  /** Parent thread ID for nested threads */
  pthid?: string;
  
  /** Sender DID */
  from?: DID;
  
  /** Recipient DIDs */
  to?: DID | DID[];
  
  /** Creation timestamp (defaults to now) */
  created_time?: number;
  
  /** Expiration timestamp */
  expires_time?: number;
  
  /** Message body content */
  body?: Record<string, unknown>;
}

/**
 * Helper function to convert a DIDComm message to a TAPMessage
 * @param didcomm Base DIDComm message
 * @returns A TAP message based on the DIDComm message type
 */
export function didcommToTAPMessage(didcomm: DIDCommMessage): TAPMessage {
  // Determine the TAP message type based on the DIDComm type
  const type = didcomm.type as TAPMessageTypeString;
  
  // Create the appropriate TAP message based on the type
  switch (type) {
    case MessageTypeMap.TRANSFER:
      return { ...didcomm, type } as TransferMessage;
    case MessageTypeMap.PAYMENT:
      return { ...didcomm, type } as PaymentMessage;
    case MessageTypeMap.AUTHORIZE:
      return { ...didcomm, type } as AuthorizeMessage;
    case MessageTypeMap.REJECT:
      return { ...didcomm, type } as RejectMessage;
    case MessageTypeMap.SETTLE:
      return { ...didcomm, type } as SettleMessage;
    case MessageTypeMap.CANCEL:
      return { ...didcomm, type } as CancelMessage;
    case MessageTypeMap.REVERT:
      return { ...didcomm, type } as RevertMessage;
    case MessageTypeMap.ADD_AGENTS:
      return { ...didcomm, type } as AddAgentsMessage;
    case MessageTypeMap.REPLACE_AGENT:
      return { ...didcomm, type } as ReplaceAgentMessage;
    case MessageTypeMap.REMOVE_AGENT:
      return { ...didcomm, type } as RemoveAgentMessage;
    case MessageTypeMap.UPDATE_POLICIES:
      return { ...didcomm, type } as UpdatePoliciesMessage;
    case MessageTypeMap.UPDATE_PARTY:
      return { ...didcomm, type } as UpdatePartyMessage;
    case MessageTypeMap.CONFIRM_RELATIONSHIP:
      return { ...didcomm, type } as ConfirmRelationshipMessage;
    case MessageTypeMap.CONNECT:
      return { ...didcomm, type } as ConnectMessage;
    case MessageTypeMap.AUTHORIZATION_REQUIRED:
      return { ...didcomm, type } as AuthorizationRequiredMessage;
    case MessageTypeMap.COMPLETE:
      return { ...didcomm, type } as CompleteMessage;
    default:
      // Default to the base TAPMessage
      return { ...didcomm, type } as TAPMessage;
  }
}