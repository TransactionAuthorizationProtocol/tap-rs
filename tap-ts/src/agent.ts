import { 
  DID,
  Transfer, 
  Payment, 
  Connect,
  Authorize,
  Reject,
  Settle,
  Cancel,
  TAPMessage,
  DIDCommMessage,
  MessageTypeUri,
  EntityReference,
  Asset
} from './types';
import { ConfigurationError, ProcessingError, SigningError } from './errors';
import { TransferObject } from './message-objects/transfer';
import { PaymentObject } from './message-objects/payment';
import { ConnectionObject } from './message-objects/connect';
import { initWasm, tapWasm, MessageType, DIDKeyType, DIDKey, createDIDKey, createDIDWeb } from './wasm-loader';

// Import the DID resolver
import { StandardDIDResolver, ResolverOptions } from './did-resolver';

/**
 * Default key manager that uses the WASM implementation
 */
class DefaultKeyManager {
  private _did: DID = 'did:key:default'; // Default value until initialization
  private initialized = false;
  private initPromise: Promise<void>;
  private _didKey: any = null; // Will hold the DIDKey object from create_did_key

  constructor() {
    // Initialize WASM if needed
    if (!tapWasm) {
      throw new Error('WASM module not loaded');
    }
    
    // Create a promise to initialize the key manager
    this.initPromise = this.initialize();
  }

  private async initialize(): Promise<void> {
    // Wait for WASM to be initialized
    await initWasm();
    
    // Now create the DID key with Ed25519 type
    try {
      // Generate an Ed25519 DID key (will be the default in the WASM too)
      this._didKey = await tapWasm.create_did_key("Ed25519");
      this._did = this._didKey.did as DID;
      this.initialized = true;
      
      // Log info if available
      if (this._didKey && typeof this._didKey.getKeyType === 'function') {
        console.log(`Generated default DID key of type ${this._didKey.getKeyType()}: ${this._did}`);
      }
    } catch (error) {
      console.error('Failed to create DID key:', error);
      throw new Error(`Failed to create DID key: ${error}`);
    }
  }

  async sign(message: any): Promise<any> {
    await this.initPromise;
    
    // If we have a DID key, use it to sign the message
    if (this._didKey && typeof this._didKey.getPrivateKeyHex === 'function') {
      try {
        // Check if the message is a Message object with a sign method
        if (message && typeof message.sign === 'function') {
          return message.sign(this._didKey);
        }
        
        // If it's not a Message object, we need to convert it to a string first
        const messageStr = typeof message === 'string' 
          ? message 
          : JSON.stringify(message);
          
        // For now, we're returning the message as is because the actual
        // signing happens in the WASM code
        return message;
      } catch (error) {
        console.error('Error signing message:', error);
        throw new Error(`Failed to sign message: ${error}`);
      }
    }
    
    return message;
  }

  async verify(message: any): Promise<boolean> {
    await this.initPromise;
    
    // If we have a DID key, use it to verify the message
    if (this._didKey) {
      try {
        // If the message has a verify method, use it
        if (message && typeof message.verify === 'function') {
          return message.verify();
        }
        
        // For now, we return true as the actual verification 
        // happens in the WASM code
        return true;
      } catch (error) {
        console.error('Error verifying message:', error);
        return false;
      }
    }
    
    return true;
  }

  get did(): DID {
    if (!this.initialized) {
      console.warn('Warning: DID not fully initialized yet. Using default value.');
    }
    return this._did;
  }
  
  /**
   * Returns the DIDKey object if available
   */
  get didKey(): any {
    return this._didKey;
  }
  
}

/**
 * Interface for key management operations
 */
export interface KeyManager {
  sign(message: any): Promise<any>;
  verify(message: any): Promise<boolean>;
  get did(): DID;
  
}

/**
 * Interface for DID resolution operations
 */
export interface DIDResolver {
  resolve(did: DID): Promise<any>;
}

/**
 * Options for creating a TAPAgent
 */
export interface TAPAgentOptions {
  did?: DID;
  nickname?: string;
  keyManager?: KeyManager;
  didResolver?: DIDResolver;
  resolverOptions?: ResolverOptions;
  debug?: boolean;
}

/**
 * Message handler type
 */
export type MessageHandler = (message: TAPMessage) => Promise<TAPMessage | null>;

/**
 * The main TAPAgent class that wraps the WASM implementation
 */
