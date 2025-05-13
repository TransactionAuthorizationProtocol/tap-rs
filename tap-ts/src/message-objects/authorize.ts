import { TAPAgent } from '../agent';
import { Authorize, Settle } from '@taprsvp/types';
import { BaseMessageObject } from './base-message';
import { SettleObject } from './settle';
import { tapWasm, MessageType } from '../wasm-loader';

/**
 * Authorization message object with fluent response interface
 */
export class AuthorizeObject extends BaseMessageObject {
  /**
   * Create a settlement message for this authorization
   */
  settle(params: Omit<Settle, '@type' | '@context'>): SettleObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the settle response
    const message = this.agent.getWasmAgent().create_message(MessageType.Settle);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the original recipient
    if (this.to && this.to.length > 0) {
      this.agent.getWasmAgent().set_to(message, this.to[0]);
    }
    
    // Set settle body
    message.set_settle_body({
      settlementId: params.settlementId,
      amount: params.amount,
      '@type': 'Settle',
      '@context': 'https://tap.rsvp/schema/1.0'
    });
    
    // Sign the message
    this.agent.getWasmAgent().sign_message(message);
    
    // Create and return the settle object
    return new SettleObject(this.agent, message);
  }
}