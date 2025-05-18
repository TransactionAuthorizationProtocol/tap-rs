//! Utility functions for TAP core
//!
//! This module provides utility functions used throughout the TAP core library.

use crate::error::Error;
use crate::message::attachment::Attachment as TapAttachment;
use crate::Result;
use didcomm::{
    Attachment as DidcommAttachment, AttachmentData as DidcommAttachmentData, Base64AttachmentData,
    JsonAttachmentData,
};
use std::time::SystemTime;

/// Gets the current time as a unix timestamp (seconds since the epoch)
///
/// # Returns
///
/// * `Result<u64>` - The current time as a unix timestamp, or an error if
///   the system time cannot be determined.
pub fn get_current_time() -> Result<u64> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| Error::ParseError(format!("Failed to get current time: {}", e)))
        .map(|duration| duration.as_secs())
}

/// Converts a TAP attachment into a DIDComm attachment.
///
/// # Arguments
///
/// * `tap_attachment` - A reference to the TAP Attachment.
///
/// # Returns
///
/// * `Option<DidcommAttachment>` - The converted DIDComm attachment, or None if input data is missing.
pub fn convert_tap_attachment_to_didcomm(
    tap_attachment: &TapAttachment,
) -> Option<DidcommAttachment> {
    // Convert TAP AttachmentData (struct with optional fields)
    // to DIDComm AttachmentData (enum Base64 | Json | Links | ...)
    let didcomm_data_result = tap_attachment.data.as_ref().and_then(|data| {
        if let Some(b64) = &data.base64 {
            // Assuming didcomm::AttachmentData::Base64 exists and takes (value, Option<hash>)
            // We don't have hash info from TAP attachment, so pass None.
            // Need to confirm the exact signature of didcomm::AttachmentData::Base64
            // For now, let's assume it takes { value: String }
            Some(DidcommAttachmentData::Base64 {
                value: Base64AttachmentData {
                    base64: b64.clone(),
                    jws: None,
                },
            })
        } else {
            // Refactor using Option::map as suggested by clippy
            data.json
                .as_ref()
                .map(|json_val| DidcommAttachmentData::Json {
                    value: JsonAttachmentData {
                        json: json_val.clone(),
                        jws: None,
                    },
                })
        }
    });

    // Only create attachment if data is present (as per DIDComm spec)
    didcomm_data_result.map(|data| DidcommAttachment {
        id: Some(tap_attachment.id.clone()),
        media_type: Some(tap_attachment.media_type.clone()),
        data,
        description: None,  // TAP attachment doesn't have description
        filename: None,     // TAP attachment doesn't have filename
        format: None,       // TAP attachment doesn't have format
        lastmod_time: None, // TAP attachment doesn't have lastmod_time
        byte_count: None,   // TAP attachment doesn't have byte_count
    })
}
