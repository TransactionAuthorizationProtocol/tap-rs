/**
 * Node implementation for TAP-TS
 * 
 * This module provides the TapNode class for TAP-TS.
 */

import { TapError, ErrorType } from './error.ts';
import { NodeConfig, MessageMetadata } from './types.ts';
import { Message, MessageSubscriber } from './message.ts';
import { Agent } from './agent.ts';
import wasmLoader from './wasm/mod.ts';

/**
 * TAP Node class
 * 
 * A Node represents a TAP network participant that can host multiple agents.
 */
export class TapNode {
  private wasmNode: any;
  private agents: Map<string, Agent> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  
  /**
   * Create a new TapNode instance
   * 
   * @param config - Optional node configuration
   */
  constructor(config?: NodeConfig) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: 'WASM module is not loaded',
      });
    }
    
    const module = wasmLoader.getModule();
    let wasmConfig: any;
    
    if (config) {
      wasmConfig = new module.NodeConfig();
      
      if (config.debug) {
        wasmConfig.set_debug(config.debug);
      }
      
      if (config.network?.peers?.length) {
        wasmConfig.set_network(config.network.peers);
      }
    }
    
    this.wasmNode = new module.TapNode(wasmConfig);
    
    // Set up the message handling
    this.setupMessageHandling();
  }
  
  /**
   * Set up message handling
   */
  private setupMessageHandling(): void {
    // The WASM node will call this function when it receives a message
    const messageCallback = async (wasmMessage: any, wasmMetadata: any) => {
      try {
        // Convert the WASM message to a Message instance
        const message = Message.fromJSON(wasmMessage);
        
        // Convert the WASM metadata to a MessageMetadata object
        const metadata: MessageMetadata = typeof wasmMetadata === 'object' ? wasmMetadata : {};
        
        // Notify all subscribers
        for (const subscriber of this.messageSubscribers) {
          try {
            subscriber(message, metadata);
          } catch (error) {
            console.error('Error in message subscriber:', error);
          }
        }
        
        // If the message has a target agent, forward it
        if (metadata.toDid && this.agents.has(metadata.toDid)) {
          const agent = this.agents.get(metadata.toDid)!;
          await agent.processMessage(message, metadata);
        }
      } catch (error) {
        console.error('Error processing message:', error);
      }
    };
    
    // Register the callback with the WASM node
    this.wasmNode.subscribe_to_messages(messageCallback);
  }
  
  /**
   * Register an agent with this node
   * 
   * @param agent - Agent to register
   * @throws {TapError} If the agent is already registered
   */
  registerAgent(agent: Agent): void {
    const did = agent.did;
    
    if (this.agents.has(did)) {
      throw new TapError({
        type: ErrorType.AGENT_ALREADY_EXISTS,
        message: `Agent with DID ${did} is already registered`,
      });
    }
    
    // Register with the WASM node
    try {
      this.wasmNode.register_agent(agent.getWasmAgent());
    } catch (error) {
      throw new TapError({
        type: ErrorType.INVALID_STATE,
        message: 'Failed to register agent with WASM node',
        cause: error,
      });
    }
    
    // Add to our local map
    this.agents.set(did, agent);
  }
  
  /**
   * Unregister an agent from this node
   * 
   * @param did - DID of the agent to unregister
   * @returns True if the agent was unregistered, false if not found
   */
  unregisterAgent(did: string): boolean {
    if (!this.agents.has(did)) {
      return false;
    }
    
    // Unregister from the WASM node
    const result = this.wasmNode.unregister_agent(did);
    
    // Remove from our local map
    this.agents.delete(did);
    
    return result;
  }
  
  /**
   * Get an agent by DID
   * 
   * @param did - DID of the agent to get
   * @returns Agent instance or undefined if not found
   */
  getAgent(did: string): Agent | undefined {
    return this.agents.get(did);
  }
  
  /**
   * Get the DIDs of all registered agents
   * 
   * @returns Array of agent DIDs
   */
  getAgentDIDs(): string[] {
    return Array.from(this.agents.keys());
  }
  
  /**
   * Send a message from one agent to another
   * 
   * @param fromDid - DID of the sender
   * @param toDid - DID of the recipient
   * @param message - Message to send
   * @returns Promise that resolves to the packed message string
   * @throws {TapError} If the sender agent is not found
   */
  async sendMessage(fromDid: string, toDid: string, message: Message): Promise<string> {
    if (!this.agents.has(fromDid)) {
      throw new TapError({
        type: ErrorType.AGENT_NOT_FOUND,
        message: `Agent with DID ${fromDid} not found`,
      });
    }
    
    try {
      const packedMessage = await this.wasmNode.send_message(
        fromDid,
        toDid,
        message.getWasmMessage()
      );
      
      return packedMessage;
    } catch (error) {
      throw new TapError({
        type: ErrorType.MESSAGE_SEND_ERROR,
        message: 'Failed to send message',
        cause: error,
      });
    }
  }
  
  /**
   * Process a received message
   * 
   * @param message - Message to process
   * @param metadata - Optional message metadata
   * @returns Promise that resolves when the message is processed
   */
  async processMessage(message: Message | string, metadata?: MessageMetadata): Promise<void> {
    const wasmMessage = typeof message === 'string' ? message : message.getWasmMessage();
    const wasmMetadata = metadata || {};
    
    await this.wasmNode.process_message(wasmMessage, wasmMetadata);
  }
  
  /**
   * Subscribe to all messages processed by this node
   * 
   * @param subscriber - Subscriber function
   * @returns Unsubscribe function
   */
  subscribeToMessages(subscriber: MessageSubscriber): () => void {
    this.messageSubscribers.add(subscriber);
    
    // Return an unsubscribe function
    return () => {
      this.messageSubscribers.delete(subscriber);
    };
  }
  
  /**
   * Get the underlying WASM node
   * 
   * @returns WASM node
   */
  getWasmNode(): any {
    return this.wasmNode;
  }
}
