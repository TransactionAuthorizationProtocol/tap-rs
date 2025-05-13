// Initialize WASM module
import { initWasm } from './wasm-loader';
import { DIDKey, DIDKeyType, createDIDKey, createDIDWeb } from './wasm-loader';

// Try to initialize the WASM module
initWasm().catch(error => {
  console.error('Failed to initialize TAP-WASM module:', error);
});

// Export the TAPAgent class and related interfaces
export { 
  TAPAgent, 
  TAPAgentOptions, 
  MessageHandler, 
  KeyManager, 
  DIDResolver
} from './agent';

// Export DID resolver
export { 
  StandardDIDResolver, 
  ResolverOptions, 
  createResolver, 
  defaultResolver 
} from './did-resolver';

// Export error classes
export {
  TAPError,
  SigningError,
  ValidationError,
  NetworkError,
  ConfigurationError,
  ProcessingError
} from './errors';

// Export message objects
export { TransferObject } from './message-objects/transfer';
export { PaymentObject } from './message-objects/payment';
export { ConnectionObject } from './message-objects/connect';
export { AuthorizeObject } from './message-objects/authorize';
export { RejectObject } from './message-objects/reject';
export { CancelObject } from './message-objects/cancel';
export { SettleObject } from './message-objects/settle';
export { RevertObject } from './message-objects/revert';
export { BaseMessageObject } from './message-objects/base-message';

// Export DID generation functions
export { DIDKey, DIDKeyType, createDIDKey, createDIDWeb };

// Export CLI tools
export * from './cli';

// Re-export types from @taprsvp/types
export * from '@taprsvp/types';