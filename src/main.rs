//! DX Media CLI - Universal Digital Asset Acquisition
//!
//! Usage:
//!   dx search "sunset mountains" --type image
//!   dx download openverse:abc123
//!   dx providers --available

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match dx_media::cli::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");

            // Print chain of errors
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                eprintln!("  Caused by: {cause}");
                source = std::error::Error::source(cause);
            }

            ExitCode::FAILURE
        }
    }
}
