# TAP.rs Usage Guide

## Overview 
The **Transaction Authorization Protocol (TAP)** is a decentralized off-chain protocol that allows multiple participants in a blockchain transaction to identify each other and collaboratively authorize or reject the transaction *before* on-chain settlement ([tap.md](prds/tap.md)). In essence, TAP adds an **authorization layer** on top of the blockchain’s settlement layer, enabling counterparties (originators and beneficiaries, and their service providers) to coordinate safely and privately without modifying on-chain mechanisms ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=horizing,altering%20the%20underlying%20permissionless%20blockchain)). This approach helps solve real-world challenges like regulatory compliance and fraud prevention while preserving the trustless nature of blockchain transactions. 

`tap.rs` is the Rust implementation of TAP, providing developers with a way to create TAP agents and process TAP messages programmatically. It implements the TAP messaging flows using **DIDComm v2** – a secure messaging standard based on decentralized identifiers (DIDs) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=%5BDIDComm%20v2%5D%28https%3A%2F%2Fidentity.foundation%2Fdidcomm,a%20number%20of%20important%20features)). All TAP messages in `tap.rs` are DIDComm v2 compliant, meaning each message is a JSON envelope with standard fields (`id`, `type`, `from`, `to`, `body`, etc.) and can be cryptographically signed and encrypted as needed ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,2.md%20at%20main%20%C2%B7)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,band)). By leveraging DIDComm, `tap.rs` ensures **transport independence** (messages can be sent over HTTP, email, etc.) and end-to-end security for TAP communications ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,2.md%23%3A~%3Atext%3DTAIP%2Cwithin%2520the%2520context%2520of%2520TAP)). 

In practical terms, `tap.rs` enables organizations like exchanges, banks, or even individual wallets to act as **TAP agents** that coordinate transaction approvals. The library also comes with a **TypeScript/WASM wrapper**, allowing it to run in web browsers or Node.js. This means front-end applications (e.g. a web wallet) can participate in TAP flows using the same logic, compiled to WebAssembly. In a TAP flow, each participant’s agent (built on `tap.rs`) will exchange DIDComm messages – such as a transaction proposal, approval, or rejection – with other agents. Using `tap.rs`, developers can easily create, sign, send, receive, and verify these TAP messages within their systems, integrating TAP’s multi-party authorization handshake into any blockchain application ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Overall%2C%20TAP%20introduces%20a%20generic,20it%29%29%20%28%5BTransaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.r%20svp%2FTAIPs%2Ftaip)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,20privacy)).

## Installation & Setup 

Setting up `tap.rs` involves adding the Rust crates to your project and (optionally) building the WebAssembly package for TypeScript usage. Below are step-by-step instructions for both:

### Rust Library Installation

1. **Add Dependencies:** Include the TAP crates in your Rust project’s `Cargo.toml`. The project is organized as a workspace with three main crates: `tap-didcomm-core`, `tap-didcomm-node`, and `tap-didcomm-web`. If published on crates.io, you can add for example: 

   ```toml
   [dependencies]
   tap-didcomm-core = "0.x"
   tap-didcomm-node = "0.x"
   tap-didcomm-web  = "0.x"
   ```

   *If the crates are not yet on crates.io, you can add them via GitHub:* 

   ```toml
   [dependencies]
   tap-didcomm-core = { git = "https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs", package = "tap-didcomm-core" }
   tap-didcomm-node = { git = "https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs", package = "tap-didcomm-node" }
   tap-didcomm-web  = { git = "https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs", package = "tap-didcomm-web" }
   ```

2. **Rust Toolchain:** Ensure you have Rust 2021 edition or later installed, along with Cargo (which supports workspaces) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Prerequisites)). If you plan to use the library in an async context, having an async runtime like Tokio is recommended (the library is `async/await` throughout ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Usage)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Core%20library%20that%20handles%20message,DID%20resolvers%2C%20signers%2C%20encryptors%2C%20etc))).

3. **Build and Test:** Run `cargo build` to compile the library and `cargo test` to run the included tests and ensure everything is working ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=2)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Testing)). The library uses modern Rust error handling and comes with comprehensive test coverage ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Key%20features%3A)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Testing)).

### TypeScript WASM Wrapper Setup

If you want to use `tap.rs` in a JavaScript/TypeScript environment (for example, in a web browser or a Node.js service), you can compile it to WebAssembly and use the generated wrapper:

1. **Prerequisites:** Install [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) and ensure you have Node.js and npm available (these are needed for packaging and testing the WASM build) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=,js%20and%20npm%20%28for%20testing)).

2. **Build WASM Package:** Navigate to the `tap-didcomm-rs` repository (or your project workspace) and run `wasm-pack build --target web`. This will compile the Rust code to `wasm32-unknown-unknown` and produce a `pkg/` directory with the `.wasm` binary and JavaScript binding code ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=WASM%20Build)). (Use `--target nodejs` for a Node.js environment, or build both as needed.)

3. **Integrate into Project:** You can publish the generated package to npm or use it directly. If published (e.g., as `tap-didcomm` on npm), install it via: `npm install tap-didcomm`. Otherwise, link the local `pkg` as a dependency or copy the `pkg` contents into your web app. The package includes TypeScript type definitions (via `.d.ts` files) for the WASM exports, making it convenient to use in a TypeScript project ([didcomm-node - npm](https://www.npmjs.com/package/didcomm-node#:~:text=Under%20the%20hood)) ([didcomm-node - npm](https://www.npmjs.com/package/didcomm-node#:~:text=This%20package%20is%20written%20in,helps%20in%20packaging%20and%20publishing)).

4. **Initialization:** In your frontend code, import the module and initialize the WASM. For example: 

   ```ts
   import init, { DIDCommNode, Message } from "tap-didcomm";
   // Initialize the WASM module (loads the WebAssembly binary)
   await init();
   // Now you can use DIDCommNode, Message, etc.
   ```
   
   The `init()` function (or default import call) loads the WebAssembly module. After that, the `tap.rs` API is available in your JavaScript environment via the wrapped classes/functions. You can then create agents, pack/unpack messages, and use HTTP or other channels to send the messages, just as you would in Rust.

With the library installed and set up, you’re ready to utilize its core components to handle TAP flows in your application.

## Core Components & APIs 

`tap.rs` is composed of several core components that correspond to the TAP protocol’s functionality. These include the DIDComm messaging primitives, agent management, TAP message types, and node/server abstractions. Understanding these will help you effectively use the library.

### Processing DIDComm Messages

At the heart of TAP is the exchange of DIDComm messages between agents. The **`tap-didcomm-core`** crate provides the functionality to create, pack, and unpack DIDComm v2 messages. Key points include:

- **Message Structure:** All TAP messages conform to the DIDComm v2 JSON structure ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,2.md%20at%20main%20%C2%B7)). This means every message has an `id` (unique identifier), a `type` (identifies the TAP message schema, e.g. `https://tap.rsvp/schema/1.0#Transfer`), `from` and `to` DIDs, and a `body` with message-specific fields. For example, a plaintext TAP Transfer message might look like: 

  ```json
  {
    "id": "12345-67890-abcd",
    "type": "https://tap.rsvp/schema/1.0#Transfer",
    "from": "did:web:originator.example",
    "to": ["did:web:beneficiary.example"],
    "created_time": 1678901234,
    "body": { ... }
  }
  ``` 

  This JSON is then typically signed and/or encrypted to form the actual message sent over the wire ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=%7B%20%22id%22%3A%20%2212345,)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,band)). Using DIDComm ensures that messages are self-contained and can be transported over any channel (HTTP, email, etc.) while remaining secure ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,2.md%23%3A~%3Atext%3DTAIP%2Cwithin%2520the%2520context%2520of%2520TAP)).

- **Packing and Unpacking:** The library provides asynchronous functions to **pack** a message (apply a signature and/or encryption) and **unpack** a message (verify signature and decrypt). For example, you can create a `tap_didcomm_core::Message` and then use `pack_message(...)` to sign it, producing a JWS/JWE string. Conversely, incoming packed messages can be fed to an unpack or `receive` function to retrieve the plaintext and verify it. This is all built on the `ssi` (Self-Sovereign Identity) crate internally for cryptographic operations ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Core%20library%20that%20handles%20message,DID%20resolvers%2C%20signers%2C%20encryptors%2C%20etc)). The packing mode is flexible: you can choose plaintext (for testing or non-sensitive data), signed, or fully encrypted DIDComm messages as needed.

