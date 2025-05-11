/**
 * TAP Message implementation based on the Transaction Authorization Protocol specification
 * 
 * @module TAPMessage
 */

import { TapError, ErrorType } from "./error.ts";
import { wasmLoader } from "./wasm/loader.ts";
import type { 
  DID,
  DIDCommMessage,
  DIDCommReply,
  Transfer,
  Payment,
  Authorize,
  Reject,
  Settle,
  Cancel,
  Revert,
  AddAgents,
  ReplaceAgent,
  RemoveAgent,
  UpdatePolicies,
  UpdateParty,
  ConfirmRelationship,
  Connect,
  AuthorizationRequired,
  Complete,
  TAPMessage as ITAPMessage,
  TransferMessage,
  PaymentMessage,
  AuthorizeMessage,
  RejectMessage,
  SettleMessage,
  CancelMessage,
  RevertMessage,
  AddAgentsMessage,
  ReplaceAgentMessage,
  RemoveAgentMessage,
  UpdatePoliciesMessage,
  UpdatePartyMessage,
  ConfirmRelationshipMessage,
  ConnectMessage,
  AuthorizationRequiredMessage,
  CompleteMessage
} from "../../prds/taips/packages/typescript/src/tap";

/**
 * Security modes for TAP messages
 */
export enum SecurityMode {
  PLAIN = 'plain',
  SIGNED = 'signed',
  ENCRYPTED = 'encrypted',
}

/**
 * TAP Message type string literals
 */
export const MessageTypes = {
  TRANSFER: "https://tap.rsvp/schema/1.0#Transfer",
  PAYMENT: "https://tap.rsvp/schema/1.0#Payment",
  AUTHORIZE: "https://tap.rsvp/schema/1.0#Authorize",
  REJECT: "https://tap.rsvp/schema/1.0#Reject",
  SETTLE: "https://tap.rsvp/schema/1.0#Settle",
  CANCEL: "https://tap.rsvp/schema/1.0#Cancel",
  REVERT: "https://tap.rsvp/schema/1.0#Revert",
  ADD_AGENTS: "https://tap.rsvp/schema/1.0#AddAgents",
  REPLACE_AGENT: "https://tap.rsvp/schema/1.0#ReplaceAgent",
  REMOVE_AGENT: "https://tap.rsvp/schema/1.0#RemoveAgent",
  UPDATE_POLICIES: "https://tap.rsvp/schema/1.0#UpdatePolicies",
  UPDATE_PARTY: "https://tap.rsvp/schema/1.0#UpdateParty",
  CONFIRM_RELATIONSHIP: "https://tap.rsvp/schema/1.0#ConfirmRelationship",
  CONNECT: "https://tap.rsvp/schema/1.0#Connect",
  AUTHORIZATION_REQUIRED: "https://tap.rsvp/schema/1.0#AuthorizationRequired",
  COMPLETE: "https://tap.rsvp/schema/1.0#Complete",
  PRESENTATION: "https://tap.rsvp/schema/1.0#Presentation",
  ERROR: "https://tap.rsvp/schema/1.0#Error",
} as const;

export type TAPMessageType = typeof MessageTypes[keyof typeof MessageTypes];

/**
 * Options for creating a new TAP message
 */
export interface TAPMessageOptions {
  /** Message type */
  type: TAPMessageType;
  
  /** Optional message ID (auto-generated if not provided) */
  id?: string;
  
  /** Asset ID in CAIP-19 format (for Transfer messages) */
  assetId?: string;
  
  /** Custom data to include with the message */
  customData?: Record<string, unknown>;
  
  /** Thread ID for tracking message threads */
  thid?: string;
  
  /** Parent thread ID for nested threads */
  pthid?: string;
  
  /** Creation timestamp (defaults to now) */
  created_time?: number;
  
  /** Expiration timestamp */
  expires_time?: number;
  
  /** Sender DID */
  from?: DID;
  
  /** Recipient DIDs */
  to?: DID | DID[];
  
  /** Security mode for the message */
  securityMode?: SecurityMode;
  
  /** Message body */
  body?: Record<string, unknown>;
}

/**
 * TAP Message class
 * 
 * This class represents a TAP message using the DIDComm message format.
 * It implements the TAP message types defined in the TAP specification.
 */
export class TAPMessage {
  private wasmMessage: any;
  readonly type: TAPMessageType;
  readonly id: string;
  readonly version = "1.0";
  customData?: Record<string, unknown>;
  thid?: string;
  pthid?: string;
  created_time: number;
  expires_time?: number;
  securityMode: SecurityMode = SecurityMode.PLAIN;
  
  // The message body - this holds the actual TAP message content
  body: Record<string, unknown> = {};

  /**
   * Create a new TAP message
   * 
   * @param options Message options
   */
  constructor(options: TAPMessageOptions) {
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
    this.thid = options.thid;
    this.pthid = options.pthid;
    this.created_time = options.created_time || Math.floor(Date.now() / 1000);
    this.expires_time = options.expires_time;
    this.securityMode = options.securityMode || SecurityMode.PLAIN;
    
    // Initialize the WASM message
    this.wasmMessage = new module.Message(
      this.id,
      this.type,
      this.version
    );
    
    // Set body if provided
    if (options.body) {
      this.body = { ...options.body };
      
      // Set the WASM body based on message type
      this.setWasmBody();
    }
    
    // Set asset ID if provided (for Transfer messages)
    if (options.assetId && this.type === MessageTypes.TRANSFER) {
      this.setAssetId(options.assetId);
    }
    
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
  }

