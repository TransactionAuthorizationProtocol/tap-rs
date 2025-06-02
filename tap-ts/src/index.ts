/**
 * TAP Agent TypeScript API
 * 
 * This package provides a TypeScript wrapper around the TAP WASM bindings
 * for building applications that use the Transaction Authorization Protocol.
 */

// Export the main agent class
export { TAPAgent, TAPAgentOptions, MessageHandler, DIDResolver } from './agent';

// Export the message object classes
export { BaseMessage } from './message-objects/base-message';
export { TransferMessage } from './message-objects/transfer';
export { PaymentMessage } from './message-objects/payment';
export { ConnectMessage } from './message-objects/connect';
export { AuthorizeMessage } from './message-objects/authorize';
export { RejectMessage } from './message-objects/reject';
export { SettleMessage } from './message-objects/settle';
export { CancelMessage } from './message-objects/cancel';
export { RevertMessage } from './message-objects/revert';

// Export WASM-related utilities
export {
  initWasm,
  ensureWasmInitialized,
  isInitialized,
  createAgent,
  createNode,
  generateUuid,
  DIDKeyType,
  WasmTapAgent,
  TapNode,
  MessageType,
  WasmKeyType
} from './wasm-loader';

// Export DID generation functions
export { createDIDKey, createDIDWeb } from './did-generation';

// Export the DID resolver
export { StandardDIDResolver, ResolverOptions } from './did-resolver';

// Export error types
export {
  TAPError,
  ConfigurationError,
  ProcessingError,
  SigningError,
  ValidationError,
  NetworkError
} from './errors';

// Export type definitions
export {
  DID,
  Asset,
  MessageTypeUri,
  Agent,
  Party,
  TAPMessage,
  Transfer,
  Payment,
  Connect,
  Authorize,
  Reject,
  Settle,
  Cancel,
  Revert,
  WasmMessageType
} from './types';