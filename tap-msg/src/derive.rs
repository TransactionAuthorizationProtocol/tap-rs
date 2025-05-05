/// Macro to implement TapMessage for a struct.
///
/// This macro implements the TapMessage trait for a struct that implements TapMessageBody.
/// It's a replacement for the previous derive macro in tap-msg-derive.
///
/// # Example
///
/// ```rust
/// use tap_msg::impl_tap_message;
/// use tap_msg::message::tap_message_trait::{TapMessageBody, TapMessage};
/// use tap_msg::error::Result;
/// use serde::{Serialize, Deserialize};
/// 
/// // Your struct that implements TapMessageBody
/// #[derive(Serialize, Deserialize)]
/// struct MyMessage {
///     transfer_id: String,
///     // other fields...
/// }
/// 
/// impl TapMessageBody for MyMessage {
///     fn validate(&self) -> Result<()> {
///         Ok(())
///     }
///     
///     // Note: This is a static method, not an instance method
///     fn message_type() -> &'static str {
///         "my-message"
///     }
/// }
/// 
/// // Implement TapMessage trait
/// impl_tap_message!(MyMessage);
/// ```
#[macro_export]
macro_rules! impl_tap_message {
    ($type:ty) => {
        impl $crate::message::tap_message_trait::TapMessage for $type {
            fn validate(&self) -> $crate::error::Result<()> {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(<Self as $crate::message::tap_message_trait::TapMessageBody>::message_type().to_string())
            }
            fn body_as<T: $crate::message::tap_message_trait::TapMessageBody>(&self) -> $crate::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> $crate::error::Result<didcomm::Message> {
                $crate::message::tap_message_trait::TapMessage::create_reply(self, body, creator_did)
            }
            fn message_type(&self) -> &'static str {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
            }
            fn thread_id(&self) -> Option<&str> {
                // for types with transfer_id
                Some(&self.transfer_id)
            }
            fn parent_thread_id(&self) -> Option<&str> {
                None
            }
            fn message_id(&self) -> &str {
                &self.transfer_id
            }
        }
    };
}
