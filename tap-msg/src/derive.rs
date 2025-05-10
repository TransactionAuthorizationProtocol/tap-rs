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
///     transaction_id: String,
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
                // Create the base message with creator as sender
                let mut message = body.to_didcomm(Some(creator_did))?;

                // Set the thread ID to maintain the conversation thread
                if let Some(thread_id) = self.thread_id() {
                    message.thid = Some(thread_id.to_string());
                } else {
                    // If no thread ID exists, use the original message ID as the thread ID
                    message.thid = Some(self.message_id().to_string());
                }

                // Set the parent thread ID if this thread is part of a larger transaction
                if let Some(parent_thread_id) = self.parent_thread_id() {
                    message.pthid = Some(parent_thread_id.to_string());
                }

                Ok(message)
            }
            fn message_type(&self) -> &'static str {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
            }
            fn thread_id(&self) -> Option<&str> {
                // for types with transaction_id
                Some(&self.transaction_id)
            }
            fn parent_thread_id(&self) -> Option<&str> {
                None
            }
            fn message_id(&self) -> &str {
                &self.transaction_id
            }
        }
    };
}
