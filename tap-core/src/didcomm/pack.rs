use crate::error::Result;
use crate::message::TapMessage;
use didcomm::Message;

/// Packs a TAP message into a DIDComm message.
///
/// This function wraps the TAP message into a DIDComm envelope that can be
/// encrypted and authenticated according to the DIDComm spec.
///
/// # Arguments
///
/// * `message` - The TAP message to pack
/// * `from_did` - The DID of the sender (required for authenticated messages)
/// * `to_dids` - The DIDs of the recipients (required for encrypted messages)
///
/// # Returns
///
/// A packed DIDComm message as a JSON string
pub async fn pack_tap_message(
    message: &TapMessage,
    from_did: Option<&str>,
    to_dids: &[&str],
) -> Result<String> {
    // Convert TapMessage to JSON
    let body = serde_json::to_value(message)
        .map_err(|e| crate::error::Error::SerializationError(e.to_string()))?;

    // Build a DIDComm message using the proper API
    let mut message_builder =
        Message::build(message.id.clone(), message.message_type.to_string(), body);

    // Add sender if authentication is needed
    if let Some(from) = from_did {
        message_builder = message_builder.from(from.to_string());
    }

    // Add recipients if encryption is needed
    if !to_dids.is_empty() {
        let to_dids_owned: Vec<String> = to_dids.iter().map(|&s| s.to_string()).collect();
        message_builder = message_builder.to_many(to_dids_owned);
    }

    // Add created time if specified - convert from RFC3339 to unix timestamp
    let created_time = chrono::DateTime::parse_from_rfc3339(&message.created_time)
        .map_err(|e| crate::error::Error::ParseError(e.to_string()))?;
    let unix_timestamp = created_time.timestamp() as u64;
    message_builder = message_builder.created_time(unix_timestamp);

    // Add expires time if specified
    if let Some(expires_time) = &message.expires_time {
        let expires = chrono::DateTime::parse_from_rfc3339(expires_time)
            .map_err(|e| crate::error::Error::ParseError(e.to_string()))?;
        let expires_unix = expires.timestamp() as u64;
        message_builder = message_builder.expires_time(expires_unix);
    }

    // Finalize the message
    let didcomm_message = message_builder.finalize();

    // Pack the message (this would normally include encryption and signing depending on the security settings)
    let packed_msg = serde_json::to_string(&didcomm_message)
        .map_err(|e| crate::error::Error::SerializationError(e.to_string()))?;

    Ok(packed_msg)
}
