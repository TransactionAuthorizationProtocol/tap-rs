import { TAPAgent } from "../agent";
import { DID, TAPMessage, MessageTypeUri } from "../types";

/**
 * Base class for all TAP message objects
 * Provides common functionality for all message types
 */
export abstract class BaseMessage {
  protected agent: TAPAgent;
  protected message: any;

  /**
   * Create a new base message
   */
  constructor(agent: TAPAgent, message: any) {
    this.agent = agent;
    this.message = message;
  }

  /**
   * Get the message ID
   */
  get id(): string {
    return this.message.id;
  }

  /**
   * Get the message type
   */
  get type(): MessageTypeUri {
    return this.message.type as MessageTypeUri;
  }

  /**
   * Get the sender DID
   */
  get from(): DID {
    return this.message.from as DID;
  }

  /**
   * Get the recipient DIDs
   */
  get to(): DID[] {
    return Array.isArray(this.message.to) ? this.message.to : [];
  }

  /**
   * Set the recipient DIDs
   */
  setTo(dids: DID | DID[]): this {
    if (Array.isArray(dids)) {
      this.message.to = dids;
    } else {
      this.message.to = [dids];
    }
    return this;
  }

  /**
   * Get the message body
   */
  get body(): any {
    return this.message.body;
  }

  /**
   * Set the thread ID (thid)
   */
  setThreadId(thid: string): this {
    this.message.thid = thid;
    return this;
  }

  /**
   * Get the thread ID (thid)
   */
  get threadId(): string | undefined {
    return this.message.thid;
  }

  /**
   * Set the parent thread ID (pthid)
   */
  setParentThreadId(pthid: string): this {
    this.message.pthid = pthid;
    return this;
  }

  /**
   * Get the parent thread ID (pthid)
   */
  get parentThreadId(): string | undefined {
    return this.message.pthid;
  }

  /**
   * Pack the message for transmission
   */
  async pack(): Promise<{ message: string, metadata: any }> {
    return await this.agent.packMessage(this.toJSON());
  }

  /**
   * Get the raw message that can be sent over the wire
   */
  toJSON(): TAPMessage {
    return {
      id: this.message.id,
      type: this.message.type,
      from: this.message.from,
      to: this.message.to || [],
      created_time: this.message.created || Date.now(),
      expires_time: this.message.expires,
      body: this.message.body,
      thid: this.message.thid,
      pthid: this.message.pthid,
    };
  }
}