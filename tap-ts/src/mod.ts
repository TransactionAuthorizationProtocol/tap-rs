/**
 * TAP-TS: TypeScript WASM Wrapper for the Transaction Authorization Protocol
 * 
 * This module provides the main entry point for the TAP-TS library.
 */

// Re-export error handling
export * from './error.ts';

// Re-export types
export * from './types.ts';

// Re-export message handling
export * from './message.ts';

// Re-export agent
export * from './agent.ts';

// Re-export node
export * from './node.ts';

// Re-export DID resolution
export * from './did/mod.ts';

// Re-export WASM module
export { default as wasmLoader } from './wasm/mod.ts';

// Export version
export const VERSION = '0.1.0';
