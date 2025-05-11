/**
 * TAP-TS Main Module (TAP Specification Implementation)
 * 
 * This is the main entry point for the TAP-TS library based on the
 * Transaction Authorization Protocol specification.
 */

// Export the new TAP implementation
export {
  TAPMessage,
  TAPMessageType,
  TAPMessages,
  SecurityMode,
  MessageTypes,
  type TAPMessageHandler,
  type TAPMessageOptions
} from "./TAPMessage.ts";

export {
  TAPAgent,
  createTAPAgent,
  type TAPAgentOptions
} from "./TAPAgent.ts";

export {
  TAPNode,
  createTAPNode,
  type TAPNodeOptions
} from "./TAPNode.ts";

// Export error types
export {
  TapError,
  ErrorType
} from "./error.ts";

// Export WASM loader
export {
  wasmLoader,
  WasmEvent
} from "./wasm/loader.ts";

// Re-export types from the TAP specification
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
  TAPMessage as ITAPMessage,
} from "../../prds/taips/packages/typescript/src/tap";