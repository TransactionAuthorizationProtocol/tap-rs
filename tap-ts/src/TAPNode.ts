/**
 * TAP Node implementation based on the Transaction Authorization Protocol specification
 * 
 * @module TAPNode
 */

import { TapError, ErrorType } from "./error.ts";
import { wasmLoader } from "./wasm/loader.ts";
import { TAPMessage, TAPMessageType, TAPMessageHandler } from "./TAPMessage.ts";
import { TAPAgent } from "./TAPAgent.ts";
import type { DID } from "../../prds/taips/packages/typescript/src/tap";

/**
 * TAP Node configuration options
 */
export interface TAPNodeOptions {
  /** Debug mode flag */
  debug?: boolean;
  
  /** Node ID */
  id?: string;
  
  /** Known peer DIDs */
  peers?: DID[];
}

/**
 * TAP Node class
 * 
 * A node in the TAP network that can manage multiple agents,
 * route messages, and connect to other nodes.
 */
export class TAPNode {
  private wasmNode: any;
  private debug: boolean;
  private id: string;
  private agents: Map<DID, TAPAgent> = new Map();
  private messageHandlers: Map<TAPMessageType, TAPMessageHandler[]> = new Map();
  private globalHandlers: TAPMessageHandler[] = [];
  private peers: Set<DID> = new Set();
  
  /**
   * Create a new TAP node
   * 
   * @param options Node configuration options
   */
  constructor(options: TAPNodeOptions = {}) {
    if (!wasmLoader.moduleIsLoaded()) {
      throw new TapError({
        type: ErrorType.WASM_NOT_LOADED,
        message: "WASM module not loaded",
      });
    }
    
    // Get the WASM module
    const module = wasmLoader.getModule();
    
    this.debug = options.debug ?? false;
    this.id = options.id || `node_${crypto.randomUUID().replace(/-/g, "")}`;
    
    // Create the WASM node
    const config = new module.NodeConfig();
    this.wasmNode = new module.TapNode(config);
    
    // Add peers if provided
    if (options.peers) {
      for (const peer of options.peers) {
        this.addPeer(peer);
      }
    }
    
    if (this.debug) {
      console.log(`TAPNode initialized with ID: ${this.id}`);
    }
  }
  
  /**
   * Get the node's ID
   * 
   * @returns The node's ID
   */
  getID(): string {
    return this.id;
  }
  
  /**
   * Register an agent with this node
   * 
   * @param agent Agent to register
   * @returns This node for chaining
   */
  registerAgent(agent: TAPAgent): this {
    const did = agent.getDID();
    
    if (this.agents.has(did)) {
      if (this.debug) {
        console.log(`Agent with DID ${did} already registered, replacing`);
      }
    }
    
    this.agents.set(did, agent);
    
    // Register with WASM node if needed
    try {
      this.wasmNode.register_agent(agent.wasmAgent);
    } catch (error) {
      // If the WASM binding fails, we'll still track the agent in our map
      if (this.debug) {
        console.warn(`Failed to register agent with WASM node: ${error}`);
      }
    }
    
    if (this.debug) {
      console.log(`Registered agent with DID: ${did}`);
    }
    
    return this;
  }
  
  /**
   * Unregister an agent from this node
   * 
   * @param did DID of the agent to unregister
   * @returns This node for chaining
   */
  unregisterAgent(did: DID): this {
    if (!this.agents.has(did)) {
      throw new TapError({
        type: ErrorType.AGENT_NOT_FOUND,
        message: `Agent with DID ${did} not found`,
      });
    }
    
    this.agents.delete(did);
    
    // Unregister from WASM node if needed
    try {
      this.wasmNode.unregister_agent(did);
    } catch (error) {
      // If the WASM binding fails, we'll still remove the agent from our map
      if (this.debug) {
        console.warn(`Failed to unregister agent from WASM node: ${error}`);
      }
    }
    
    if (this.debug) {
      console.log(`Unregistered agent with DID: ${did}`);
    }
    
    return this;
  }
  
  /**
   * Get an agent by DID
   * 
   * @param did DID of the agent to get
   * @returns The agent or undefined if not found
   */
  getAgent(did: DID): TAPAgent | undefined {
    return this.agents.get(did);
  }
  
