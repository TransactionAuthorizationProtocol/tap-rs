/**
 * TAP-RSVP SDK for TypeScript
 *
 * This is the main entry point for the TAP SDK.
 * It exports all the classes and functions needed to work with TAP.
 */

// Export all types from @taprsvp/types
import * as TapTypes from '@taprsvp/types';
export { TapTypes };

// Export message wrapper classes
import {
  MessageWrapper,
  MessageWrapperOptions,
  TransferWrapper,
  PaymentRequestWrapper,
  ReplyFactory
} from './agent/MessageWrapper';

export {
  MessageWrapper,
  MessageWrapperOptions,
  TransferWrapper,
  PaymentRequestWrapper,
  ReplyFactory
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