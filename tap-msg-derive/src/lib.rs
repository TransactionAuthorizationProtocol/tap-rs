//! Procedural derive macro for implementing TAP message traits.
//!
//! This crate provides the `#[derive(TapMessage)]` macro that automatically
//! implements both `TapMessage` and `MessageContext` traits based on field attributes.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

/// Procedural derive macro for implementing TapMessage, MessageContext, and optionally TapMessageBody traits.
///
/// # Usage
///
/// ## Basic Usage (TapMessage + MessageContext only)
///
/// ```ignore
/// use tap_msg::TapMessage;
/// use tap_msg::message::Participant;
/// use tap_caip::AssetId;
///
/// #[derive(TapMessage)]
/// pub struct Transfer {
///     #[tap(participant)]
///     pub originator: Participant,
///     
///     #[tap(participant)]
///     pub beneficiary: Option<Participant>,
///     
///     #[tap(participant_list)]
///     pub agents: Vec<Participant>,
///     
///     #[tap(transaction_id)]
///     pub transaction_id: String,
///     
///     // regular fields don't need attributes
///     pub amount: String,
///     pub asset: AssetId,
/// }
/// ```
///
/// ## Full Usage (includes TapMessageBody with auto-generated to_didcomm)
///
/// ```ignore
/// use tap_msg::TapMessage;
/// use tap_msg::message::Participant;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize, TapMessage)]
/// #[tap(message_type = "https://tap.rsvp/schema/1.0#transfer")]
/// pub struct Transfer {
///     #[tap(participant)]
///     pub originator: Participant,
///     
///     #[tap(participant)]
///     pub beneficiary: Option<Participant>,
///     
///     #[tap(participant_list)]
///     pub agents: Vec<Participant>,
///     
///     #[tap(transaction_id)]
///     pub transaction_id: String,
///     
///     pub amount: String,
/// }
///
/// // TapMessageBody is automatically implemented with:
/// // - message_type() returning the specified string
/// // - validate() with basic validation (can be overridden)
/// // - to_didcomm() with automatic participant extraction and message construction
/// ```
///
/// # Supported Attributes
///
/// ## Struct-level Attributes
/// - `#[tap(message_type = "url")]` - TAP message type URL (enables TapMessageBody generation)
/// - `#[tap(generated_id)]` - Indicates the message uses a generated ID
///
/// ## Field-level Attributes
/// - `#[tap(participant)]` - Single participant field (required or optional)
/// - `#[tap(participant_list)]` - Vec<Participant> field
/// - `#[tap(transaction_id)]` - Transaction ID field
/// - `#[tap(optional_transaction_id)]` - Optional transaction ID field
/// - `#[tap(thread_id)]` - Thread ID field (for thread-based messages)
#[proc_macro_derive(TapMessage, attributes(tap))]
pub fn derive_tap_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = impl_tap_message(&input);
    TokenStream::from(expanded)
}

#[proc_macro_derive(TapMessageBody, attributes(tap))]
pub fn derive_tap_message_body(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = impl_tap_message_body_only(&input);
    TokenStream::from(expanded)
}

fn impl_tap_message(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("TapMessage can only be derived for structs with named fields"),
        },
        _ => panic!("TapMessage can only be derived for structs"),
    };

    let field_info = analyze_fields(fields, &input.attrs);

    // Check if we're inside the tap-msg crate or external
    let is_internal = std::env::var("CARGO_CRATE_NAME").unwrap_or_default() == "tap_msg";

    // TapMessageBody implementation is now handled by separate derive macro
    let tap_message_body_impl: Option<TokenStream2> = None;

    let tap_message_impl = impl_tap_message_trait(
        name,
        &field_info,
        &impl_generics,
        &ty_generics,
        where_clause,
        is_internal,
    );

    let message_context_impl = impl_message_context_trait(
        name,
        &field_info,
        &impl_generics,
        &ty_generics,
        where_clause,
        is_internal,
    );

    quote! {
        #tap_message_body_impl
        #tap_message_impl
        #message_context_impl
    }
}

#[derive(Debug)]
struct FieldInfo {
    participant_fields: Vec<syn::Ident>,
    optional_participant_fields: Vec<syn::Ident>,
    participant_list_fields: Vec<syn::Ident>,
    transaction_id_field: Option<syn::Ident>,
    optional_transaction_id_field: Option<syn::Ident>,
    thread_id_field: Option<syn::Ident>,
    has_generated_id: bool,
    message_type: Option<String>,
}

