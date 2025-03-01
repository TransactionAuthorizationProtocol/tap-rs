//! DIDComm integration for TAP messages.
//!
//! This module handles the integration with DIDComm v2 for secure messaging,
//! including message encryption, decryption, signing, and verification.

mod pack;
mod unpack;

pub use pack::*;
pub use unpack::*;
