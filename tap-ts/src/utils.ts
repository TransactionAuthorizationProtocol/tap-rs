/**
 * Utility functions for TAP agent operations
 */

import { generatePrivateKey as wasmGeneratePrivateKey, generateUUID as wasmGenerateUUID, WasmKeyType } from 'tap-wasm';
import type { KeyType } from './types.js';
import { TapAgentError } from './types.js';

/**
 * Generate a new private key for the specified key type
 * @param keyType - The type of key to generate (default: Ed25519)
 * @returns Hex-encoded private key string
 */
export function generatePrivateKey(keyType: KeyType = 'Ed25519'): string {
  try {
    if (!validateKeyType(keyType)) {
      throw new TapAgentError(`Unsupported key type: ${keyType}`);
    }
    
    return wasmGeneratePrivateKey(keyType);
  } catch (error) {
    if (error instanceof TapAgentError) {
      throw error;
    }
    throw new TapAgentError('Failed to generate private key', 'KEY_GENERATION_ERROR', error as Error);
  }
}

/**
 * Generate a new UUID v4 string
 * @returns UUID v4 string
 */
export function generateUUID(): string {
  try {
    return wasmGenerateUUID();
  } catch (error) {
    throw new TapAgentError('Failed to generate UUID', 'UUID_GENERATION_ERROR', error as Error);
  }
}

/**
 * Validate DID format
 * @param did - DID string to validate
 * @returns True if valid DID format
 */
export function isValidDID(did: string | null | undefined): did is string {
  if (!did || typeof did !== 'string') {
    return false;
  }

  // Basic DID format validation: did:method:method-specific-id
  const didRegex = /^did:[a-z0-9]+:[a-zA-Z0-9._:%/-]+$/;
  return didRegex.test(did);
}

/**
 * Validate private key format (hex string, optionally prefixed with 0x)
 * @param privateKey - Private key string to validate
 * @returns True if valid private key format
 */
export function isValidPrivateKey(privateKey: string | null | undefined): privateKey is string {
  if (!privateKey || typeof privateKey !== 'string') {
    return false;
  }

  // Remove 0x prefix if present
  const cleanKey = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
  
  // Check if it's exactly 64 hex characters (32 bytes)
  const hexRegex = /^[a-fA-F0-9]{64}$/;
  return hexRegex.test(cleanKey);
}

/**
 * Validate key type
 * @param keyType - Key type to validate
 * @returns True if supported key type
 */
export function validateKeyType(keyType: string | null | undefined): keyType is KeyType {
  if (!keyType || typeof keyType !== 'string') {
    return false;
  }

  return keyType === 'Ed25519' || keyType === 'P256' || keyType === 'secp256k1';
}

/**
 * Convert key type string to WASM enum value
 * @param keyType - Key type string
 * @returns WASM key type enum value
 * @internal
 */
export function keyTypeToWasm(keyType: KeyType): number {
  switch (keyType) {
    case 'Ed25519':
      return WasmKeyType.Ed25519;
    case 'P256':
      return WasmKeyType.P256;
    case 'secp256k1':
      return WasmKeyType.Secp256k1;
    default:
      throw new TapAgentError(`Unsupported key type: ${keyType}`);
  }
}

/**
 * Clean and normalize a private key string
 * @param privateKey - Private key string (may have 0x prefix)
 * @returns Clean hex private key string
 */
export function normalizePrivateKey(privateKey: string): string {
  if (!isValidPrivateKey(privateKey)) {
    throw new TapAgentError('Invalid private key format');
  }

  // Remove 0x prefix if present and return lowercase
  return privateKey.startsWith('0x') ? privateKey.slice(2).toLowerCase() : privateKey.toLowerCase();
}

/**
 * Format a timestamp as ISO 8601 string
 * @param timestamp - Unix timestamp in milliseconds
 * @returns ISO 8601 formatted date string
 */
export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toISOString();
}

/**
 * Parse ISO 8601 date string to Unix timestamp
 * @param dateString - ISO 8601 formatted date string
 * @returns Unix timestamp in milliseconds
 */
export function parseTimestamp(dateString: string): number {
  const date = new Date(dateString);
  if (isNaN(date.getTime())) {
    throw new TapAgentError(`Invalid date format: ${dateString}`);
  }
  return date.getTime();
}

/**
 * Check if a message is within the maximum age limit
 * @param createdTime - Message creation timestamp
 * @param maxAgeSeconds - Maximum age in seconds
 * @returns True if message is within age limit
 */
export function isMessageWithinAgeLimit(createdTime: number, maxAgeSeconds: number): boolean {
  const now = Date.now();
  const messageAge = (now - createdTime) / 1000; // Convert to seconds
  return messageAge <= maxAgeSeconds;
}

/**
 * Safely stringify an object, handling circular references
 * @param obj - Object to stringify
 * @returns JSON string
 */
export function safeStringify(obj: unknown): string {
  const seen = new WeakSet();
  
  return JSON.stringify(obj, (_key, value) => {
    if (typeof value === 'object' && value !== null) {
      if (seen.has(value)) {
        throw new TapAgentError('Circular reference detected');
      }
      seen.add(value);
    }
    return value;
  });
}

/**
 * Deep clone an object
 * @param obj - Object to clone
 * @returns Cloned object
 */
export function deepClone<T>(obj: T): T {
  if (obj === null || typeof obj !== 'object') {
    return obj;
  }

  if (obj instanceof Date) {
    return new Date(obj.getTime()) as unknown as T;
  }

  if (Array.isArray(obj)) {
    return obj.map(item => deepClone(item)) as unknown as T;
  }

  const cloned = {} as T;
  for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      cloned[key] = deepClone(obj[key]);
    }
  }

  return cloned;
}

/**
 * Create a result wrapper for operations that may fail
 * @param operation - Function that may throw an error
 * @returns Result wrapper with success/error indication
 */
export async function wrapResult<T>(
  operation: () => Promise<T> | T
): Promise<{ success: true; data: T } | { success: false; error: TapAgentError }> {
  try {
    const data = await operation();
    return { success: true, data };
  } catch (error) {
    const tapError = error instanceof TapAgentError 
      ? error 
      : new TapAgentError('Operation failed', 'UNKNOWN_ERROR', error as Error);
    
    return { success: false, error: tapError };
  }
}

/**
 * Retry an operation with exponential backoff
 * @param operation - Operation to retry
 * @param maxRetries - Maximum number of retry attempts
 * @param baseDelay - Base delay in milliseconds
 * @returns Promise that resolves with operation result
 */
export async function retryWithBackoff<T>(
  operation: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 100
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt === maxRetries) {
        break;
      }

      // Exponential backoff with jitter
      const delay = baseDelay * Math.pow(2, attempt) + Math.random() * 100;
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }

  throw new TapAgentError(
    `Operation failed after ${maxRetries + 1} attempts`,
    'RETRY_EXHAUSTED',
    lastError
  );
}