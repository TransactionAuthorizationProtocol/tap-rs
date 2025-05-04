use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro to implement the TapMessage trait for body types.
#[proc_macro_derive(TapMessage)]
pub fn derive_tap_message(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let gen = quote! {
        impl tap_msg::message::tap_message_trait::TapMessage for #name {
            fn validate(&self) -> tap_msg::error::Result<()> {
                <Self as tap_msg::message::tap_message_trait::TapMessageBody>::validate(self)
            }
            fn is_tap_message(&self) -> bool {
                false
            }
            fn get_tap_type(&self) -> Option<String> {
                Some(<Self as tap_msg::message::tap_message_trait::TapMessageBody>::message_type().to_string())
            }
            fn body_as<T: tap_msg::message::tap_message_trait::TapMessageBody>(&self) -> tap_msg::error::Result<T> {
                unimplemented!()
            }
            fn get_all_participants(&self) -> Vec<String> {
                Vec::new()
            }
            fn create_reply<T: tap_msg::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> tap_msg::error::Result<tap_msg::message::tap_message_trait::Message> {
                tap_msg::message::tap_message_trait::TapMessage::create_reply(self, body, creator_did)
            }
            fn message_type(&self) -> &'static str {
                <Self as tap_msg::message::tap_message_trait::TapMessageBody>::message_type()
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

    gen.into()
}
