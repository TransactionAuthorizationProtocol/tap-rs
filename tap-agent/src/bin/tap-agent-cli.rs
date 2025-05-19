//! Binary entrypoint for the TAP Agent CLI tool
//!
//! This binary provides command-line utilities for creating and managing
//! Decentralized Identifiers (DIDs) and associated cryptographic keys.
//! 
//! When launched, this tool will check for existing keys in ~/.tap/keys.json
//! and use them if available. If no keys are found, it will default to 
//! generating ephemeral keys.

use std::sync::Arc;
use tap_agent::{agent::DefaultAgent, cli};

fn main() {
    // Run the CLI command
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
