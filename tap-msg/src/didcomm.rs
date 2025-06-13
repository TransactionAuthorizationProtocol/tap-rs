use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Wrapper for plain message. Provides helpers for message building and packing/unpacking.
/// Adapted from https://github.com/sicpa-dlab/didcomm-rust/blob/main/src/message/message.rs
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(bound = "T: Serialize + serde::de::DeserializeOwned")]
pub struct PlainMessage<T = Value> {
    /// Message id. Must be unique to the sender.
    pub id: String,

    /// Optional, if present it must be "application/didcomm-plain+json"
    #[serde(default = "default_typ")]
    pub typ: String,

    /// Message type attribute value MUST be a valid Message Type URI,
    /// that when resolved gives human readable information about the message.
    /// The attribute’s value also informs the content of the message,
    /// or example the presence of other attributes and how they should be processed.
    #[serde(rename = "type")]
    pub type_: String,

    /// Message body - strongly typed when T is specified.
    pub body: T,

    /// Sender identifier. The from attribute MUST be a string that is a valid DID
    /// or DID URL (without the fragment component) which identifies the sender of the message.
    pub from: String,

    /// Identifier(s) for recipients. MUST be an array of strings where each element
    /// is a valid DID or DID URL (without the fragment component) that identifies a member
    /// of the message’s intended audience.
    #[serde(default)]
    pub to: Vec<String>,

    /// Uniquely identifies the thread that the message belongs to.
    /// If not included the id property of the message MUST be treated as the value of the `thid`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thid: Option<String>,

    /// If the message is a child of a thread the `pthid`
    /// will uniquely identify which thread is the parent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,

    /// Custom message headers.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra_headers: HashMap<String, Value>,

    /// The attribute is used for the sender
    /// to express when they created the message, expressed in
    /// UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    /// This attribute is informative to the recipient, and may be relied on by protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<u64>,

    /// The expires_time attribute is used for the sender to express when they consider
    /// the message to be expired, expressed in UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    /// This attribute signals when the message is considered no longer valid by the sender.
    /// When omitted, the message is considered to have no expiration by the sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<u64>,

