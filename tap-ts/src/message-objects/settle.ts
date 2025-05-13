import { TAPAgent } from '../agent';
import { Settle, Revert } from '@taprsvp/types';
import { BaseMessageObject } from './base-message';
import { RevertObject } from './revert';
import { tapWasm, MessageType } from '../wasm-loader';

/**
 * Settlement message object with fluent response interface
 */
export class SettleObject extends BaseMessageObject {
  /**
   * Create a revert message for this settlement
   */
  revert(params: Omit<Revert, '@type' | '@context'>): RevertObject {
    // Create a message ID
    const id = tapWasm.generate_uuid_v4();
    
    // Create a WASM message for the revert response
    const message = this.agent.getWasmAgent().create_message(MessageType.Revert);
    
    // Set the from field
    this.agent.getWasmAgent().set_from(message);
    
    // Set the to field to the original recipient
    if (this.to && this.to.length > 0) {
      this.agent.getWasmAgent().set_to(message, this.to[0]);
    }
    
    // Set revert body
    message.set_revert_body({
      reason: params.reason,
      settlementAddress: params.settlementAddress,
      '@type': 'Revert',
      '@context': 'https://tap.rsvp/schema/1.0'
    });
    
    // Sign the message
    this.agent.getWasmAgent().sign_message(message);
    
    // Create and return the revert object
    return new RevertObject(this.agent, message);
  }
}