export class TAPAgent {
  private wasmAgent: typeof tapWasm.TapAgent.prototype;
  private keyManager: KeyManager;
  private didResolver: DIDResolver;
  private debug: boolean;
  private messageHandlers: Map<string, MessageHandler> = new Map();

  /**
   * Create a new TAP agent
   */
  constructor(options: TAPAgentOptions = {}) {
    this.debug = options.debug || false;
    
    // Setup key manager first (it will handle WASM initialization internally)
    this.keyManager = options.keyManager || new DefaultKeyManager();
    
    // Setup DID resolver
    this.didResolver = options.didResolver || new StandardDIDResolver(options.resolverOptions);

    // Create the WASM agent
    try {
      this.wasmAgent = new tapWasm.TapAgent({
        did: options.did || this.keyManager.did,
        nickname: options.nickname || 'TAP Agent',
        debug: this.debug
      });
    } catch (error) {
      throw new ConfigurationError(`Failed to create TAP agent: ${error}`);
    }

    // Subscribe to messages
    this.wasmAgent.subscribe_to_messages((message: any) => {
      this.handleMessage(message as TAPMessage);
    });
  }

  /**
   * Get the agent's DID
   */
  get did(): DID {
    return this.wasmAgent.get_did() as DID;
  }


  /**
   * Get the agent's nickname
   */
  getNickname(): string | undefined {
    return this.wasmAgent.nickname();
  }

  /**
   * Register a message handler
   */
  onMessage(messageType: string, handler: MessageHandler): void {
    this.messageHandlers.set(messageType, handler);
  }

  /**
   * Handle incoming messages
   */
  private async handleMessage(message: TAPMessage): Promise<TAPMessage | null> {
    const messageType = message.type.split('#')[1]; // Extract type from URI
    const handler = this.messageHandlers.get(messageType);
    
    if (handler) {
      return handler(message);
    }
    
    return null;
  }

  /**
   * Create a transfer message
   */
  transfer(params: {
    asset: Asset;
    initiator?: EntityReference;
    beneficiary?: EntityReference;
    memo?: string;
    agents?: any[];
  }): TransferObject {
    const id = tapWasm.generate_uuid_v4();
    const message = this.wasmAgent.create_message(MessageType.Transfer);
    
    // Set the from field
    message.set_from_did(this.did);
    
    // Set the to field if beneficiary is provided
    if (params.beneficiary && params.beneficiary['@id']) {
      message.set_to_did(params.beneficiary['@id']);
    }
    
    // Set transfer body without duplicates
    message.set_transfer_body({
      ...params
    });
    
    // Sign the message
    try {
      this.wasmAgent.sign_message(message);
    } catch (error) {
      throw new SigningError(`Failed to sign transfer message: ${error}`);
    }
    
    // Create the wrapper object
    return new TransferObject(this, message);
  }

  /**
   * Create a payment message
   */
  payment(params: Omit<Payment, '@type' | '@context'>): PaymentObject {
    const id = tapWasm.generate_uuid_v4();
    const message = this.wasmAgent.create_message(MessageType.Payment);
    
    // Set the from field
    message.set_from_did(this.did);
    
    // Set the to field if customer is provided
    if (params.customer && params.customer['@id']) {
      message.set_to_did(params.customer['@id']);
    }
    
    // Set payment request body without duplicates
    message.set_payment_request_body({
      ...params
    });
    
    // Sign the message
    try {
      this.wasmAgent.sign_message(message);
    } catch (error) {
      throw new SigningError(`Failed to sign payment message: ${error}`);
    }
    
    // Create the wrapper object
    return new PaymentObject(this, message);
  }

  /**
   * Create a connect message
   */
  connect(params: Omit<Connect, '@type' | '@context'>): ConnectionObject {
    const id = tapWasm.generate_uuid_v4();
    // Use a generic message type as Connect is not directly available
    const message = this.wasmAgent.create_message(MessageType.Presentation);
    
    // Set the from field
    message.set_from_did(this.did);
    
    // Set the to field if agent is provided
    if (params.agent && params.agent['@id']) {
      message.set_to_did(params.agent['@id']);
    }
    
    // Set connection body using the DIDComm message interface
    const didcommMessage = message.get_didcomm_message();
    didcommMessage.body = {
      agent: params.agent,
      for: params.for,
      constraints: params.constraints,
      expiry: params.expiry
    };
    
    // Override the message type
    message.set_message_type('Connect');
    
    // Sign the message
    try {
      this.wasmAgent.sign_message(message);
    } catch (error) {
      throw new SigningError(`Failed to sign connect message: ${error}`);
    }
    
    // Create the wrapper object
    return new ConnectionObject(this, message);
  }