- **Example – Packing a Message:** Below is a simple example of creating and packing a DIDComm message in Rust:
  
  ```rust
  use tap_didcomm_core::{Message, PackingType, pack_message};
  use tap_didcomm_node::{DIDCommNode, NodeConfig};

  // Suppose we have a plugin with our agent's keys and DID (see "Managing Agents")
  let plugin = /* ... initialize your DIDComm plugin ... */;

  // Create a DIDComm node (agent instance) with default config
  let node = DIDCommNode::new(NodeConfig::default(), plugin.clone());

  // Compose a Transfer message (transaction proposal)
  let tx_body = serde_json::json!({
      "asset": "eip155:1/slip44:60", 
      "amount": "100", 
      "destination": "0xBeneficiaryAddress..."
  });
  let transfer_msg = Message::new("https://tap.rsvp/schema/1.0#Transfer", tx_body)?
      .from("did:example:alice")              // Originator agent DID
      .to(vec!["did:example:bob"]);           // Beneficiary agent DID

  // Pack (sign) the message for sending
  let packed = pack_message(&transfer_msg, &plugin, PackingType::Signed).await?;
  println!("Packed DIDComm message: {}", packed);
  ```

  In this snippet, we create a new `Message`, specify the `from` and `to` DIDs, and then call `pack_message` with `PackingType::Signed` to produce a signed DIDComm message (JWM). The `plugin` provided to `pack_message` supplies the necessary cryptographic context (keys, signatures, etc.) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=let%20message%20%3D%20Message%3A%3Anew%28,did%3Aexample%3Abob)). The result `packed` would be a JSON Web Signature (JWS) object in string form that can be transmitted to the recipient. Similarly, one could use `PackingType::Encrypted` for confidentiality (which would produce a JWE envelope).

- **Receiving and Unpacking:** To process an incoming message, `tap.rs` uses the **`DIDCommNode`** abstraction (from `tap-didcomm-node` crate). You can call `node.receive(packed_message).await?` to have the node unpack and verify a message ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=%2F%2F%20Receive%20and%20process%20the,await)). If the message is valid and intended for this agent, the node will parse it and route it to the appropriate handler (more on handlers below). Under the hood, unpacking will verify the JWS signature against the sender’s DID Document keys and decrypt if needed, ensuring the message integrity and authenticity before processing ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=sender%E2%80%99s%20identity,Transaction)). The library will reject messages that fail signature verification or that come from unexpected senders, but it’s also incumbent on the application to use the agent management (next section) to ensure only trusted DIDs are accepted.

### Managing Agents

In TAP, an **Agent** is an entity (often a service or software acting on behalf of a user or institution) that participates in the transaction authorization flow. Each agent is identified by a **Decentralized Identifier (DID)**, which in turn has an associated **DID Document** containing that agent’s public keys and service endpoints ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=0are,5.md%23%3A~%3Atext%3DAgents%2520are%2520identified%2520using%2520Decentralized%2CLD%2520node%2520synta)). Managing agents in `tap.rs` involves handling their identities (DIDs and keys) and their relationships (which agents are involved in a given transaction).

Important considerations for agent management in `tap.rs`:

- **DID and Keys:** To instantiate an agent, you will typically create or use an existing DID for that agent and load the corresponding private keys for signing. `tap.rs` doesn’t hard-code any particular DID method – you can use **did:web**, **did:key**, **did:ethr**, **did:pkh**, or others as suits your use case ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=ded%29%29.%20Many%20different%20,used%20in%20TAP%20examples%20include)). For example, an exchange might use a `did:web:exchange.com` DID (trust anchored in its domain) while an individual could use a `did:pkh:eip155:1:0x...` DID that corresponds to their Ethereum address ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,10%29.%20For)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,In)). What matters is that each agent has a DID and access to the DID’s **private keys** to sign messages, and that other agents can resolve that DID to get the **public keys** to verify signatures.

- **DIDComm Plugin Architecture:** The `tap-didcomm-node` crate uses a plugin system to manage cryptographic operations and DID resolution ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Core%20library%20that%20handles%20message,DID%20resolvers%2C%20signers%2C%20encryptors%2C%20etc)). When creating a `DIDCommNode`, you supply a `plugin` object that knows how to **resolve DIDs** (find the DID Document and keys) and how to **sign/decrypt** with your agent’s keys. The library is designed to be flexible: you could implement a plugin that, for example, queries a DID resolver for did:web via HTTP, or one that retrieves private keys from a secure vault for signing. Out of the box, you might use the default plugin which leverages the `ssi` crate’s capabilities (which include support for did:key, did:web, etc.) or write a custom one. In our examples above, `plugin` is an instance providing the agent’s DID and keys.

- **Multiple Agents and Roles:** TAP flows can involve more than two agents. For instance, an originator might have two agents (say, a user’s wallet and an exchange service) that both need to authorize a transaction. The TAP message format allows listing multiple agents and their roles in the transaction ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,5.md%23%3A~%3Atext%3DThe%2520following%2520are%2520the%2520attributes%2Carray)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=3.md%23%3A~%3Atext%3DAgents%2520can%2520have%2520specific%2520roles%2Cthe%2520execution%2520of%2520a%2520t%20ransaction%29%29.%20For%20example%2C%20TAIP,chain%20transaction%20or%20receive%20funds)). As a developer, you might manage this by running a `DIDCommNode` for each agent identity under your control. For example, if your institution has an “oracle” service and a “compliance” service that both act as agents, you could instantiate two nodes (each with its own DID and keys). The `agents` array in the Transfer message will list all agents’ DIDs, and each node will only respond to messages addressed to its DID. `tap.rs` ensures that a node only processes messages intended for its DID (`node.receive()` will check the `to` field or the encryption envelope to ensure it’s the recipient).

- **Agent State and Storage:** `tap.rs` itself focuses on message exchange and doesn’t mandate how you store agent state (such as whether a given transaction has been authorized yet). As a developer, you may maintain an application-level state machine or database tracking the progress of each TAP transaction thread (from Transfer -> Authorize -> Settle or Reject). The `thid` (thread ID) present in each message links them to the original Transfer request ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=4%23%3A~%3Atext%3D,4%23%3A~%3Atext%3DMessages%2520implement%2520TAIP%2Cattribute%2520of%2520the%2520message%29%29%20%28%5BTransaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip)), which you can use as a key to track conversation state. When managing multiple agents, ensure each agent knows when a transaction is completed or aborted, so it can stop waiting for further messages on that thread.

In summary, managing agents in `tap.rs` involves configuring each agent’s identity (DID and keys) via the plugin system and potentially running multiple `DIDCommNode` instances if you have multiple agents. Each node will take care of signing outgoing messages with its DID and verifying incoming messages from other agents’ DIDs. The developer’s responsibility is to set up proper DID resolvers (for example, ensuring your node can resolve `did:web` DIDs by fetching `.well-known` URLs, resolve `did:ethr` via an Ethereum RPC if needed, etc.) and secure the private keys for signing (more on security later). 

### TAP Message Types: Transfer, Authorize, Settle, Reject

TAP defines four core message types that make up the transaction authorization flow ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Developers%20integrating%20TAP%20should%20primarily,can)). `tap.rs` fully supports creating and handling each of these messages. Understanding their purpose and content is important for using the library effectively:

- **Transfer:** This is the transaction proposal message, initiating a TAP flow. It’s typically sent by the Originator’s agent to the Beneficiary’s agent to describe the intended transaction ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,3.md%23%3A~%3Atext%3DThis%2520specification%2520provides%2520the%2520messaging%2Ca%2520Transaction%252)). The Transfer message contains the *who, what, and how* of the transaction – e.g., the asset to be transferred, the amount, and (optionally) the identities of the parties and agents involved. In TAP (per TAIP-3), the Transfer’s `body` may include fields like `asset` (a chain-agnostic asset identifier, using [CAIP-19] format), `amount` (or `amountSubunits` for fungibles), party identifiers for originator/beneficiary, and possibly a `settlementAddress` ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,message%20if%20they%20approve)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Transaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip)). In `tap.rs`, you will create a Transfer message by instantiating a `Message` with type `"https://tap.rsvp/schema/1.0#Transfer"` and populating the body JSON accordingly. This message essentially says: *“I want to send X amount of Y asset to Z. Here are the details.”* Once sent, the originator’s agent will await an **Authorize** or **Reject** in response. (If the originator already included a settlement address, the beneficiary’s agent knows where the funds would go; if not, the beneficiary can provide an address in the Authorize.)

