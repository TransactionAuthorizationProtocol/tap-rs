/**
 * Agent module for TAP-TS
 * 
 * @module agent
 */

import { Message, MessageHandler, MessageSubscriber, MessageType } from "./message.ts";
import { TapError, ErrorType } from "./error.ts";
import type { AgentOptions, MessageMetadata } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";

/**
 * Generate a random ID
 * 
 * @returns A random ID string
 */
function generateUUID(): string {
  return crypto.randomUUID().replace(/-/g, "");
}

/**
 * Agent class for TAP
 * 
 * This class represents a TAP agent, which is a participant in the TAP network.
 * It can send and receive messages, and handle authorization requests and responses.
 */
export class Agent {
  private wasmAgent: any;
  private _did: string;
  private _id: string;
  private messageHandlers: Map<MessageType, Set<MessageHandler>> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  private isInitialized = false;
  public isReady = false;
  
  /**
   * Get the agent's ID
   */
  get id(): string {
    return this._id;
  }
  
  /**
   * Get the agent's DID
   */
  get did(): string {
    return this._did;
  }

  /**
   * Create a new Agent instance
   * 
   * @param options - Agent options
   */
  constructor(options: AgentOptions) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    this._did = options.did;
    this._id = options.id || `agent_${generateUUID()}`;
    const module = wasmLoader.getModule();
    
    // Create a new WASM agent instance
    this.wasmAgent = new module.Agent(this.did);
    
    // Initialize message handler maps for each message type in the TAP protocol
    this.messageHandlers.set(MessageType.TRANSFER, new Set());
    this.messageHandlers.set(MessageType.PRESENTATION, new Set());
    this.messageHandlers.set(MessageType.AUTHORIZE, new Set());
    this.messageHandlers.set(MessageType.REJECT, new Set());
    this.messageHandlers.set(MessageType.SETTLE, new Set());
    this.messageHandlers.set(MessageType.ADD_AGENTS, new Set());
    this.messageHandlers.set(MessageType.ERROR, new Set());
    
