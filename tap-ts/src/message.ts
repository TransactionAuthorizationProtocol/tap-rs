/**
 * Message module for TAP-TS
 * 
 * @module message
 */

import { TapError, ErrorType } from "./error.ts";
import type { MessageMetadata } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";


/**
 * Agent involved in a transaction
 */
interface Agent {
  /** DID of the agent */
  "@id": string;
  
  /** Optional role of the agent in the transaction */
  role?: string;
}

/**
 * Transfer message data structure (TAIP-3)
 */
interface TransferData {
  /** Asset ID in CAIP-19 format */
  asset: string;
  
  /** Originator information */
  originator: Agent;
  
  /** Beneficiary information (optional) */
  beneficiary?: Agent;
  
  /** Amount as a decimal string */
  amount: string;
  
  /** Agents involved in the transaction */
  agents: Agent[];
  
  /** Optional settled transaction ID */
  settlementId?: string;
  
  /** Optional memo or note for the transaction */
  memo?: string;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Message types for TAP following the standard specifications
 * These are the official message types as defined in the TAP protocol
 */
export enum MessageType {
  // Core message types based on TAP standard
  TRANSFER = 'https://tap.rsvp/schema/1.0#Transfer',
  REQUEST_PRESENTATION = 'https://tap.rsvp/schema/1.0#RequestPresentation',
  PRESENTATION = 'https://tap.rsvp/schema/1.0#Presentation',
  AUTHORIZE = 'https://tap.rsvp/schema/1.0#Authorize',
  REJECT = 'https://tap.rsvp/schema/1.0#Reject',
  SETTLE = 'https://tap.rsvp/schema/1.0#Settle',
  ADD_AGENTS = 'https://tap.rsvp/schema/1.0#AddAgents',
  ERROR = 'https://tap.rsvp/schema/1.0#Error',
}

/**
 * Security mode for messages
 */
export enum SecurityMode {
  PLAIN = 'plain',
  SIGNED = 'signed',
  ENCRYPTED = 'encrypted',
}

/**
 * Options for creating a new message
 */
export interface MessageOptions {
  /** Message type */
  type: MessageType;
  
  /** Optional message ID (auto-generated if not provided) */
  id?: string;
  
  /** Asset ID in CAIP-19 format (for Transfer messages) */
  assetId?: string;
  
  /** Custom data to include with the message */
  customData?: Record<string, unknown>;
  
  /** Thread ID for tracking message threads */
  threadId?: string;
  
  /** Correlation ID for tracking related messages */
  correlation?: string;
  
  /** Creation timestamp (defaults to now) */
  created?: number;
  
  /** Expiration timestamp */
  expires?: number;
  
  /** Sender DID */
  from?: string;
  
  /** Recipient DIDs */
  to?: string | string[];
  
  /** Security mode for the message */
  securityMode?: SecurityMode;
}

/**
 * TAP Message class
 * 
 * This class represents a TAP message using the DIDComm message format.
 */
export class Message {
  private wasmMessage: any;
  type: MessageType;
  id: string;
  version = "1.0";
  customData?: Record<string, unknown>;
  threadId?: string;
  correlation?: string;
  created: number;
  expires?: number;
  securityMode: SecurityMode = SecurityMode.PLAIN;
  // This is made public to allow the agent to set asset information directly
  _data: Record<string, unknown> = {};

  /**
   * Create a new TAP message
   * 
   * @param options Message options
   */
  constructor(options: MessageOptions) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    // Get the WASM module
    const module = wasmLoader.getModule();
    
    this.type = options.type;
    this.id = options.id || `msg_${generateUuid()}`;
    this.customData = options.customData;
    this.threadId = options.threadId;
    this.correlation = options.correlation;
    this.created = options.created || Date.now();
    this.expires = options.expires;
    this.securityMode = options.securityMode || SecurityMode.PLAIN;
    
    // Create the WASM message
    this.wasmMessage = new module.Message(
      this.id,
      this.type,
      this.version
    );
    
    // Set sender and recipient if provided
    if (options.from) {
      this.from = options.from;
    }
    
    if (options.to) {
      const toArray = Array.isArray(options.to) ? options.to : [options.to];
      if (toArray.length > 0) {
        // Set recipients
        this.to = toArray;
      }
    }
    
    // Set assetId if provided (for Transfer messages)
    if (options.assetId && this.type === MessageType.TRANSFER) {
      this.setAssetId(options.assetId);
    }
  }