- **Authorize:** This message is sent by an agent to approve/accept the proposed transaction. In practice, the **beneficiary’s agent** is usually the one to Authorize (approving the incoming transfer), but any agent in the flow can send an Authorize if they have a say in approval ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Transaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip)). For example, a beneficiary exchange’s agent would send Authorize to confirm that the details in the Transfer are acceptable (correct destination account, compliance checks passed, etc.). On the originator’s side, an agent might also send an Authorize if internal policy requires dual control (e.g., a second-factor approval for large transfers) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Transaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip)). An Authorize message will reference the original Transfer (via the `thid` thread ID) and may carry additional info such as the settlement address if it wasn’t provided earlier ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Transaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FTAIPs%2Ftaip,20OPTIONAL)). In `tap.rs`, after receiving a Transfer, your beneficiary-side node might generate an Authorize message (type `"https://tap.rsvp/schema/1.0#Authorize"`) in response. You’d include any necessary fields in the body, like `settlementAddress` (so the originator knows where to send funds) or just an approval indicator. Once an Authorize is sent, the transaction is basically green-lit from that agent’s perspective. (Multiple Authorize messages can be involved if more than one agent must approve.)

- **Settle:** This message is sent by the originator’s side to indicate that the transaction is being executed on-chain (or has been executed). Typically, once the originator has received the necessary Authorize(s), they will broadcast the blockchain transaction. The originator’s agent (e.g. an exchange) then sends a **Settle** message to the beneficiary’s agent saying, in effect, *“The transfer is now settling on-chain.”* ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Authorize%20messages%20before%20sending%20Settle)). The Settle message can include a reference like a transaction hash or ID (`settlementId`) so that the beneficiary can track the on-chain transaction ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,eip155%3A1%3Atx%2F0x3edb98c24...7c33)). In `tap.rs`, you create a Settle message with type `"https://tap.rsvp/schema/1.0#Settle"`, and include fields such as the `settlementId` (formatted per CAIP-220, e.g., including chain id and tx hash) if available ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,eip155%3A1%3Atx%2F0x3edb98c24...7c33)). If the originator sends Settle *before* the transaction is mined (as a notification), it might omit the `settlementId` initially, but it should send another Settle with the actual tx hash once known ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=could%20look%20like%3A%20,4%23%3A~%3Atext%3D%252A%2520%2560%2540type%2560%2520%2Cone%2520agent%2520representing%2520the%2520originator)). Upon receiving a Settle, the beneficiary’s agent (and any other agents) will mark the TAP flow as complete/finalized. The Settle message effectively concludes the TAP handshake: the transfer is moving to the blockchain for final settlement ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Authorize%20messages%20before%20sending%20Settle)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=user%E2%80%99s%20wallet%20or%20an%20exchange%E2%80%99s,settlementId)). (Note: in some scenarios, multiple Settle messages might be sent – e.g., if two originator-side agents both send Settle confirmations, or if an agent sends a preliminary Settle and then an updated one with details.)

- **Reject:** This message can be sent by any participant to abort the transaction. A **Reject** indicates that the transaction should not proceed ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,perhaps%20the)). For example, a beneficiary agent might send a Reject if the Transfer details don’t match expectations or violate some policy (wrong amount, KYC failed, etc.), or an originator agent could reject its own request if it detects an issue (perhaps the beneficiary’s response was unacceptable or risk factors changed) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,perhaps%20the)). A Reject, like Authorize, is tied to the original Transfer’s thread and signals a negative response. The Reject message may include a human-readable `reason` in its body to explain why (e.g., `"Beneficiary name mismatch"` or `"Risk threshold exceeded"`) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,counterparty%20to%20log%20or%20display)). In `tap.rs`, you’d create a Reject message with type `"https://tap.rsvp/schema/1.0#Reject"` and include a `reason` if applicable. When a Reject is received by the originator’s side, they should consider the TAP flow terminated (no on-chain action will be taken unless they choose to modify the request and start a new TAP flow) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Reject%20halts%20the%20authorization%20flow,the%20transaction%20with%20corrected%20info)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,to%20try%20again%20with%20modifications)). Essentially, any required party’s Reject kills the transaction authorization; the originator would typically cancel the pending transaction or try again with new parameters outside of this thread.

In summary, these four message types form the core API of TAP. The library doesn’t force a specific sequence but the intended flow is: **Transfer → (zero or more Authorize) → either Settle or Reject** ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Overall%2C%20TAP%20introduces%20a%20generic,20it%29%29%20%28%5BTransaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.r%20svp%2FTAIPs%2Ftaip)). Using `tap.rs`, you will be constructing and consuming these message types. The library ensures each message’s envelope has the correct `type` and thread linking, but it’s up to your application logic to send them in the correct order and handle cases like missing approvals or rejects. The TAP specification recommends that after a Reject, no one should proceed to settlement ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,to%20try%20again%20with%20modifications)), and that originators wait for approval before sending Settle ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=that%20%E2%80%9CI%20am%20now%20settling,Authorize%20messages%20before%20sending%20Settle)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,20privacy)). By adhering to that flow in your usage of `tap.rs`, you ensure compliance with the protocol’s expectations.

### Running TAP Nodes and HTTP Services (tap-node and tap-http)

To facilitate communication between agents, `tap.rs` provides an abstraction of a **TAP node** (DIDComm node) and an optional HTTP service wrapper. This corresponds to running an agent that can asynchronously handle incoming and outgoing messages, and if needed, expose a network endpoint for other agents to reach it.

**TAP Node (DIDComm Node):** The TAP node is represented by the `DIDCommNode` struct (in `tap-didcomm-node`). A DIDComm node in this context is essentially an agent instance capable of receiving, processing, and sending DIDComm messages. Internally, `DIDCommNode` is built to integrate with the Actix actor model for asynchronous message handling ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=,architecture%20for%20custom%20message%20handlers)). However, you don’t need to use Actix directly to use it – you can interact with `DIDCommNode` via async methods as shown earlier (e.g., `node.receive(...)`). The node can be thought of as a stateful message router for a given agent:

- When you call `DIDCommNode::new(config, plugin)`, you create a node with a certain configuration (which might include things like an agent DID or routing rules in the `NodeConfig`) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=use%20tap_didcomm_core%3A%3A,NodeConfig)). You also pass in the `plugin` which the node will use for cryptographic tasks and DID resolution.
- The node can **receive messages** (via `node.receive(packed_message)`) which will unpack and verify the message and then determine what to do with it. By default, `tap.rs` includes a simple handler that could, for instance, log the message or store it. You can extend the node by registering custom handlers for different message types if you want to automate responses. For example, you could write a handler such that when a Transfer message arrives, the node automatically calls your business logic to decide and send an Authorize or Reject.
- The `DIDCommNode` supports running in WASM as well, so the same logic can handle messages in a browser context.
- If using Actix, the node can be started as an actor which will then receive messages via the actor system. The library provides integration for this, but it’s optional. 

In many cases, you might not need to manually deal with the actor model. Instead, you might simply use the node’s async API within an async runtime. For example, you could spawn a background task that listens for incoming HTTP requests (using the `tap-http` server below) and calls `node.receive` for each message, and another task that sends out messages via HTTP when needed. 

**TAP HTTP Service (`tap-http`):** To easily enable network communication, the `tap-didcomm-web` crate provides an HTTP server implementation (`DIDCommServer`) that wraps a DIDComm node ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=tap)). This is akin to a “TAP HTTP node” that listens on a port for incoming DIDComm messages and routes them to the node for processing. Key features of the HTTP server include CORS support and integration with Actix-Web framework for robust performance ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=tap)):

