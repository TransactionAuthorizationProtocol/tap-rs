/**
 * Agent module for TAP-TS
 * 
 * @module agent
 */

import { Message, MessageType } from "./message.ts";
import { TapError, ErrorType } from "./error.ts";
import { MessageCallback, MessageMetadata, MessageSubscriber } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";

/**
 * Generate a UUID
 * 
 * @returns A UUID string
 */
function generateUuid(): string {
  return crypto.randomUUID();
}

/**
 * Message handler function type
 */
export type MessageHandler = (message: Message, metadata?: MessageMetadata) => Promise<void>;

/**
 * TAP Agent class
 * 
 * An Agent represents a participant in the TAP network.
 */
export class Agent {
  private _id: string;
  private _did: string;
  private _nickname?: string;
  private messageHandlers: Map<MessageType, MessageCallback> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  /** WASM agent instance */
  private wasmAgent: any;

  /**
   * Create a new Agent instance
   * 
   * @param config - Agent configuration
   */
  constructor(config: any) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: 'WASM module not loaded',
      });
    }
    
    this._did = config.did || this.generateDid();
    this._id = config.id || `agent_${generateUuid()}`;
    this._nickname = config.nickname;
    
    const module = wasmLoader.getModule();
    
    // Create the WASM agent instance
    this.wasmAgent = new module.Agent(config);
    
    // Set up the message handling
    this.setupMessageHandling();
  }
  
  /**
   * Set up message handling
   */
  private setupMessageHandling(): void {
    // The WASM agent will call this function when it receives a message
    const messageCallback = async (wasmMessage: any, wasmMetadata: any) => {
      try {
        // Convert the WASM message to a Message instance
        const message = Message.fromJSON(wasmMessage);
        
        // Convert the WASM metadata to a MessageMetadata object
        const metadata: MessageMetadata = typeof wasmMetadata === 'object' ? wasmMetadata : {};
        
        // Call the appropriate message handler
        const handler = this.messageHandlers.get(message.type);
        if (handler) {
          await handler(message, metadata);
        }
        
        // Notify all subscribers
        for (const subscriber of this.messageSubscribers) {
          try {
            subscriber(message, metadata);
          } catch (error) {
            console.error('Error in message subscriber:', error);
          }
        }
      } catch (error) {
        console.error('Error processing message:', error);
      }
    };
    
    // Register the callback with the WASM agent
    this.wasmAgent.subscribe_to_messages(messageCallback);
  }
  
  /**
   * Get the agent's DID
   * 
   * @returns The agent's DID
   */
  get did(): string {
    return this._did;
  }
  
  /**
   * Get the agent's nickname
   * 
   * @returns The agent's nickname, if set
   */
  get nickname(): string | undefined {
    return this._nickname;
  }
  
  /**
   * Set the agent's nickname
   */
  set nickname(value: string | undefined) {
    this._nickname = value;
  }
  
  /**
   * Get the agent's id
   * 
   * @returns The agent's id
   */
  get id(): string {
    return this._id;
  }
  
  /**
   * Register a message handler for a specific message type
   * 
   * @param messageType - Message type to handle
   * @param handler - Handler function
   */
  registerMessageHandler(messageType: MessageType, handler: MessageHandler): void {
    this.messageHandlers.set(messageType, handler);
    
    // Also register with the WASM agent
    const module = wasmLoader.getModule();
    let wasmMessageType;
    
    switch (messageType) {
      case MessageType.AUTHORIZATION_REQUEST:
        wasmMessageType = module.MessageType.AuthorizationRequest;
        break;
      case MessageType.AUTHORIZATION_RESPONSE:
        wasmMessageType = module.MessageType.AuthorizationResponse;
        break;
      case MessageType.PING:
        wasmMessageType = module.MessageType.Ping;
        break;
      default:
        throw new TapError({
          type: ErrorType.INVALID_ARGUMENT,
          message: `Invalid message type: ${messageType}`,
        });
    }
    
    this.wasmAgent.register_message_handler(wasmMessageType, async (wasmMessage: any) => {
      // This callback is used internally by the WASM agent
      // The actual handling is done in setupMessageHandling
    });
  }
  
  /**
   * Process a received message
   * 
   * @param message - Message to process
   * @param metadata - Optional message metadata
   * @returns A promise that resolves when the message is processed
   */
  async processMessage(message: Message, metadata?: MessageMetadata): Promise<void> {
    const wasmMessage = message.getWasmMessage();
    const wasmMetadata = metadata || {};
    
    await this.wasmAgent.process_message(wasmMessage, wasmMetadata);
  }
  
  /**
   * Handle a message
   * 
   * @param message - Message to handle
   * @param metadata - Optional message metadata
   * @returns Promise that resolves when the message is handled
   */
  async handleMessage(message: Message, metadata?: MessageMetadata): Promise<void> {
    return this.processMessage(message, metadata);
  }
  
  /**
   * Subscribe to all messages processed by this agent
   * 
   * @param subscriber - Subscriber function
   * @returns An unsubscribe function
   */
  subscribeToMessages(subscriber: MessageSubscriber): () => void {
    this.messageSubscribers.add(subscriber);
    
    // Return an unsubscribe function
    return () => {
      this.messageSubscribers.delete(subscriber);
    };
  }
  
  /**
   * Subscribe to messages received by this agent
   * 
   * @param subscriber - Subscriber function
   */
  subscribe(subscriber: MessageSubscriber): void {
    this.messageSubscribers.add(subscriber);
  }
  
  /**
   * Check if the agent is ready
   * 
   * @returns True if the agent is ready, false otherwise
   */
  get isReady(): boolean {
    return true; // Assume the agent is always ready in this implementation
  }
  
  /**
   * Get the underlying WASM agent
   * 
   * @returns WASM agent
   */
  getWasmAgent(): any {
    return this.wasmAgent;
  }
  
  /**
   * Create a new message
   * 
   * @param options - Message options
   * @returns A new Message instance
   */
  createMessage(options: {
    type: MessageType;
    to?: string[];
    data?: Record<string, unknown>;
  }): Message {
    return new Message({
      type: options.type,
      from: this.did,
      to: options.to,
      customData: options.data,
    });
  }
  
  /**
   * Create an authorization request message
   * 
   * @param options - Authorization request options
   * @returns A new Message instance
   */
  createAuthorizationRequest(options: {
    to: string[];
    protocol: string;
    callbackUrl?: string;
    resources?: string[];
    purpose?: string;
    expires?: number;
  }): Message {
    const message = this.createMessage({
      type: MessageType.AUTHORIZATION_REQUEST,
      to: options.to,
    });
    
    // In a real implementation, we would set the authorization request data
    
    return message;
  }
  
  /**
   * Create an authorization response message
   * 
   * @param options - Authorization response options
   * @returns A new Message instance
   */
  createAuthorizationResponse(options: {
    requestId: string;
    approved: boolean;
    reason?: string;
  }): Message {
    const message = this.createMessage({
      type: MessageType.AUTHORIZATION_RESPONSE,
    });
    
    // In a real implementation, we would set the authorization response data
    
    return message;
  }
  
  /**
   * Register a handler for a specific message type
   * 
   * @param type - Message type to handle
   * @param handler - Handler function
   */
  registerHandler(type: MessageType, handler: MessageCallback): void {
    this.messageHandlers.set(type, handler);
  }
  
  /**
   * Unregister a handler for a specific message type
   * 
   * @param type - Message type
   * @returns True if the handler was unregistered, false if it wasn't found
   */
  unregisterHandler(type: MessageType): boolean {
    return this.messageHandlers.delete(type);
  }
  
  /**
   * Check if a handler exists for a specific message type
   * 
   * @param type - Message type to check
   * @returns True if a handler exists for the message type
   */
  hasHandler(type: MessageType): boolean {
    return this.messageHandlers.has(type);
  }
  
  private generateDid(): string {
    // In a real implementation, we would generate a DID
    return 'did:example:1234567890';
  }
}
