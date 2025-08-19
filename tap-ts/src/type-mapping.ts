/**
 * Type mapping functions between TypeScript and WASM/Rust types
 */

import type { DIDCommMessage, MessageAttachment, TapMessageTypeName } from './types.js';
import { TapAgentError } from './types.js';
import { safeStringify } from './utils.js';

/**
 * WASM message format (uses 'type' field)
 */
interface WasmMessage {
  id: string;
  type: string;
  from?: string;
  to?: string[];
  created_time?: number;
  expires_time?: number;
  thid?: string;
  pthid?: string;
  body: unknown;
  attachments?: MessageAttachment[];
  headers?: Record<string, unknown>;
  [key: string]: unknown;
}

/**
 * Valid TAP message types and their URI mappings
 */
const TAP_MESSAGE_TYPES = new Set<string>([
  'Transfer',
  'Payment', 
  'Authorize',
  'Reject',
  'Settle',
  'Cancel',
  'Revert',
  'Connect',
  'Escrow',
  'Capture',
  'AddAgents',
  'ReplaceAgent',
  'RemoveAgent',
  'UpdatePolicies',
  'UpdateParty',
  'ConfirmRelationship',
  'AuthorizationRequired',
  'Presentation',
  'TrustPing',
  'BasicMessage',
]);

const TAP_MESSAGE_URIS = new Set<string>(
  Array.from(TAP_MESSAGE_TYPES).map(type => `https://tap.rsvp/schema/1.0#${type}`)
);

/**
 * Convert TypeScript DIDComm message to WASM format
 * @param message - TypeScript DIDComm message
 * @returns WASM-compatible message format
 */
export function convertToWasmMessage<T = unknown>(message: DIDCommMessage<T>): WasmMessage {
  try {
    // Validate required fields
    if (!message.id || typeof message.id !== 'string') {
      throw new TapAgentError("Invalid message structure: missing required field 'id'");
    }
    
    if (!message.type || typeof message.type !== 'string') {
      throw new TapAgentError("Invalid message structure: missing required field 'type'");
    }

    if (message.body === undefined) {
      throw new TapAgentError("Invalid message structure: missing required field 'body'");
    }

    // Check for circular references
    try {
      safeStringify(message);
    } catch (error) {
      if (error instanceof TapAgentError && error.message.includes('Circular reference')) {
        throw error;
      }
      throw new TapAgentError('Invalid message structure: unable to serialize', 'SERIALIZATION_ERROR', error as Error);
    }

    // Convert to WASM format
    const wasmMessage: WasmMessage = {
      id: message.id,
      type: message.type,
      body: message.body,
    };

    // Add optional fields
    if (message.from !== undefined) {
      wasmMessage.from = message.from;
    }
    
    if (message.to !== undefined) {
      wasmMessage.to = message.to;
    }
    
    if (message.created_time !== undefined) {
      wasmMessage.created_time = message.created_time;
    }
    
    if (message.expires_time !== undefined) {
      wasmMessage.expires_time = message.expires_time;
    }
    
    if (message.thid !== undefined) {
      wasmMessage.thid = message.thid;
    }
    
    if (message.pthid !== undefined) {
      wasmMessage.pthid = message.pthid;
    }
    
    if (message.attachments !== undefined) {
      wasmMessage.attachments = message.attachments;
    }
    
    if (message.headers !== undefined) {
      wasmMessage.headers = message.headers;
    }

    return wasmMessage;
  } catch (error) {
    if (error instanceof TapAgentError) {
      throw error;
    }
    throw new TapAgentError('Failed to convert message to WASM format', 'CONVERSION_ERROR', error as Error);
  }
}

/**
 * Convert WASM message format to TypeScript DIDComm message
 * @param wasmMessage - WASM message format
 * @returns TypeScript DIDComm message
 */
export function convertFromWasmMessage<T = unknown>(wasmMessage: WasmMessage): DIDCommMessage<T> {
  try {
    // Validate required fields
    if (!wasmMessage.id || typeof wasmMessage.id !== 'string') {
      throw new TapAgentError("Invalid WASM message structure: missing required field 'id'");
    }
    
    if (!wasmMessage.type || typeof wasmMessage.type !== 'string') {
      throw new TapAgentError("Invalid WASM message structure: missing required field 'type'");
    }

    if (wasmMessage.body === undefined) {
      throw new TapAgentError("Invalid WASM message structure: missing required field 'body'");
    }

    // Convert from WASM format
    const message: DIDCommMessage<T> = {
      id: wasmMessage.id,
      type: wasmMessage.type,
      body: wasmMessage.body as T,
    };

    // Add optional fields
    if (wasmMessage.from !== undefined) {
      message.from = wasmMessage.from;
    }
    
    if (wasmMessage.to !== undefined) {
      message.to = wasmMessage.to;
    }
    
    if (wasmMessage.created_time !== undefined) {
      message.created_time = wasmMessage.created_time;
    }
    
    if (wasmMessage.expires_time !== undefined) {
      message.expires_time = wasmMessage.expires_time;
    }
    
    if (wasmMessage.thid !== undefined) {
      message.thid = wasmMessage.thid;
    }
    
    if (wasmMessage.pthid !== undefined) {
      message.pthid = wasmMessage.pthid;
    }
    
    if (wasmMessage.attachments !== undefined) {
      message.attachments = wasmMessage.attachments;
    }
    
    if (wasmMessage.headers !== undefined) {
      message.headers = wasmMessage.headers;
    }

    return message;
  } catch (error) {
    if (error instanceof TapAgentError) {
      throw error;
    }
    throw new TapAgentError('Failed to convert message from WASM format', 'CONVERSION_ERROR', error as Error);
  }
}

