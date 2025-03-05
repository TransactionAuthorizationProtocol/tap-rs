# TAP-TS Upgrade Guide

## Changes Made to the TypeScript Implementation

The TAP-TS TypeScript implementation has been updated to conform to the TAP specification and to remove legacy message types and fields. Key changes include:

1. Removal of `ledgerId` references throughout the codebase
2. Removal of legacy message types (`AUTHORIZATION_REQUEST`, `AUTHORIZATION_RESPONSE`)
3. Implementation of standard TAP message types:
   - Transfer: `https://tap.rsvp/schema/1.0#Transfer`
   - RequestPresentation: `https://tap.rsvp/schema/1.0#RequestPresentation`
   - Presentation: `https://tap.rsvp/schema/1.0#Presentation`
   - Authorize: `https://tap.rsvp/schema/1.0#Authorize`
   - Reject: `https://tap.rsvp/schema/1.0#Reject`
   - Settle: `https://tap.rsvp/schema/1.0#Settle`
   - AddAgents: `https://tap.rsvp/schema/1.0#AddAgents`
   - Error: `https://tap.rsvp/schema/1.0#Error`
4. Addition of proper getter/setter methods for standard message data:
   - `setTransferData` / `getTransferData`
   - `setAuthorizeData` / `getAuthorizeData`
   - `setRejectData` / `getRejectData`
   - `setSettleData` / `getSettleData`
5. Update from method-based to property-based accessors for `from` and `to` fields

## Required Changes to the WASM Bindings

The Rust WASM bindings (`tap-wasm` crate) need to be updated to maintain compatibility with the TypeScript implementation. The following changes are required:

1. Update the Message struct to remove legacy fields:
   - Remove `ledger_id` field
   - Remove `authorization_request` and `authorization_response` fields
   - Update the Message constructor to no longer require a `ledger_id` parameter

2. Add standard message body data fields:
   - Add `transfer_body`, `authorize_body`, `reject_body`, `settle_body` fields
   - Implement getter/setter methods for these new fields

3. Update the MessageType enum to use standard TAP types:
   - Remove legacy types like `AuthorizationRequest` and `AuthorizationResponse`
   - Add standard types like `Transfer`, `Authorize`, `Reject`, etc.

4. Update the Agent struct to align with the TypeScript Agent class:
   - Ensure Agent properties match between the two implementations
   - Update message handling methods to work with standard message types

5. Fix field access in the JS bindings:
   - Update references to `additional_fields` to the proper field name
   - Correct signature verification to use the proper signature format

6. Deprecate and eventually remove the legacy authorization methods:
   - Mark `set_authorization_request` and related methods as deprecated
   - Provide clear guidance to use standard methods instead

## Testing and Verification

After making the above changes to the `tap-wasm` crate, the following steps should be taken:

1. Run `cargo build --package tap-wasm` to verify compilation
2. Update WASM tests to use standard message types
3. Ensure the TypeScript implementation can correctly interact with the WASM bindings
4. Run TypeScript tests to verify functionality
5. Update examples to demonstrate the use of standard message types

## Migration Guide for Users

For users migrating from previous versions of TAP-TS to the new version:

1. Replace `AUTHORIZATION_REQUEST` and `AUTHORIZATION_RESPONSE` with standard TAP message types:
   - Use `AUTHORIZE` instead of `AUTHORIZATION_RESPONSE` for approvals
   - Use `REJECT` instead of `AUTHORIZATION_RESPONSE` for rejections
   - Use `TRANSFER` instead of `AUTHORIZATION_REQUEST` for transfer requests
   
2. Replace legacy methods with standard TAP methods:
   - Use `setTransferData` instead of `setAuthorizationRequestData`
   - Use `setAuthorizeData` or `setRejectData` instead of `setAuthorizationResponseData`
   
3. Remove references to `ledgerId`:
   - Use proper CAIP identifiers for assets and chains
   - Update message creation to not require `ledgerId`

4. Update from/to access:
   - Use `message.from = "did:example:123"` instead of `message.from("did:example:123")`
   - Use `message.to = ["did:example:456"]` instead of `message.to(["did:example:456"])`

5. Update message data validation:
   - Ensure validation logic handles the new message structure
   - Update UI code to render the new message formats