- **Starting the server:** You configure it with a host and port (and optional CORS settings) via `ServerConfig`, then instantiate `DIDCommServer::new(server_config, node_config, plugin)` and run it ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=Using%20the%20Web%20Server)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=let%20server%20%3D%20DIDCommServer%3A%3Anew,await)). This will internally create a `DIDCommNode` (with the provided `node_config` and `plugin`) and bind an HTTP server. For example:

  ```rust
  use tap_didcomm_web::{DIDCommServer, ServerConfig, CorsConfig};
  
  let server_cfg = ServerConfig {
      host: "0.0.0.0".to_string(),
      port: 8080,
      cors: CorsConfig::default()
  };
  let node_cfg = NodeConfig::default();
  let server = DIDCommServer::new(server_cfg, node_cfg, plugin);
  server.run().await?;  // This starts listening on 0.0.0.0:8080
  ```
  
  By default, the server exposes a RESTful API endpoint to receive messages (e.g., an HTTP POST endpoint where other agents can POST DIDComm messages) ([TransactionAuthorizationProtocol/tap-didcomm-rs - GitHub](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs//#:~:text=TransactionAuthorizationProtocol%2Ftap,Comprehensive%20error%20handling%3B%20Logging)). The exact routes are defined in the implementation (commonly something like POST `/messages` or `/` for the inbox – you can consult the documentation or configure the route). The CORS settings ensure that web clients can call the endpoint if needed (useful for browser-based agents).

- **Sending messages via HTTP:** When you want to send a DIDComm message to another agent, you would use that agent’s endpoint (which you might get from its DID Document’s service entries). For instance, if the counterparty is running a `tap-http` server at `https://agency.beneficiary.com/tap`, you would send your packed message as an HTTP POST to that URL. You can use any HTTP client for this (`reqwest` in Rust, or `fetch` in a browser, etc.). The `tap.rs` library doesn’t send HTTP for you (it leaves transport to you), but it prepares the message and envelope. In practice, you might integrate by having your node, upon producing a `packed` message to send, make an HTTP request to the peer’s endpoint. Similarly, when your `DIDCommServer` receives a POST, it will unpack the message and you can handle it in the node.

- **Asynchronous handling:** The HTTP server runs asynchronously. It will accept incoming connections and spawn handlers so that multiple messages can be processed concurrently. Inside, each incoming message is handed to the DIDComm node (which can process asynchronously, including awaiting any external calls like DID resolution). This design allows TAP agents to handle many authorization workflows in parallel without blocking, which is important for services like exchanges that might be coordinating multiple transactions at once.

- **Logging and Errors:** The `tap-http` server includes logging middleware and comprehensive error handling out-of-the-box ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=HTTP%20server%20implementation%20wrapping%20a,Features%20include)). If a message fails to unpack or verify, the server will respond with an error status. During development, having server logs enabled will help trace the flow of messages and diagnose issues (like invalid signatures or malformed messages).

Running a TAP HTTP server is optional but highly convenient for real-world deployments: it effectively gives your TAP agent an “address” (URL) where it can be reached by counterparties. In a typical deployment, each institution (VASP, wallet service, etc.) would run a TAP HTTP service at a well-known endpoint (advertised via their DID Document’s `service` section). Self-custodial wallets or browser-based agents might not run an HTTP server themselves; instead, they can communicate with others by directly calling the others’ servers or through a mediator. Because `tap.rs` is transport-agnostic, you could also use other channels (for example, integrate with an email or messaging queue if needed), but HTTP is the primary mechanism provided for simplicity.

**Summary:** Use `DIDCommNode` to manage your agent’s message processing logic, and use `DIDCommServer` (`tap-http`) if you need to expose an HTTP interface for receiving messages. Together, these allow your TAP agent to run continuously and handle the full lifecycle of a TAP conversation asynchronously.

## Examples & Usage Scenarios

This section provides concrete examples of how to use `tap.rs` in both Rust and TypeScript, covering common scenarios: setting up an agent, signing messages, running a TAP node, sending/receiving via HTTP, and using the library in a browser context. The examples assume some setup as described above (e.g., you have appropriate DIDs, keys, and configuration in place).

#### 1. Creating an Agent and Signing a Message (Rust)

In this example, we’ll create a simple TAP agent in Rust, prepare a Transfer message, and sign it. We assume you have generated or obtained a DID and its corresponding key for this agent (for illustration, we use placeholder values):

```rust
use tap_didcomm_core::{Message, PackingType, pack_message};
use tap_didcomm_node::{DIDCommNode, NodeConfig};
// Suppose these are our agent's identity details:
let agent_did = "did:web:originator.example";      // The agent's DID
// `plugin` is an object that knows the agent's DID, holds its private key, and can resolve other DIDs.
let plugin = setup_plugin_with_key(agent_did, my_private_key)?;

// Initialize the DIDComm node (agent) with default settings
let node = DIDCommNode::new(NodeConfig::default(), plugin.clone());

// Create a Transfer message proposing a transaction
let transfer_body = serde_json::json!({
    "asset": "eip155:1/slip44:60",         // Asset: ETH on Ethereum mainnet
    "amount": "2500000000000000000",      // Amount in wei (2.5 ETH)
    "originator": { "@id": "did:example:alice" },  // Originator party info (optional)
    "beneficiary": { "@id": "did:example:bob" }    // Beneficiary party info (optional)
});
let transfer_msg = Message::new("https://tap.rsvp/schema/1.0#Transfer", transfer_body)?
    .from(agent_did)
    .to(vec!["did:web:beneficiary.example"]);

// Sign the message using our agent's DID keys
let signed_transfer = pack_message(&transfer_msg, &plugin, PackingType::Signed).await?;

// `signed_transfer` is a JWS string that can be sent to the beneficiary's agent.
// For demonstration, print the compact JWS (it will be a long string of characters):
println!("Signed Transfer JWS: {}", signed_transfer);
```

In this snippet, `setup_plugin_with_key` represents whatever mechanism you use to create a `plugin` that implements DID resolution and signing (this could be provided by `tap.rs` or written by you to interface with your key storage). We then create a `Message` of type `Transfer` and fill in some example body fields: asset and amount (notice we used the smallest subunit for a fungible asset, as recommended by the TAP spec), and even included originator/beneficiary identifiers for context. We set the `.from()` to our agent’s DID and `.to()` to the counterparty’s DID. Calling `pack_message(... PackingType::Signed)` produces a signed message. At this point, the `signed_transfer` can be sent to the beneficiary agent over a transport (e.g., HTTP). The beneficiary’s agent will later unpack and verify this message.

#### 2. Running a TAP Node and Handling Incoming Messages (Rust)

Continuing from the previous setup, let’s consider receiving a response. Suppose our originator agent now gets an incoming DIDComm message (for example, an **Authorize** from the beneficiary). We can use the same `DIDCommNode` to handle it:

```rust
# use tap_didcomm_core::{PackedMessage};
# use tap_didcomm_node::DIDCommNode;
// ... continuing from previous setup ...
# let plugin = plugin.clone();
# let node = node;
# let incoming_authorize_jwm = get_incoming_message(); // placeholder: obtain the JWS/JWE from network
// Receive and process the incoming message
if let Err(err) = node.receive(&incoming_authorize_jwm).await {
    eprintln!("Failed to process incoming message: {:?}", err);
} else {
    println!("Incoming message processed successfully.");
    // You might inspect the node's state or logs to see what the message was.
}
```

Here, `incoming_authorize_jwm` is a placeholder for the packed message we received (perhaps via an HTTP POST to our server). Calling `node.receive(&incoming_authorize_jwm).await` will cause the node to: decrypt (if needed), verify the signature, and determine the message type. If it’s an Authorize, for instance, the default behavior might simply log it or mark it. In a real application, you’d likely have a custom handler or logic: for example, on receiving an Authorize, your originator agent might automatically proceed to send a Settle. You could implement this by extending `DIDCommNode` or by checking the unpacked message after `receive`. The `receive` method will return `Ok(())` if the message is valid and was handled. You can retrieve the last message or embed handlers by customizing the node (for instance, through the plugin or by subclassing the actor). 

**Asynchronous Handling:** Because all of this is `async`, you can incorporate it into an async runtime. For instance, if using Tokio, your main loop could `await` on `server.run()` (which runs indefinitely handling HTTP) and concurrently have other tasks. The `node.receive` will be invoked for each incoming message (as shown above), and you can respond by constructing and sending new messages with your node as needed.

#### 3. Sending and Receiving via HTTP (Rust & cURL)

Using the `tap-didcomm-web` server makes the above easier by handling the HTTP details. Let’s demonstrate how two agents (originator and beneficiary) might communicate over HTTP using `tap.rs`:

- **Originator (Agent A)** runs a server on `localhost:8080`.
- **Beneficiary (Agent B)** runs a server on `localhost:9090`.

Each knows the other’s DID (which includes a service endpoint URL). We’ll simulate Agent A sending a Transfer to Agent B via HTTP, and Agent B responding with Authorize, also via HTTP.

**Agent A (Originator) setup and send:**

```rust
use tap_didcomm_web::{DIDCommServer, ServerConfig, CorsConfig};
// (Assume plugin_a is set up for Agent A's DID and keys)
let server_a = DIDCommServer::new(
    ServerConfig { host: "127.0.0.1".into(), port: 8080, cors: CorsConfig::default() },
    NodeConfig::default(),
    plugin_a
);
tokio::spawn(async {
    server_a.run().await.expect("Server A failed");
});
// Once the server is running, send a Transfer message to Agent B:
let transfer_msg = /* construct Message as shown above */;
let packed = pack_message(&transfer_msg, &plugin_a, PackingType::Signed).await?;
// Use an HTTP client to POST the packed message to B's endpoint (localhost:9090)
let client = reqwest::Client::new();
let res = client.post("http://127.0.0.1:9090/messages")
    .header("Content-Type", "application/json")
    .body(packed.to_string())
    .send().await?;
if res.status().is_success() {
    println!("Transfer sent to Agent B");
}
```

