/**
 * Message module for TAP-TS
 * 
 * @module message
 */

import { TapError, ErrorType } from "./error.ts";
import type { MessageMetadata } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";


/**
 * Participant involved in a transaction
 */
interface Participant {
  /** DID of the participant */
  "@id": string;
  
  /** Optional role of the participant in the transaction */
  role?: string;
}

/**
 * Transfer message data structure (TAIP-3)
 */
interface TransferData {
  /** Asset ID in CAIP-19 format */
  asset: string;
  
  /** Originator information */
  originator: Participant;
  
  /** Beneficiary information (optional) */
  beneficiary?: Participant;
  
  /** Amount as a decimal string */
  amount: string;
  
  /** Agents involved in the transaction */
  agents: Participant[];
  
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
  // Identity verification message type (TAIP-8)
  PRESENTATION = 'https://tap.rsvp/schema/1.0#Presentation',
  // Transaction response message types (TAIP-4)
  AUTHORIZE = 'https://tap.rsvp/schema/1.0#Authorize',
  REJECT = 'https://tap.rsvp/schema/1.0#Reject',
  SETTLE = 'https://tap.rsvp/schema/1.0#Settle',
  // Agent management message types (TAIP-5)
  ADD_AGENTS = 'https://tap.rsvp/schema/1.0#AddAgents',
  REPLACE_AGENT = 'https://tap.rsvp/schema/1.0#ReplaceAgent',
  REMOVE_AGENT = 'https://tap.rsvp/schema/1.0#RemoveAgent',
  // Policy management message type (TAIP-7)
  UPDATE_POLICIES = 'https://tap.rsvp/schema/1.0#UpdatePolicies',
  // Party update message type (TAIP-6)
  UPDATE_PARTY = 'https://tap.rsvp/schema/1.0#UpdateParty',
  // Agent relationship confirmation (TAIP-9)
  CONFIRM_RELATIONSHIP = 'https://tap.rsvp/schema/1.0#confirmrelationship',
  // Error message type
  ERROR = 'https://tap.rsvp/schema/1.0#Error',
}

/**
 * Structure for TAIP-4 Reject data
 */
export interface RejectData {
  /** Transfer ID that is being rejected */
  transfer_id: string;
  
  /** Rejection code */
  code: string;
  
  /** Rejection description */
  description: string;
  
  /** Optional note */
  note?: string;
  
  /** Optional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for Error message body
 */
export interface ErrorBody {
  /** Error code */
  code: string;
  
  /** Error description */
  description: string;
  
  /** Original message ID that caused this error, if applicable */
  original_message_id?: string;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for Policy Types (TAIP-7)
 */
export interface Policy {
  /** Policy type */
  '@type': string;
  
  /** Optional list of DIDs this policy applies to */
  from?: string[];
  
  /** Optional list of roles this policy applies to */
  from_role?: string[];
  
  /** Optional list of agent types this policy applies to */
  from_agent?: string[];
  
  /** Optional human-readable purpose for this requirement */
  purpose?: string;

  /** Additional fields may be present based on policy type */
  [key: string]: any;
}

/**
 * Structure for TAIP-5 AddAgents message data
 */
export interface AddAgentsData {
  /** ID of the transfer to add agents to */
  transfer_id: string;
  
  /** Agents to add */
  agents: Participant[];
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for TAIP-5 ReplaceAgent message data
 */
export interface ReplaceAgentData {
  /** ID of the transfer to replace agent in */
  transfer_id: string;
  
  /** DID of the original agent to replace */
  original: string;
  
  /** Replacement agent */
  replacement: Participant;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for TAIP-5 RemoveAgent message data
 */
export interface RemoveAgentData {
  /** ID of the transfer to remove agent from */
  transfer_id: string;
  
  /** DID of the agent to remove */
  agent: string;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for TAIP-7 UpdatePolicies message data
 */
export interface UpdatePoliciesData {
  /** ID of the transfer to update policies for */
  transfer_id: string;
  
  /** Policies to apply */
  policies: Policy[];
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for TAIP-9 ConfirmRelationship message data
 */
export interface ConfirmRelationshipData {
  /** ID of the transfer related to this message */
  transfer_id: string;
  
  /** DID of the agent whose relationship is being confirmed */
  agent_id: string;
  
  /** DID of the entity that the agent acts on behalf of */
  for_id: string;
  