fn analyze_fields(
    fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>,
    struct_attrs: &[syn::Attribute],
) -> FieldInfo {
    let mut field_info = FieldInfo {
        participant_fields: Vec::new(),
        optional_participant_fields: Vec::new(),
        participant_list_fields: Vec::new(),
        transaction_id_field: None,
        optional_transaction_id_field: None,
        thread_id_field: None,
        has_generated_id: false,
        message_type: None,
    };

    // First check struct-level attributes
    for attr in struct_attrs {
        if attr.path().is_ident("tap") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("generated_id") {
                    field_info.has_generated_id = true;
                } else if meta.path.is_ident("message_type") {
                    if let Ok(lit) = meta.value() {
                        if let Ok(lit_str) = lit.parse::<syn::LitStr>() {
                            field_info.message_type = Some(lit_str.value());
                        }
                    }
                }
                Ok(())
            });
        }
    }

    for field in fields {
        let field_name = field.ident.as_ref().expect("Field must have a name");

        for attr in &field.attrs {
            if attr.path().is_ident("tap") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("participant") {
                        // Check if the field type is Option<Participant>
                        if is_optional_type(&field.ty) {
                            field_info
                                .optional_participant_fields
                                .push(field_name.clone());
                        } else {
                            field_info.participant_fields.push(field_name.clone());
                        }
                    } else if meta.path.is_ident("participant_list") {
                        field_info.participant_list_fields.push(field_name.clone());
                    } else if meta.path.is_ident("transaction_id") {
                        field_info.transaction_id_field = Some(field_name.clone());
                    } else if meta.path.is_ident("optional_transaction_id") {
                        field_info.optional_transaction_id_field = Some(field_name.clone());
                    } else if meta.path.is_ident("thread_id") {
                        field_info.thread_id_field = Some(field_name.clone());
                    } else if meta.path.is_ident("generated_id") {
                        field_info.has_generated_id = true;
                    }
                    Ok(())
                });
            }
        }
    }

    field_info
}

fn is_optional_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn impl_tap_message_trait(
    name: &syn::Ident,
    field_info: &FieldInfo,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    is_internal: bool,
) -> TokenStream2 {
    let thread_id_impl = generate_thread_id_impl(field_info);
    let message_id_impl = generate_message_id_impl(field_info);
    let get_all_participants_impl = generate_get_all_participants_impl(field_info);

    let crate_path = if is_internal {
        quote! { crate }
    } else {
        quote! { ::tap_msg }
    };

    // message_type is no longer part of TapMessage trait

    quote! {
        impl #impl_generics #crate_path::message::tap_message_trait::TapMessage for #name #ty_generics #where_clause {
            fn validate(&self) -> #crate_path::error::Result<()> {
                <Self as #crate_path::message::tap_message_trait::TapMessageBody>::validate(self)
            }

            fn is_tap_message(&self) -> bool {
                <Self as #crate_path::message::tap_message_trait::TapMessageBody>::message_type()
                    .starts_with("https://tap.rsvp/schema/1.0#")
            }

            fn get_tap_type(&self) -> Option<String> {
                Some(
                    <Self as #crate_path::message::tap_message_trait::TapMessageBody>::message_type()
                        .to_string(),
                )
            }

            fn body_as<T: #crate_path::message::tap_message_trait::TapMessageBody>(
                &self,
            ) -> #crate_path::error::Result<T> {
                unimplemented!()
            }

            fn get_all_participants(&self) -> Vec<String> {
                #get_all_participants_impl
            }

            fn create_reply<T: #crate_path::message::tap_message_trait::TapMessageBody>(
                &self,
                body: &T,
                creator_did: &str,
            ) -> #crate_path::error::Result<#crate_path::didcomm::PlainMessage> {
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
                #thread_id_impl
            }

            fn parent_thread_id(&self) -> Option<&str> {
                None
            }

            fn message_id(&self) -> &str {
                #message_id_impl
            }
        }
    }
}

