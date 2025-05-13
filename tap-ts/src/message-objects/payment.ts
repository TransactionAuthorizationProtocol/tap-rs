import { TAPAgent } from '../agent';
import { Payment, Authorize, Reject, Cancel } from '@taprsvp/types';
import { BaseMessageObject } from './base-message';
import { AuthorizeObject } from './authorize';
import { RejectObject } from './reject';
import { CancelObject } from './cancel';
import { tapWasm, MessageType } from '../wasm-loader';

/**
 * Payment request message object with fluent response interface
 */
export class PaymentObject extends BaseMessageObject {
  /**
   * Create an authorization response to this payment request
   */
  authorize(params: Omit<Authorize, '@type' | '@context'>): AuthorizeObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the authorize response
    const message = this.agent.getWasmAgent().create_message(MessageType.Authorize);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the merchant's DID
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
   * Create a rejection response to this payment request
   */
  reject(params: Omit<Reject, '@type' | '@context'>): RejectObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the reject response
    const message = this.agent.getWasmAgent().create_message(MessageType.Reject);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the merchant's DID
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
  
  /**
   * Create a cancel response to this payment request
   */
  cancel(params: Omit<Cancel, '@type' | '@context'>): CancelObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the cancel response
    const message = this.agent.getWasmAgent().create_message(MessageType.Cancel);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the recipients
    if (this.to && this.to.length > 0) {
      this.agent.getWasmAgent().set_to(message, this.to[0]);
    }
    
    // Set cancel body
    message.set_cancel_body({
      reason: params.reason,
      '@type': 'Cancel',
      '@context': 'https://tap.rsvp/schema/1.0'
    });
    
    // Sign the message
    this.agent.getWasmAgent().sign_message(message);
    
    // Create and return the cancel object
    return new CancelObject(this.agent, message);
  }
}