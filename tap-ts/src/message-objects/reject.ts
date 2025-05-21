import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";

/**
 * RejectMessage - Represents a TAP Reject message
 */
export class RejectMessage extends BaseMessage {
  /**
   * Create a new reject message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the reason for the rejection
   */
  get reason(): string {
    return this.body.reason;
  }

  /**
   * Set the reason for the rejection
   */
  setReason(reason: string): this {
    this.body.reason = reason;
    return this;
  }
}