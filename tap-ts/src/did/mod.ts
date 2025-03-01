/**
 * DID resolution for TAP-TS
 * 
 * This module provides DID resolution capabilities for TAP-TS.
 * Uses standard npm packages for DID resolution.
 */

// Re-export resolver types, interfaces and functions
export * from './resolver.ts';

// Export the default resolver
export { default as didResolver } from './resolver.ts';
