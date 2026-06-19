//! QR Code generation and reading tool.
//!
//! Generate QR codes from text/URLs using qrencode command.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// QR Code error correction level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QrErrorCorrection {
    /// ~7% error recovery.
    Low,
    /// ~15% error recovery.
    #[default]
    Medium,
    /// ~25% error recovery.
    Quartile,
    /// ~30% error recovery.
    High,
}

impl QrErrorCorrection {
    fn to_arg(&self) -> &str {
        match self {
            Self::Low => "L",
            Self::Medium => "M",
            Self::Quartile => "Q",
            Self::High => "H",
        }
    }
}

/// QR Code generation options.
#[derive(Debug, Clone)]
pub struct QrCodeOptions {
    /// Size multiplier for the QR code.
    pub size: u32,
    /// Error correction level.
    pub error_correction: QrErrorCorrection,
    /// Margin in modules.
    pub margin: u32,
}

impl Default for QrCodeOptions {
    fn default() -> Self {
        Self {
            size: 10,
            error_correction: QrErrorCorrection::default(),
            margin: 4,
        }
    }
}

impl QrCodeOptions {
    /// Create options with specific size.
    pub fn with_size(size: u32) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }

    /// Set error correction level.
    pub fn with_error_correction(mut self, level: QrErrorCorrection) -> Self {
        self.error_correction = level;
        self
    }

    /// Set margin.
    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }
}

/// Generate a QR code image from text data.
///
/// Requires `qrencode` command to be installed.
///
/// # Arguments
/// * `data` - The text/URL to encode
/// * `output` - Path to save the QR code image
/// * `size` - Size multiplier for the output
///
/// # Example
/// ```no_run
/// use dx_media::tools::image::qrcode::generate_qr;
///
/// // Generate QR code for a URL
/// generate_qr("https://example.com", "qr_code.png", 10).unwrap();
/// ```
pub fn generate_qr<P: AsRef<Path>>(data: &str, output: P, size: u32) -> Result<ToolOutput> {
    generate_qr_with_options(data, output, QrCodeOptions::with_size(size))
}

/// Generate a QR code with detailed options.
///
/// Uses `qrencode` command-line tool.
pub fn generate_qr_with_options<P: AsRef<Path>>(
    data: &str,
    output: P,
    options: QrCodeOptions,
) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let status = Command::new("qrencode")
        .args([
            "-o",
            output_path.to_str().unwrap_or("output.png"),
            "-s",
            &options.size.to_string(),
            "-l",
            options.error_correction.to_arg(),
            "-m",
            &options.margin.to_string(),
            data,
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute qrencode: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "qrencode command failed".to_string(),
        });
    }

    Ok(
        ToolOutput::success_with_path("QR code generated successfully", output_path)
            .with_metadata("data_length", data.len().to_string())
            .with_metadata(
                "error_correction",
                options.error_correction.to_arg().to_string(),
            ),
    )
}

/// Generate QR code to SVG format.
pub fn generate_qr_svg<P: AsRef<Path>>(data: &str, output: P, size: u32) -> Result<ToolOutput> {
    let output_path = output.as_ref();

    let status = Command::new("qrencode")
        .args([
            "-t",
            "SVG",
            "-o",
            output_path.to_str().unwrap_or("output.svg"),
            "-s",
            &size.to_string(),
            data,
        ])
        .status()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute qrencode: {}", e),
        })?;

    if !status.success() {
        return Err(DxError::Internal {
            message: "qrencode SVG command failed".to_string(),
        });
    }

    Ok(ToolOutput::success_with_path(
        "QR code SVG generated successfully",
        output_path,
    ))
}

/// Decode a QR code from an image.
///
/// Uses `zbarimg` command-line tool.
pub fn decode_qr<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();

    let output = Command::new("zbarimg")
        .args(["--raw", "-q", input_path.to_str().unwrap_or("")])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute zbarimg: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "No QR code found in image".to_string(),
        });
    }

    let decoded = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(ToolOutput::success(format!("Decoded: {}", decoded))
        .with_metadata("decoded_data", decoded)
        .with_metadata("source", input_path.display().to_string()))
}

/// Generate QR code with text encoding to terminal (ASCII art).
pub fn generate_qr_ascii(data: &str) -> Result<ToolOutput> {
    let output = Command::new("qrencode")
        .args(["-t", "ANSIUTF8", data])
        .output()
        .map_err(|e| DxError::Internal {
            message: format!("Failed to execute qrencode: {}", e),
        })?;

    if !output.status.success() {
        return Err(DxError::Internal {
            message: "qrencode ASCII command failed".to_string(),
        });
    }

    let ascii = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(ToolOutput::success(ascii))
}

/// Batch generate QR codes from a list of data.
pub fn generate_qr_batch<P: AsRef<Path>>(
    data_list: &[&str],
    output_dir: P,
    options: QrCodeOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();

    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut generated = 0;
    for (i, data) in data_list.iter().enumerate() {
        let output_path = output_dir.join(format!("qr_{:04}.png", i));
        generate_qr_with_options(data, &output_path, options.clone())?;
        generated += 1;
    }

    Ok(ToolOutput::success(format!(
        "Generated {} QR codes in {}",
        generated,
        output_dir.display()
    ))
    .with_metadata("count", generated.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_correction_arg() {
        assert_eq!(QrErrorCorrection::Low.to_arg(), "L");
        assert_eq!(QrErrorCorrection::Medium.to_arg(), "M");
        assert_eq!(QrErrorCorrection::Quartile.to_arg(), "Q");
        assert_eq!(QrErrorCorrection::High.to_arg(), "H");
    }

    #[test]
    fn test_options_builder() {
        let opts = QrCodeOptions::with_size(20)
            .with_error_correction(QrErrorCorrection::High)
            .with_margin(2);

        assert_eq!(opts.size, 20);
        assert_eq!(opts.error_correction, QrErrorCorrection::High);
        assert_eq!(opts.margin, 2);
    }
}
