## Introduction

The `@taprsvp/tap` library is a TypeScript SDK for the **Transaction Authorization Protocol (TAP)**—a decentralized protocol for multi-party transaction authorization. It wraps a Rust core (via WebAssembly) to combine **performance** and **security** with a **developer-friendly TS API**. All TAP message types and data models are supported, following the TAP spec.

## Message Types & Data Structures

**Complete TAP Model Support:** Implements all TAP message types and data structures as defined in the TAP specification. Categories include:

* **Transaction Messages:** `Transfer`, `Payment`
* **Authorization Flow:** `Authorize`, `Complete`, `Settle`, `Reject`, `Cancel`, `Revert`
* **Participant Management:** `UpdateAgent`, `UpdateParty`, `AddAgents`, `ReplaceAgent`, `RemoveAgent`
* **Relationship Proofs:** `ConfirmRelationship`
* **Policy Messages:** `UpdatePolicies`
* **Connection Messages:** `Connect`, `AuthorizationRequired`

**TypeScript Interfaces:** Each message body and data element is represented by a strict TS interface. Example: Transfer body:

```ts
interface TransferBody {
  asset: string;
  amount: string;
  originator: Party;
  beneficiary?: Party;
  agents: Agent[];
  settlementId?: string;
  memo?: string;
  purpose?: string;
  categoryPurpose?: string;
  expiry?: string;
}
```

Every message envelope extends a generic base:

```ts
abstract class DIDCommMessage<Body> {
  readonly id: string;
  readonly type: string;
  from!: DID;
  to: string[] = [];
  thid?: string;
  created_time!: number;
  expires_time?: number;
  body: Body;
  constructor(type: string, body: Body) { ... }
  _prepareEnvelope(agentDid: string) { ... }
}
```

## Class-Based High-Level API

### `TAPAgent`

**Single entry-point for crypto & DID** operations: instantiate one agent per application context.

```ts
import { TAPAgent } from "@taprsvp/tap";
const agent = new TAPAgent({
  did: "did:web:merchant.example",
  signer: mySigner,
  resolver: createDefaultResolver(),
});
```

No global; methods `sign()` and `verify()` always use your `agent` instance.

### Message Classes

#### Starter Messages (start new threads)

* **Transfer**
* **Payment**
* **Connect**

```ts
import { Transfer, Payment, Connect } from "@taprsvp/tap";

const transfer = new Transfer({
  asset: "eip155:1/slip44:60",
  amount: "1.23",
  originator: { did: "did:web:alice.bank" },
  beneficiary: { did: "did:web:bob.exchange" },
  agents: [ { "@id": "did:web:alice.bank" }, { "@id": "did:web:bob.exchange" } ],
});

const payment = new Payment({
  asset: "eip155:1/erc20:0xA0b8...6eB48",
  amount: "100.00",
  merchant: { did: "did:web:merchant.example", name: "Example Store", mcc: "5812" },
  invoice: "https://example.com/invoice/123",
});
```

Constructors handle:

* UUIDv4 `id`
* Correct `type` URI
* `from`, `created_time` on signing
* Optional `expires_time`
* Thread ID (`thid`) left undefined

#### Reply Methods on Instances

Starter classes provide methods to spawn replies, automatically setting `thid = this.id`:

```ts
// On Transfer instance:
const auth   = transfer.authorize();
const rej    = transfer.reject("reason");
const settle = transfer.settle("eip155:1:tx/...", "1.23");
const cancel = transfer.cancel("user_requested");
const revert = transfer.revert({ settlementAddress: "...", reason: "..." });

// On Payment instance:
const complete = payment.complete({ settlementAddress: "...", amount: "95.50" });
const settle2  = payment.settle("eip155:1:tx/...");
const cancel2  = payment.cancel("customer declined");
```

Each reply class (`Authorize`, `Complete`, `Settle`, etc.) extends `DIDCommMessage`:

```ts
export class Authorize extends DIDCommMessage<AuthorizeBody> {
  constructor(body: AuthorizeBody = {}, opts: { thid: string }) {
    super("https://tap.rsvp/schema/1.0#Authorize", body);
    this.thid = opts.thid;
  }
}
```

## Agent Methods

```ts
// Signing:
await agent.sign(transfer);
// Internally calls message._prepareEnvelope(agent.did) + attaches JWS

// Verifying:
const ok = await agent.verify(incomingMsg);
if (!ok) throw new Error("Invalid signature");
```

## WASM Bridge & Initialization

* **Async ESM initialization** via `await TapWasm.initialize()` inside library.
* Uses `WebAssembly.instantiateStreaming` in browsers and native import in Node.
* High-level classes call into WASM for serialization, packing, and cryptography.

## Packaging & Structure

Single npm package `@taprsvp/tap` with:

* `dist/wasm/` — compiled Rust WASM + bindings
* `src/models/` — TS interfaces for TAP messages & data elements
* `src/agent/` — `TAPAgent` class + default resolver setup
* `src/api/messages/` — message classes (`Transfer.ts`, `Payment.ts`, etc.)
* `src/utils/` — UUID, date utils, error classes

## TypeScript Best Practices

* **Strict typing** for all messages and bodies
* **JSDoc** on public APIs
* **Custom error classes** (`TapError`, `ValidationError`, etc.)
* **Unit tests** verifying TS➡️Rust parity and spec compliance

## Example Flow

```ts
// 1) Agent setup
const agent = new TAPAgent({ did, signer, resolver });

// 2) Start Transfer
const transfer = new Transfer({ ... });
await agent.sign(transfer);
send(transfer);

// 3) On receiver side:
const received = await receive();
await agent.verify(received);
const authorizeMsg = received.authorize();
await agent.sign(authorizeMsg);
send(authorizeMsg);
```
