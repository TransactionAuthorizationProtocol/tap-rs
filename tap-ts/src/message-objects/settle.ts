import { TAPAgent } from "../agent";
import { BaseMessage } from "./base-message";

/**
 * SettleMessage - Represents a TAP Settle message
 */
export class SettleMessage extends BaseMessage {
  /**
   * Create a new settle message
   */
  constructor(agent: TAPAgent, message: any) {
    super(agent, message);
  }

  /**
   * Get the settlement ID
   */
  get settlementId(): string {
    return this.body.settlementId;
  }

  /**
   * Set the settlement ID
   */
  setSettlementId(settlementId: string): this {
    this.body.settlementId = settlementId;
    return this;
  }

  /**
   * Get the amount that was settled
   */
  get amount(): string | undefined {
    return this.body.amount;
  }

  /**
   * Set the amount that was settled
   */
  setAmount(amount: string): this {
    this.body.amount = amount;
    return this;
  }
}