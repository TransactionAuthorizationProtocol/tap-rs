/**
 * Agent implementation for TAP-TS
 * 
 * This module provides the Agent class for TAP-TS.
 */

import { TapError, ErrorType } from './error.ts';
import { AgentConfig, MessageType, MessageMetadata } from './types.ts';
import { Message, MessageHandler, MessageSubscriber } from './message.ts';
import wasmLoader from './wasm/mod.ts';

/**
 * TAP Agent class
 * 
 * An Agent represents a participant in the TAP network.
 */
export class Agent {
  private wasmAgent: any;
  private messageHandlers: Map<MessageType, MessageHandler> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  
  /**
   * Create a new Agent instance
   * 
   * @param config - Agent configuration
   */
  constructor(config: AgentConfig) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: 'WASM module is not loaded',
      });
    }
    
    const module = wasmLoader.getModule();
    const wasmConfig = new module.AgentConfig(config.did);
    
    if (config.nickname) {
      wasmConfig.set_nickname(config.nickname);
    }
    
    if (config.debug) {
      wasmConfig.set_debug(config.debug);
    }
    
    this.wasmAgent = new module.Agent(wasmConfig);
    
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
    return this.wasmAgent.did;
  }
  
  /**
   * Get the agent's nickname
   * 
   * @returns The agent's nickname, if set
   */
  get nickname(): string | undefined {
    return this.wasmAgent.nickname;
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
   * Get the underlying WASM agent
   * 
   * @returns WASM agent
   */
  getWasmAgent(): any {
    return this.wasmAgent;
  }
}
