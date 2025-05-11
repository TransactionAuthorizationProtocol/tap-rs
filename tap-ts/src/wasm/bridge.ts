/**
 * WASM Bridge
 * 
 * This module provides a bridge between TypeScript and the Rust WASM implementation.
 * It handles initialization and provides wrappers around the WASM functions.
 */

// Dynamically import the appropriate WASM module based on environment
async function loadWasmModule() {
  try {
    if (typeof window !== 'undefined') {
      // Browser environment
      return await import('@taprsvp/tap/wasm');
    } else {
      // Node.js environment
      return await import('@taprsvp/tap/wasm');
    }
  } catch (error) {
    console.error('Failed to load WASM module:', error);
    throw new Error('Failed to initialize TAP WASM module. Make sure the WASM files are correctly built and included in your distribution.');
  }
}

let wasmModule: any = null;

/**
 * Initialize the WASM module
 * This function should be called before using any TAP functionality
 */
export async function initialize() {
  if (wasmModule) return wasmModule;
  
  const module = await loadWasmModule();
  wasmModule = await module.initialize();
  return wasmModule;
}

/**
 * Get the initialized WASM module
 * Throws an error if the module hasn't been initialized
 */
export function getWasmModule() {
  if (!wasmModule) {
    throw new Error('WASM module not initialized. Call initialize() first.');
  }
  return wasmModule;
}

/**
 * Create a new TAP message using the WASM module
 */
export async function createMessage(id: string, type: string, version: string) {
  const wasm = await getWasmModule();
  return new wasm.Message(id, type, version);
}

/**
 * Create a new TAP agent using the WASM module
 */
export async function createAgent(did: string, key: string) {
  const wasm = await getWasmModule();
  return new wasm.TapAgent(did, key);
}

/**
 * Generate a UUID v4 using the WASM module
 */
export async function generateUuid() {
  const wasm = await getWasmModule();
  return wasm.generate_uuid_v4();
}

/**
 * Create a DID key using the WASM module
 */
export async function createDidKey() {
  const wasm = await getWasmModule();
  return wasm.create_did_key();
}