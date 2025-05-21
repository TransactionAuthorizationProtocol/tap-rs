import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";

/**
 * RevertMessage - Represents a TAP Revert message
 */
export class RevertMessage extends BaseMessage {
  /**
   * Create a new revert message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the settlement address to revert
   */
  get settlementAddress(): string {
    return this.body.settlementAddress;
  }

  /**
   * Set the settlement address to revert
   */
  setSettlementAddress(settlementAddress: string): this {
    this.body.settlementAddress = settlementAddress;
    return this;
  }

  /**
   * Get the reason for the reversion
   */
  get reason(): string {
    return this.body.reason;
  }

  /**
   * Set the reason for the reversion
   */
  setReason(reason: string): this {
    this.body.reason = reason;
    return this;
  }
}