    this.isInitialized = true;
    this.isReady = true;
  }

  /**
   * Register a handler for a specific message type
   * 
   * @param type - Message type to handle
   * @param handler - Handler function to call when a message of the given type is received
   * @returns This agent instance for chaining
   */
  registerHandler(type: MessageType, handler: MessageHandler): this {
    let handlers = this.messageHandlers.get(type);
    if (!handlers) {
      handlers = new Set<MessageHandler>();
      this.messageHandlers.set(type, handlers);
    }
    handlers.add(handler);
    return this;
  }

  /**
   * Subscribe to all messages
   * 
   * @param subscriber - Subscriber function to call for all messages
   * @returns This agent instance for chaining
   */
  subscribe(subscriber: MessageSubscriber): this {
    this.messageSubscribers.add(subscriber);
    return this;
  }

  /**
   * Unsubscribe from all messages
   * 
   * @param subscriber - Subscriber function to remove
   * @returns This agent instance for chaining
   */
  unsubscribe(subscriber: MessageSubscriber): this {
    this.messageSubscribers.delete(subscriber);
    return this;
  }

  /**
   * Check if a handler exists for a specific message type
   * 
   * @param type - Message type to check
   * @returns True if there's at least one handler for the message type
   */
  hasHandler(type: MessageType): boolean {
    const handlers = this.messageHandlers.get(type);
    return handlers !== undefined && handlers.size > 0;
  }

  /**
   * Unregister a handler for a specific message type
   * 
   * @param type - Message type to unregister the handler for
   * @param handler - Handler function to remove
   * @returns This agent instance for chaining
   */
  unregisterHandler(type: MessageType, handler: MessageHandler): this {
    const handlers = this.messageHandlers.get(type);
    if (handlers) {
      handlers.delete(handler);
    }
    return this;
  }
  
  /**
   * Unregister all handlers for a specific message type
   * 
   * @param type - Message type to unregister all handlers for
   * @returns True if handlers were unregistered, false if there were none
   */
  unregisterAllHandlers(type: MessageType): boolean {
    const handlers = this.messageHandlers.get(type);
    if (handlers && handlers.size > 0) {
      handlers.clear();
      return true;
    }
    return false;
  }

  /**
   * Process a message
   * 
   * This method routes the message to the appropriate handlers and subscribers.
   * 
   * @param message - Message to process
   * @param metadata - Optional metadata to pass to handlers
   * @returns Promise that resolves when all handlers have processed the message
   */
  async processMessage(message: Message, metadata?: MessageMetadata): Promise<void> {
    // In a real implementation, we would call the WASM agent to process the message
    // but for now we'll just skip that step since we're focused on removing PING
    try {
      // Skip the WASM processing for now - will need to be implemented later
      // No need to call this.wasmAgent.process_message() here
    } catch (error) {
      console.error("Error processing message:", error);
      throw new TapError({
        type: ErrorType.MESSAGE_PROCESSING_ERROR,
        message: "Error processing message",
        cause: error,
      });
    }
    
    // Then call message handlers for the specific type
    const type = message.getType();
    const handlers = this.messageHandlers.get(type);
    
    if (handlers && handlers.size > 0) {
      const handlerPromises = Array.from(handlers).map(handler => 
        Promise.resolve().then(() => handler(message, metadata))
      );
      
      await Promise.all(handlerPromises);
    }
    
    // Finally, call all subscribers
    if (this.messageSubscribers.size > 0) {
      const subscriberPromises = Array.from(this.messageSubscribers).map(subscriber => 
        Promise.resolve().then(() => subscriber(message, metadata))
      );
      
      await Promise.all(subscriberPromises);
    }
  }

  /**
   * Send a message
   * 
   * @param message - Message to send
   * @param options - Optional send options
   * @returns Promise that resolves when the message has been sent
   */
  async sendMessage(message: Message): Promise<void> {
    // Ensure the message has our DID as the sender
    message.from = this.did;
    
    try {
      // Sign the message
      message.sign(this);
      
      // Send the message using the WASM agent
      await this.wasmAgent.send_message(message.getWasmMessage());
      
      return Promise.resolve();
    } catch (error) {
      console.error("Error sending message:", error);
      throw new TapError({
        type: ErrorType.MESSAGE_SENDING_ERROR,
        message: "Error sending message",
        cause: error,
      });
    }
  }

  /**
   * Sign a message using the underlying WASM agent
   * 
   * @param message - Message or WASM message to sign
   * @returns The signed message
   */
  signMessage(message: Message | any): Message | any {
    try {
      if (message instanceof Message) {
        // If it's a Message instance, get the WASM message
        const wasmMessage = message.getWasmMessage();
        
        // Make sure the from DID is set to this agent's DID
        if (message.from !== this.did) {
          message.from = this.did;
        }
        
        // Sign using the WASM agent
        this.wasmAgent.sign_message(wasmMessage);
        return message;
      } else {
        // Otherwise, assume it's a WASM message
        // Ensure from field is set correctly
        if (message.from_did && message.from_did() !== this.did) {
          message.set_from_did(this.did);
        } else if (message.set_from_did) {
          message.set_from_did(this.did);
        }
        
        this.wasmAgent.sign_message(message);
        return message;
      }
    } catch (error) {
      console.error("Error signing message:", error);
      throw new TapError({
        type: ErrorType.MESSAGE_SIGNING_ERROR,
        message: "Error signing message",
        cause: error,
      });
    }
  }

  /**
   * Create a new message
   * 
   * @param typeOrOptions - Message type or message options object
   * @param options - Additional message options
   * @returns A new Message instance
   */
  createMessage(typeOrOptions: MessageType | { type: MessageType }, options?: Record<string, any>): Message {
    let type: MessageType;
    let combinedOptions: Record<string, any> = {};
    
    if (typeof typeOrOptions === 'object') {
      // If first parameter is an options object with a type
      type = typeOrOptions.type;
      combinedOptions = { ...typeOrOptions };
    } else {
      // If first parameter is just the message type
      type = typeOrOptions;
      combinedOptions = options || {};
    }
    
    // Create a new message with the given type
    const message = new Message({
      type,
      from: this.did,
      ...combinedOptions,
    });
    
    // Handle specific message types
    switch (type) {
      case MessageType.TRANSFER:
        // For Transfer messages, set transfer data if provided
        if (combinedOptions.transferData) {
          message.setTransferData(combinedOptions.transferData);
        } 
        // Or just set the assetId if that's what was provided
        else if (combinedOptions.assetId) {
          message.setAssetId(combinedOptions.assetId);
        }
        break;
        
      case MessageType.AUTHORIZE:
        // For Authorize messages, set authorize data if provided
        if (combinedOptions.authorizeData) {
          message.setAuthorizeData(combinedOptions.authorizeData);
        }
        break;
        
      case MessageType.REJECT:
        // For Reject messages, set reject data if provided
        if (combinedOptions.rejectData) {
          message.setRejectData(combinedOptions.rejectData);
        }
        break;
        
      case MessageType.SETTLE:
        // For Settle messages, set settle data if provided
        if (combinedOptions.settleData) {
          message.setSettleData(combinedOptions.settleData);
        }
        break;
        
      case MessageType.CANCEL:
        // For Cancel messages, set cancel data if provided
        if (combinedOptions.cancelData) {
          message.setCancelData(combinedOptions.cancelData);
        }
        break;
        
      case MessageType.REVERT:
        // For Revert messages, set revert data if provided
        if (combinedOptions.revertData) {
          message.setRevertData(combinedOptions.revertData);
        }
        break;
    }
    
    return message;
  }

  /**
   * Get the agent's DID
   * 
   * @returns The agent's DID
   */
  getDid(): string {
    return this.did;
  }

  /**
   * Get the underlying WASM agent
   * 
   * @returns The WASM agent
   */
  getWasmAgent(): any {
    return this.wasmAgent;
  }
}