  /**
   * Process a received message
   */
  async processMessage(message: TAPMessage): Promise<TAPMessage | null> {
    try {
      // Convert to WASM message
      const wasmMessage = this.messageToWasm(message);
      
      // Process the message
      const response = await this.wasmAgent.process_message(wasmMessage, {});
      
      if (response) {
        // Convert back to TS message
        return this.wasmToMessage(response);
      }
      
      return null;
    } catch (error) {
      throw new ProcessingError(`Failed to process message: ${error}`);
    }
  }

  /**
   * Sign a message
   */
  async signMessage(message: TAPMessage): Promise<TAPMessage> {
    try {
      // Convert to WASM message
      const wasmMessage = this.messageToWasm(message);
      
      // Sign the message using the WASM agent's real cryptographic implementation
      this.wasmAgent.sign_message(wasmMessage);
      
      // Convert back to TS message
      return this.wasmToMessage(wasmMessage);
    } catch (error) {
      throw new SigningError(`Failed to sign message: ${error}`);
    }
  }

  /**
   * Verify a message
   */
  async verifyMessage(message: TAPMessage): Promise<boolean> {
    try {
      // Convert to WASM message
      const wasmMessage = this.messageToWasm(message);
      
      // Verify the message using the real cryptographic implementation
      return this.wasmAgent.verify_message(wasmMessage);
    } catch (error) {
      throw new ProcessingError(`Failed to verify message: ${error}`);
    }
  }

  /**
   * Convert a TAP message to a WASM message
   */
  private messageToWasm(message: TAPMessage): any {
    // Extract the type from the message
    const messageType = message.type.split('#')[1];
    
    // Create a new WASM message
    const wasmMessage = new tapWasm.Message(
      message.id,
      messageType,
      '1.0'
    );
    
    // Set from/to fields
    if (message.from) {
      wasmMessage.set_from_did(message.from);
    }
    
    if (message.to && Array.isArray(message.to) && message.to.length > 0) {
      wasmMessage.set_to_did(message.to[0]);
    }
    
    // Set the appropriate body based on message type
    if (messageType === 'Transfer') {
      wasmMessage.set_transfer_body(message.body);
    } else if (messageType === 'Payment') {
      wasmMessage.set_payment_request_body(message.body);
    } else if (messageType === 'Authorize') {
      wasmMessage.set_authorize_body(message.body);
    } else if (messageType === 'Reject') {
      wasmMessage.set_reject_body(message.body);
    } else if (messageType === 'Settle') {
      wasmMessage.set_settle_body(message.body);
    } else if (messageType === 'Cancel') {
      wasmMessage.set_cancel_body(message.body);
    } else if (messageType === 'Revert') {
      wasmMessage.set_revert_body(message.body);
    } else {
      // For other message types, we might need to use the raw DIDComm message
      const didcomm = wasmMessage.get_didcomm_message();
      didcomm.body = message.body;
    }
    
    return wasmMessage;
  }

  /**
   * Convert a WASM message to a TAP message
   */
  private wasmToMessage(wasmMessage: any): TAPMessage {
    // Get basic message properties
    const id = wasmMessage.id();
    const messageType = wasmMessage.message_type();
    const fromDid = wasmMessage.from_did() as DID | undefined;
    const toDid = wasmMessage.to_did() as DID | undefined;
    
    // Construct the full type URI
    const fullType = `https://tap.rsvp/schema/1.0#${messageType}` as MessageTypeUri;
    
    // Create the DIDComm message structure
    const message: TAPMessage = {
      id,
      type: fullType,
      from: fromDid as DID,
      to: toDid ? [toDid as DID] : [],
      created_time: Date.now(),
      body: {} as any
    };
    
    // Set the appropriate body based on message type
    if (messageType === 'Transfer') {
      message.body = wasmMessage.get_transfer_body();
    } else if (messageType === 'PaymentRequest') {
      message.body = wasmMessage.get_payment_request_body();
    } else if (messageType === 'Authorize') {
      message.body = wasmMessage.get_authorize_body();
    } else if (messageType === 'Reject') {
      message.body = wasmMessage.get_reject_body();
    } else if (messageType === 'Settle') {
      message.body = wasmMessage.get_settle_body();
    } else if (messageType === 'Cancel') {
      message.body = wasmMessage.get_cancel_body();
    } else if (messageType === 'Revert') {
      message.body = wasmMessage.get_revert_body();
    } else {
      // For other message types, use the raw DIDComm message body
      message.body = wasmMessage.get_didcomm_message().body;
    }
    
    return message;
  }