  /** Role of the agent in the transaction (optional) */
  role?: string;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Structure for TAIP-6 UpdateParty message data
 * 
 * The UpdateParty message allows transaction participants to update party information
 * within an existing transfer without creating a new transaction. This is essential
 * for maintaining transaction integrity while allowing flexibility in participant details.
 * 
 * Common use cases include:
 * - Updating account information for a beneficiary
 * - Changing participant roles or credentials
 * - Correcting information after compliance verification
 * 
 * @example
 * ```typescript
 * // Create an UpdateParty message
 * const message = new Message({ type: MessageType.UPDATE_PARTY });
 * 
 * // Set the UpdateParty data
 * message.setUpdatePartyData({
 *   transfer_id: "transfer-123",
 *   party_type: "originator",
 *   party: {
 *     "@id": "did:key:z6MkpDYxrwJw5WoD1o4YVfthJJgZfxrECpW6Da6QCWagRHLx",
 *     role: "business_account",
 *     name: "Acme Corp"
 *   },
 *   note: "Updating role after compliance check"
 * });
 * ```
 */
export interface UpdatePartyData {
  /** ID of the transfer related to this message */
  transfer_id: string;
  
  /** Type of party being updated (e.g., "originator", "beneficiary") */
  party_type: string;
  
  /** Updated participant information */
  party: Participant;
  
  /** Optional note about the update */
  note?: string;
  
