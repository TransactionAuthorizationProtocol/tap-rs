/**
 * TAP-RSVP SDK for TypeScript
 * 
 * This is the main entry point for the TAP SDK.
 * It exports all the classes and functions needed to work with TAP.
 */

// Export message types
export * from './models/types';

// Export message classes
export * from './api/messages/Transfer';
export * from './api/messages/Payment';
export * from './api/messages/Authorize';
export * from './api/messages/Complete';
export * from './api/messages/Settle';
export * from './api/messages/Reject';
export * from './api/messages/Cancel';
export * from './api/messages/Revert';
export * from './api/messages/Connect';

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