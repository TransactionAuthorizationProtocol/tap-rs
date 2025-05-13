import { TAPAgent } from '../agent';
import { Connect, Authorize, Reject } from '@taprsvp/types';
import { BaseMessageObject } from './base-message';
import { AuthorizeObject } from './authorize';
import { RejectObject } from './reject';
import { tapWasm, MessageType } from '../wasm-loader';

/**
 * Connect message object with fluent response interface
 */
export class ConnectionObject extends BaseMessageObject {
  /**
   * Create an authorization response to this connect request
   */
  authorize(params: Omit<Authorize, '@type' | '@context'>): AuthorizeObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the authorize response
    const message = this.agent.getWasmAgent().create_message(MessageType.Authorize);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the requesting agent
    if (this.from) {
      this.agent.getWasmAgent().set_to(message, this.from);
    }
    
    // Set authorize body
    message.set_authorize_body({
      settlementAddress: params.settlementAddress,
      expiry: params.expiry,
      '@type': 'Authorize',
      '@context': 'https://tap.rsvp/schema/1.0'
    });
    
    // Sign the message
    this.agent.getWasmAgent().sign_message(message);
    
    // Create and return the authorize object
    return new AuthorizeObject(this.agent, message);
  }
  
  /**
   * Create a rejection response to this connect request
   */
  reject(params: Omit<Reject, '@type' | '@context'>): RejectObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the reject response
    const message = this.agent.getWasmAgent().create_message(MessageType.Reject);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the requesting agent
    if (this.from) {
      this.agent.getWasmAgent().set_to(message, this.from);
    }
    
    // Set reject body
    message.set_reject_body({
      reason: params.reason,
      '@type': 'Reject',
      '@context': 'https://tap.rsvp/schema/1.0'
    });
    
    // Sign the message
    this.agent.getWasmAgent().sign_message(message);
    
    // Create and return the reject object
    return new RejectObject(this.agent, message);
  }
}