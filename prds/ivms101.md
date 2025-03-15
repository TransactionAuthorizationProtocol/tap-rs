Below is the updated PRD with a new section for the `tap-ivms101` module. At the end, you’ll find a checklist with single-point tasks in markdown format to track the implementation of this module.

---

# Rust TAP Implementation PRD and Best Practices

This document outlines the product requirements for a Rust implementation of the Transaction Authorization Protocol (TAP), targeting payment-related use cases and Travel Rule messaging. The project is structured as a Rust workspace with multiple crates. Each item below is represented as a checklist (using markdown checkboxes) to track progress. Additionally, best practices for Rust development and references/URLs to useful specifications are provided at each step.

---

*(Previous sections covering tap-msg, tap-agent, caip, tap-node, tap-http, tap-ts, tap-cli, and overall testing & validation remain unchanged.)*

---

## tap-ivms101 Module

The `tap-ivms101` module integrates IVMS101 data handling directly into `tap-msg`. This module enables the secure and compliant exchange of identity information between TAP agents by combining IVMS101 data with TAP’s PresentationRequest and Presentation messages (as described in TAIP-8). The module will:

- **PresentationRequest Message:**  
  - Require specific IVMS101 person information (e.g. full name, address, national ID) about either the originator or beneficiary.
  - Dynamically specify which fields are required, based on transaction context or regulatory policy.

- **Presentation Message:**  
  - Include IVMS101 person data in response to a PresentationRequest.
  - Support full serialization and deserialization of IVMS101 data into the standard JSON format.

- **Integration:**  
  - Integrate directly into `tap-msg` so that TAP messages can include IVMS101 data seamlessly.
  - Ensure that the PresentationRequest/Presentation messages are packed/unpacked via DIDComm v2 like all other TAP messages.

- **Verifiable Credential and Presentation:**  
  - Convert IVMS101 data into a W3C Verifiable Credential that is tied to the subject’s DID (originator or beneficiary).
  - Wrap the credential inside a Verifiable Presentation.
  - Attach the Verifiable Presentation to the Presentation message so that the recipient can verify the data per TAIP-8.

- **Dynamic and Selective Disclosure:**  
  - Allow configurable, dynamic handling of required IVMS101 fields.
  - Enable selective disclosure so that only the requested fields are shared.

---

### Single-Point Tasks for tap-ivms101 Module

- [ ] **Define IVMS101 Data Models:**  
  - Create new rust crate tap-ivms101 to contain IVMS101 data models.
  - Create rust structs for the entire IVMS101 standard defined here https://cdn.prod.website-files.com/648841abc97f28489cc3f2ce/6656e9c60c3029989dcd7431_IVMS101.2023%20interVASP%20data%20model%20standard.pdf
  
- [ ] **Implement Serialization/Deserialization:**  
  - Develop functions to serialize IVMS101 data into the standardized JSON format.
  - Develop functions to deserialize incoming IVMS101 JSON into the corresponding Rust data models.

- [ ] **Create PresentationRequest Message API:**  
  - Design and implement an API function (e.g., `create_presentation_request()`) that constructs a PresentationRequest message.
  - Ensure the function requires specific IVMS101 fields (e.g., name, address, ID) for either the originator or beneficiary, based on dynamic configuration.

- [ ] **Create Presentation Message API:**  
  - Design and implement an API function (e.g., `create_presentation()`) that builds a Presentation message using the ssi crate.
  - Ensure this function accepts IVMS101 data and produces a message that includes the data as a Verifiable Presentation.

- [ ] **Integrate with tap-msg Message Handling:**  
  - Extend tap-msg’s packing/unpacking logic to recognize and Presentation messages that contain IVMS101 data.
  - Ensure that these messages are handled dynamically as part of the TAP flow.

- [ ] **Verifiable Credential Generation:**  
  - Implement functionality to convert IVMS101 Person data into a Verifiable Credential using the ssi crate
  - Bind the credential to the appropriate DID (originator or beneficiary) and sign it with the issuer’s key.

- [ ] **Wrap Credential in Verifiable Presentation:**  
  - Implement functionality to wrap the Verifiable Credential into a Verifiable Presentation using the ssi crate.
  - Ensure the Verifiable Presentation includes necessary proofs (e.g., a nonce or audience restriction) per W3C standards.

- [ ] **Attach Verifiable Presentation to Presentation Message:**  
  - Extend the Presentation message builder to include the generated Verifiable Presentation as a DIDComm attachment.
  - Ensure the message conforms to TAIP-8 standards.

- [ ] **Dynamic Field Requirement & Selective Disclosure Support:**  
  - Implement configuration options to dynamically set which IVMS101 fields are required in a PresentationRequest.
  - Ensure the Presentation function only includes the fields requested (selective disclosure).

- [ ] **Validation & Verification:**  
  - Implement functions to validate incoming Verifiable Credentials and Presentations.
  - Ensure that the signatures are verified, and the IVMS101 data meets the requested schema requirements.

- [ ] **Testing & Documentation:**  
  - Write unit tests for serialization, deserialization, and the creation of PresentationRequest and Presentation messages.
  - Develop integration tests to simulate an end-to-end IVMS101 data exchange within a TAP flow.
  - Document the API usage with examples and reference the IVMS101 and TAIP-8 specifications.

---

## References & Useful Links for IVMS101 Integration

- **IVMS101 Standard:**  
  - [IVMS101 Overview and Schema Specifications](https://github.com/InterVASP/IVMS101)  
  - *(Ensure to check for the latest version and guidelines.)*

- **TAIP-8 (Selective Disclosure & Presentation Messages):**  
  - [TAIP-8 Documentation in the TAP GitHub Repository](https://github.com/TransactionAuthorizationProtocol/TAIPs)  

- **W3C Verifiable Credentials:**  
  - [W3C Verifiable Credentials Data Model](https://www.w3.org/TR/vc-data-model/)  
  - [W3C Verifiable Presentations](https://www.w3.org/TR/vc-data-model/#verifiable-presentations)

- **DIDComm v2 Specifications:**  
  - [DIDComm Messaging v2 by the Decentralized Identity Foundation](https://identity.foundation/didcomm-messaging/spec/)

- **tap.rs Repository:**  
  - [tap-didcomm-rs GitHub Repository](https://github.com/TransactionAuthorizationProtocol/tap-didcomm-rs)

