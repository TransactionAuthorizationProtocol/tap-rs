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
/// #[tap(message_type = "https://tap.rsvp/schema/1.0#Transfer", initiator, authorizable, transactable)]
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
/// // - Authorizable trait (if authorizable attribute is present)
/// // - Transaction trait (if transactable attribute is present)
/// ```
///
/// # Supported Attributes
///
/// ## Struct-level Attributes
/// - `#[tap(message_type = "url")]` - TAP message type URL (enables TapMessageBody generation)
/// - `#[tap(initiator)]` - Marks this as a conversation-initiating message
/// - `#[tap(authorizable)]` - Auto-generates Authorizable trait implementation
/// - `#[tap(transactable)]` - Auto-generates Transaction trait implementation
/// - `#[tap(builder)]` - Auto-generates builder pattern
///
/// ## Field-level Attributes
/// - `#[tap(participant)]` - Single participant field (required or optional)
/// - `#[tap(participant_list)]` - Vec<Participant> field
/// - `#[tap(transaction_id)]` - Transaction ID field (creates new transaction for initiators)
/// - `#[tap(thread_id)]` - Thread ID field (references existing transaction for replies)
/// - `#[tap(connection_id)]` - Connection ID field (for linking to Connect messages)
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

    let tap_message_body_impl = if field_info.message_type.is_some() {
        impl_tap_message_body_trait(
            name,
            &field_info,
            &impl_generics,
            &ty_generics,
            where_clause,
            is_internal,
        )
    } else {
        quote! {}
    };

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

    let authorizable_impl = if field_info.is_authorizable {
        impl_authorizable_trait(
            name,
            &field_info,
            &impl_generics,
            &ty_generics,
            where_clause,
            is_internal,
        )
    } else {
        quote! {}
    };

    let transaction_impl = if field_info.is_transactable {
        impl_transaction_trait(
            name,
            &field_info,
            &impl_generics,
            &ty_generics,
            where_clause,
            is_internal,
        )
    } else {
        quote! {}
    };

    let connectable_impl = if field_info.connection_id_field.is_some() || field_info.is_initiator {
        impl_connectable_trait(
            name,
            &field_info,
            &impl_generics,
            &ty_generics,
            where_clause,
            is_internal,
        )
    } else {
        quote! {}
    };

    quote! {
        #tap_message_impl
        #message_context_impl
        #tap_message_body_impl
        #authorizable_impl
        #transaction_impl
        #connectable_impl
    }
}