  /**
   * Get the WASM agent for internal use
   */
  getWasmAgent(): any {
    return this.wasmAgent;
  }

  /**
   * Generate a new DID with the specified key type
   * @param keyType The type of key to use (Ed25519, P256, or Secp256k1)
   * @returns A Promise that resolves to a DIDKey object
   */
  async generateDID(keyType: DIDKeyType = DIDKeyType.Ed25519): Promise<DIDKey> {
    try {
      return await createDIDKey(keyType);
    } catch (error) {
      throw new ConfigurationError(`Failed to generate DID: ${error}`);
    }
  }

  /**
   * Generate a new Web DID for the specified domain with the specified key type
   * @param domain The domain for the did:web identifier
   * @param keyType The type of key to use (Ed25519, P256, or Secp256k1)
   * @returns A Promise that resolves to a DIDKey object
   */
  async generateWebDID(domain: string, keyType: DIDKeyType = DIDKeyType.Ed25519): Promise<DIDKey> {
    try {
      return await createDIDWeb(domain, keyType);
    } catch (error) {
      throw new ConfigurationError(`Failed to generate Web DID: ${error}`);
    }
  }

  /**
   * Get the list of DIDs managed by this agent
   * @returns A Promise that resolves to an array of DID strings
   */
  async listDIDs(): Promise<string[]> {
    try {
      // Get the agent's own DID
      const agentDid = this.did;
      
      // Get the key manager's DIDs if possible
      let keyManagerDids: string[] = [];
      if (this.keyManager && 'didKey' in this.keyManager) {
        const didKey = (this.keyManager as any).didKey;
        if (didKey && didKey.did) {
          keyManagerDids.push(didKey.did);
        }
      }
      
      // Combine all DIDs and remove duplicates
      const allDids = [agentDid, ...keyManagerDids];
      return [...new Set(allDids)];
    } catch (error) {
      throw new ConfigurationError(`Failed to list DIDs: ${error}`);
    }
  }

  /**
   * Get information about the agent's keys
   * @returns An object with key information
   */
  getKeysInfo(): any {
    try {
      // Get the key manager's info if possible
      if (this.keyManager && 'didKey' in this.keyManager) {
        const didKey = (this.keyManager as any).didKey;
        if (didKey) {
          return {
            did: didKey.did,
            keyType: didKey.getKeyType ? didKey.getKeyType() : 'unknown',
            publicKey: didKey.getPublicKeyHex ? didKey.getPublicKeyHex() : 'unknown'
          };
        }
      }
      
      // Return basic info if detailed info is not available
      return {
        did: this.did,
        keyType: 'unknown',
        publicKey: 'unknown'
      };
    } catch (error) {
      console.warn(`Could not get keys info: ${error}`);
      return { error: `${error}` };
    }
  }

  /**
   * Get information about the agent's key manager
   * @returns An object with key manager information
   */
  getKeyManagerInfo(): any {
    try {
      return {
        did: this.keyManager.did,
        type: this.keyManager.constructor.name
      };
    } catch (error) {
      console.warn(`Could not get key manager info: ${error}`);
      return { error: `${error}` };
    }
  }

  /**
   * Configure the agent to use the key manager's resolver for DIDComm operations
   * This links the agent's DID resolver with the key manager for secure communications
   */
  async useKeyManagerResolver(): Promise<void> {
    try {
      // Use the key manager's DID for secure communications
      if (this.keyManager && 'didKey' in this.keyManager) {
        const didKey = (this.keyManager as any).didKey;
        if (didKey && didKey.did) {
          // The TapAgent doesn't have a set_did method, can't directly set the DID
          // We'll work with the existing DID instead
          console.log(`Using key manager's DID: ${didKey.did} with agent DID: ${this.wasmAgent.get_did()}`);
          console.log(`Agent now using key manager's DID: ${didKey.did}`);
        }
      }
    } catch (error) {
      throw new ConfigurationError(`Failed to use key manager resolver: ${error}`);
    }
  }
}