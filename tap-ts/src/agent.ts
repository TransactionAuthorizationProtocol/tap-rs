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
 * Agent class for TAP
 * 
 * This class represents a TAP agent, which is a participant in the TAP network.
 * It can send and receive messages, and handle authorization requests and responses.
 */
export class Agent {
  private wasmAgent: any;
  private did: string;
  private messageHandlers: Map<MessageType, Set<MessageHandler>> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  private isInitialized = false;

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
    
    this.did = options.did;
    const module = wasmLoader.getModule();
    
    // Create a new WASM agent instance
    this.wasmAgent = new module.Agent(this.did);
    
    // Initialize message handler maps for each message type
    this.messageHandlers.set(MessageType.PING, new Set());
    this.messageHandlers.set(MessageType.PONG, new Set());
    this.messageHandlers.set(MessageType.AUTHORIZATION_REQUEST, new Set());
    this.messageHandlers.set(MessageType.AUTHORIZATION_RESPONSE, new Set());
    
    this.isInitialized = true;
  }

  /**
   * Register a handler for a specific message type
   * 
   * @param type - Message type to handle
   * @param handler - Handler function to call when a message of the given type is received
   * @returns This agent instance for chaining
   */
  registerHandler(type: MessageType, handler: MessageHandler): this {
    const handlers = this.messageHandlers.get(type);
    if (handlers) {
      handlers.add(handler);
    }
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
   * Process a message
   * 
   * This method routes the message to the appropriate handlers and subscribers.
   * 
   * @param message - Message to process
   * @param metadata - Optional metadata to pass to handlers
   * @returns Promise that resolves when all handlers have processed the message
   */
  async processMessage(message: Message, metadata?: MessageMetadata): Promise<void> {
    // First call the WASM agent to process the message
    try {
      this.wasmAgent.process_message(message.getWasmMessage());
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
    message.from(this.did);
    
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
   * Sign a message
   * 
   * @param message - Message or WASM message to sign
   * @returns The signed message
   */
  signMessage(message: Message | any): Message | any {
    try {
      if (message instanceof Message) {
        // If it's a Message instance, get the WASM message
        const wasmMessage = message.getWasmMessage();
        this.wasmAgent.sign_message(wasmMessage);
        return message;
      } else {
        // Otherwise, assume it's a WASM message
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
   * @param type - Message type
   * @param options - Additional message options
   * @returns A new Message instance
   */
  createMessage(type: MessageType, options?: Record<string, any>): Message {
    return new Message({
      type,
      from: this.did,
      ...options,
    });
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