    /// from_prior is a compactly serialized signed JWT containing FromPrior value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_prior: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

/// Type alias for backward compatibility - PlainMessage with Value body
pub type UntypedPlainMessage = PlainMessage<Value>;

const PLAINTEXT_TYP: &str = "application/didcomm-plain+json";

fn default_typ() -> String {
    PLAINTEXT_TYP.to_string()
}

// Implementation for generic PlainMessage<T>
impl<T> PlainMessage<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Create a new PlainMessage with the given body
    pub fn new(id: String, type_: String, body: T, from: String) -> Self {
        Self {
            id,
            typ: default_typ(),
            type_,
            body,
            from,
            to: vec![],
            thid: None,
            pthid: None,
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Builder method to set recipients
    pub fn with_recipients(mut self, to: Vec<String>) -> Self {
        self.to = to;
        self
    }

    /// Builder method to add a single recipient
    pub fn with_recipient(mut self, recipient: &str) -> Self {
        self.to.push(recipient.to_string());
        self
    }

    /// Builder method to set thread ID
    pub fn with_thread_id(mut self, thid: Option<String>) -> Self {
        self.thid = thid;
        self
    }

    /// Builder method to set parent thread ID
    pub fn with_parent_thread_id(mut self, pthid: Option<String>) -> Self {
        self.pthid = pthid;
        self
    }

    /// Builder method to set expiration time
    pub fn with_expires_at(mut self, expires_time: u64) -> Self {
        self.expires_time = Some(expires_time);
        self
    }

    /// Builder method to add attachments
    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    /// Builder method to add a custom header
    pub fn with_header(mut self, key: String, value: Value) -> Self {
        self.extra_headers.insert(key, value);
        self
    }
}

// Implementation specific to typed PlainMessage where T implements TapMessageBody
impl<T> PlainMessage<T>
where
    T: crate::message::TapMessageBody + serde::Serialize + serde::de::DeserializeOwned,
{
    /// Create a new typed message
    pub fn new_typed(body: T, from: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            typ: default_typ(),
            type_: T::message_type().to_string(),
            body,
            from: from.to_string(),
            to: vec![],
            thid: None,
            pthid: None,
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Convert to untyped PlainMessage for serialization/transport
    pub fn to_plain_message(self) -> crate::error::Result<PlainMessage<Value>> {
        // First serialize the body with the @type field
        let mut body_value = serde_json::to_value(&self.body)?;

        // Ensure @type is set in the body
        if let Some(body_obj) = body_value.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                Value::String(T::message_type().to_string()),
            );
        }

        Ok(PlainMessage {
            id: self.id,
            typ: self.typ,
            type_: self.type_,
            body: body_value,
            from: self.from,
            to: self.to,
            thid: self.thid,
            pthid: self.pthid,
            created_time: self.created_time,
            expires_time: self.expires_time,
            from_prior: self.from_prior,
            attachments: self.attachments,
            extra_headers: self.extra_headers,
        })
    }

    /// Extract recipients based on the message body participants
    pub fn extract_participants(&self) -> Vec<String> {
        let mut participants = vec![];

        // Try to extract from MessageContext first if implemented
        if let Some(ctx_participants) = self.try_extract_from_context() {
            participants = ctx_participants;
        } else {
            // Fallback to TapMessageBody::to_didcomm
            if let Ok(plain_msg) = self.body.to_didcomm(&self.from) {
                participants = plain_msg.to;
            }
        }

        // Add any explicitly set recipients
        for recipient in &self.to {
            if !participants.contains(recipient) {
                participants.push(recipient.clone());
            }
        }

        participants
    }

    /// Try to extract participants using MessageContext if available
    fn try_extract_from_context(&self) -> Option<Vec<String>> {
        // This is a compile-time check - if T implements MessageContext,
        // we can use it. Otherwise, this will return None.
        //
        // In practice, this would need to be implemented using trait objects
        // or type erasure, but for now we'll use the TapMessageBody approach
        // and let individual message types override this behavior.
        None
    }
}

// Implementation for PlainMessage<T> where T implements both TapMessageBody and MessageContext
impl<T> PlainMessage<T>
where
    T: crate::message::TapMessageBody
        + crate::message::MessageContext
        + serde::Serialize
        + serde::de::DeserializeOwned,
{
    /// Extract participants using MessageContext
    pub fn extract_participants_with_context(&self) -> Vec<String> {
        self.body.participant_dids()
    }

    /// Create a typed message with automatic recipient detection
    pub fn new_typed_with_context(body: T, from: &str) -> Self {
        let participants = body.participant_dids();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            typ: default_typ(),
            type_: T::message_type().to_string(),
            body,
            from: from.to_string(),
            to: participants.into_iter().filter(|did| did != from).collect(),
            thid: None,
            pthid: None,
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            expires_time: None,
            from_prior: None,
            attachments: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Get routing hints from the message body
    pub fn routing_hints(&self) -> crate::message::RoutingHints {
        self.body.routing_hints()
    }

    /// Get transaction context from the message body
    pub fn transaction_context(&self) -> Option<crate::message::TransactionContext> {
        self.body.transaction_context()
    }
}

// Implementation for PlainMessage<Value> (untyped)
impl PlainMessage<Value> {
    /// Create a typed message from an untyped PlainMessage
    pub fn from_untyped(plain_msg: PlainMessage<Value>) -> Self {
        plain_msg
    }

    /// Try to parse the body into a specific TAP message type
    pub fn parse_body<T: crate::message::TapMessageBody>(
        self,
    ) -> crate::error::Result<PlainMessage<T>> {
        // Check type matches
        if self.type_ != T::message_type() {
            return Err(crate::error::Error::Validation(format!(
                "Type mismatch: expected {}, got {}",
                T::message_type(),
                self.type_
            )));
        }

        // Parse the body
        let typed_body: T = serde_json::from_value(self.body)?;

        Ok(PlainMessage {
            id: self.id,
            typ: self.typ,
            type_: self.type_,
            body: typed_body,
            from: self.from,
            to: self.to,
            thid: self.thid,
            pthid: self.pthid,
            created_time: self.created_time,
            expires_time: self.expires_time,
            from_prior: self.from_prior,
            attachments: self.attachments,
            extra_headers: self.extra_headers,
        })
    }

    /// Parse into the TapMessage enum for runtime dispatch
    pub fn parse_tap_message(
        &self,
    ) -> crate::error::Result<crate::message::tap_message_enum::TapMessage> {
        crate::message::tap_message_enum::TapMessage::from_plain_message(self)
    }
}

/// Extension trait for PlainMessage to work with typed messages
pub trait PlainMessageExt<T> {
    /// Convert to a typed message
    fn into_typed(self) -> PlainMessage<T>;

    /// Try to parse as a specific message type
    fn parse_as<U: crate::message::TapMessageBody>(self) -> crate::error::Result<PlainMessage<U>>;
}

impl PlainMessageExt<Value> for PlainMessage<Value> {
    fn into_typed(self) -> PlainMessage<Value> {
        self
    }

    fn parse_as<U: crate::message::TapMessageBody>(self) -> crate::error::Result<PlainMessage<U>> {
        self.parse_body()
    }
}

/// Helper to convert between typed messages and TapMessage enum
impl<T: crate::message::TapMessageBody> TryFrom<PlainMessage<T>>
    for crate::message::tap_message_enum::TapMessage
where
    crate::message::tap_message_enum::TapMessage: From<T>,
{
    type Error = crate::error::Error;

    fn try_from(typed: PlainMessage<T>) -> crate::error::Result<Self> {
        // This would require implementing From<T> for TapMessage for each message type
        // For now, we'll use the parse approach
        typed.to_plain_message()?.parse_tap_message()
    }
}

/// Message for out-of-band invitations (TAIP-2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutOfBand {
    /// Goal code for the invitation.
    #[serde(rename = "goal_code")]
    pub goal_code: String,

    /// Invitation message ID.
    pub id: String,

    /// Label for the invitation.
    pub label: String,

    /// Accept option for the invitation.
    pub accept: Option<String>,

    /// The DIDComm services to connect to.
    pub services: Vec<serde_json::Value>,
}

/// Simple attachment data for a TAP message.
///
/// This structure represents a simplified version of attachment data
/// that directly contains base64 or JSON without the complexity of the
/// full AttachmentData enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleAttachmentData {
    /// Base64-encoded data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base64: Option<String>,