fn impl_message_context_trait(
    name: &syn::Ident,
    field_info: &FieldInfo,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    is_internal: bool,
) -> TokenStream2 {
    let participants_impl = generate_participants_impl(field_info);
    let transaction_context_impl = generate_transaction_context_impl(field_info, is_internal);

    let crate_path = if is_internal {
        quote! { crate }
    } else {
        quote! { ::tap_msg }
    };

    quote! {
        impl #impl_generics #crate_path::message::MessageContext for #name #ty_generics #where_clause {
            fn participants(&self) -> Vec<&#crate_path::message::Participant> {
                #participants_impl
            }

            fn transaction_context(&self) -> Option<#crate_path::message::TransactionContext> {
                #transaction_context_impl
            }
        }
    }
}

fn generate_participants_impl(field_info: &FieldInfo) -> TokenStream2 {
    let mut participant_pushes = Vec::new();

    // Add required participants
    for field in &field_info.participant_fields {
        participant_pushes.push(quote! {
            participants.push(&self.#field);
        });
    }

    // Add optional participants
    for field in &field_info.optional_participant_fields {
        participant_pushes.push(quote! {
            if let Some(ref participant) = self.#field {
                participants.push(participant);
            }
        });
    }

    // Add participant lists
    for field in &field_info.participant_list_fields {
        participant_pushes.push(quote! {
            participants.extend(&self.#field);
        });
    }

    quote! {
        let mut participants = Vec::new();
        #(#participant_pushes)*
        participants
    }
}

fn generate_get_all_participants_impl(field_info: &FieldInfo) -> TokenStream2 {
    let mut participant_extracts = Vec::new();

    // Add required participants
    for field in &field_info.participant_fields {
        participant_extracts.push(quote! {
            participants.push(self.#field.id.clone());
        });
    }

    // Add optional participants
    for field in &field_info.optional_participant_fields {
        participant_extracts.push(quote! {
            if let Some(ref participant) = self.#field {
                participants.push(participant.id.clone());
            }
        });
    }

    // Add participant lists
    for field in &field_info.participant_list_fields {
        participant_extracts.push(quote! {
            for participant in &self.#field {
                participants.push(participant.id.clone());
            }
        });
    }

    quote! {
        let mut participants = Vec::new();
        #(#participant_extracts)*
        participants
    }
}

fn generate_thread_id_impl(field_info: &FieldInfo) -> TokenStream2 {
    if let Some(thread_field) = &field_info.thread_id_field {
        quote! { self.#thread_field.as_deref() }
    } else if let Some(tx_field) = &field_info.transaction_id_field {
        quote! { Some(&self.#tx_field) }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! { self.#opt_tx_field.as_deref() }
    } else {
        quote! { None }
    }
}

fn generate_message_id_impl(field_info: &FieldInfo) -> TokenStream2 {
    if let Some(tx_field) = &field_info.transaction_id_field {
        quote! { &self.#tx_field }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! {
            if let Some(ref id) = self.#opt_tx_field {
                id
            } else {
                &self.id
            }
        }
    } else if let Some(thread_field) = &field_info.thread_id_field {
        quote! {
            if let Some(ref thid) = self.#thread_field {
                thid
            } else {
                &self.id
            }
        }
    } else if field_info.has_generated_id {
        quote! {
            // For types without an ID field, we'll use a static string
            // This isn't ideal but it satisfies the API contract
            static FALLBACK_ID: &str = "00000000-0000-0000-0000-000000000000";
            FALLBACK_ID
        }
    } else {
        quote! { &self.transaction_id }
    }
}

fn generate_transaction_context_impl(field_info: &FieldInfo, is_internal: bool) -> TokenStream2 {
    let crate_path = if is_internal {
        quote! { crate }
    } else {
        quote! { ::tap_msg }
    };

    if let Some(tx_field) = &field_info.transaction_id_field {
        quote! {
            Some(#crate_path::message::TransactionContext::new(
                self.#tx_field.clone(),
                <Self as #crate_path::message::tap_message_trait::TapMessageBody>::message_type().to_string(),
            ))
        }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! {
            self.#opt_tx_field.as_ref().map(|tx_id| {
                #crate_path::message::TransactionContext::new(
                    tx_id.clone(),
                    <Self as #crate_path::message::tap_message_trait::TapMessageBody>::message_type().to_string(),
                )
            })
        }
    } else {
        quote! { None }
    }
}

