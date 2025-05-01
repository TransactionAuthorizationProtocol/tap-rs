# Supported Message Formats

This document provides detailed information about the message formats supported by the TAP (Transaction Authorization Protocol) implementation and any deviations from the standard specifications.

## DIDComm Messaging Support

The TAP implementation uses the [DIDComm Messaging v2](https://identity.foundation/didcomm-messaging/spec/) protocol as its transport layer, with specific TAP message types defined as extensions of this protocol.

### Supported DIDComm Features

- **Plain Messages**: Full support for unencrypted DIDComm messages
- **Message Attachments**: Support for both base64 and JSON attachment formats
- **Message Headers**: Support for standard DIDComm headers including `id`, `type`, `from`, `to`, `thid`
- **Routing**: Support for multi-recipient routing with `to` arrays

## TAP Message Types

The following message types are fully supported in our implementation:

| Message Type | Message URI | Standard Compliance | Notes |
|--------------|-------------|---------------------|-------|
| Transfer | `https://tap.rsvp/schema/1.0#Transfer` | Full | Supports both object and string representations of AssetId |
| Authorize | `https://tap.rsvp/schema/1.0#Authorize` | Full | - |
| Reject | `https://tap.rsvp/schema/1.0#Reject` | Full | - |
| Settle | `https://tap.rsvp/schema/1.0#Settle` | Full | Implementation is tolerant of format issues |
| AddAgents | `https://tap.rsvp/schema/1.0#AddAgents` | Full | - |
| ReplaceAgent | `https://tap.rsvp/schema/1.0#ReplaceAgent` | Full | - |
| RemoveAgent | `https://tap.rsvp/schema/1.0#RemoveAgent` | Full | - |
| UpdatePolicies | `https://tap.rsvp/schema/1.0#UpdatePolicies` | Full | - |
| UpdateParty | `https://tap.rsvp/schema/1.0#UpdateParty` | Full | - |
| ConfirmRelationship | `https://tap.rsvp/schema/1.0#ConfirmRelationship` | Full | - |
| OutOfBand | `https://didcomm.org/out-of-band/2.0/invitation` | Full | - |
| DIDCommPresentation | `https://didcomm.org/present-proof/3.0/presentation` | Full | Fully compliant with standard DIDComm present-proof protocol |
| PaymentRequest | `https://tap.rsvp/schema/1.0#PaymentRequest` | Full | TAIP-14 support with currency/asset options and embedded invoice |
| Invoice | Embedded in PaymentRequest | Full | TAIP-16 compliant structured invoice with line items and tax details |
| AuthorizationRequired | `https://tap.rsvp/schema/1.0#AuthorizationRequired` | Full | - |
| Connect | `https://tap.rsvp/schema/1.0#Connect` | Full | - |

## Message Validation

All message types enforce the following validation rules:

- Required fields must be present and have correct types
- IDs must be valid format (UUIDs, DIDs, etc. as appropriate)
- Attachments must have proper format (ID, media type, data)
- References to other messages (e.g., `thid`) must be present where required

## Extensions and Deviations

Our implementation includes the following extensions to the standard specifications:

1. **Enhanced Validation**: Additional validation logic beyond what's required in specifications
2. **Tolerance for Format Issues**: Some message types (e.g., Settle) are tolerant of minor format issues
3. **Additional Fields**: Support for additional metadata fields not specified in the standards
4. **Invoice Support**: Full implementation of TAIP-16 structured invoice with validation
5. **Embedded Invoices**: Integration of invoice objects within payment requests

## Compatibility Testing

All message types are tested against the test vectors provided in the TAP Interoperability Profile specification. See [compatibility.md](../prds/compatibility.md) for detailed compatibility status.

## Usage Examples

For examples of creating and processing each message type, please refer to the unit tests in the [tests directory](./tests/).