fn impl_connectable_trait(
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

    if let Some(ref conn_field) = field_info.connection_id_field {
        // Has explicit connection_id field
        quote! {
            impl #impl_generics #crate_path::message::tap_message_trait::Connectable for #name #ty_generics #where_clause {
                fn with_connection(&mut self, connect_id: &str) -> &mut Self {
                    self.#conn_field = Some(connect_id.to_string());
                    self
                }

                fn has_connection(&self) -> bool {
                    self.#conn_field.is_some()
                }

                fn connection_id(&self) -> Option<&str> {
                    self.#conn_field.as_deref()
                }
            }
        }
    } else {
        // Initiator messages don't have connections
        quote! {
            impl #impl_generics #crate_path::message::tap_message_trait::Connectable for #name #ty_generics #where_clause {
                fn with_connection(&mut self, _connect_id: &str) -> &mut Self {
                    // Initiator messages don't have connection IDs
                    self
                }

                fn has_connection(&self) -> bool {
                    false
                }

                fn connection_id(&self) -> Option<&str> {
                    None
                }
            }
        }
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
    optional_thread_id_field: Option<syn::Ident>,
    connection_id_field: Option<syn::Ident>,
    has_generated_id: bool,
    message_type: Option<String>,
    is_initiator: bool,
    is_authorizable: bool,
    is_transactable: bool,
    generate_builder: bool,
    custom_validation: bool,
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
        optional_thread_id_field: None,
        connection_id_field: None,
        has_generated_id: false,
        message_type: None,
        is_initiator: false,
        is_authorizable: false,
        is_transactable: false,
        generate_builder: false,
        custom_validation: false,
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
                } else if meta.path.is_ident("initiator") {
                    field_info.is_initiator = true;
                } else if meta.path.is_ident("authorizable") {
                    field_info.is_authorizable = true;
                } else if meta.path.is_ident("transactable") {
                    field_info.is_transactable = true;
                } else if meta.path.is_ident("builder") {
                    field_info.generate_builder = true;
                } else if meta.path.is_ident("custom_validation") {
                    field_info.custom_validation = true;
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
                        // Check if the field type is Option<String>
                        if is_optional_type(&field.ty) {
                            field_info.optional_thread_id_field = Some(field_name.clone());
                        } else {
                            field_info.thread_id_field = Some(field_name.clone());
                        }
                    } else if meta.path.is_ident("connection_id") {
                        field_info.connection_id_field = Some(field_name.clone());
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

                // message is already PlainMessage<Value> from to_didcomm

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
    if let Some(ref thread_field) = field_info.thread_id_field {
        quote! { Some(&self.#thread_field) }
    } else if let Some(ref opt_thread_field) = field_info.optional_thread_id_field {
        quote! { self.#opt_thread_field.as_deref() }
    } else if field_info.is_initiator {
        // Initiators don't have a thread_id - they start the thread
        quote! { None }
    } else if let Some(ref tx_field) = field_info.transaction_id_field {
        quote! { Some(&self.#tx_field) }
    } else if let Some(ref opt_tx_field) = field_info.optional_transaction_id_field {
        quote! { self.#opt_tx_field.as_deref() }
    } else {
        quote! { None }
    }
}

fn generate_message_id_impl(field_info: &FieldInfo) -> TokenStream2 {
    if let Some(tx_field) = &field_info.transaction_id_field {
        quote! { &self.#tx_field }
    } else if let Some(thread_field) = &field_info.thread_id_field {
        quote! { &self.#thread_field }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! {
            self.#opt_tx_field.as_deref().unwrap_or("")
        }
    } else {
        quote! {
            // For types without an ID field, we'll use a static string
            // This isn't ideal but it satisfies the API contract
            static FALLBACK_ID: &str = "00000000-0000-0000-0000-000000000000";
            FALLBACK_ID
        }
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
        .expect("message_type attribute is required for TapMessageBody");

    let validate_impl = if field_info.custom_validation {
        // If custom_validation is specified, delegate to a validate_<struct_name_lowercase> method
        let method_name = syn::Ident::new(
            &format!("validate_{}", name.to_string().to_lowercase()),
            name.span(),
        );
        quote! {
            self.#method_name()
        }
    } else {
        quote! {
            // Basic validation - users can override this by implementing custom validation
            Ok(())
        }
    };

    let to_didcomm_impl = generate_to_didcomm_impl(field_info, is_internal);

    quote! {
        impl #impl_generics #crate_path::message::tap_message_trait::TapMessageBody for #name #ty_generics #where_clause {
            fn message_type() -> &'static str {
                #message_type
            }

            fn validate(&self) -> #crate_path::error::Result<()> {
                #validate_impl
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
    let thread_assignment = if field_info.is_initiator {
        // Initiators create a new thread - their transaction_id becomes the thid for replies
        if let Some(tx_field) = &field_info.transaction_id_field {
            quote! {
                thid: Some(self.#tx_field.clone()),
            }
        } else {
            quote! {
                thid: None,
            }
        }
    } else if let Some(thread_field) = &field_info.thread_id_field {
        // Reply messages use thread_id to reference the existing transaction
        quote! {
            thid: Some(self.#thread_field.clone()),
        }
    } else if let Some(opt_thread_field) = &field_info.optional_thread_id_field {
        // Optional thread_id field
        quote! {
            thid: self.#opt_thread_field.clone(),
        }
    } else if let Some(tx_field) = &field_info.transaction_id_field {
        // Fallback for backwards compatibility
        quote! {
            thid: Some(self.#tx_field.clone()),
        }
    } else if let Some(opt_tx_field) = &field_info.optional_transaction_id_field {
        quote! {
            thid: self.#opt_tx_field.clone(),
        }
    } else {
        quote! {
            thid: None,
        }
    };

    // Generate pthid assignment for connection linking
    let pthid_assignment = if let Some(conn_field) = &field_info.connection_id_field {
        quote! {
            pthid: self.#conn_field.clone(),
        }
    } else {
        quote! {
            pthid: None,
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
            #pthid_assignment
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

fn impl_authorizable_trait(
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

    // Get the transaction ID field access
    let tx_id_access = if let Some(ref tx_field) = field_info.transaction_id_field {
        quote! { &self.#tx_field }
    } else if let Some(ref thread_field) = field_info.thread_id_field {
        quote! { &self.#thread_field }
    } else {
        quote! { "" }
    };

    quote! {
        impl #impl_generics #crate_path::message::tap_message_trait::Authorizable for #name #ty_generics #where_clause {
            fn authorize(
                &self,
                creator_did: &str,
                settlement_address: Option<&str>,
                expiry: Option<&str>,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::Authorize> {
                let authorize = #crate_path::message::Authorize::with_all(#tx_id_access, settlement_address, expiry);
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&authorize, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, authorize)
            }

            fn cancel(&self, creator_did: &str, by: &str, reason: Option<&str>) -> #crate_path::didcomm::PlainMessage<#crate_path::message::Cancel> {
                let cancel = if let Some(reason) = reason {
                    #crate_path::message::Cancel::with_reason(#tx_id_access, by, reason)
                } else {
                    #crate_path::message::Cancel::new(#tx_id_access, by)
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&cancel, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, cancel)
            }

            fn reject(&self, creator_did: &str, reason: &str) -> #crate_path::didcomm::PlainMessage<#crate_path::message::Reject> {
                let reject = #crate_path::message::Reject {
                    transaction_id: (#tx_id_access).to_string(),
                    reason: reason.to_string(),
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&reject, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, reject)
            }
        }
    }
}

fn impl_transaction_trait(
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

    // Get the transaction ID field access
    let tx_id_access = if let Some(ref tx_field) = field_info.transaction_id_field {
        quote! { &self.#tx_field }
    } else if let Some(ref thread_field) = field_info.thread_id_field {
        quote! { &self.#thread_field }
    } else {
        quote! { "" }
    };

    quote! {
        impl #impl_generics #crate_path::message::tap_message_trait::Transaction for #name #ty_generics #where_clause {
            fn settle(
                &self,
                creator_did: &str,
                settlement_id: &str,
                amount: Option<&str>,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::Settle> {
                let settle = #crate_path::message::Settle {
                    transaction_id: (#tx_id_access).to_string(),
                    settlement_id: settlement_id.to_string(),
                    amount: amount.map(|s| s.to_string()),
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&settle, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, settle)
            }

            fn revert(
                &self,
                creator_did: &str,
                settlement_address: &str,
                reason: &str,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::Revert> {
                let revert = #crate_path::message::Revert {
                    transaction_id: (#tx_id_access).to_string(),
                    settlement_address: settlement_address.to_string(),
                    reason: reason.to_string(),
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&revert, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, revert)
            }

            fn add_agents(&self, creator_did: &str, agents: Vec<#crate_path::message::Participant>) -> #crate_path::didcomm::PlainMessage<#crate_path::message::AddAgents> {
                let add_agents = #crate_path::message::AddAgents {
                    transaction_id: (#tx_id_access).to_string(),
                    agents,
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&add_agents, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, add_agents)
            }

            fn replace_agent(
                &self,
                creator_did: &str,
                original_agent: &str,
                replacement: #crate_path::message::Participant,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::ReplaceAgent> {
                let replace_agent = #crate_path::message::ReplaceAgent {
                    transaction_id: (#tx_id_access).to_string(),
                    original: original_agent.to_string(),
                    replacement,
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&replace_agent, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, replace_agent)
            }

            fn remove_agent(&self, creator_did: &str, agent: &str) -> #crate_path::didcomm::PlainMessage<#crate_path::message::RemoveAgent> {
                let remove_agent = #crate_path::message::RemoveAgent {
                    transaction_id: (#tx_id_access).to_string(),
                    agent: agent.to_string(),
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&remove_agent, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, remove_agent)
            }

            fn update_party(
                &self,
                creator_did: &str,
                party_type: &str,
                party: #crate_path::message::Participant,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::UpdateParty> {
                let update_party = #crate_path::message::UpdateParty {
                    transaction_id: (#tx_id_access).to_string(),
                    party_type: party_type.to_string(),
                    party,
                    context: None,
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&update_party, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, update_party)
            }

            fn update_policies(
                &self,
                creator_did: &str,
                policies: Vec<#crate_path::message::Policy>,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::UpdatePolicies> {
                let update_policies = #crate_path::message::UpdatePolicies {
                    transaction_id: (#tx_id_access).to_string(),
                    policies,
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&update_policies, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, update_policies)
            }

            fn confirm_relationship(
                &self,
                creator_did: &str,
                agent_did: &str,
                relationship_type: &str,
            ) -> #crate_path::didcomm::PlainMessage<#crate_path::message::ConfirmRelationship> {
                let confirm_relationship = #crate_path::message::ConfirmRelationship {
                    transaction_id: (#tx_id_access).to_string(),
                    agent_id: agent_did.to_string(),
                    relationship_type: relationship_type.to_string(),
                };
                let original_message = self
                    .to_didcomm(creator_did)
                    .expect("Failed to create DIDComm message");
                let reply = original_message
                    .create_reply(&confirm_relationship, creator_did)
                    .expect("Failed to create reply");
                #crate_path::message::tap_message_trait::typed_plain_message(reply, confirm_relationship)
            }
        }
    }
}

