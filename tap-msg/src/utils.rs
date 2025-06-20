//! Utility functions for TAP core
//!
//! This module provides utility functions used throughout the TAP core library.

pub mod name_hash;

use crate::error::Error;
use crate::Result;
use std::time::SystemTime;

pub use name_hash::{hash_name, NameHashable};

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
