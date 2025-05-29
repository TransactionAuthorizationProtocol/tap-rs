/// Macro to implement TapMessage for a struct.
///
/// This macro implements the TapMessage trait for a struct that implements TapMessageBody.
/// It supports different message field structures with specialized variants.
///
/// # Variants
///
/// ## Basic (Standard transaction_id field)
///
/// Use this variant for message types with a required `transaction_id: String` field:
///
/// ```ignore
/// // Example usage - this won't be run as a test
/// use tap_msg::impl_tap_message;
/// impl_tap_message!(Transfer);
/// ```
///
/// ## Optional Transaction ID
///
/// Use this variant for message types with an optional `transaction_id: Option<String>` field:
///
/// ```ignore
/// // Example usage - this won't be run as a test
/// use tap_msg::impl_tap_message;
/// impl_tap_message!(Presentation, optional_transaction_id);
/// ```
///
/// ## Thread-based Messages
///
/// Use this variant for message types with a `thid: Option<String>` field but no transaction_id:
///
/// ```ignore
/// // Example usage - this won't be run as a test
/// use tap_msg::impl_tap_message;
/// impl_tap_message!(DIDCommPresentation, thread_based);
/// ```
///
/// ## Generated ID
///
/// Use this variant for message types with neither transaction_id nor thread_id fields:
///
/// ```ignore
/// // Example usage - this won't be run as a test
/// use tap_msg::impl_tap_message;
/// impl_tap_message!(ErrorBody, generated_id);
/// ```
///
/// # Complete Example
///
/// ```ignore
/// // Example usage - this won't be run as a test
/// use tap_msg::impl_tap_message;
/// use tap_msg::message::tap_message_trait::{TapMessageBody, TapMessage};
/// use tap_msg::error::Result;
/// use serde::{Serialize, Deserialize};
/// use crate::didcomm::PlainMessage;
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
///
///     fn to_didcomm(&self, from_did: &str) -> Result<PlainMessage> {
///         // Implementation omitted
///         unimplemented!()
///     }
/// }
///
/// // Implement TapMessage trait
/// impl_tap_message!(MyMessage);
/// ```
#[macro_export]
macro_rules! impl_tap_message {
    // For types with a required transaction_id field (most common case)
    ($type:ty) => {
        impl $crate::message::tap_message_trait::TapMessage for $type {
            fn validate(&self) -> $crate::error::Result<()> {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(
                    <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
                        .to_string(),
                )
            }
            fn body_as<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
            ) -> $crate::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> $crate::error::Result<$crate::didcomm::PlainMessage> {
                // Create the base message with creator as sender
                let mut message = body.to_didcomm(creator_did)?;

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

    // For types with an optional transaction_id field
    ($type:ty, optional_transaction_id) => {
        impl $crate::message::tap_message_trait::TapMessage for $type {
            fn validate(&self) -> $crate::error::Result<()> {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(
                    <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
                        .to_string(),
                )
            }
            fn body_as<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
            ) -> $crate::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> $crate::error::Result<$crate::didcomm::PlainMessage> {
                // Create the base message with creator as sender
                let mut message = body.to_didcomm(creator_did)?;

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
            fn thread_id(&self) -> Option<&str> {
                self.transaction_id.as_deref()
            }
            fn parent_thread_id(&self) -> Option<&str> {
                None
            }
            fn message_id(&self) -> &str {
                if let Some(ref id) = self.transaction_id {
                    id
                } else {
                    &self.id
                }
            }
        }
    };

    // For types with a thread_id field instead of transaction_id
    ($type:ty, thread_based) => {
        impl $crate::message::tap_message_trait::TapMessage for $type {
            fn validate(&self) -> $crate::error::Result<()> {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(
                    <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
                        .to_string(),
                )
            }
            fn body_as<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
            ) -> $crate::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> $crate::error::Result<$crate::didcomm::PlainMessage> {
                // Create the base message with creator as sender
                let mut message = body.to_didcomm(creator_did)?;

                // Set the thread ID to maintain the conversation thread
                if let Some(thread_id) = self.thread_id() {
                    message.thid = Some(thread_id.to_string());
                } else {
                    // If no thread ID exists, use the original message ID as the thread ID
                    message.thid = Some(self.message_id().to_string());
                }

                Ok(message)
            }
            fn thread_id(&self) -> Option<&str> {
                self.thid.as_deref()
            }
            fn parent_thread_id(&self) -> Option<&str> {
                None
            }
            fn message_id(&self) -> &str {
                if let Some(ref thid) = self.thid {
                    thid
                } else {
                    &self.id
                }
            }
        }
    };

    // For types with neither transaction_id nor thread_id (generated ID)
    ($type:ty, generated_id) => {
        impl $crate::message::tap_message_trait::TapMessage for $type {
            fn validate(&self) -> $crate::error::Result<()> {
                <Self as $crate::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(
                    <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type()
                        .to_string(),
                )
            }
            fn body_as<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
            ) -> $crate::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: $crate::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> $crate::error::Result<$crate::didcomm::PlainMessage> {
                // Create the base message with creator as sender
                let message = body.to_didcomm(creator_did)?;

                // For types without thread/transaction ID, we don't set thread ID on replies

                Ok(message)
            }
            fn thread_id(&self) -> Option<&str> {
                None
            }
            fn parent_thread_id(&self) -> Option<&str> {
                None
            }
            fn message_id(&self) -> &str {
                // For types without an ID field, we'll use a static string
                // This isn't ideal but it satisfies the API contract
                // In real usage, these message types should be wrapped in a TapMessage
                // implementation that provides a proper ID
                static FALLBACK_ID: &str = "00000000-0000-0000-0000-000000000000";
                FALLBACK_ID
            }
        }
    };
}