  /**
   * Set the WASM body based on the message type
   */
  private setWasmBody(): void {
    if (!this.body || Object.keys(this.body).length === 0) {
      return;
    }
    
    try {
      switch (this.type) {
        case MessageTypes.TRANSFER:
          if (this.wasmMessage.set_transfer_body) {
            this.wasmMessage.set_transfer_body(this.body);
          }
          break;
        case MessageTypes.PAYMENT:
          if (this.wasmMessage.set_payment_request_body) {
            this.wasmMessage.set_payment_request_body(this.body);
          }
          break;
        case MessageTypes.AUTHORIZE:
          if (this.wasmMessage.set_authorize_body) {
            this.wasmMessage.set_authorize_body(this.body);
          }
          break;
        case MessageTypes.REJECT:
          if (this.wasmMessage.set_reject_body) {
            this.wasmMessage.set_reject_body(this.body);
          }
          break;
        case MessageTypes.SETTLE:
          if (this.wasmMessage.set_settle_body) {
            this.wasmMessage.set_settle_body(this.body);
          }
          break;
        case MessageTypes.CANCEL:
          if (this.wasmMessage.set_cancel_body) {
            this.wasmMessage.set_cancel_body(this.body);
          }
          break;
        case MessageTypes.REVERT:
          if (this.wasmMessage.set_revert_body) {
            this.wasmMessage.set_revert_body(this.body);
          }
          break;
        case MessageTypes.ADD_AGENTS:
          if (this.wasmMessage.set_add_agents_body) {
            this.wasmMessage.set_add_agents_body(this.body);
          }
          break;
        case MessageTypes.REPLACE_AGENT:
          if (this.wasmMessage.set_replace_agent_body) {
            this.wasmMessage.set_replace_agent_body(this.body);
          }
          break;
        case MessageTypes.REMOVE_AGENT:
          if (this.wasmMessage.set_remove_agent_body) {
            this.wasmMessage.set_remove_agent_body(this.body);
          }
          break;
        case MessageTypes.UPDATE_POLICIES:
          if (this.wasmMessage.set_update_policies_body) {
            this.wasmMessage.set_update_policies_body(this.body);
          }
          break;
        case MessageTypes.UPDATE_PARTY:
          if (this.wasmMessage.set_update_party_body) {
            this.wasmMessage.set_update_party_body(this.body);
          }
          break;
        case MessageTypes.CONFIRM_RELATIONSHIP:
          if (this.wasmMessage.set_confirm_relationship_body) {
            this.wasmMessage.set_confirm_relationship_body(this.body);
          }
          break;
        case MessageTypes.PRESENTATION:
          if (this.wasmMessage.set_presentation_body) {
            this.wasmMessage.set_presentation_body(this.body);
          }
          break;
        case MessageTypes.ERROR:
          if (this.wasmMessage.set_error_body) {
            this.wasmMessage.set_error_body(this.body);
          }
          break;
        // Add other message types as they become supported in the WASM backend
      }
    } catch (error) {
      console.warn(`Error setting ${this.type} body in WASM`, error);
    }
  }

