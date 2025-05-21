import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";

/**
 * CancelMessage - Represents a TAP Cancel message
 */
export class CancelMessage extends BaseMessage {
  /**
   * Create a new cancel message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the reason for the cancellation
   */
  get reason(): string | undefined {
    return this.body.reason;
  }

  /**
   * Set the reason for the cancellation
   */
  setReason(reason: string): this {
    this.body.reason = reason;
    return this;
  }
}