    /// JSON data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<serde_json::Value>,
}

/// Attachment for a TAP message.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Attachment {
    /// A JSON object that gives access to the actual content of the attachment.
    /// Can be based on base64, json or external links.
    pub data: AttachmentData,

    /// Identifies attached content within the scope of a given message.
    ///  Recommended on appended attachment descriptors. Possible but generally unused
    ///  on embedded attachment descriptors. Never required if no references to the attachment
    ///  exist; if omitted, then there is no way to refer to the attachment later in the thread,
    ///  in error messages, and so forth. Because id is used to compose URIs, it is recommended
    ///  that this name be brief and avoid spaces and other characters that require URI escaping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// A human-readable description of the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A hint about the name that might be used if this attachment is persisted as a file.
    /// It is not required, and need not be unique. If this field is present and mime-type is not,
    /// the extension on the filename may be used to infer a MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// Describes the MIME type of the attached content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,

    /// Describes the format of the attachment if the mime_type is not sufficient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// A hint about when the content in this attachment was last modified
    /// in UTC Epoch Seconds (seconds since 1970-01-01T00:00:00Z UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod_time: Option<u64>,

    /// Mostly relevant when content is included by reference instead of by value.
    /// Lets the receiver guess how expensive it will be, in time, bandwidth, and storage,
    /// to fully fetch the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byte_count: Option<u64>,
}

impl Attachment {
    pub fn base64(base64: String) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Base64 {
            value: Base64AttachmentData { base64, jws: None },
        })
    }

    pub fn json(json: Value) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Json {
            value: JsonAttachmentData { json, jws: None },
        })
    }

    pub fn links(links: Vec<String>, hash: String) -> AttachmentBuilder {
        AttachmentBuilder::new(AttachmentData::Links {
            value: LinksAttachmentData {
                links,
                hash,
                jws: None,
            },
        })
    }
}

pub struct AttachmentBuilder {
    data: AttachmentData,
    id: Option<String>,
    description: Option<String>,
    filename: Option<String>,
    media_type: Option<String>,
    format: Option<String>,
    lastmod_time: Option<u64>,
    byte_count: Option<u64>,
}

impl AttachmentBuilder {
    fn new(data: AttachmentData) -> Self {
        AttachmentBuilder {
            data,
            id: None,
            description: None,
            filename: None,
            media_type: None,
            format: None,
            lastmod_time: None,
            byte_count: None,
        }
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }

