/**
 * UUID generation utilities
 */
import { v4 as uuidv4 } from 'uuid';
import { generateUuid } from '../wasm/bridge';

/**
 * Generates a UUID v4
 * Uses WASM implementation if available, falls back to JavaScript implementation
 * 
 * @returns A new UUID v4 string
 */
export async function generateUUID(): Promise<string> {
  try {
    // Try using the WASM implementation first
    return await generateUuid();
  } catch (error) {
    // Fall back to JavaScript implementation if WASM is not available
    return uuidv4();
  }
}

/**
 * Generates a message ID in the format 'msg_' + UUIDv4
 * 
 * @returns A new message ID string
 */
export async function generateMessageId(): Promise<string> {
  const uuid = await generateUUID();
  return `msg_${uuid}`;
}