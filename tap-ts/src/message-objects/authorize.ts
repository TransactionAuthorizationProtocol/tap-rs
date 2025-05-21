import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";

/**
 * AuthorizeMessage - Represents a TAP Authorize message
 */
export class AuthorizeMessage extends BaseMessage {
  /**
   * Create a new authorize message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the reason for the authorization
   */
  get reason(): string | undefined {
    return this.body.reason;
  }

  /**
   * Set the reason for the authorization
   */
  setReason(reason: string): this {
    this.body.reason = reason;
    return this;
  }

  /**
   * Get the settlement address for the authorization
   */
  get settlementAddress(): string | undefined {
    return this.body.settlementAddress;
  }

  /**
   * Set the settlement address for the authorization
   */
  setSettlementAddress(settlementAddress: string): this {
    this.body.settlementAddress = settlementAddress;
    return this;
  }

  /**
   * Get the expiry timestamp for the authorization
   */
  get expiry(): string | undefined {
    return this.body.expiry;
  }

  /**
   * Set the expiry timestamp for the authorization
   */
  setExpiry(expiry: string): this {
    this.body.expiry = expiry;
    return this;
  }
}