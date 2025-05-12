/**
 * TAP-RSVP SDK for TypeScript
 *
 * This is the main entry point for the TAP SDK.
 * It exports all the classes and functions needed to work with TAP.
 */

// Export all types from @taprsvp/types
// Only import the types and export them, don't re-export from models/types
import * as TapTypes from '@taprsvp/types';
export { TapTypes };

// Export message implementation classes with different names to avoid conflicts
import { Transfer as TransferImpl } from './api/messages/Transfer';
import { Payment as PaymentImpl } from './api/messages/Payment';
import { Authorize as AuthorizeImpl } from './api/messages/Authorize';
import { Complete as CompleteImpl } from './api/messages/Complete';
import { Settle as SettleImpl } from './api/messages/Settle';
import { Reject as RejectImpl } from './api/messages/Reject';
import { Cancel as CancelImpl } from './api/messages/Cancel';
import { Revert as RevertImpl } from './api/messages/Revert';
import { Connect as ConnectImpl } from './api/messages/Connect';

// Re-export implementation classes with different names
export {
  TransferImpl,
  PaymentImpl,
  AuthorizeImpl,
  CompleteImpl,
  SettleImpl,
  RejectImpl,
  CancelImpl,
  RevertImpl,
  ConnectImpl
};

// Export agent
export * from './agent/TAPAgent';

// Export utilities
export * from './utils/errors';
export * from './utils/uuid';
export * from './utils/date';

// Initialize WASM - this will be used internally by the classes
import { initialize } from './wasm/bridge';

// Initialize WASM module when this module is imported
let initPromise: Promise<any> | null = null;

export function ensureInitialized(): Promise<any> {
  if (!initPromise) {
    initPromise = initialize();
  }
  return initPromise;
}

// Auto-initialize
ensureInitialized().catch(err => {
  console.error('Error initializing TAP WASM module:', err);
});