**Agent B (Beneficiary) setup and auto-respond:**

```rust
// (Assume plugin_b is set up for Agent B's DID and keys)
let server_b = DIDCommServer::new(
    ServerConfig { host: "127.0.0.1".into(), port: 9090, cors: CorsConfig::default() },
    NodeConfig::default(),
    plugin_b
);
// Suppose we add a simple handler such that when a Transfer is received, B automatically Authorizes:
server_b.node().register_handler("https://tap.rsvp/schema/1.0#Transfer", |node, msg| {
    // This is pseudocode for registering a handler in the node
    // In practice, you might subclass DIDCommNode or use an actor message
    let transfer = msg; // unpacked Message
    println!("Received Transfer: {:?}", transfer.body);
    // Prepare an Authorize response
    let auth_msg = Message::new("https://tap.rsvp/schema/1.0#Authorize", serde_json::json!({}))
        .unwrap()
        .from(node.did().to_string())
        .to(vec![transfer.from.clone().unwrap()]);
    let packed_auth = futures::executor::block_on( pack_message(&auth_msg, node.plugin(), PackingType::Signed) ).unwrap();
    // send it back via HTTP (to Agent A's service endpoint)
    let _ = futures::executor::block_on( reqwest::Client::new()
             .post("http://127.0.0.1:8080/messages")
             .header("Content-Type", "application/json")
             .body(packed_auth.to_string())
             .send() );
});
server_b.run().await?;
```

*(The above handler registration is illustrative – the actual API to register handlers may differ. You could also achieve auto-response by overriding an Actor’s handler method in an Actix context.)*

In summary, Agent B’s server will listen on port 9090. When Agent A’s HTTP POST arrives at `/messages` with the Transfer JWS, Server B will pass it to B’s node. We registered a handler for Transfer that logs the content and immediately creates an Authorize message in response. It then uses an HTTP client to POST the Authorize to Agent A’s `/messages` endpoint (localhost:8080). Agent A’s server receives the Authorize and processes it (here we didn’t show A’s handler, but A could then log it and proceed to send a Settle similarly).

**Using cURL or other clients:** If you want to test the HTTP interface manually, you can use `curl` commands. For example, assuming you have a packed message saved in `transfer.json` file, you could do:

```bash
curl -X POST http://127.0.0.1:9090/messages \
     -H "Content-Type: application/json" \
     -d @transfer.json
```

The response from the server will indicate success or failure (and in case of failure, might include an error message). 

The HTTP API makes it easy to integrate with systems in any language – they just need to POST the correct DIDComm message to the agent’s endpoint. Meanwhile, the Rust `tap.rs` internals handle the crypto and validation.

#### 4. Using the TypeScript Wrapper in a Browser Application

One powerful feature of `tap.rs` is that the same logic can run in a web browser via WebAssembly. This enables scenarios like a non-custodial web wallet participating in TAP with an exchange. Let’s outline a simple example of using the TypeScript API in a browser context.

Imagine we have built the WASM package and included it in our web app (either via bundler or directly). We will:

- Initialize the WASM module.
- Create an agent (node) instance.
- Generate a Transfer message and sign it.
- Send it via HTTP (using `fetch`) to the counterparty.
- (We won’t fully show receiving in the browser, but note it could similarly handle incoming messages with the same library.)

**Browser JS/TS code:**

```ts
import initTap, { DIDCommNode, Message, Packing } from "./tap_didcomm.js";  // assuming local build or npm package

// 1. Initialize the WASM module
await initTap();  // this loads the WebAssembly, must be awaited

// 2. Create a DIDComm node for our agent (e.g., the user's wallet agent)
const node = new DIDCommNode(); 
// (If needed, configure the node or plugin for keys. This example assumes 
// the WASM module has been built with an embedded key for simplicity, or you might 
// call node.setKey or similar if provided by the API)

// 3. Define our DIDs for the parties
const originatorDID = "did:pkh:eip155:1:0xUserWalletAddress";   // user's self-hosted wallet DID
const beneficiaryDID = "did:web:exchange.example";             // exchange's agent DID

// 4. Create a Transfer message
let transfer = new Message("https://tap.rsvp/schema/1.0#Transfer", {
    asset: "eip155:1/slip44:60",
    amount: "2500000000000000000",
    originator: { "@id": originatorDID },
    beneficiary: { "@id": beneficiaryDID }
});
transfer = transfer.from(originatorDID).to([beneficiaryDID]);

// 5. Pack (sign) the message. We use Packing.Signed for a signed JWM.
const packedTransfer = await node.pack(transfer, Packing.Signed);

// 6. Send the packed message to the beneficiary's TAP HTTP endpoint via fetch
await fetch("https://api.exchange.example/tap/messages", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: packedTransfer
});
console.log("Transfer message sent from browser wallet to exchange.");
```

A few notes on the above example:

- We used `did:pkh` for the originator (user) DID, representing their blockchain address ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,In)). In a real app, the user’s wallet might derive a DID from their key or use a did:key. The WASM module would need to know the user’s private key to sign. This could be done by having the user input a key or by integrating with a browser wallet’s signing capability (one could envision a callback to sign the bytes). For simplicity, assume the WASM node was preloaded with the key.
- The `DIDCommNode` in WASM likely has methods to set a secrets resolver or key (depending on implementation). In some DIDComm WASM libraries (like SICPA’s didcomm-rust WASM), the approach is to supply secrets via an API. The specifics for `tap.rs` TypeScript wrapper should be checked in its documentation. Our example glosses over this by assuming a default key.
- We send the HTTP request to `https://api.exchange.example/tap/messages` which is the exchange’s TAP endpoint. In practice, the DID Document of `did:web:exchange.example` would tell us the exact URL to post to (which could be something like `https://exchange.example/.well-known/tap` or a specific path). We would use that dynamically, but here we hard-coded for clarity.

On the receiving side, if the exchange responds with an Authorize, how would our browser agent get it? Since our browser isn’t running a server, one approach is **long-polling or WebSocket**. For example, the browser could periodically check an inbox service or maintain a WebSocket connection to a relay that holds messages for it. Another approach is that the exchange, knowing the user’s agent is not reachable directly, could communicate Authorize through an out-of-band mechanism (like redirecting the user to a link or sending it via an existing channel). These are higher-level architecture decisions. The important point is that `tap.rs` in the browser can **unpack** any message it receives using the same APIs (e.g., `node.unpack(packedMessage)` or simply `node.receive(packedMessage)` if such exists in the WASM binding).

Using the TypeScript wrapper in a Node.js environment would look similar, except you might use `require()` or different import syntax and you wouldn’t need CORS. The key benefit is that the **TypeScript API mirrors the Rust API**, so developers can apply consistent logic across backend and frontend. This is enabled by WASM and the fact that `tap.rs` is written in Rust with portability in mind ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=A%20modular%20DIDComm%20v2%20library,packing%2Funpacking%2C%20signing%2C%20encryption%2C%20and%20verification)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=,Detailed%20documentation)).

## Integration with Existing Systems

TAP is meant to integrate into the existing blockchain and compliance ecosystem, not exist in isolation. Here we discuss how to integrate `tap.rs` (and TAP in general) with various systems and standards:

### DID Resolvers and DID Methods

Because TAP relies on Decentralized Identifiers for naming agents, integrating with **DID infrastructure** is crucial. `tap.rs` is flexible about which DID methods you use, but you need to configure resolvers for those methods so that your agent can look up counterparty DID Documents (for verifying signatures and finding service endpoints).