  /**
   * Get the message ID
   * 
   * @returns Message ID
   */
  getId(): string {
    return this.wasmMessage.id();
  }

  /**
   * Get the message type
   * 
   * @returns Message type
   */
  getType(): MessageType {
    return this.type;
  }

  /**
   * Get the message version
   * 
   * @returns Message version
   */
  getVersion(): string {
    return this.wasmMessage.version();
  }

  /**
   * Set the asset ID (CAIP-19 format) for Transfer messages
   * 
   * @param assetId - Asset ID in CAIP-19 format (e.g., "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
   * @returns This message for chaining
   * @throws If the message type is not Transfer
   */
  setAssetId(assetId: string): this {
    if (this.type !== MessageType.TRANSFER) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set asset ID on ${this.type} message`,
      });
    }
    
    this._data.asset = assetId;
    return this;
  }
  
  /**
   * Get the asset ID for Transfer messages
   * 
   * @returns The CAIP-19 asset ID or undefined if not set
   */
  getAssetId(): string | undefined {
    return this._data.asset as string | undefined;
  }
  
  /**
   * Set Transfer data according to TAIP-3
   * 
   * @param data - Transfer data object
   * @returns This message for chaining
   * @throws If the message type is not Transfer
   */
  setTransferData(data: TransferData): this {
    if (this.type !== MessageType.TRANSFER) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Transfer data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_transfer_body) {
      try {
        this.wasmMessage.set_transfer_body(data);
      } catch (error) {
        console.warn("Error setting transfer body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get Transfer data for TAIP-3 Transfer messages
   * 
   * @returns TransferData object or undefined if not set or not a Transfer message
   */
  getTransferData(): TransferData | undefined {
    if (this.type !== MessageType.TRANSFER) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_transfer_body) {
      try {
        const wasmTransferData = this.wasmMessage.get_transfer_body();
        if (wasmTransferData) {
          return wasmTransferData as TransferData;
        }
      } catch (error) {
        console.warn("Error getting transfer body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for a Transfer
    if (!this._data.asset || !this._data.originator) {
      return undefined;
    }
    
    return this._data as unknown as TransferData;
  }
  
  /**
   * Set Authorize data according to TAIP-4
   * 
   * @param data - Authorize data object
   * @returns This message for chaining
   * @throws If the message type is not Authorize
   */
  setAuthorizeData(data: { transfer_id: string; note?: string; metadata?: Record<string, unknown> }): this {
    if (this.type !== MessageType.AUTHORIZE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Authorize data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_authorize_body) {
      try {
        this.wasmMessage.set_authorize_body(data);
      } catch (error) {
        console.warn("Error setting authorize body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get Authorize data for TAIP-4 Authorize messages
   * 
   * @returns Authorize data object or undefined if not set or not an Authorize message
   */
  getAuthorizeData(): { transfer_id: string; note?: string; metadata?: Record<string, unknown> } | undefined {
    if (this.type !== MessageType.AUTHORIZE) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_authorize_body) {
      try {
        const wasmAuthorizeData = this.wasmMessage.get_authorize_body();
        if (wasmAuthorizeData) {
          return wasmAuthorizeData as { transfer_id: string; note?: string; metadata?: Record<string, unknown> };
        }
      } catch (error) {
        console.warn("Error getting authorize body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields
    if (!this._data.transfer_id) {
      return undefined;
    }
    
    return this._data as { transfer_id: string; note?: string; metadata?: Record<string, unknown> };
  }
  
  /**
   * Set Reject data according to TAIP-4
   * 
   * @param data - Reject data object
   * @returns This message for chaining
   * @throws If the message type is not Reject
   */
  setRejectData(data: { transfer_id: string; code: string; description: string; note?: string; metadata?: Record<string, unknown> }): this {
    if (this.type !== MessageType.REJECT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Reject data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_reject_body) {
      try {
        this.wasmMessage.set_reject_body(data);
      } catch (error) {
        console.warn("Error setting reject body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get Reject data for TAIP-4 Reject messages
   * 
   * @returns Reject data object or undefined if not set or not a Reject message
   */
  getRejectData(): { transfer_id: string; code: string; description: string; note?: string; metadata?: Record<string, unknown> } | undefined {
    if (this.type !== MessageType.REJECT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_reject_body) {
      try {
        const wasmRejectData = this.wasmMessage.get_reject_body();
        if (wasmRejectData) {
          return wasmRejectData as { transfer_id: string; code: string; description: string; note?: string; metadata?: Record<string, unknown> };
        }
      } catch (error) {
        console.warn("Error getting reject body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields
    if (!this._data.transfer_id || !this._data.code || !this._data.description) {
      return undefined;
    }
    
    return this._data as { transfer_id: string; code: string; description: string; note?: string; metadata?: Record<string, unknown> };
  }
  
  /**
   * Set Settle data according to TAIP-4
   * 
   * @param data - Settle data object
   * @returns This message for chaining
   * @throws If the message type is not Settle
   */
  setSettleData(data: { transfer_id: string; transaction_id: string; transaction_hash?: string; block_height?: number; note?: string; metadata?: Record<string, unknown> }): this {
    if (this.type !== MessageType.SETTLE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Settle data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_settle_body) {
      try {
        this.wasmMessage.set_settle_body(data);
      } catch (error) {
        console.warn("Error setting settle body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get Settle data for TAIP-4 Settle messages
   * 
   * @returns Settle data object or undefined if not set or not a Settle message
   */
  getSettleData(): { transfer_id: string; transaction_id: string; transaction_hash?: string; block_height?: number; note?: string; metadata?: Record<string, unknown> } | undefined {
    if (this.type !== MessageType.SETTLE) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_settle_body) {
      try {
        const wasmSettleData = this.wasmMessage.get_settle_body();
        if (wasmSettleData) {
          return wasmSettleData as { transfer_id: string; transaction_id: string; transaction_hash?: string; block_height?: number; note?: string; metadata?: Record<string, unknown> };
        }
      } catch (error) {
        console.warn("Error getting settle body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields
    if (!this._data.transfer_id || !this._data.transaction_id) {
      return undefined;
    }
    
    return this._data as { transfer_id: string; transaction_id: string; transaction_hash?: string; block_height?: number; note?: string; metadata?: Record<string, unknown> };
  }
  

  // Legacy authorization methods have been removed and replaced with standard TAP types
  // If you need authorization functionality, use the AUTHORIZE and REJECT message types

  /**
   * Get the underlying WASM message
   * 
   * @returns WASM message
   */
  getWasmMessage(): any {
    return this.wasmMessage;
  }

  private _fromDid?: string;
  private _toDids?: string[];

  /**
   * Get the sender DID
   */
  get from(): string | undefined {
    return this._fromDid || this.wasmMessage.from_did();
  }

  /**
   * Set the sender DID
   */
  set from(value: string) {
    this._fromDid = value;
    this.wasmMessage.set_from_did(value);
  }

  /**
   * Get the recipient DIDs
   */
  get to(): string[] | undefined {
    const toDid = this.wasmMessage.to_did();
    return toDid ? [toDid] : this._toDids;
  }

  /**
   * Set the recipient DIDs
   */
  set to(value: string[]) {
    this._toDids = value;
    // Support only the first recipient for now (WASM binding limitation)
    if (value && value.length > 0) {
      this.wasmMessage.set_to_did(value[0]);
    } else {
      this.wasmMessage.set_to_did(null);
    }
  }

  /**
   * Set recipient DIDs for the message (method form)
   * 
   * @param value - DIDs to set
   * @returns This message for chaining
   */
  toRecipients(value: string[]): this {
    this.to = value;
    return this;
  }

  /**
   * Set sender DID for the message (method form)
   * 
   * @param value - DID to set
   * @returns This message for chaining
   */
  fromSender(value: string): this {
    this.from = value;
    return this;
  }

  /**
   * Sign the message using the agent's keys
   * Directly relies on the WASM implementation for signing
   * 
   * @param agent - Agent to sign the message with
   * @returns This message for chaining
   */
  sign(agent: any): this {
    if (this.securityMode === SecurityMode.PLAIN) {
      this.securityMode = SecurityMode.SIGNED;
    }
    
    try {
      // Use the agent's sign_message method which calls the WASM implementation
      agent.signMessage(this.wasmMessage);
    } catch (error) {
      throw new TapError({
        type: ErrorType.MESSAGE_SIGNING_ERROR,
        message: "Failed to sign message using agent",
        cause: error
      });
    }
    
    return this;
  }

  /**
   * Encrypt the message for the specified recipients
   * 
   * @param agent - Agent to encrypt the message with
   * @param recipients - Recipients to encrypt for (defaults to message's to field)
   * @returns Encrypted message
   */
  async encrypt(agent: any, recipients?: string[]): Promise<this> {
    this.securityMode = SecurityMode.ENCRYPTED;
    
    // In the future, implement actual encryption here
    // For now, just mark it as encrypted
    
    return this;
  }

  /**
   * Verify the message signature
   * Uses the WASM implementation for verification
   * 
   * @returns True if the message signature is valid, false if verification fails or isn't available
   */
  verify(): boolean {
    try {
      if (this.wasmMessage.verify_message) {
        return this.wasmMessage.verify_message(true);
      } else {
        console.warn("verify_message not available on WASM message");
        return false; // Security first: if we can't verify, assume it's not valid
      }
    } catch (error) {
      console.error("Error verifying message:", error);
      return false;
    }
  }

  /**
   * Decode an encrypted message
   * 
   * @param agent - Agent to decrypt the message with
   * @returns Decrypted message
   */
  async decrypt(agent: any): Promise<this> {
    // In the future, implement actual decryption here
    // For now, just mark it as plain
    this.securityMode = SecurityMode.PLAIN;
    
    return this;
  }

  /**
   * Create a message from bytes
   * 
   * @param bytes - Message bytes
   * @returns A new Message instance
   */
  static fromBytes(bytes: Uint8Array): Message {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    const wasmMessage = module.Message.fromBytes(bytes);
    
    // Create a new Message instance with basic properties
    const message = new Message({
      id: wasmMessage.id(),
      type: wasmMessage.message_type() as MessageType,
    });
    
    // Replace the WASM message with the one we got from bytes
    message.wasmMessage = wasmMessage;
    
    return message;
  }

  /**
   * Create a message from a JSON string
   * 
   * @param json - Message JSON
   * @returns A new Message instance
   */
  static fromJSON(json: string): Message {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    const wasmMessage = module.Message.fromJson(json);
    
    // Create a new Message instance with basic properties
    const message = new Message({
      id: wasmMessage.id(),
      type: wasmMessage.message_type() as MessageType,
    });
    
    // Replace the WASM message with the one we got from JSON
    message.wasmMessage = wasmMessage;
    
    return message;
  }

  /**
   * Create a message from raw data
   * 
   * @param data - Raw message data
   * @returns A new Message instance
   */
  static fromData(data: unknown): Message {
    if (typeof data === 'string') {
      return Message.fromJSON(data);
    }
    
    if (data instanceof Uint8Array) {
      return Message.fromBytes(data);
    }
    
    // Convert object to JSON string and parse
    return Message.fromJSON(JSON.stringify(data));
  }

  /**
   * Convert to JSON
   * 
   * @returns JSON representation of the message
   */
  toJSON(): Record<string, unknown> {
    const base = {
      id: this.getId(),
      type: this.getType(),
      version: this.getVersion(),
      created: this.created,
      expires: this.expires,
      threadId: this.threadId,
      correlation: this.correlation,
      securityMode: this.securityMode,
    };
    
    // Add from/to if present
    if (this.from) {
      Object.assign(base, { from: this.from });
    }
    
    if (this.to) {
      Object.assign(base, { to: this.to });
    }
    
    // Add message-specific data if present
    if (this._data && Object.keys(this._data).length > 0) {
      Object.assign(base, { data: this._data });
    }
    
    // Add custom data if present
    if (this.customData) {
      Object.assign(base, { customData: this.customData });
    }
    
    return base;
  }

  /**
   * Convert to a string (JSON)
   * 
   * @returns JSON string representation of the message
   */
  toString(): string {
    return JSON.stringify(this.toJSON());
  }

  /**
   * Convert to bytes
   * 
   * @returns Uint8Array containing the message bytes
   */
  toBytes(): Uint8Array {
    return this.wasmMessage.to_bytes();
  }
}

/**
 * Type for message handler functions
 */
export type MessageHandler = (message: Message, metadata?: MessageMetadata) => void | Promise<void>;

/**
 * Type for message subscriber functions
 */
export type MessageSubscriber = (message: Message, metadata?: MessageMetadata) => void | Promise<void>;

/**
 * Generate a random ID
 * 
 * @returns A random ID string
 */
function generateUuid(): string {
  return crypto.randomUUID().replace(/-/g, "");
}