  /**
   * Set the asset ID for Transfer messages
   * 
   * @param assetId Asset ID in CAIP-19 format
   * @returns This message for chaining
   * @throws If the message type is not Transfer
   */
  setAssetId(assetId: string): this {
    if (this.type !== MessageTypes.TRANSFER) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set asset ID on ${this.type} message`,
      });
    }
    
    this.body.asset = assetId;
    
    // Update the WASM body
    this.setWasmBody();
    
    return this;
  }

  /**
   * Get the asset ID for Transfer messages
   * 
   * @returns The asset ID or undefined if not set
   */
  getAssetId(): string | undefined {
    if (this.type !== MessageTypes.TRANSFER) {
      return undefined;
    }
    
    return this.body.asset as string | undefined;
  }

  /**
   * Set the Transfer message body according to the TAP specification
   * 
   * @param transfer Transfer message body
   * @returns This message for chaining
   * @throws If the message type is not Transfer
   */
  setTransfer(transfer: Transfer): this {
    if (this.type !== MessageTypes.TRANSFER) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Transfer body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...transfer };
    
    // Update the WASM body
    if (this.wasmMessage.set_transfer_body) {
      try {
        this.wasmMessage.set_transfer_body(transfer);
      } catch (error) {
        console.warn("Error setting transfer body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Transfer message body
   * 
   * @returns The Transfer message body or undefined if not a Transfer message
   */
  getTransfer(): Transfer | undefined {
    if (this.type !== MessageTypes.TRANSFER) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_transfer_body) {
      try {
        const wasmTransferData = this.wasmMessage.get_transfer_body();
        if (wasmTransferData && typeof wasmTransferData === 'object') {
          return wasmTransferData as Transfer;
        }
      } catch (error) {
        console.warn("Error getting transfer body from WASM", error);
      }
    }
    
    return this.body as Transfer;
  }

  /**
   * Set the Payment message body according to the TAP specification
   * 
   * @param payment Payment message body
   * @returns This message for chaining
   * @throws If the message type is not Payment
   */
  setPayment(payment: Payment): this {
    if (this.type !== MessageTypes.PAYMENT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Payment body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...payment };
    
    // Update the WASM body
    if (this.wasmMessage.set_payment_request_body) {
      try {
        this.wasmMessage.set_payment_request_body(payment);
      } catch (error) {
        console.warn("Error setting payment request body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Payment message body
   * 
   * @returns The Payment message body or undefined if not a Payment message
   */
  getPayment(): Payment | undefined {
    if (this.type !== MessageTypes.PAYMENT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_payment_request_body) {
      try {
        const wasmPaymentData = this.wasmMessage.get_payment_request_body();
        if (wasmPaymentData && typeof wasmPaymentData === 'object') {
          return wasmPaymentData as Payment;
        }
      } catch (error) {
        console.warn("Error getting payment request body from WASM", error);
      }
    }
    
    return this.body as Payment;
  }

  /**
   * Set the Authorize message body according to the TAP specification
   * 
   * @param authorize Authorize message body
   * @returns This message for chaining
   * @throws If the message type is not Authorize
   */
  setAuthorize(authorize: Authorize): this {
    if (this.type !== MessageTypes.AUTHORIZE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Authorize body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...authorize };
    
    // Update the WASM body
    if (this.wasmMessage.set_authorize_body) {
      try {
        this.wasmMessage.set_authorize_body(authorize);
      } catch (error) {
        console.warn("Error setting authorize body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Authorize message body
   * 
   * @returns The Authorize message body or undefined if not an Authorize message
   */
  getAuthorize(): Authorize | undefined {
    if (this.type !== MessageTypes.AUTHORIZE) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_authorize_body) {
      try {
        const wasmAuthorizeData = this.wasmMessage.get_authorize_body();
        if (wasmAuthorizeData && typeof wasmAuthorizeData === 'object') {
          return wasmAuthorizeData as Authorize;
        }
      } catch (error) {
        console.warn("Error getting authorize body from WASM", error);
      }
    }
    
    return this.body as Authorize;
  }

  /**
   * Set the Reject message body according to the TAP specification
   * 
   * @param reject Reject message body
   * @returns This message for chaining
   * @throws If the message type is not Reject
   */
  setReject(reject: Reject): this {
    if (this.type !== MessageTypes.REJECT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Reject body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...reject };
    
    // Update the WASM body
    if (this.wasmMessage.set_reject_body) {
      try {
        this.wasmMessage.set_reject_body(reject);
      } catch (error) {
        console.warn("Error setting reject body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Reject message body
   * 
   * @returns The Reject message body or undefined if not a Reject message
   */
  getReject(): Reject | undefined {
    if (this.type !== MessageTypes.REJECT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_reject_body) {
      try {
        const wasmRejectData = this.wasmMessage.get_reject_body();
        if (wasmRejectData && typeof wasmRejectData === 'object') {
          return wasmRejectData as Reject;
        }
      } catch (error) {
        console.warn("Error getting reject body from WASM", error);
      }
    }
    
    return this.body as Reject;
  }

  /**
   * Set the Settle message body according to the TAP specification
   * 
   * @param settle Settle message body
   * @returns This message for chaining
   * @throws If the message type is not Settle
   */
  setSettle(settle: Settle): this {
    if (this.type !== MessageTypes.SETTLE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Settle body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...settle };
    
    // Update the WASM body
    if (this.wasmMessage.set_settle_body) {
      try {
        this.wasmMessage.set_settle_body(settle);
      } catch (error) {
        console.warn("Error setting settle body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Settle message body
   * 
   * @returns The Settle message body or undefined if not a Settle message
   */
  getSettle(): Settle | undefined {
    if (this.type !== MessageTypes.SETTLE) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_settle_body) {
      try {
        const wasmSettleData = this.wasmMessage.get_settle_body();
        if (wasmSettleData && typeof wasmSettleData === 'object') {
          return wasmSettleData as Settle;
        }
      } catch (error) {
        console.warn("Error getting settle body from WASM", error);
      }
    }
    
    return this.body as Settle;
  }

  /**
   * Set the Cancel message body according to the TAP specification
   * 
   * @param cancel Cancel message body
   * @returns This message for chaining
   * @throws If the message type is not Cancel
   */
  setCancel(cancel: Cancel): this {
    if (this.type !== MessageTypes.CANCEL) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Cancel body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...cancel };
    
    // Update the WASM body
    if (this.wasmMessage.set_cancel_body) {
      try {
        this.wasmMessage.set_cancel_body(cancel);
      } catch (error) {
        console.warn("Error setting cancel body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Cancel message body
   * 
   * @returns The Cancel message body or undefined if not a Cancel message
   */
  getCancel(): Cancel | undefined {
    if (this.type !== MessageTypes.CANCEL) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_cancel_body) {
      try {
        const wasmCancelData = this.wasmMessage.get_cancel_body();
        if (wasmCancelData && typeof wasmCancelData === 'object') {
          return wasmCancelData as Cancel;
        }
      } catch (error) {
        console.warn("Error getting cancel body from WASM", error);
      }
    }
    
    return this.body as Cancel;
  }

  /**
   * Set the Revert message body according to the TAP specification
   * 
   * @param revert Revert message body
   * @returns This message for chaining
   * @throws If the message type is not Revert
   */
  setRevert(revert: Revert): this {
    if (this.type !== MessageTypes.REVERT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Revert body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...revert };
    
    // Update the WASM body
    if (this.wasmMessage.set_revert_body) {
      try {
        this.wasmMessage.set_revert_body(revert);
      } catch (error) {
        console.warn("Error setting revert body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Revert message body
   * 
   * @returns The Revert message body or undefined if not a Revert message
   */
  getRevert(): Revert | undefined {
    if (this.type !== MessageTypes.REVERT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_revert_body) {
      try {
        const wasmRevertData = this.wasmMessage.get_revert_body();
        if (wasmRevertData && typeof wasmRevertData === 'object') {
          return wasmRevertData as Revert;
        }
      } catch (error) {
        console.warn("Error getting revert body from WASM", error);
      }
    }
    
    return this.body as Revert;
  }

  /**
   * Set the AddAgents message body according to the TAP specification
   * 
   * @param addAgents AddAgents message body
   * @returns This message for chaining
   * @throws If the message type is not AddAgents
   */
  setAddAgents(addAgents: AddAgents): this {
    if (this.type !== MessageTypes.ADD_AGENTS) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set AddAgents body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...addAgents };
    
    // Update the WASM body
    if (this.wasmMessage.set_add_agents_body) {
      try {
        this.wasmMessage.set_add_agents_body(addAgents);
      } catch (error) {
        console.warn("Error setting add_agents body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the AddAgents message body
   * 
   * @returns The AddAgents message body or undefined if not an AddAgents message
   */
  getAddAgents(): AddAgents | undefined {
    if (this.type !== MessageTypes.ADD_AGENTS) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_add_agents_body) {
      try {
        const wasmAddAgentsData = this.wasmMessage.get_add_agents_body();
        if (wasmAddAgentsData && typeof wasmAddAgentsData === 'object') {
          return wasmAddAgentsData as AddAgents;
        }
      } catch (error) {
        console.warn("Error getting add_agents body from WASM", error);
      }
    }
    
    return this.body as AddAgents;
  }

  /**
   * Set the ReplaceAgent message body according to the TAP specification
   * 
   * @param replaceAgent ReplaceAgent message body
   * @returns This message for chaining
   * @throws If the message type is not ReplaceAgent
   */
  setReplaceAgent(replaceAgent: ReplaceAgent): this {
    if (this.type !== MessageTypes.REPLACE_AGENT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set ReplaceAgent body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...replaceAgent };
    
    // Update the WASM body
    if (this.wasmMessage.set_replace_agent_body) {
      try {
        this.wasmMessage.set_replace_agent_body(replaceAgent);
      } catch (error) {
        console.warn("Error setting replace_agent body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the ReplaceAgent message body
   * 
   * @returns The ReplaceAgent message body or undefined if not a ReplaceAgent message
   */
  getReplaceAgent(): ReplaceAgent | undefined {
    if (this.type !== MessageTypes.REPLACE_AGENT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_replace_agent_body) {
      try {
        const wasmReplaceAgentData = this.wasmMessage.get_replace_agent_body();
        if (wasmReplaceAgentData && typeof wasmReplaceAgentData === 'object') {
          return wasmReplaceAgentData as ReplaceAgent;
        }
      } catch (error) {
        console.warn("Error getting replace_agent body from WASM", error);
      }
    }
    
    return this.body as ReplaceAgent;
  }

  /**
   * Set the RemoveAgent message body according to the TAP specification
   * 
   * @param removeAgent RemoveAgent message body
   * @returns This message for chaining
   * @throws If the message type is not RemoveAgent
   */
  setRemoveAgent(removeAgent: RemoveAgent): this {
    if (this.type !== MessageTypes.REMOVE_AGENT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set RemoveAgent body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...removeAgent };
    
    // Update the WASM body
    if (this.wasmMessage.set_remove_agent_body) {
      try {
        this.wasmMessage.set_remove_agent_body(removeAgent);
      } catch (error) {
        console.warn("Error setting remove_agent body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the RemoveAgent message body
   * 
   * @returns The RemoveAgent message body or undefined if not a RemoveAgent message
   */
  getRemoveAgent(): RemoveAgent | undefined {
    if (this.type !== MessageTypes.REMOVE_AGENT) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_remove_agent_body) {
      try {
        const wasmRemoveAgentData = this.wasmMessage.get_remove_agent_body();
        if (wasmRemoveAgentData && typeof wasmRemoveAgentData === 'object') {
          return wasmRemoveAgentData as RemoveAgent;
        }
      } catch (error) {
        console.warn("Error getting remove_agent body from WASM", error);
      }
    }
    
    return this.body as RemoveAgent;
  }

  /**
   * Set the UpdatePolicies message body according to the TAP specification
   * 
   * @param updatePolicies UpdatePolicies message body
   * @returns This message for chaining
   * @throws If the message type is not UpdatePolicies
   */
  setUpdatePolicies(updatePolicies: UpdatePolicies): this {
    if (this.type !== MessageTypes.UPDATE_POLICIES) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set UpdatePolicies body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...updatePolicies };
    
    // Update the WASM body
    if (this.wasmMessage.set_update_policies_body) {
      try {
        this.wasmMessage.set_update_policies_body(updatePolicies);
      } catch (error) {
        console.warn("Error setting update_policies body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the UpdatePolicies message body
   * 
   * @returns The UpdatePolicies message body or undefined if not an UpdatePolicies message
   */
  getUpdatePolicies(): UpdatePolicies | undefined {
    if (this.type !== MessageTypes.UPDATE_POLICIES) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_update_policies_body) {
      try {
        const wasmUpdatePoliciesData = this.wasmMessage.get_update_policies_body();
        if (wasmUpdatePoliciesData && typeof wasmUpdatePoliciesData === 'object') {
          return wasmUpdatePoliciesData as UpdatePolicies;
        }
      } catch (error) {
        console.warn("Error getting update_policies body from WASM", error);
      }
    }
    
    return this.body as UpdatePolicies;
  }

  /**
   * Set the UpdateParty message body according to the TAP specification
   * 
   * @param updateParty UpdateParty message body
   * @returns This message for chaining
   * @throws If the message type is not UpdateParty
   */
  setUpdateParty(updateParty: UpdateParty): this {
    if (this.type !== MessageTypes.UPDATE_PARTY) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set UpdateParty body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...updateParty };
    
    // Update the WASM body
    if (this.wasmMessage.set_update_party_body) {
      try {
        this.wasmMessage.set_update_party_body(updateParty);
      } catch (error) {
        console.warn("Error setting update_party body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the UpdateParty message body
   * 
   * @returns The UpdateParty message body or undefined if not an UpdateParty message
   */
  getUpdateParty(): UpdateParty | undefined {
    if (this.type !== MessageTypes.UPDATE_PARTY) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_update_party_body) {
      try {
        const wasmUpdatePartyData = this.wasmMessage.get_update_party_body();
        if (wasmUpdatePartyData && typeof wasmUpdatePartyData === 'object') {
          return wasmUpdatePartyData as UpdateParty;
        }
      } catch (error) {
        console.warn("Error getting update_party body from WASM", error);
      }
    }
    
    return this.body as UpdateParty;
  }

  /**
   * Set the ConfirmRelationship message body according to the TAP specification
   * 
   * @param confirmRelationship ConfirmRelationship message body
   * @returns This message for chaining
   * @throws If the message type is not ConfirmRelationship
   */
  setConfirmRelationship(confirmRelationship: ConfirmRelationship): this {
    if (this.type !== MessageTypes.CONFIRM_RELATIONSHIP) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set ConfirmRelationship body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...confirmRelationship };
    
    // Update the WASM body
    if (this.wasmMessage.set_confirm_relationship_body) {
      try {
        this.wasmMessage.set_confirm_relationship_body(confirmRelationship);
      } catch (error) {
        console.warn("Error setting confirm_relationship body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the ConfirmRelationship message body
   * 
   * @returns The ConfirmRelationship message body or undefined if not a ConfirmRelationship message
   */
  getConfirmRelationship(): ConfirmRelationship | undefined {
    if (this.type !== MessageTypes.CONFIRM_RELATIONSHIP) {
      return undefined;
    }
    
    // Try to get from WASM first
    if (this.wasmMessage.get_confirm_relationship_body) {
      try {
        const wasmConfirmRelationshipData = this.wasmMessage.get_confirm_relationship_body();
        if (wasmConfirmRelationshipData && typeof wasmConfirmRelationshipData === 'object') {
          return wasmConfirmRelationshipData as ConfirmRelationship;
        }
      } catch (error) {
        console.warn("Error getting confirm_relationship body from WASM", error);
      }
    }
    
    return this.body as ConfirmRelationship;
  }

  /**
   * Set the Connect message body according to the TAP specification
   * 
   * @param connect Connect message body
   * @returns This message for chaining
   * @throws If the message type is not Connect
   */
  setConnect(connect: Connect): this {
    if (this.type !== MessageTypes.CONNECT) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Connect body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...connect };
    
    // If WASM implementation is available, set it
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.set_connect_body) {
      try {
        this.wasmMessage.set_connect_body(connect);
      } catch (error) {
        console.warn("Error setting connect body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Connect message body
   * 
   * @returns The Connect message body or undefined if not a Connect message
   */
  getConnect(): Connect | undefined {
    if (this.type !== MessageTypes.CONNECT) {
      return undefined;
    }
    
    // Try to get from WASM if available
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.get_connect_body) {
      try {
        const wasmConnectData = this.wasmMessage.get_connect_body();
        if (wasmConnectData && typeof wasmConnectData === 'object') {
          return wasmConnectData as Connect;
        }
      } catch (error) {
        console.warn("Error getting connect body from WASM", error);
      }
    }
    
    return this.body as Connect;
  }

  /**
   * Set the AuthorizationRequired message body according to the TAP specification
   * 
   * @param authorizationRequired AuthorizationRequired message body
   * @returns This message for chaining
   * @throws If the message type is not AuthorizationRequired
   */
  setAuthorizationRequired(authorizationRequired: AuthorizationRequired): this {
    if (this.type !== MessageTypes.AUTHORIZATION_REQUIRED) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set AuthorizationRequired body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...authorizationRequired };
    
    // If WASM implementation is available, set it
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.set_authorization_required_body) {
      try {
        this.wasmMessage.set_authorization_required_body(authorizationRequired);
      } catch (error) {
        console.warn("Error setting authorization_required body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the AuthorizationRequired message body
   * 
   * @returns The AuthorizationRequired message body or undefined if not an AuthorizationRequired message
   */
  getAuthorizationRequired(): AuthorizationRequired | undefined {
    if (this.type !== MessageTypes.AUTHORIZATION_REQUIRED) {
      return undefined;
    }
    
    // Try to get from WASM if available
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.get_authorization_required_body) {
      try {
        const wasmAuthorizationRequiredData = this.wasmMessage.get_authorization_required_body();
        if (wasmAuthorizationRequiredData && typeof wasmAuthorizationRequiredData === 'object') {
          return wasmAuthorizationRequiredData as AuthorizationRequired;
        }
      } catch (error) {
        console.warn("Error getting authorization_required body from WASM", error);
      }
    }
    
    return this.body as AuthorizationRequired;
  }

  /**
   * Set the Complete message body according to the TAP specification
   * 
   * @param complete Complete message body
   * @returns This message for chaining
   * @throws If the message type is not Complete
   */
  setComplete(complete: Complete): this {
    if (this.type !== MessageTypes.COMPLETE) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_TYPE,
        message: `Cannot set Complete body on ${this.type} message`,
      });
    }
    
    // Set the body
    this.body = { ...complete };
    
    // If WASM implementation is available, set it
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.set_complete_body) {
      try {
        this.wasmMessage.set_complete_body(complete);
      } catch (error) {
        console.warn("Error setting complete body in WASM", error);
      }
    }
    
    return this;
  }

  /**
   * Get the Complete message body
   * 
   * @returns The Complete message body or undefined if not a Complete message
   */
  getComplete(): Complete | undefined {
    if (this.type !== MessageTypes.COMPLETE) {
      return undefined;
    }
    
    // Try to get from WASM if available
    // Currently not supported but prepared for future implementation
    if (this.wasmMessage.get_complete_body) {
      try {
        const wasmCompleteData = this.wasmMessage.get_complete_body();
        if (wasmCompleteData && typeof wasmCompleteData === 'object') {
          return wasmCompleteData as Complete;
        }
      } catch (error) {
        console.warn("Error getting complete body from WASM", error);
      }
    }
    
    return this.body as Complete;
  }

  private _from?: DID;
  private _to?: DID[];

  /**
   * Get the sender DID
   */
  get from(): DID | undefined {
    return this._from || (this.wasmMessage.from_did ? this.wasmMessage.from_did() : undefined) as DID;
  }

  /**
   * Set the sender DID
   */
  set from(value: DID) {
    this._from = value;
    if (this.wasmMessage.set_from_did) {
      this.wasmMessage.set_from_did(value);
    }
  }

  /**
   * Get the recipient DIDs
   */
  get to(): DID[] | undefined {
    const toDid = this.wasmMessage.to_did ? this.wasmMessage.to_did() : undefined;
    return toDid ? [toDid as DID] : this._to;
  }

  /**
   * Set the recipient DIDs
   */
  set to(value: DID[]) {
    this._to = value;
    // Support only the first recipient for now (WASM binding limitation)
    if (value && value.length > 0 && this.wasmMessage.set_to_did) {
      this.wasmMessage.set_to_did(value[0]);
    } else if (this.wasmMessage.set_to_did) {
      this.wasmMessage.set_to_did(null);
    }
  }

  /**
   * Sign the message using the provided agent
   * 
   * @param agent Agent to sign the message with
   * @returns This message for chaining
   * @throws If signing fails
   */
  sign(agent: any): this {
    if (this.securityMode === SecurityMode.PLAIN) {
      this.securityMode = SecurityMode.SIGNED;
    }
    
    try {
      if (agent.signMessage) {
        agent.signMessage(this.wasmMessage);
      } else {
        throw new Error("Agent does not have signMessage method");
      }
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
   * Verify the message signature
   * 
   * @returns True if the signature is valid, false otherwise
   */
  verify(): boolean {
    try {
      if (this.wasmMessage.verify_message) {
        return this.wasmMessage.verify_message(true);
      } else {
        console.warn("verify_message not available on WASM message");
        return false;
      }
    } catch (error) {
      console.error("Error verifying message:", error);
      return false;
    }
  }

  /**
   * Encrypt the message for the specified recipients
   * 
   * @param agent Agent to encrypt the message with
   * @param recipients Optional recipients to encrypt for (defaults to message's to field)
   * @returns Encrypted message
   */
  async encrypt(agent: any, recipients?: DID[]): Promise<this> {
    // Set the security mode to encrypted
    this.securityMode = SecurityMode.ENCRYPTED;
    
    // In the future, implement actual encryption here
    // This is a placeholder for when the WASM implementation supports encryption
    
    return this;
  }

  /**
   * Decrypt an encrypted message
   * 
   * @param agent Agent to decrypt the message with
   * @returns Decrypted message
   */
  async decrypt(agent: any): Promise<this> {
    // In the future, implement actual decryption here
    // This is a placeholder for when the WASM implementation supports decryption
    
    this.securityMode = SecurityMode.PLAIN;
    
    return this;
  }

  /**
   * Convert the message to a standard DIDComm message
   * 
   * @returns DIDComm message representation of this TAP message
   */
  toDIDCommMessage(): DIDCommMessage {
    const didcomm: DIDCommMessage = {
      id: this.id,
      type: this.type,
      body: this.body,
      created_time: this.created_time,
      from: this.from,
      to: this.to || [],
    };
    
    // Add optional fields if they exist
    if (this.thid) didcomm.thid = this.thid;
    if (this.pthid) didcomm.pthid = this.pthid;
    if (this.expires_time) didcomm.expires_time = this.expires_time;
    
    return didcomm;
  }

  /**
   * Convert to a specific TAP message type based on the current type
   * 
   * @returns The appropriate TAP message type
   */
  toTAPMessage(): ITAPMessage {
    const didcomm = this.toDIDCommMessage();
    
    switch (this.type) {
      case MessageTypes.TRANSFER:
        return { ...didcomm, type: this.type } as TransferMessage;
      case MessageTypes.PAYMENT:
        return { ...didcomm, type: this.type } as PaymentMessage;
      case MessageTypes.AUTHORIZE:
        return { ...didcomm, type: this.type, thid: this.thid! } as AuthorizeMessage;
      case MessageTypes.REJECT:
        return { ...didcomm, type: this.type, thid: this.thid! } as RejectMessage;
      case MessageTypes.SETTLE:
        return { ...didcomm, type: this.type, thid: this.thid! } as SettleMessage;
      case MessageTypes.CANCEL:
        return { ...didcomm, type: this.type, thid: this.thid! } as CancelMessage;
      case MessageTypes.REVERT:
        return { ...didcomm, type: this.type, thid: this.thid! } as RevertMessage;
      case MessageTypes.ADD_AGENTS:
        return { ...didcomm, type: this.type, thid: this.thid! } as AddAgentsMessage;
      case MessageTypes.REPLACE_AGENT:
        return { ...didcomm, type: this.type, thid: this.thid! } as ReplaceAgentMessage;
      case MessageTypes.REMOVE_AGENT:
        return { ...didcomm, type: this.type, thid: this.thid! } as RemoveAgentMessage;
      case MessageTypes.UPDATE_POLICIES:
        return { ...didcomm, type: this.type, thid: this.thid! } as UpdatePoliciesMessage;
      case MessageTypes.UPDATE_PARTY:
        return { ...didcomm, type: this.type, thid: this.thid! } as UpdatePartyMessage;
      case MessageTypes.CONFIRM_RELATIONSHIP:
        return { ...didcomm, type: this.type, thid: this.thid! } as ConfirmRelationshipMessage;
      case MessageTypes.CONNECT:
        return { ...didcomm, type: this.type } as ConnectMessage;
      case MessageTypes.AUTHORIZATION_REQUIRED:
        return { ...didcomm, type: this.type, thid: this.thid! } as AuthorizationRequiredMessage;
      case MessageTypes.COMPLETE:
        return { ...didcomm, type: this.type, thid: this.thid! } as CompleteMessage;
      default:
        return { ...didcomm, type: this.type } as ITAPMessage;
    }
  }

  /**
   * Convert to JSON
   * 
   * @returns JSON representation of the message
   */
  toJSON(): Record<string, unknown> {
    const tapMessage = this.toTAPMessage();
    return tapMessage as Record<string, unknown>;
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
    if (this.wasmMessage && this.wasmMessage.to_bytes) {
      return this.wasmMessage.to_bytes();
    }
    
    // Fallback to JSON serialization
    const json = this.toString();
    return new TextEncoder().encode(json);
  }

  /**
   * Create a message from bytes
   * 
   * @param bytes Message bytes
   * @returns A new TAPMessage instance
   */
  static fromBytes(bytes: Uint8Array): TAPMessage {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    
    try {
      // Try to use the WASM implementation first
      const wasmMessage = module.Message.fromBytes(bytes);
      
      // Create a new TAPMessage instance with the appropriate type
      const message = new TAPMessage({
        id: wasmMessage.id(),
        type: wasmMessage.message_type() as TAPMessageType,
      });
      
      // Replace the WASM message with the one we got from bytes
      message.wasmMessage = wasmMessage;
      
      return message;
    } catch (error) {
      // Fallback to JSON parsing
      const jsonString = new TextDecoder().decode(bytes);
      return TAPMessage.fromJSON(jsonString);
    }
  }

  /**
   * Create a message from a JSON string
   * 
   * @param json JSON string representation of a TAP message
   * @returns A new TAPMessage instance
   */
  static fromJSON(json: string): TAPMessage {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    const module = wasmLoader.getModule();
    
    try {
      // Try to parse as a DIDComm message
      const parsed = JSON.parse(json);
      
      // Validate basic structure
      if (!parsed.type || !parsed.id) {
        throw new Error("Invalid TAP message format");
      }
      
      try {
        // Try to use the WASM implementation first
        const wasmMessage = module.Message.fromJson(json);
        
        // Create a new TAPMessage instance with the appropriate type
        const message = new TAPMessage({
          id: wasmMessage.id(),
          type: wasmMessage.message_type() as TAPMessageType,
        });
        
        // Replace the WASM message with the one we got from JSON
        message.wasmMessage = wasmMessage;
        
        return message;
      } catch (wasmError) {
        // Fallback to manual construction
        const tapMessage = new TAPMessage({
          id: parsed.id,
          type: parsed.type as TAPMessageType,
          from: parsed.from,
          to: parsed.to,
          thid: parsed.thid,
          pthid: parsed.pthid,
          created_time: parsed.created_time,
          expires_time: parsed.expires_time,
          body: parsed.body || {},
        });
        
        return tapMessage;
      }
    } catch (error) {
      throw new TapError({
        type: ErrorType.MESSAGE_PARSE_ERROR,
        message: "Failed to parse TAP message from JSON",
        cause: error,
      });
    }
  }

  /**
   * Create a new TAP message from a DIDComm message
   * 
   * @param didcomm DIDComm message
   * @returns A new TAPMessage instance
   */
  static fromDIDComm(didcomm: DIDCommMessage): TAPMessage {
    if (!didcomm.type || !didcomm.id) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE_FORMAT,
        message: "Invalid DIDComm message format",
      });
    }
    
    const tapMessage = new TAPMessage({
      id: didcomm.id,
      type: didcomm.type as TAPMessageType,
      from: didcomm.from,
      to: didcomm.to,
      thid: didcomm.thid,
      pthid: didcomm.pthid,
      created_time: didcomm.created_time,
      expires_time: didcomm.expires_time,
      body: didcomm.body || {},
    });
    
    return tapMessage;
  }
}

/**
 * Generate a random UUID
 * 
 * @returns A random UUID string with hyphens removed
 */
function generateUuid(): string {
  return crypto.randomUUID().replace(/-/g, "");
}

/**
 * Type for TAP message handler functions
 */
export type TAPMessageHandler = (message: TAPMessage, metadata?: Record<string, unknown>) => void | Promise<void>;

/**
 * Helper factory functions to create specific TAP message types
 */
export const TAPMessages = {
  /**
   * Create a new Transfer message
   * 
   * @param transfer Transfer message body
   * @param options Additional message options
   * @returns A new Transfer message
   */
  createTransfer(transfer: Transfer, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.TRANSFER,
      ...options,
    });
    
    message.setTransfer(transfer);
    
    return message;
  },
  
  /**
   * Create a new Payment message
   * 
   * @param payment Payment message body
   * @param options Additional message options
   * @returns A new Payment message
   */
  createPayment(payment: Payment, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.PAYMENT,
      ...options,
    });
    
    message.setPayment(payment);
    
    return message;
  },
  
  /**
   * Create a new Authorize message
   * 
   * @param authorize Authorize message body
   * @param thid Thread ID of the message being authorized
   * @param options Additional message options
   * @returns A new Authorize message
   */
  createAuthorize(authorize: Authorize, thid: string, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.AUTHORIZE,
      thid,
      ...options,
    });
    
    message.setAuthorize(authorize);
    
    return message;
  },
  
  /**
   * Create a new Reject message
   * 
   * @param reject Reject message body
   * @param thid Thread ID of the message being rejected
   * @param options Additional message options
   * @returns A new Reject message
   */
  createReject(reject: Reject, thid: string, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.REJECT,
      thid,
      ...options,
    });
    
    message.setReject(reject);
    
    return message;
  },
  
  /**
   * Create a new Settle message
   * 
   * @param settle Settle message body
   * @param thid Thread ID of the message being settled
   * @param options Additional message options
   * @returns A new Settle message
   */
  createSettle(settle: Settle, thid: string, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.SETTLE,
      thid,
      ...options,
    });
    
    message.setSettle(settle);
    
    return message;
  },
  
  /**
   * Create a new Cancel message
   * 
   * @param cancel Cancel message body
   * @param thid Thread ID of the message being cancelled
   * @param options Additional message options
   * @returns A new Cancel message
   */
  createCancel(cancel: Cancel, thid: string, options: Partial<TAPMessageOptions> = {}): TAPMessage {
    const message = new TAPMessage({
      type: MessageTypes.CANCEL,
      thid,
      ...options,
    });
    
    message.setCancel(cancel);
    
    return message;
  }
};