- **Supported DID Methods:** TAP doesn’t mandate a specific DID method; you can use whatever is suitable ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=ded%29%29.%20Many%20different%20,used%20in%20TAP%20examples%20include)). Common choices include:
  - **did:web:** often used by institutions like exchanges or banks. A did:web DID (e.g., `did:web:example.com`) is resolved by fetching a JSON document from a well-known URL on that domain. Integrating this in `tap.rs` means your DID resolver plugin should perform an HTTPS GET to `https://example.com/.well-known/did.json` (or the path corresponding to the DID) and parse the DID Document. The `ssi` crate used by `tap.rs` has built-in support for did:web resolution, which you can enable or extend. Using did:web anchors the trust in the domain’s DNS/SSL, which is practical for VASPs who already have web infrastructure ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,10%29.%20For)).
  - **did:pkh:** suitable for identifying blockchain addresses in a DID format ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,In)). For example, `did:pkh:eip155:1:0xabc...` represents an Ethereum address on mainnet. The DID Document for a did:pkh contains the address’s public key (if obtainable) or simply the fact that it’s a blockchain account. `ssi` can generate a rudimentary DID Document for did:pkh (the public key might not be directly extractable from just an address without an associated signature, so often did:pkh is used as an identifier and the verification happens by having that address sign something). In TAP, did:pkh is useful for self-hosted wallets – you could use the user’s own crypto address as their DID in the protocol, though you’ll need an alternate channel to get their public key for DIDComm (perhaps pre-exchange of keys or use did:key instead for actual messaging).
  - **did:key:** a simple method where the DID itself encodes a public key. This is very convenient for testing and for cases where a lightweight DID is needed. For example, `did:key:z6Mk...` is derived from an Ed25519 key. `tap.rs` (via `ssi`) supports did:key out of the box. If your agents use did:key, you can generate a new DID per agent easily and exchange them. The drawback is did:key DIDs don’t have an easy way to discover service endpoints (no lookup other than exchanging the DIDDoc out-of-band), but you can include the service endpoint manually in the DID Document you share.
  - **did:ethr, did:ion, etc.:** If you use blockchain-based DIDs like did:ethr (Ethereum registry) or did:ion (Bitcoin sidetree), integration might require connecting to their networks. For did:ethr, you’d need a Web3 provider to read the DID Document from the smart contract registry. `tap.rs` plugin architecture allows you to implement this: for instance, you could use the `didkit` or `ssi` capabilities to resolve did:ethr by providing an RPC URL. Did:ion resolution would involve the Ion DID resolution library or service. These methods are more decentralized but add complexity. Use them if your use case or ecosystem demands it.

- **Implementing a Custom Resolver:** To integrate with these DID methods, you might implement the `DIDResolver` trait (if provided by the library) in your plugin. The plugin could maintain a map of known DIDs to DID Documents (for cached or static relationships) and fall back to network calls for unknown ones. For example, when `node.receive()` gets a message from `did:web:bank.com`, the plugin’s resolver method will be invoked; you can then fetch `https://bank.com/.well-known/did.json`, parse it (perhaps using `ssi::did::Document` parsing functions), and return the public keys to the `tap.rs` core for signature verification ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=5.md%23%3A~%3Atext%3DAgents,https%3A%2F%2Fgithub.com%2FTransactionAuthorizationProtocol%2FTAIPs%2Fblob%2Fmain%2FTAIPs%2Ftai%20p)). 

- **Ensuring Up-to-date DID Docs:** Integrate with the CA authorities or blockchain events as needed to keep DID Documents current. If a partner rotates their keys (updates their DID Document), your resolver should fetch the latest version. Some DID methods (like did:web) don’t have a built-in versioning, so coordination is off-chain (maybe via communication or documentation). Others (like did:ion) have internal versioning or require refresh via a blockchain. Make sure your integration accounts for that – for instance, always fetch did:web fresh at the start of a TAP flow, or cache for only a short period.

By properly integrating DID resolution, `tap.rs` will be able to **verify signatures** on incoming messages (using the sender’s public key from their DID Document) and **encrypt messages** to recipients (using their public encryption keys from the DID Document). This is fundamental to the security of TAP, as it confirms you’re talking to who you think you are, and only they can read the messages ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=5.md%23%3A~%3Atext%3DAgents,https%3A%2F%2Fgithub.com%2FTransactionAuthorizationProtocol%2FTAIPs%2Fblob%2Fmain%2FTAIPs%2Ftai%20p)).

### VASP Compliance and Travel Rule Integration

One of TAP’s primary use cases is to facilitate compliance with regulations like the FATF **Travel Rule** for VASPs (Virtual Asset Service Providers). `tap.rs` can be integrated with a VASP’s compliance systems to automate the exchange of required information alongside transaction approvals ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,20privacy)). Here’s how:

- **Exchanging Travel Rule Data:** In a VASP-to-VASP transfer, each side needs to share **originator and beneficiary identity information** (such as names, account numbers, government ID numbers, etc.) as required by regulations ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,20privacy)). TAP messages (Transfer and/or Authorize) support attaching this data in a private, secure way. The TAP spec suggests using *Selective Disclosure* or *Verifiable Credentials* for this purpose (see TAIP-8). In practice, this means you might include an attachment or a credential in the **Transfer** message body that contains the sender’s Travel Rule information. Or the beneficiary VASP’s agent might respond with a request for info, and the originator agent provides it. `tap.rs` can carry arbitrary JSON in the message body, so you can integrate your compliance payloads directly. For example, you could add a field `originatorInfo` containing a JSON object of PII (encrypted if the channel between VASPs isn’t fully confidential, though typically the DIDComm encryption covers it). Because all TAP messages are end-to-end encrypted, this PII stays confidential between the VASPs ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,while%20still%20fulfilling%20compliance%20checks)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=6.md%23%3A~%3Atext%3DAdditional,W3C%20Verifiable%20Credentials%2FDIDComm%20attachments%20typically)).

- **Policy Checks and Automation:** A VASP’s backend compliance system (e.g., a Travel Rule solution or database of blacklisted addresses) can hook into the TAP agent’s workflow. Using `tap.rs`, when a Transfer is received by a beneficiary VASP’s agent, you can parse the originator’s info and run it through sanctions screening, KYC verification, etc., before deciding to Authorize. If everything checks out, your agent (via code) sends an Authorize; if something fails, it sends a Reject with reason (e.g., “Beneficiary name mismatch” or “Regulatory block”). On the originator side, if an Authorize comes in with additional requirements (perhaps the beneficiary asks for more info via a custom field or a follow-up message), the originator’s agent can supply that (maybe as another message or by including in Settle if allowed). Essentially, `tap.rs` serves as the messaging bus between two compliance departments, with the advantage that it’s structured and secure.

- **Travel Rule Specific Protocols:** While TAP itself is not a Travel Rule protocol per se, it can transport Travel Rule messages. Some jurisdictions have specific formats (like IVMS101 for identity data). You can embed an IVMS101 payload in TAP. Or if both VASPs support it, they could use a verifiable credential format for the identity info. TAIP-8 outlines a way to request and deliver verifiable credentials during a TAP flow ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=6.md%23%3A~%3Atext%3DAdditional,W3C%20Verifiable%20Credentials%2FDIDComm%20attachments%20typically)), which could be your Travel Rule compliance info. `tap.rs` doesn’t enforce how you format this data – it gives you the channel to exchange it securely. Thus, integrating might simply be: call your compliance API to get an IVMS101 JSON for the customer, attach it to the Transfer message; on the other side, parse that JSON and feed it into your verification system.

- **Audit Trail:** TAP provides an audit log of messages that can prove compliance actions were taken off-chain. Ensure you persist the messages (or at least the pertinent info) in your system of record. For example, log the fact that “Transaction X was approved by Beneficiary VASP Y at time T with Travel Rule info attached.” Since messages are signed, you even have cryptographic proof of what was exchanged, which is valuable for audits or dispute resolution.

- **Triggering On-Chain Transaction:** Only once all parties have Authorized (including internal approvals) will the originator broadcast the on-chain transaction ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,20privacy)). You can integrate `tap.rs` with your exchange’s transaction submission queue such that a transaction is only released from the queue when a corresponding TAP flow yields the required Authorize messages. Conversely, if a Reject is received, your system should remove or mark the transaction as cancelled. Many VASPs have a “pre-transfer hold” for Travel Rule – TAP fits exactly in that window to decide whether to proceed.

By integrating `tap.rs` in this way, VASPs can comply with the Travel Rule **privately and automatically**. They exchange required customer data directly, not on the public blockchain, fulfilling regulations without compromising user privacy ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,the%20beneficiary%20exchange%20has%20vetted)). TAP essentially acts as a secure tunnel for compliance info and mutual confirmation of addresses before the funds move, reducing errors and ensuring both sides are in agreement.

### Web & Mobile Clients via WASM

Not only institutions benefit from TAP – end-users with self-custodial wallets can also participate to improve security of transfers. The WebAssembly support in `tap.rs` means you can integrate TAP into **web browsers or mobile apps**, bringing them into the TAP network:

