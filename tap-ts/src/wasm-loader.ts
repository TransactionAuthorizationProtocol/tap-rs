/**
 * Utility module to handle WebAssembly initialization
 */

// Import the WASM module
import * as tapWasm from 'tap-wasm';
import __wbg_init from 'tap-wasm';

// Track initialization state
let initialized = false;
let initializationPromise: Promise<void> | null = null;

/**
 * Initialize the WebAssembly module
 * This should be called before any other operations
 */
export function initWasm(): Promise<void> {
  if (initialized) {
    return Promise.resolve();
  }
  
  if (initializationPromise) {
    return initializationPromise;
  }
  
  initializationPromise = new Promise<void>((resolve, reject) => {
    __wbg_init()
      .then(() => {
        // After WASM is loaded, initialize the module
        if (typeof tapWasm.init === 'function') {
          tapWasm.init();
        } else if (typeof tapWasm.init_tap_wasm === 'function') {
          tapWasm.init_tap_wasm();
        } else {
          reject(new Error('Cannot find WASM initialization function'));
          return;
        }
        
        initialized = true;
        resolve();
      })
      .catch(error => {
        reject(new Error(`Failed to initialize WASM module: ${error}`));
      });
  });
  
  return initializationPromise;
}

/**
 * Check if the WASM module has been initialized
 */
export function isInitialized(): boolean {
  return initialized;
}

// Object mapping for message types
export const MessageType = {
  Transfer: tapWasm.MessageType.Transfer,
  Payment: tapWasm.MessageType.PaymentRequest,
  Presentation: tapWasm.MessageType.Presentation,
  Authorize: tapWasm.MessageType.Authorize,
  Reject: tapWasm.MessageType.Reject,
  Settle: tapWasm.MessageType.Settle,
  AddAgents: tapWasm.MessageType.AddAgents,
  ReplaceAgent: tapWasm.MessageType.ReplaceAgent,
  RemoveAgent: tapWasm.MessageType.RemoveAgent,
  UpdatePolicies: tapWasm.MessageType.UpdatePolicies,
  UpdateParty: tapWasm.MessageType.UpdateParty,
  ConfirmRelationship: tapWasm.MessageType.ConfirmRelationship,
  Error: tapWasm.MessageType.Error,
  Unknown: tapWasm.MessageType.Unknown,
  // Add missing types
  Cancel: 6, // Using ReplaceAgent as a temporary substitute 
  Revert: 7  // Using RemoveAgent as a temporary substitute
};

// Re-export the entire module for ease of use
export { tapWasm };