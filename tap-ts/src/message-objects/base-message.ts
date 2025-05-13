import { TAPAgent } from '../agent';
import { DID, TAPMessage, MessageTypeUri } from '../types/index';
import { NetworkError } from '../errors';

/**
 * Base class for all message objects
 */
export abstract class BaseMessageObject {
  id: string;
  from?: DID;
  to?: DID[];
  type: MessageTypeUri;
  created_time: number;
  body: any;
  
  // Reference to the TAP agent
  protected agent: TAPAgent;
  
  // Reference to the WASM message
  protected wasmMessage: any;
  
  constructor(agent: TAPAgent, wasmMessage: any) {
    this.agent = agent;
    this.wasmMessage = wasmMessage;
    
    // Extract metadata from WASM message
    this.id = wasmMessage.id();
    this.from = wasmMessage.from_did() as DID;
    const toDid = wasmMessage.to_did() as DID | undefined;
    this.to = toDid ? [toDid] : [];
    this.type = `https://tap.rsvp/schema/1.0#${wasmMessage.message_type()}` as MessageTypeUri;
    this.created_time = Date.now();
    
    // Extract body based on message type
    const messageType = wasmMessage.message_type();
    if (messageType === 'Transfer') {
      this.body = wasmMessage.get_transfer_body();
    } else if (messageType === 'PaymentRequest') {
      this.body = wasmMessage.get_payment_request_body();
    } else if (messageType === 'Authorize') {
      this.body = wasmMessage.get_authorize_body();
    } else if (messageType === 'Reject') {
      this.body = wasmMessage.get_reject_body();
    } else if (messageType === 'Settle') {
      this.body = wasmMessage.get_settle_body();
    } else if (messageType === 'Cancel') {
      this.body = wasmMessage.get_cancel_body();
    } else if (messageType === 'Revert') {
      this.body = wasmMessage.get_revert_body();
    } else {
      // For other message types, use the raw DIDComm message body
      this.body = wasmMessage.get_didcomm_message().body;
    }
  }
  
  /**
   * Get the raw message
   */
  getMessage(): TAPMessage {
    return {
      id: this.id,
      type: this.type,
      from: this.from as DID,
      to: this.to || [],
      created_time: this.created_time,
      body: this.body
    };
  }
  
  /**
   * Send the message
   * Note: In a real implementation, this would connect to a transport mechanism
   */
  async send(): Promise<void> {
    try {
      // This is a placeholder. In a real implementation, you would:
      // 1. Check if the message is properly signed
      // 2. Use a transport layer to send the message
      // 3. Handle any errors or retries
      
      // For now, we'll just log that the message would be sent
      if (typeof console !== 'undefined') {
        console.log(`Sending message ${this.id} of type ${this.type}`);
      }
    } catch (error) {
      throw new NetworkError(`Failed to send message: ${error}`);
    }
  }
  
  /**
   * Sign the message
   */
  async sign(): Promise<this> {
    this.agent.getWasmAgent().sign_message(this.wasmMessage);
    return this;
  }
  
  /**
   * Verify the message signature
   */
  async verify(): Promise<boolean> {
    return this.agent.getWasmAgent().verify_message(this.wasmMessage);
  }
  
  /**
   * Returns a JSON representation of the message
   */
  toJSON(): any {
    return {
      id: this.id,
      type: this.type,
      from: this.from,
      to: this.to,
      created_time: this.created_time,
      body: this.body
    };
  }
  
  /**
   * Returns a string representation of the message
   */
  toString(): string {
    return JSON.stringify(this.toJSON(), null, 2);
  }
}