  /** Timestamp of the update */
  timestamp?: string;
  
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Security modes for TAP messages
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
  setRejectData(data: RejectData): this {
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
  getRejectData(): RejectData | undefined {
    if (this.type !== MessageType.REJECT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_reject_body) {
      try {
        const wasmRejectData = this.wasmMessage.get_reject_body();
        if (wasmRejectData) {
          return wasmRejectData as RejectData;
        }
      } catch (error) {
        console.warn("Error getting reject body from WASM", error);
      }
    }
    
    if (!this._data || Object.keys(this._data).length === 0) {
      return undefined;
    }
    
    // Construct a properly typed RejectData object from the raw data
    const data = this._data as Record<string, unknown>;
    if (
      typeof data.transfer_id === 'string' &&
      typeof data.code === 'string' &&
      typeof data.description === 'string'
    ) {
      const rejectData: RejectData = {
        transfer_id: data.transfer_id,
        code: data.code,
        description: data.description
      };
      
      // Add optional fields if present
      if (typeof data.note === 'string') {
        rejectData.note = data.note;
      }
      
      if (data.metadata && typeof data.metadata === 'object') {
        rejectData.metadata = data.metadata as Record<string, unknown>;
      }
      
      return rejectData;
    }
    
    return undefined;
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

  /**
   * Set Error data 
   * 
   * @param data - Error data object
   * @returns This message for chaining
   * @throws If the message type is not Error
   */
  setErrorData(data: ErrorBody): this {
    if (this.type !== MessageType.ERROR) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Error data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_error_body) {
      try {
        this.wasmMessage.set_error_body(data);
      } catch (error) {
        console.warn("Error setting error body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get Error data
   * 
   * @returns Error data object or undefined if not set or not an Error message
   */
  getErrorData(): ErrorBody | undefined {
    if (this.type !== MessageType.ERROR) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_error_body) {
      try {
        const wasmErrorData = this.wasmMessage.get_error_body();
        if (wasmErrorData) {
          return wasmErrorData as ErrorBody;
        }
      } catch (error) {
        console.warn("Error getting error body from WASM", error);
      }
    }
    
    if (!this._data || Object.keys(this._data).length === 0) {
      return undefined;
    }
    
    // Construct a properly typed ErrorBody object from the raw data
    const data = this._data as Record<string, unknown>;
    if (
      typeof data.code === 'string' &&
      typeof data.description === 'string'
    ) {
      const errorBody: ErrorBody = {
        code: data.code,
        description: data.description
      };
      
      // Add optional fields if present
      if (typeof data.original_message_id === 'string') {
        errorBody.original_message_id = data.original_message_id;
      }
      
      if (data.metadata && typeof data.metadata === 'object') {
        errorBody.metadata = data.metadata as Record<string, unknown>;
      }
      
      return errorBody;
    }
    
    return undefined;
  }

  /**
   * Set AddAgents data according to TAIP-5
   * 
   * @param data - AddAgents data object
   * @returns This message for chaining
   * @throws If the message type is not AddAgents
   */
  setAddAgentsData(data: AddAgentsData): this {
    if (this.type !== MessageType.ADD_AGENTS) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set AddAgents data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_add_agents_body) {
      try {
        this.wasmMessage.set_add_agents_body(data);
      } catch (error) {
        console.warn("Error setting add_agents body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get AddAgents data for TAIP-5 AddAgents messages
   * 
   * @returns AddAgentsData object or undefined if not set or not an AddAgents message
   */
  getAddAgentsData(): AddAgentsData | undefined {
    if (this.type !== MessageType.ADD_AGENTS) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_add_agents_body) {
      try {
        const wasmAddAgentsData = this.wasmMessage.get_add_agents_body();
        if (wasmAddAgentsData) {
          return wasmAddAgentsData as AddAgentsData;
        }
      } catch (error) {
        console.warn("Error getting add_agents body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for AddAgents
    if (!this._data.transfer_id || !this._data.agents) {
      return undefined;
    }
    
    return this._data as unknown as AddAgentsData;
  }

  /**
   * Set ReplaceAgent data according to TAIP-5
   * 
   * @param data - ReplaceAgent data object
   * @returns This message for chaining
   * @throws If the message type is not ReplaceAgent
   */
  setReplaceAgentData(data: ReplaceAgentData): this {
    if (this.type !== MessageType.REPLACE_AGENT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set ReplaceAgent data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_replace_agent_body) {
      try {
        this.wasmMessage.set_replace_agent_body(data);
      } catch (error) {
        console.warn("Error setting replace_agent body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get ReplaceAgent data for TAIP-5 ReplaceAgent messages
   * 
   * @returns ReplaceAgentData object or undefined if not set or not a ReplaceAgent message
   */
  getReplaceAgentData(): ReplaceAgentData | undefined {
    if (this.type !== MessageType.REPLACE_AGENT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_replace_agent_body) {
      try {
        const wasmReplaceAgentData = this.wasmMessage.get_replace_agent_body();
        if (wasmReplaceAgentData) {
          return wasmReplaceAgentData as ReplaceAgentData;
        }
      } catch (error) {
        console.warn("Error getting replace_agent body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for ReplaceAgent
    if (!this._data.transfer_id || !this._data.original || !this._data.replacement) {
      return undefined;
    }
    
    return this._data as unknown as ReplaceAgentData;
  }

  /**
   * Set RemoveAgent data according to TAIP-5
   * 
   * @param data - RemoveAgent data object
   * @returns This message for chaining
   * @throws If the message type is not RemoveAgent
   */
  setRemoveAgentData(data: RemoveAgentData): this {
    if (this.type !== MessageType.REMOVE_AGENT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set RemoveAgent data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_remove_agent_body) {
      try {
        this.wasmMessage.set_remove_agent_body(data);
      } catch (error) {
        console.warn("Error setting remove_agent body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get RemoveAgent data for TAIP-5 RemoveAgent messages
   * 
   * @returns RemoveAgentData object or undefined if not set or not a RemoveAgent message
   */
  getRemoveAgentData(): RemoveAgentData | undefined {
    if (this.type !== MessageType.REMOVE_AGENT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_remove_agent_body) {
      try {
        const wasmRemoveAgentData = this.wasmMessage.get_remove_agent_body();
        if (wasmRemoveAgentData) {
          return wasmRemoveAgentData as RemoveAgentData;
        }
      } catch (error) {
        console.warn("Error getting remove_agent body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for RemoveAgent
    if (!this._data.transfer_id || !this._data.agent) {
      return undefined;
    }
    
    return this._data as unknown as RemoveAgentData;
  }

  /**
   * Set UpdatePolicies data according to TAIP-7
   * 
   * @param data - UpdatePolicies data object
   * @returns This message for chaining
   * @throws If the message type is not UpdatePolicies
   */
  setUpdatePoliciesData(data: UpdatePoliciesData): this {
    if (this.type !== MessageType.UPDATE_POLICIES) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set UpdatePolicies data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_update_policies_body) {
      try {
        this.wasmMessage.set_update_policies_body(data);
      } catch (error) {
        console.warn("Error setting update_policies body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get UpdatePolicies data for TAIP-7 UpdatePolicies messages
   * 
   * @returns UpdatePoliciesData object or undefined if not set or not an UpdatePolicies message
   */
  getUpdatePoliciesData(): UpdatePoliciesData | undefined {
    if (this.type !== MessageType.UPDATE_POLICIES) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_update_policies_body) {
      try {
        const wasmUpdatePoliciesData = this.wasmMessage.get_update_policies_body();
        if (wasmUpdatePoliciesData) {
          return wasmUpdatePoliciesData as UpdatePoliciesData;
        }
      } catch (error) {
        console.warn("Error getting update_policies body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for UpdatePolicies
    if (!this._data.transfer_id || !this._data.policies) {
      return undefined;
    }
    
    return this._data as unknown as UpdatePoliciesData;
  }
  
  /**
   * Set ConfirmRelationship data for TAIP-9 ConfirmRelationship messages
   * 
   * @param data The ConfirmRelationship data to set
   * @returns this (chainable)
   * @throws {TapError} If the message type is not CONFIRM_RELATIONSHIP
   */
  setConfirmRelationshipData(data: ConfirmRelationshipData): this {
    if (this.type !== MessageType.CONFIRM_RELATIONSHIP) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set ConfirmRelationship data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_confirm_relationship_body) {
      try {
        this.wasmMessage.set_confirm_relationship_body(data);
      } catch (error) {
        console.warn("Error setting confirm_relationship body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get ConfirmRelationship data for TAIP-9 ConfirmRelationship messages
   * 
   * @returns ConfirmRelationshipData object or undefined if not set or not a ConfirmRelationship message
   */
  getConfirmRelationshipData(): ConfirmRelationshipData | undefined {
    if (this.type !== MessageType.CONFIRM_RELATIONSHIP) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_confirm_relationship_body) {
      try {
        const wasmConfirmRelationshipData = this.wasmMessage.get_confirm_relationship_body();
        if (wasmConfirmRelationshipData) {
          return wasmConfirmRelationshipData as ConfirmRelationshipData;
        }
      } catch (error) {
        console.warn("Error getting confirm_relationship body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for ConfirmRelationship
    if (!this._data.transfer_id || !this._data.agent_id || !this._data.for) {
      return undefined;
    }
    
    return this._data as unknown as ConfirmRelationshipData;
  }

  /**
   * Set UpdateParty data according to TAIP-6
   * 
   * This method allows setting data for an UpdateParty message (TAIP-6), which
   * enables participants to update party information in an existing transfer.
   * The UpdateParty message is critical for scenarios where participant details
   * need to change after a transfer has been initiated.
   * 
   * @param data - UpdateParty data object containing required fields:
   *   - transfer_id: Identifier of the transfer to update
   *   - party_type: Type of the party being updated (e.g., "originator", "beneficiary")
   *   - party: The updated participant information
   *   - note: (Optional) A note explaining the reason for the update
   *   - timestamp: (Optional) When the update was made
   *   - metadata: (Optional) Additional data as key-value pairs
   * 
   * @returns This message for chaining
   * @throws {TapError} If the message type is not UPDATE_PARTY
   * 
   * @example
   * ```typescript
   * const message = new Message({ type: MessageType.UPDATE_PARTY });
   * message.setUpdatePartyData({
   *   transfer_id: "transfer-abc123",
   *   party_type: "beneficiary",
   *   party: {
   *     "@id": "did:key:z6MkrF9z7GeZZuUXR5tUFoCnEKJxBtChfYNNVn4TvXKBi6XQ",
   *     role: "customer",
   *     name: "Alice Smith",
   *     account: {
   *       id: "GB29NWBK60161331926819",
   *       bank_id: "NWBKGB2L"
   *     }
   *   },
   *   note: "Updated account details after verification"
   * });
   * ```
   */
  setUpdatePartyData(data: UpdatePartyData): this {
    if (this.type !== MessageType.UPDATE_PARTY) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set UpdateParty data on ${this.type} message`,
      });
    }
    
    // Store the data
    Object.assign(this._data, data);
    
    // Use the WASM implementation if available
    if (this.wasmMessage.set_update_party_body) {
      try {
        this.wasmMessage.set_update_party_body(data);
      } catch (error) {
        console.warn("Error setting update_party body in WASM", error);
      }
    }
    
    return this;
  }
  
  /**
   * Get UpdateParty data for TAIP-6 UpdateParty messages
   * 
   * Retrieves the UpdateParty data from a message, which includes information
   * about party updates in an existing transfer. This method first attempts
   * to retrieve data from the WASM implementation if available, then falls back
   * to the TypeScript implementation.
   * 
   * @returns UpdatePartyData object or undefined if not set or not an UpdateParty message
   * 
   * @example
   * ```typescript
   * // Retrieve data from an UpdateParty message
   * const updatePartyData = message.getUpdatePartyData();
   * if (updatePartyData) {
   *   console.log(`Updating ${updatePartyData.party_type} in transfer ${updatePartyData.transfer_id}`);
   *   console.log(`New party ID: ${updatePartyData.party["@id"]}`);
   *   if (updatePartyData.note) {
   *     console.log(`Update reason: ${updatePartyData.note}`);
   *   }
   * }
   * ```
   */
  getUpdatePartyData(): UpdatePartyData | undefined {
    if (this.type !== MessageType.UPDATE_PARTY) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_update_party_body) {
      try {
        const wasmUpdatePartyData = this.wasmMessage.get_update_party_body();
        if (wasmUpdatePartyData) {
          return wasmUpdatePartyData as UpdatePartyData;
        }
      } catch (error) {
        console.warn("Error getting update_party body from WASM", error);
      }
    }
    
    // Check if we have the minimum required fields for UpdateParty
    if (!this._data.transfer_id || !this._data.party_type || !this._data.party) {
      return undefined;
    }
    
    return this._data as unknown as UpdatePartyData;
  }

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
