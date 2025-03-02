/**
 * Node implementation for TAP-TS
 * 
 * This module provides the TapNode class for TAP-TS.
 */

/**
 * TAP Node implementation
 * 
 * @module node
 */

import { Agent } from "./agent.ts";
import { Message } from "./message.ts";
import { TapError, ErrorType } from "./error.ts";
import { MessageMetadata, MessageSubscriber } from "./types.ts";
import { wasmLoader } from "./wasm/loader.ts";

/**
 * TapNode configuration interface
 */
interface NodeConfig {
  /**
   * Node ID
   */
  id?: string;
  
  /**
   * Debugging flag
   */
  debug?: boolean;
}

/**
 * TAP Node class
 * 
 * A Node represents a participant in the TAP network that can host multiple agents.
 */
export class TapNode {
  private _id: string;
  private agents: Map<string, Agent> = new Map();
  private didToIdMap: Map<string, string> = new Map();
  private messageSubscribers: Set<MessageSubscriber> = new Set();
  
  /**
   * Create a new TapNode instance
   * 
   * @param config - Node configuration
   */
  constructor(config: NodeConfig = {}) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    this._id = config.id || `node_${Date.now()}`;
  }
  
  /**
   * Get the node's ID
   * 
   * @returns The node's ID
   */
  get id(): string {
    return this._id;
  }
  
  /**
   * Register an agent with this node
   * 
   * @param agent - Agent to register
   * @throws If an agent with the same ID is already registered
   */
  registerAgent(agent: Agent): void {
    if (this.agents.has(agent.id)) {
      throw new TapError({
        type: ErrorType.AGENT_ALREADY_REGISTERED,
        message: `Agent with ID ${agent.id} is already registered`,
      });
    }
    
    this.agents.set(agent.id, agent);
    this.didToIdMap.set(agent.did, agent.id);
    
    // Subscribe to the agent's messages
    agent.subscribe((message, metadata) => {
      this.broadcastMessage(message, metadata);
    });
  }
  
  /**
   * Unregister an agent from this node
   * 
   * @param agentId - ID of the agent to unregister
   * @returns True if the agent was unregistered, false if it wasn't found
   */
  unregisterAgent(agentId: string): boolean {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new TapError({
        type: ErrorType.AGENT_NOT_FOUND,
        message: `Agent with ID ${agentId} not found`,
      });
    }
    
    this.agents.delete(agentId);
    this.didToIdMap.delete(agent.did);
    
    return true;
  }
  
  /**
   * Get all registered agents
   * 
   * @returns Map of agent IDs to agents
   */
  getAgents(): Map<string, Agent> {
    return this.agents;
  }
  
  /**
   * Get DIDs of all registered agents
   * 
   * @returns Array of DIDs
   */
  getAgentDIDs(): string[] {
    return Array.from(this.didToIdMap.keys());
  }
  
  /**
   * Find an agent by its DID
   * 
   * @param did - DID to search for
   * @returns Agent with the matching DID, or undefined if not found
   */
  getAgentByDID(did: string): Agent | undefined {
    const agentId = this.didToIdMap.get(did);
    if (!agentId) {
      return undefined;
    }
    
    return this.agents.get(agentId);
  }
  
  /**
   * Send a message from one agent to another
   * 
   * @param fromDID - DID of the sending agent
   * @param toDID - DID of the receiving agent
   * @param message - Message to send
   * @throws If the sending or receiving agent is not found
   */
  async sendMessage(fromDID: string, toDID: string, message: Message): Promise<void> {
    const fromAgent = this.getAgentByDID(fromDID);
    if (!fromAgent) {
      throw new TapError({
        type: ErrorType.AGENT_NOT_FOUND,
        message: `Agent with DID ${fromDID} not found`,
      });
    }
    
    const toAgent = this.getAgentByDID(toDID);
    if (!toAgent) {
      throw new TapError({
        type: ErrorType.AGENT_NOT_FOUND,
        message: `Agent with DID ${toDID} not found`,
      });
    }
    
    // Set message from/to if not already set
    if (!message.from) {
      message.from = fromDID;
    }
    
    if (!message.to || message.to.length === 0) {
      message.to = [toDID];
    }
    
    // Deliver the message
    await this.handleAgentMessage(fromAgent, toAgent, message);
  }
  
  /**
   * Subscribe to messages
   * 
   * @param subscriber - Subscriber function
   * @returns True if the subscriber was added, false if it was already subscribed
   */
  subscribe(subscriber: MessageSubscriber): boolean {
    if (this.messageSubscribers.has(subscriber)) {
      return false;
    }
    
    this.messageSubscribers.add(subscriber);
    return true;
  }
  
  /**
   * Unsubscribe from messages
   * 
   * @param subscriber - Subscriber function
   * @returns True if the subscriber was removed, false if it wasn't found
   */
  unsubscribe(subscriber: MessageSubscriber): boolean {
    return this.messageSubscribers.delete(subscriber);
  }
  
  /**
   * Subscribe to messages (compatibility method for tests)
   * 
   * @param subscriber - Subscriber function
   * @returns Unsubscribe function
   */
  subscribeToMessages(subscriber: MessageSubscriber): () => boolean {
    this.subscribe(subscriber);
    return () => this.unsubscribe(subscriber);
  }
  
  /**
   * Broadcast a message to all subscribers
   * 
   * @param message - Message to broadcast
   * @param metadata - Optional message metadata
   */
  private broadcastMessage(message: Message, metadata?: MessageMetadata): void {
    for (const subscriber of this.messageSubscribers) {
      try {
        subscriber(message, metadata);
      } catch (error) {
        console.error("Error in message subscriber:", error);
      }
    }
  }
  
  /**
   * Handle sending a message from one agent to another
   * 
   * @param fromAgent - Source agent
   * @param toAgent - Destination agent
   * @param message - Message to send
   */
  private async handleAgentMessage(fromAgent: Agent, toAgent: Agent, message: Message): Promise<void> {
    await toAgent.processMessage(message);
  }
}