- **Browser Wallet Integration:** Suppose you have a browser-based crypto wallet (or a browser extension). By embedding `tap.rs` (compiled to WASM), the wallet can act as a TAP agent for the user. For example, when a user wants to withdraw from Exchange A to their wallet, Exchange A can initiate a TAP Transfer to the user’s wallet agent. If the wallet’s web app is open (or the extension is running), it receives the TAP message (maybe via a push notification or by the user polling). The wallet can then display to the user: “Exchange A is requesting to send you 2.5 ETH. Do you approve and confirm your address?” If the user clicks approve, the wallet’s TAP agent (running in WASM) sends an Authorize message back to the exchange’s agent ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=custodial%20wallet%20service%20,and%20security%20of%20customer%20on)). The exchange, upon receiving this Authorize, now knows the user is ready and the address is correct (since the Authorize could carry the `settlementAddress` which the wallet’s agent knows – essentially confirming “send the funds here, I’m expecting them”). This dramatically reduces the chance of the user losing funds by mistake, because if something was wrong (say the user didn’t initiate this or the address was wrong), the user would Reject and the exchange would not send the crypto ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=custodial%20wallet%20service%20,and%20security%20of%20customer%20on)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=intent,20Drama)).

- **Mobile Wallets:** On mobile, one could compile `tap.rs` to a native library or use the WASM in a WebView. This would allow mobile non-custodial wallets to also speak TAP. For instance, a mobile wallet app could incorporate a TAP listener that checks an inbox (through the wallet’s backend or via push) for incoming TAP requests. Using the same logic, the mobile user authorizes or rejects. The actual networking might be handled by a lightweight cloud service that the wallet trusts (since mobile devices aren’t always online to run an HTTP server). That service could store an encrypted message for the user until the device comes online. Once retrieved, the WASM can unpack it on-device (so the keys stay with the user) and the user can respond.

- **WASM Performance:** Modern browsers can handle cryptographic operations in WASM efficiently. `tap.rs` uses strong crypto (e.g., X25519, Ed25519, AES-GCM), but these run quite fast in WASM thanks to optimizations. This means even a relatively slow device can handle a TAP flow (which involves a few signatures and encryption operations) almost instantly. From the user’s perspective, approving a transaction via TAP should only introduce minimal delay (a second or two for messages to round-trip over the internet).

- **Integration Considerations:** When integrating into a web app, be mindful of **security**. The private key for the user’s DID should be securely stored (in the browser, this might be in local storage protected by a password, or derived from the user’s wallet seed). You wouldn’t want the WASM module’s memory leaking the key. Use the best practices of the wallet (some wallets might offload signing to hardware or a secure element; TAP can accommodate that by allowing an external signing function in the plugin). Also, ensure the browser only communicates with known endpoints (like the exchange’s HTTPS URL obtained from a DID Document) to avoid any Man-in-the-Middle. The DIDComm messages are encrypted anyway, but transport security and correct DID resolution protect against spoofing.

In summary, `tap.rs`’s WebAssembly support breaks down the barrier between backend services and user-facing clients in the TAP network. A user’s browser or phone can become a TAP agent, which means **end-to-end** authorization flows: the person who ultimately owns the funds can have a say in authorizing a transaction *before* it’s broadcast. This leads to safer crypto transfers (no more surprise withdrawals or deposits to wrong addresses) and a seamless integration of compliance checks directly with user interaction (e.g., the user could confirm identity info as part of the TAP exchange if ever needed). By integrating TAP into web and mobile clients, developers can offer a much richer and safer payment experience than traditional blind sends to blockchain addresses ([TAP - The Transaction Authorization Protocol for public blockchains](https://tap.rsvp/#:~:text=Avoid%20loss%20of%20customer%20funds,loss%20or%20theft%20of%20funds)) ([TAP - The Transaction Authorization Protocol for public blockchains](https://tap.rsvp/#:~:text=Transactions%20between%20people%20rather%20than,cryptographic%20addresses)).

## Security & Best Practices 

Security is paramount in TAP, as it deals with pre-transaction coordination and sensitive data. When using `tap.rs`, keep in mind the following security guidelines and best practices to ensure your implementation remains robust:

- **Message Signing & Verification:** Always use cryptographic signatures on TAP messages, and always verify them. By design, every TAP message **must be signed** by the sender’s DID private key ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,band)). The `tap.rs` library handles signing when you pack a message (if you choose `Signed` or `Encrypted` packing), and verification when unpacking. As a developer, you should enforce that incoming messages have valid signatures and that the `from` field matches the key that signed it. Do not accept any message that fails verification or claims to be from a DID that it isn’t. This ensures **authenticity and integrity** – no attacker can alter a message or inject a fake message into your TAP flow without detection ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,band)). Also, use the thread `thid` checks: only accept Authorize/Settle/Reject that properly reference an existing Transfer you initiated and from parties you expect ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=sender%E2%80%99s%20identity,Transaction)). For example, if you initiated a Transfer with agents A and B, and suddenly you get a message from C, that should be discarded as invalid ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=sender%E2%80%99s%20identity,Transaction)).

- **End-to-End Encryption:** It’s highly recommended to encrypt TAP messages (use `PackingType::Encrypted` or Authenticated Encryption) whenever possible. DIDComm supports anoncrypt (anonymous encryption to a DID, hiding sender) and authcrypt (authenticated encryption, revealing sender to recipient). Encryption ensures **confidentiality**: only intended participants can read the message contents ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,internet%2C%20encryption%20prevents%20eavesdroppers%20from)). This is important if your messages contain any private information (e.g., personal data for compliance or even just the fact that two parties are transacting). `tap.rs` via `ssi` supports strong encryption algorithms (like ECDH-1PU + XC20P or A256GCM) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=,521%29%20and%20X25519)) ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=,Compressed%20NIST%20curve%20point%20support)). Use them. In practice, this means when calling `pack_message`, prefer `PackingType::Encrypted` (with signing inside encryption if you need both). The overhead is minor compared to the security gain. Additionally, run your HTTP servers over TLS (HTTPS) so that the channel is also encrypted ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=it%E2%80%99s%20sent%20over%20the%20public,https%3A%2F%2Fgithub.com%2FTransactionA)). While DIDComm encryption is end-to-end, TLS adds another layer (“defense in depth”) and protects against traffic analysis or an attacker who might try to meddle with the HTTP transport ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=it%E2%80%99s%20sent%20over%20the%20public,https%3A%2F%2Fgithub.com%2FTransactionA)). Between DIDComm encryption and TLS, your TAP traffic should remain confidential and untampered.

- **Secure Key Management:** The private keys corresponding to your agents’ DIDs are the keys to the kingdom. Protect them rigorously:
  - On the server side (e.g., exchange or bank agents), **store keys in secure hardware** if possible. Using HSMs (Hardware Security Modules) or secure enclaves to perform signing can prevent keys from being stolen even if your server is compromised ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=suspicious%20messages%2C%20and%20possibly%20requiring,own%20risk%20engine%20to%20authorize)). `tap.rs` can integrate with such setups by providing a custom plugin that interfaces with your HSM (for instance, the plugin’s sign method can call out to the HSM).
  - Do not hard-code keys in code or config files. Use environment secrets or secure vault services to inject keys at runtime, and keep them encrypted at rest.
  - For browser/mobile agents, the user’s keys should be derived from a seed phrase or provided via a wallet. Use the Web Crypto API or similar to store keys, and consider user passphrases to encrypt them. A compromised browser context could potentially leak keys, so minimize exposure (e.g., zero out WASM memory after use if possible, and don’t persist private keys in plain text).
  - Implement **DID rotation** procedures. If an agent’s key is suspected to be compromised, rotate its DID Document to a new key and distribute the new DID info to counterparties (out-of-band or via an update message if appropriate). Because TAP is off-chain, you might not have an automated DID rotation built in (unless using did:ion or something with support). But you could, for instance, include in a Reject or a special message: “I am rotating keys, please refresh my DID Document.” More simply, maintain an out-of-band channel to notify important partners.
  - Consider **multi-signature approvals** for high-value transactions ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=suspicious%20messages%2C%20and%20possibly%20requiring,own%20risk%20engine%20to%20authorize)). TAP allows multiple agents to Authorize. So for a big transfer, you might require two separate private keys (perhaps held by different departments or devices) to both send Authorize. This way, if one key/agent is compromised, the attacker still can’t fully authorize a fraudulent transaction alone ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,wallet%20being%20hacked%20%E2%80%93%20TAP)).
  