    pub fn media_type(mut self, media_type: String) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }

    pub fn lastmod_time(mut self, lastmod_time: u64) -> Self {
        self.lastmod_time = Some(lastmod_time);
        self
    }

    pub fn byte_count(mut self, byte_count: u64) -> Self {
        self.byte_count = Some(byte_count);
        self
    }

    pub fn jws(mut self, jws: String) -> Self {
        match self.data {
            AttachmentData::Base64 { ref mut value } => value.jws = Some(jws),
            AttachmentData::Json { ref mut value } => value.jws = Some(jws),
            AttachmentData::Links { ref mut value } => value.jws = Some(jws),
        }

        self
    }

    pub fn finalize(self) -> Attachment {
        Attachment {
            data: self.data,
            id: self.id,
            description: self.description,
            filename: self.filename,
            media_type: self.media_type,
            format: self.format,
            lastmod_time: self.lastmod_time,
            byte_count: self.byte_count,
        }
    }
}

// Attention: we are using untagged enum serialization variant.
// Serde will try to match the data against each variant in order and the
// first one that deserializes successfully is the one returned.
// It should work as we always have discrimination here.

/// Represents attachment data in Base64, embedded Json or Links form.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum AttachmentData {
    Base64 {
        #[serde(flatten)]
        value: Base64AttachmentData,
    },
    Json {
        #[serde(flatten)]
        value: JsonAttachmentData,
    },
    Links {
        #[serde(flatten)]
        value: LinksAttachmentData,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Base64AttachmentData {
    /// Base64-encoded data, when representing arbitrary content inline.
    pub base64: String,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct JsonAttachmentData {
    /// Directly embedded JSON data.
    pub json: Value,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LinksAttachmentData {
    /// A list of one or more locations at which the content may be fetched.
    pub links: Vec<String>,

    /// The hash of the content encoded in multi-hash format. Used as an integrity check for the attachment.
    pub hash: String,

    /// A JSON Web Signature over the content of the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jws: Option<String>,
}

#[cfg(test)]
mod tests {
    use core::panic;
    use serde_json::json;

    use super::*;

    #[test]
    fn attachment_base64_works() {
        let attachment = Attachment::base64("ZXhhbXBsZQ==".to_owned())
            .id("example-1".to_owned())
            .description("example-1-description".to_owned())
            .filename("attachment-1".to_owned())
            .media_type("message/example".to_owned())
            .format("json".to_owned())
            .lastmod_time(10000)
            .byte_count(200)
            .jws("jws".to_owned())
            .finalize();

        let data = match attachment.data {
            AttachmentData::Base64 { ref value } => value,
            _ => panic!("data isn't base64."),
        };

        assert_eq!(data.base64, "ZXhhbXBsZQ==");
        assert_eq!(data.jws, Some("jws".to_owned()));
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }

    #[test]
    fn attachment_json_works() {
        let attachment = Attachment::json(json!("example"))
            .id("example-1".to_owned())
            .description("example-1-description".to_owned())
            .filename("attachment-1".to_owned())
            .media_type("message/example".to_owned())
            .format("json".to_owned())
            .lastmod_time(10000)
            .byte_count(200)
            .jws("jws".to_owned())
            .finalize();

        let data = match attachment.data {
            AttachmentData::Json { ref value } => value,
            _ => panic!("data isn't json."),
        };

        assert_eq!(data.json, json!("example"));
        assert_eq!(data.jws, Some("jws".to_owned()));
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }

    #[test]
    fn attachment_links_works() {
        let attachment = Attachment::links(
            vec!["http://example1".to_owned(), "https://example2".to_owned()],
            "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c".to_owned(),
        )
        .id("example-1".to_owned())
        .description("example-1-description".to_owned())
        .filename("attachment-1".to_owned())
        .media_type("message/example".to_owned())
        .format("json".to_owned())
        .lastmod_time(10000)
        .byte_count(200)
        .jws("jws".to_owned())
        .finalize();

        let data = match attachment.data {
            AttachmentData::Links { ref value } => value,
            _ => panic!("data isn't links."),
        };

        assert_eq!(
            data.links,
            vec!["http://example1".to_owned(), "https://example2".to_owned()]
        );

        assert_eq!(
            data.hash,
            "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c".to_owned()
        );

        assert_eq!(data.jws, Some("jws".to_owned()));
        assert_eq!(attachment.id, Some("example-1".to_owned()));

        assert_eq!(
            attachment.description,
            Some("example-1-description".to_owned())
        );

        assert_eq!(attachment.filename, Some("attachment-1".to_owned()));
        assert_eq!(attachment.media_type, Some("message/example".to_owned()));
        assert_eq!(attachment.format, Some("json".to_owned()));
        assert_eq!(attachment.lastmod_time, Some(10000));
        assert_eq!(attachment.byte_count, Some(200));
    }
}