  /**
   * Get all registered agents' DIDs
   * 
   * @returns Array of agent DIDs
   */
  getAgentDIDs(): DID[] {
    return Array.from(this.agents.keys());
  }
  
  /**
   * Register a handler for a specific message type
   * 
   * @param type Message type to handle
   * @param handler Handler function
   * @returns This node for chaining
   */
  handleMessage(type: TAPMessageType, handler: TAPMessageHandler): this {
    if (!this.messageHandlers.has(type)) {
      this.messageHandlers.set(type, []);
    }
    
    this.messageHandlers.get(type)!.push(handler);
    
    if (this.debug) {
      console.log(`Registered handler for message type: ${type}`);
    }
    
    return this;
  }
  
  /**
   * Register a handler for all message types
   * 
   * @param handler Handler function
   * @returns This node for chaining
   */
  handleAllMessages(handler: TAPMessageHandler): this {
    this.globalHandlers.push(handler);
    
    if (this.debug) {
      console.log("Registered global message handler");
    }
    
    return this;
  }
  
  /**
   * Process a received message
   * 
   * @param message Message to process
   * @param metadata Optional message metadata
   */
  async processMessage(message: TAPMessage, metadata: Record<string, unknown> = {}): Promise<void> {
    if (this.debug) {
      console.log(`Node processing message (${message.type}) from ${message.from || 'unknown'}`);
    }
    
    // Call type-specific handlers
    const typeHandlers = this.messageHandlers.get(message.type);
    if (typeHandlers) {
      for (const handler of typeHandlers) {
        try {
          await handler(message, metadata);
        } catch (error) {
          console.error(`Error in message handler for type ${message.type}:`, error);
        }
      }
    }
    
    // Call global handlers
    for (const handler of this.globalHandlers) {
      try {
        await handler(message, metadata);
      } catch (error) {
        console.error("Error in global message handler:", error);
      }
    }
    
    // Route to appropriate agents based on the recipient(s)
    if (message.to && message.to.length > 0) {
      for (const recipient of message.to) {
        const agent = this.agents.get(recipient);
        if (agent) {
          await agent.processMessage(message, metadata);
        } else if (this.debug) {
          console.log(`No agent found for recipient: ${recipient}`);
        }
      }
    } else if (this.debug) {
      console.log("Message has no recipients");
    }
  }
  
  /**
   * Send a message through this node
   * 
   * @param message Message to send
   * @param options Sending options
   * @returns Promise resolving when the message is sent
   */
  async sendMessage(message: TAPMessage, options: Record<string, unknown> = {}): Promise<void> {
    if (this.debug) {
      console.log(`Sending message (${message.type}) to ${message.to ? message.to.join(', ') : 'unknown'}`);
    }
    
    if (!message.to || message.to.length === 0) {
      throw new TapError({
        type: ErrorType.INVALID_MESSAGE,
        message: "Message has no recipients",
      });
    }
    
    // In a real implementation, this would route the message to the appropriate recipients
    // For now, we'll just log it
    
    return Promise.resolve();
  }
  
  /**
   * Add a peer node
   * 
   * @param did DID of the peer node
   * @returns This node for chaining
   */
  addPeer(did: DID): this {
    this.peers.add(did);
    
    if (this.debug) {
      console.log(`Added peer: ${did}`);
    }
    
    return this;
  }
  
  /**
   * Remove a peer node
   * 
   * @param did DID of the peer node
   * @returns This node for chaining
   */
  removePeer(did: DID): this {
    if (this.peers.has(did)) {
      this.peers.delete(did);
      
      if (this.debug) {
        console.log(`Removed peer: ${did}`);
      }
    }
    
    return this;
  }
  
  /**
   * Get all peer DIDs
   * 
   * @returns Array of peer DIDs
   */
  getPeers(): DID[] {
    return Array.from(this.peers);
  }
}

/**
 * Create a new TAP node
 * 
 * @param options Optional configuration options
 * @returns A new TAP node
 */
export function createTAPNode(options: Partial<TAPNodeOptions> = {}): TAPNode {
  return new TAPNode(options as TAPNodeOptions);
}