- **Agent Identity Verification:** Establish trust in your counterparties’ DIDs. Before you start exchanging TAP messages with a counterparty, you should **authenticate their DID out-of-band**. For example, if you’re dealing with a new exchange, you might retrieve their DID (did:web or did:pkh) from a known directory or directly from them through an authenticated channel. Verify a did:web by checking the TLS certificate of the domain matches who you think it is. For did:pkh (an address), you might require the counterparty to prove control of that address (e.g., sign a challenge with the private key). While DIDComm ensures the message is from whoever controls that DID’s keys, it doesn’t tell you if that DID is the entity you intend unless you have some external assurance (trust on first use, a registry of trusted DIDs, etc.). Thus, maintain a list of trusted counterparty DIDs and have processes to validate new ones (similar to exchanging PGP keys or obtaining an API key).
  
- **Limiting Exposure of Data:** Only include sensitive data in TAP messages when necessary, and even then, prefer to send it **after** establishing that it’s needed. For instance, don’t automatically include full Travel Rule PII in every Transfer if the counterparty might not need it (maybe they only need it for large transfers). You can negotiate that via TAP if needed (TAIP-8 style request). If you do include, make sure it’s encrypted (again, DIDComm encryption ensures only the legit recipient sees it, not an eavesdropper or the blockchain) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=6.md%23%3A~%3Atext%3DAdditional,W3C%20Verifiable%20Credentials%2FDIDComm%20attachments%20typically)). Also, use the `reason` field in Reject messages judiciously – it’s plaintext for the recipient, but don’t reveal too much about your internal systems or logic in it (and it should never contain sensitive PII, since multiple parties might see it).
  
- **Prevent Replay Attacks:** The DIDComm spec along with TAP’s threading and message IDs help with this. Each message has a unique `id`, and every response carries a `thid` (thread id) linking to the original Transfer ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=4%23%3A~%3Atext%3D,4%23%3A~%3Atext%3DThere%2520are%2520three%2520primary%2520actions%2Can%2520agent%2520can%2520take)). Ensure your implementation checks that it has not seen a given `id` before, to avoid processing the same message twice if an attacker replays it. `tap.rs` could handle some of this (for example, storing thread state), but if not, you might implement a cache of recent message IDs. Because messages are signed, an attacker can’t modify them, but they could try to resend an old Authorize to trick you. However, if you’ve already settled that thread, you should ignore further messages on it. It’s good practice to include (or have internal) expiration times – e.g., the `expires_time` in DIDComm or an application-level timeout for how long you’ll wait for Authorize. If something comes *way* later unexpectedly, you likely drop it.
  
- **Transaction Finality and On-Chain Matching:** When a Settle message is received with a `settlementId` (tx hash), your agent should verify that hash on-chain corresponds to the expected transaction (correct amounts, addresses). This step is post-TAP but important: someone could theoretically authorize then send a different transaction. TAP can’t prevent a malicious originator from deviating at the last step (e.g., sending a different amount) – but it provides the info to detect it ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=sending%20a%20transaction%20that%20wasn%E2%80%99t,20risk%29%29%20%28%5BTransaction%20Authorization%20Protocol%5D%28https%3A%2F%2Ftap.rsvp%2FT%20AIPs%2Ftaip)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=settle%20to%20the%20beneficiary%20without,to%20Authorize%2C%20you%20expect%20the)). Your beneficiary agent upon seeing the actual tx hash can check the blockchain; if something is wrong, you at least have evidence and can take action (outside of TAP). Generally, though, if all parties followed TAP, the on-chain tx should match what was agreed (and originators have a reputational incentive to not cheat, as highlighted by the spec’s game theory reasoning ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=settle%20to%20the%20beneficiary%20without,to%20Authorize%2C%20you%20expect%20the)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=settle%20after%20everyone%20authorized%2C%20risks,you%20won%E2%80%99t%20back%20out%20arbitrarily))).
  
- **Operational Security:** Finally, treat your TAP infrastructure with the same care as you treat your hot wallet infrastructure. While TAP doesn’t directly move funds, a compromise in TAP (e.g., an attacker sending false Authorize or intercepting messages) could lead to misrouting funds or loss of funds if combined with other issues. Monitor your TAP node’s activity. Log all messages and keep an eye out for anomalies (like an unexpected Reject or an unknown DID contacting you). Use firewall rules to only allow your server to communicate with known partner endpoints if possible. 

By following these best practices, you ensure that integrating `tap.rs` adds security to your transactions rather than introducing new vulnerabilities. TAP is designed with a strong security model (end-to-end signatures and encryption, decentralized identity, explicit authorization) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=TAP%20is%20designed%20with%20security,and%20how%20they%20are%20mitigated)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=,Implementations%20should%20use%20DIDComm%E2%80%99s)) – but it must be used correctly. Each participant in a TAP flow is responsible for validating every message before acting on it ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=all%20conditions%20met%29,security%20of%20the%20underlying%20blockchain)). In short: **trust, but verify** every step. If you do so, you gain a robust safeguard against fraud, mistakes, and compliance issues in your blockchain transactions.

## References & Further Reading

- **TAP Whitepaper & Specifications (TAIPs):** *Transaction Authorization Protocol* – Comprehensive documentation of TAP can be found in the TAIP (Transaction Authorization Improvement Proposals) repository ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=The%20,https%3A%2F%2Fgithub.com%2FTransactionA)) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=Developers%20integrating%20TAP%20should%20primarily,can)). This includes the main protocol spec (message formats and flows) and extensions like TAIP-8 (Selective Disclosure) and TAIP-9 (Relationship Proofs) ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=6.md%23%3A~%3Atext%3DAdditional,W3C%20Verifiable%20Credentials%2FDIDComm%20attachments%20typically)). *(GitHub: TransactionAuthorizationProtocol/TAIPs)*

- **tap-didcomm-rs (GitHub Repository):** The source code for `tap.rs` (Rust DIDComm implementation for TAP) is available on GitHub ([GitHub - TransactionAuthorizationProtocol/tap-didcomm-rs: General purpose Rust implementation of DIDComm for use in native and typescript (WASM) use cases](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs#:~:text=A%20modular%20DIDComm%20v2%20library,packing%2Funpacking%2C%20signing%2C%20encryption%2C%20and%20verification)). It contains the Rust crates, examples, and README documentation on usage. This is useful for deep dives into the implementation or if you need to extend the library.

- **DIDComm v2 Specification:** To understand the messaging layer beneath TAP, refer to the DIDComm v2 spec by the DIF ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=%5BDIDComm%20v2%5D%28https%3A%2F%2Fidentity.foundation%2Fdidcomm,a%20number%20of%20important%20features)). It covers message formats, packing (JWM, JWE, JWS), routing, etc., which `tap.rs` relies on. *(See: “DIDComm Messaging v2” by the Decentralized Identity Foundation.)*

- **DID Core Specification (W3C):** For details on Decentralized Identifiers, DID Documents, and resolution, see the W3C DID Core spec ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=TAP%20builds%20on%20the%20existing,6.md%23%3A~%3Atext%3DParties%2520are%2520identified%2520using%2520an%2Cdecentralized%2520manner%252C%252)). This will help in implementing custom DID resolvers or understanding how did:web, did:key, did:ethr, and others function in terms of representation.

- **FATF Travel Rule Guidance:** If integrating TAP for compliance, it may help to read up on the Travel Rule requirements. The FATF guidelines (June 2019 Guidance for Virtual Assets and VASPs) outline what information needs to be exchanged between originator and beneficiary institutions. TAP can be a means to satisfy these – understanding the requirements will inform what data to include in TAP messages. *(Reference: FATF Guidance on Virtual Assets and VASPs, 2019.)*

- **Notabene Blog – TAP Use Cases:** The company Notabene (one of the contributors to TAP) has articles on how TAP improves crypto transaction safety and compliance. These can give insights into real-world scenarios (e.g., preventing mis-sent funds, enabling safe exchange-to-wallet transfers). They complement the technical perspective with a business rationale. 

- **Aries RFCs (for related concepts):** TAP is not built on Aries, but it shares some concepts with the Hyperledger Aries protocols (which also use DIDComm). If you are familiar with or interested in Aries, looking at Aries RFCs for “coordinate transfer” or “out-of-band communication” might provide additional context on designing user interactions for things like requesting authorization.

By exploring the above resources, you can gain a deeper understanding of TAP’s design and how to best implement it. This will help ensure your `tap.rs` integration is both compliant with the spec and maximally effective in practice. Good luck with building safe and authorized crypto transaction workflows!
 ([tap.md](file://file-E6THofSLZyEuWMHe5VKwGA#:~:text=The%20,https%3A%2F%2Fgithub.com%2FTransactionA))