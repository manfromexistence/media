//! Unified Media CLI - Search and download media, icons, and fonts
//!
//! Usage:
//!   media search "sunset" --type image
//!   media icon search "home" --limit 10
//!   media font search "roboto"
//!   media tools image convert input.png output.jpg

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match dx_media::cli_unified::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if requested_json_output() {
                let mut causes = Vec::new();
                let mut source = e.source();
                while let Some(cause) = source {
                    causes.push(cause.to_string());
                    source = cause.source();
                }

                eprintln!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": e.to_string(),
                        "causes": causes,
                    })
                );

                return ExitCode::FAILURE;
            }

            eprintln!("Error: {e}");

            // Print chain of errors
            let mut source = e.source();
            while let Some(cause) = source {
                eprintln!("  Caused by: {cause}");
                source = cause.source();
            }

            ExitCode::FAILURE
        }
    }
}

fn requested_json_output() -> bool {
    let args: Vec<_> = std::env::args_os().collect();
    args.windows(2).any(|window| {
        let key = window[0].to_string_lossy();
        let value = window[1].to_string_lossy();
        (key == "--format" || key == "-f") && value.eq_ignore_ascii_case("json")
    }) || args.iter().any(|arg| {
        let arg = arg.to_string_lossy();
        arg.eq_ignore_ascii_case("--format=json") || arg.eq_ignore_ascii_case("-f=json")
    })
}