/// Implements MessageContext trait for structs with participant fields.
///
/// This macro generates MessageContext implementations based on the struct fields.
/// It automatically detects Participant fields and provides participant extraction.
///
/// # Usage
///
/// ```ignore
/// use tap_msg::{impl_message_context, MessageContext};
///
/// struct Transfer {
///     originator: Participant,
///     beneficiary: Option<Participant>,
///     agents: Vec<Participant>,
///     transaction_id: String,
/// }
///
/// impl_message_context!(Transfer,
///     participants: [originator, (beneficiary optional), (agents list)],
///     transaction_id: transaction_id
/// );
/// ```
///
/// # Syntax
///
/// - `field` - Required Participant field
/// - `(field optional)` - Optional Participant field (Option<Participant>)
/// - `(field list)` - Participant list field (Vec<Participant>)
/// - `transaction_id: field_name` - Transaction ID field
#[macro_export]
macro_rules! impl_message_context {
    ($type:ty,
        participants: [$($participant_spec:tt),*],
        transaction_id: $tx_field:ident
    ) => {
        impl $crate::message::MessageContext for $type {
            fn participants(&self) -> Vec<&$crate::message::Participant> {
                let mut participants = Vec::new();

                $(
                    impl_message_context!(@add_participant participants, self, $participant_spec);
                )*

                participants
            }

            fn transaction_context(&self) -> Option<$crate::message::TransactionContext> {
                Some($crate::message::TransactionContext::new(
                    self.$tx_field.clone(),
                    <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type().to_string(),
                ))
            }
        }
    };

    // Helper for adding single participants
    (@add_participant $participants:ident, $self:ident, $field:ident) => {
        $participants.push(&$self.$field);
    };

    // Helper for adding optional participants
    (@add_participant $participants:ident, $self:ident, ($field:ident optional)) => {
        if let Some(ref participant) = $self.$field {
            $participants.push(participant);
        }
    };

    // Helper for adding participant lists
    (@add_participant $participants:ident, $self:ident, ($field:ident list)) => {
        $participants.extend(&$self.$field);
    };
}

/// Enhanced macro for implementing both TapMessage and MessageContext traits.
///
/// This macro combines TapMessage implementation with automatic MessageContext
/// generation based on field analysis.
///
/// # Usage
///
/// ```ignore
/// use tap_msg::impl_tap_message_with_context;
///
/// struct Transfer {
///     originator: Participant,
///     beneficiary: Option<Participant>,
///     agents: Vec<Participant>,
///     transaction_id: String,
/// }
///
/// impl_tap_message_with_context!(Transfer,
///     participants: [originator, (beneficiary optional), (agents list)],
///     transaction_id: transaction_id
/// );
/// ```
#[macro_export]
macro_rules! impl_tap_message_with_context {
    ($type:ty,
        participants: [$($participant_spec:tt),*],
        transaction_id: $tx_field:ident
    ) => {
        // Implement TapMessage first
        $crate::impl_tap_message!($type);

        // Then implement MessageContext
        $crate::impl_message_context!($type,
            participants: [$($participant_spec),*],
            transaction_id: $tx_field
        );
    };

    // Variant for optional transaction ID
    ($type:ty,
        participants: [$($participant_spec:tt),*],
        transaction_id: ($tx_field:ident optional)
    ) => {
        // Implement TapMessage with optional transaction ID
        $crate::impl_tap_message!($type, optional_transaction_id);

        // Implement MessageContext with optional transaction
        impl $crate::message::MessageContext for $type {
            fn participants(&self) -> Vec<&$crate::message::Participant> {
                let mut participants = Vec::new();

                $(
                    $crate::impl_message_context!(@add_participant participants, self, $participant_spec);
                )*

                participants
            }

            fn transaction_context(&self) -> Option<$crate::message::TransactionContext> {
                self.$tx_field.as_ref().map(|tx_id| {
                    $crate::message::TransactionContext::new(
                        tx_id.clone(),
                        <Self as $crate::message::tap_message_trait::TapMessageBody>::message_type().to_string(),
                    )
                })
            }
        }
    };
}
