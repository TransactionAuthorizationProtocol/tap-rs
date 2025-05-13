//! Binary entrypoint for the TAP Agent CLI tool
//!
//! This binary provides command-line utilities for creating and managing
//! Decentralized Identifiers (DIDs) and associated cryptographic keys.

use tap_agent::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