fn impl_tap_message_body_trait(
    name: &syn::Ident,
    field_info: &FieldInfo,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
    is_internal: bool,
) -> TokenStream2 {
    let crate_path = if is_internal {
        quote! { crate }
    } else {
        quote! { ::tap_msg }
    };

    let message_type = field_info
        .message_type
        .as_ref()
        .expect("Message type should be present");
    let to_didcomm_impl = generate_to_didcomm_impl(field_info, is_internal);

    quote! {
        impl #impl_generics #crate_path::message::tap_message_trait::TapMessageBody for #name #ty_generics #where_clause {
            fn message_type() -> &'static str {
                #message_type
            }

            fn validate(&self) -> #crate_path::error::Result<()> {
                // Basic validation - users can override this by implementing TapMessageBody manually
                Ok(())
            }

            fn to_didcomm(&self, from_did: &str) -> #crate_path::error::Result<#crate_path::didcomm::PlainMessage> {
                #to_didcomm_impl
            }
        }
    }
}

fn generate_to_didcomm_impl(field_info: &FieldInfo, is_internal: bool) -> TokenStream2 {
    let crate_path = if is_internal {
        quote! { crate }
    } else {
        quote! { ::tap_msg }
    };

    // Generate participant extraction
    let participant_extraction = if !field_info.participant_fields.is_empty()
        || !field_info.optional_participant_fields.is_empty()
        || !field_info.participant_list_fields.is_empty()
    {
        let mut extracts = Vec::new();

        // Required participants
        for field in &field_info.participant_fields {
            extracts.push(quote! {
                recipient_dids.push(self.#field.id.clone());
            });
        }

        // Optional participants
        for field in &field_info.optional_participant_fields {
            extracts.push(quote! {
                if let Some(ref participant) = self.#field {
                    recipient_dids.push(participant.id.clone());
                }
            });
        }

        // Participant lists
        for field in &field_info.participant_list_fields {
            extracts.push(quote! {
                for participant in &self.#field {
                    recipient_dids.push(participant.id.clone());
                }
            });
        }

        quote! {
            let mut recipient_dids = Vec::new();
            #(#extracts)*

            // Remove duplicates and sender
            recipient_dids.sort();
            recipient_dids.dedup();
            recipient_dids.retain(|did| did != from_did);
        }
    } else {
        quote! {
            let recipient_dids: Vec<String> = Vec::new();
        }
    };

    // Generate thread ID assignment
    let thread_assignment = if let Some(tx_field) = &field_info.transaction_id_field {
        quote! {
            thid: Some(self.#tx_field.clone()),
        }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! {
            thid: self.#opt_tx_field.clone(),
        }
    } else if let Some(thread_field) = &field_info.thread_id_field {
        quote! {
            thid: self.#thread_field.clone(),
        }
    } else {
        quote! {
            thid: None,
        }
    };

    quote! {
        // Serialize the message body to JSON
        let mut body_json = serde_json::to_value(self)
            .map_err(|e| #crate_path::error::Error::SerializationError(e.to_string()))?;

        // Ensure the @type field is correctly set in the body
        if let Some(body_obj) = body_json.as_object_mut() {
            body_obj.insert(
                "@type".to_string(),
                serde_json::Value::String(Self::message_type().to_string()),
            );
        }

        // Extract recipient DIDs from participants
        #participant_extraction

        let now = chrono::Utc::now().timestamp() as u64;

        // Create the PlainMessage
        Ok(#crate_path::didcomm::PlainMessage {
            id: uuid::Uuid::new_v4().to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: Self::message_type().to_string(),
            body: body_json,
            from: from_did.to_string(),
            to: recipient_dids,
            #thread_assignment
            pthid: None,
            created_time: Some(now),
            expires_time: None,
            extra_headers: std::collections::HashMap::new(),
            from_prior: None,
            attachments: None,
        })
    }
}

fn impl_tap_message_body_only(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("TapMessageBody can only be derived for structs with named fields"),
        },
        _ => panic!("TapMessageBody can only be derived for structs"),
    };

    let field_info = analyze_fields(fields, &input.attrs);

    // Check if we're inside the tap-msg crate or external
    let is_internal = std::env::var("CARGO_CRATE_NAME").unwrap_or_default() == "tap_msg";

    // TapMessageBody can only be derived if message_type is specified
    if field_info.message_type.is_none() {
        panic!("TapMessageBody derive macro requires #[tap(message_type = \"...\")] attribute");
    }

    impl_tap_message_body_trait(
        name,
        &field_info,
        &impl_generics,
        &ty_generics,
        where_clause,
        is_internal,
    )
}