/**
 * Validate TAP message type
 * @param messageType - Message type string or URI
 * @returns True if valid TAP message type
 */
export function validateTapMessageType(messageType: string | null | undefined): messageType is TapMessageTypeName | string {
  if (!messageType || typeof messageType !== 'string') {
    return false;
  }

  // Check if it's a simple type name
  if (TAP_MESSAGE_TYPES.has(messageType)) {
    return true;
  }

  // Check if it's a full URI
  if (TAP_MESSAGE_URIS.has(messageType)) {
    return true;
  }

  return false;
}

/**
 * Extract message type name from URI or return as-is if already a type name
 * @param messageType - Message type URI or name
 * @returns Message type name
 */
export function extractMessageTypeName(messageType: string): TapMessageTypeName {
  // If it's already a type name, return as-is
  if (TAP_MESSAGE_TYPES.has(messageType)) {
    return messageType as TapMessageTypeName;
  }

  // If it's a URI, extract the type name
  if (messageType.startsWith('https://tap.rsvp/schema/')) {
    const hashIndex = messageType.lastIndexOf('#');
    if (hashIndex !== -1) {
      const typeName = messageType.substring(hashIndex + 1);
      if (TAP_MESSAGE_TYPES.has(typeName)) {
        return typeName as TapMessageTypeName;
      }
    }
  }

  throw new TapAgentError(`Invalid or unsupported message type: ${messageType}`);
}

/**
 * Convert message type name to full URI
 * @param typeName - Message type name
 * @returns Full message type URI
 */
export function messageTypeToUri(typeName: TapMessageTypeName | string): string {
  if (!validateTapMessageType(typeName)) {
    throw new TapAgentError(`Invalid message type: ${typeName}`);
  }

  // If already a URI, return as-is
  if (typeName.startsWith('https://')) {
    return typeName;
  }

  // Convert type name to URI
  return `https://tap.rsvp/schema/1.0#${typeName}`;
}

/**
 * Validate message structure against TAP schema requirements
 * @param message - Message to validate
 * @returns True if valid, throws error if invalid
 */
export function validateMessageStructure<T = unknown>(message: DIDCommMessage<T>): boolean {
  // Basic structure validation
  if (!message.id || typeof message.id !== 'string') {
    throw new TapAgentError('Message must have a valid id field');
  }

  if (!message.type || typeof message.type !== 'string') {
    throw new TapAgentError('Message must have a valid type field');
  }

  if (!validateTapMessageType(message.type)) {
    throw new TapAgentError(`Unsupported message type: ${message.type}`);
  }

  if (message.body === undefined || message.body === null) {
    throw new TapAgentError('Message must have a body field');
  }

  // Validate optional fields
  if (message.from !== undefined && (typeof message.from !== 'string' || !message.from)) {
    throw new TapAgentError('Message from field must be a non-empty string if provided');
  }

  if (message.to !== undefined) {
    if (!Array.isArray(message.to)) {
      throw new TapAgentError('Message to field must be an array if provided');
    }
    if (message.to.some(recipient => typeof recipient !== 'string' || !recipient)) {
      throw new TapAgentError('Message to field must contain non-empty strings');
    }
  }

  if (message.created_time !== undefined && (typeof message.created_time !== 'number' || message.created_time < 0)) {
    throw new TapAgentError('Message created_time field must be a positive number if provided');
  }

  if (message.thid !== undefined && (typeof message.thid !== 'string' || !message.thid)) {
    throw new TapAgentError('Message thid field must be a non-empty string if provided');
  }

  if (message.pthid !== undefined && (typeof message.pthid !== 'string' || !message.pthid)) {
    throw new TapAgentError('Message pthid field must be a non-empty string if provided');
  }

  // Validate attachments if present
  if (message.attachments !== undefined) {
    if (!Array.isArray(message.attachments)) {
      throw new TapAgentError('Message attachments field must be an array if provided');
    }

    message.attachments.forEach((attachment, index) => {
      if (!attachment.data || typeof attachment.data !== 'object') {
        throw new TapAgentError(`Attachment ${index} must have a data field`);
      }

      if (attachment.data.content === undefined) {
        throw new TapAgentError(`Attachment ${index} data must have a content field`);
      }
    });
  }

  return true;
}

/**
 * Deep merge two message objects, with the second object taking precedence
 * @param base - Base message
 * @param override - Override message properties
 * @returns Merged message
 */
export function mergeMessages<T = unknown>(
  base: DIDCommMessage<T>,
  override: Partial<DIDCommMessage<T>>
): DIDCommMessage<T> {
  const merged = { ...base };

  Object.keys(override).forEach(key => {
    const overrideValue = (override as any)[key];
    if (overrideValue !== undefined) {
      if (Array.isArray(overrideValue)) {
        (merged as any)[key] = [...overrideValue];
      } else if (typeof overrideValue === 'object' && overrideValue !== null) {
        (merged as any)[key] = { ...overrideValue };
      } else {
        (merged as any)[key] = overrideValue;
      }
    }
  });

  return merged;
}