//! PDF encryption and decryption.
//!
//! Add password protection, encrypt, and decrypt PDF documents.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::process::Command;

/// PDF encryption strength.
#[derive(Debug, Clone, Copy, Default)]
pub enum EncryptionStrength {
    /// 40-bit RC4 (weak, maximum compatibility).
    Rc4_40,
    /// 128-bit RC4.
    #[default]
    Rc4_128,
    /// 128-bit AES.
    Aes128,
    /// 256-bit AES (strongest).
    Aes256,
}

impl EncryptionStrength {
    /// Get pdftk encryption level.
    fn pdftk_level(&self) -> &'static str {
        match self {
            EncryptionStrength::Rc4_40 => "40bit",
            EncryptionStrength::Rc4_128 => "128bit",
            EncryptionStrength::Aes128 => "128bitaes",
            EncryptionStrength::Aes256 => "256bitaes",
        }
    }

    /// Get qpdf key length.
    fn qpdf_key_length(&self) -> u32 {
        match self {
            EncryptionStrength::Rc4_40 => 40,
            EncryptionStrength::Rc4_128 => 128,
            EncryptionStrength::Aes128 => 128,
            EncryptionStrength::Aes256 => 256,
        }
    }

    /// Whether to use AES.
    fn is_aes(&self) -> bool {
        matches!(
            self,
            EncryptionStrength::Aes128 | EncryptionStrength::Aes256
        )
    }
}

/// PDF permissions.
#[derive(Debug, Clone)]
pub struct PdfPermissions {
    /// Allow printing.
    pub printing: bool,
    /// Allow high-quality printing.
    pub high_quality_print: bool,
    /// Allow modifying content.
    pub modify_contents: bool,
    /// Allow copying text.
    pub copy_contents: bool,
    /// Allow adding annotations.
    pub modify_annotations: bool,
    /// Allow filling forms.
    pub fill_forms: bool,
    /// Allow extracting content for accessibility.
    pub accessibility: bool,
    /// Allow assembling document.
    pub assemble: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            printing: true,
            high_quality_print: true,
            modify_contents: false,
            copy_contents: true,
            modify_annotations: true,
            fill_forms: true,
            accessibility: true,
            assemble: false,
        }
    }
}

impl PdfPermissions {
    /// No permissions - most restrictive.
    pub fn none() -> Self {
        Self {
            printing: false,
            high_quality_print: false,
            modify_contents: false,
            copy_contents: false,
            modify_annotations: false,
            fill_forms: false,
            accessibility: false,
            assemble: false,
        }
    }

    /// All permissions - least restrictive.
    pub fn all() -> Self {
        Self {
            printing: true,
            high_quality_print: true,
            modify_contents: true,
            copy_contents: true,
            modify_annotations: true,
            fill_forms: true,
            accessibility: true,
            assemble: true,
        }
    }

    /// View and print only.
    pub fn view_and_print() -> Self {
        Self {
            printing: true,
            high_quality_print: true,
            modify_contents: false,
            copy_contents: false,
            modify_annotations: false,
            fill_forms: false,
            accessibility: true,
            assemble: false,
        }
    }
}

/// Encryption options.
#[derive(Debug, Clone)]
pub struct EncryptOptions {
    /// User password (required to open).
    pub user_password: String,
    /// Owner password (required to change permissions).
    pub owner_password: String,
    /// Encryption strength.
    pub strength: EncryptionStrength,
    /// Permissions.
    pub permissions: PdfPermissions,
}

impl EncryptOptions {
    /// Create with same password for user and owner.
    pub fn with_password(password: &str) -> Self {
        Self {
            user_password: password.to_string(),
            owner_password: password.to_string(),
            strength: EncryptionStrength::default(),
            permissions: PdfPermissions::default(),
        }
    }

    /// Create with separate passwords.
    pub fn with_passwords(user: &str, owner: &str) -> Self {
        Self {
            user_password: user.to_string(),
            owner_password: owner.to_string(),
            strength: EncryptionStrength::default(),
            permissions: PdfPermissions::default(),
        }
    }
}

/// Encrypt a PDF with password.
///
/// # Arguments
/// * `input` - Input PDF path
/// * `output` - Output PDF path
/// * `password` - Password to set
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::pdf_encrypt;
///
/// pdf_encrypt::encrypt("doc.pdf", "encrypted.pdf", "secret123").unwrap();
/// ```
pub fn encrypt<P: AsRef<Path>>(input: P, output: P, password: &str) -> Result<ToolOutput> {
    encrypt_with_options(input, output, EncryptOptions::with_password(password))
}

/// Encrypt PDF with full options.
pub fn encrypt_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: EncryptOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    // Try pdftk first
    if let Ok(result) = encrypt_with_pdftk(input_path, output_path, &options) {
        return Ok(result);
    }

    // Try qpdf
    if let Ok(result) = encrypt_with_qpdf(input_path, output_path, &options) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "PDF encryption failed. Install pdftk or qpdf.".to_string(),
        source: None,
    })
}

/// Encrypt using pdftk.
fn encrypt_with_pdftk(input: &Path, output: &Path, options: &EncryptOptions) -> Result<ToolOutput> {
    let mut cmd = Command::new("pdftk");
    cmd.arg(input)
        .arg("output")
        .arg(output)
        .arg("encrypt_{}bit")
        .arg(options.strength.pdftk_level());

    // Set passwords
    if !options.user_password.is_empty() {
        cmd.arg("user_pw").arg(&options.user_password);
    }
    if !options.owner_password.is_empty() {
        cmd.arg("owner_pw").arg(&options.owner_password);
    }

    // Set permissions
    let mut allow = Vec::new();
    if options.permissions.printing {
        allow.push("Printing");
    }
    if options.permissions.modify_contents {
        allow.push("ModifyContents");
    }
    if options.permissions.copy_contents {
        allow.push("CopyContents");
    }
    if options.permissions.modify_annotations {
        allow.push("ModifyAnnotations");
    }
    if options.permissions.fill_forms {
        allow.push("FillIn");
    }
    if options.permissions.accessibility {
        allow.push("ScreenReaders");
    }
    if options.permissions.assemble {
        allow.push("Assembly");
    }
    if options.permissions.high_quality_print {
        allow.push("DegradedPrinting");
    }

    if !allow.is_empty() {
        cmd.arg("allow").args(&allow);
    }

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "pdftk encryption failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "PDF encrypted with pdftk",
        output,
    ))
}

/// Encrypt using qpdf.
fn encrypt_with_qpdf(input: &Path, output: &Path, options: &EncryptOptions) -> Result<ToolOutput> {
    let mut cmd = Command::new("qpdf");

    cmd.arg("--encrypt")
        .arg(&options.user_password)
        .arg(&options.owner_password)
        .arg(options.strength.qpdf_key_length().to_string());

    // Add permission flags
    if !options.permissions.printing {
        cmd.arg("--print=none");
    }
    if !options.permissions.modify_contents {
        cmd.arg("--modify=none");
    }
    if !options.permissions.copy_contents {
        cmd.arg("--extract=n");
    }
    if !options.permissions.modify_annotations {
        cmd.arg("--annotate=n");
    }

    if options.strength.is_aes() {
        cmd.arg("--use-aes=y");
    }

    cmd.arg("--").arg(input).arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run qpdf: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "qpdf encryption failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "PDF encrypted with qpdf",
        output,
    ))
}

/// Decrypt a PDF.
///
/// # Arguments
/// * `input` - Encrypted PDF path
/// * `output` - Output decrypted PDF path
/// * `password` - Password to decrypt
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::pdf_encrypt;
///
/// pdf_encrypt::decrypt("encrypted.pdf", "decrypted.pdf", "secret123").unwrap();
/// ```
pub fn decrypt<P: AsRef<Path>>(input: P, output: P, password: &str) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "Input file not found".to_string(),
            source: None,
        });
    }

    // Try qpdf first (better decryption support)
    if let Ok(result) = decrypt_with_qpdf(input_path, output_path, password) {
        return Ok(result);
    }

    // Try pdftk
    if let Ok(result) = decrypt_with_pdftk(input_path, output_path, password) {
        return Ok(result);
    }

    Err(DxError::Config {
        message: "PDF decryption failed. Wrong password or install pdftk/qpdf.".to_string(),
        source: None,
    })
}

/// Decrypt using qpdf.
fn decrypt_with_qpdf(input: &Path, output: &Path, password: &str) -> Result<ToolOutput> {
    let mut cmd = Command::new("qpdf");
    cmd.arg("--decrypt")
        .arg("--password=".to_owned() + password)
        .arg(input)
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run qpdf: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "qpdf decryption failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "PDF decrypted with qpdf",
        output,
    ))
}

/// Decrypt using pdftk.
fn decrypt_with_pdftk(input: &Path, output: &Path, password: &str) -> Result<ToolOutput> {
    let mut cmd = Command::new("pdftk");
    cmd.arg(input)
        .arg("input_pw")
        .arg(password)
        .arg("output")
        .arg(output);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run pdftk: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: format!(
                "pdftk decryption failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "PDF decrypted with pdftk",
        output,
    ))
}

/// Check if PDF is encrypted.
pub fn is_encrypted<P: AsRef<Path>>(input: P) -> Result<bool> {
    let input_path = input.as_ref();

    if !input_path.exists() {
        return Err(DxError::FileIo {
            path: input_path.to_path_buf(),
            message: "File not found".to_string(),
            source: None,
        });
    }

    // Try qpdf --show-encryption
    let mut cmd = Command::new("qpdf");
    cmd.arg("--show-encryption").arg(input_path);

    if let Ok(result) = cmd.output() {
        let output = String::from_utf8_lossy(&result.stdout);
        return Ok(!output.contains("File is not encrypted"));
    }

    // Try pdftk dump_data
    let mut cmd = Command::new("pdftk");
    cmd.arg(input_path).arg("dump_data");

    if let Ok(result) = cmd.output() {
        // If pdftk fails with password error, it's encrypted
        return Ok(!result.status.success());
    }

    // Cannot determine, assume not encrypted
    Ok(false)
}

/// Remove restrictions from PDF (keep encryption).
pub fn remove_restrictions<P: AsRef<Path>>(
    input: P,
    output: P,
    owner_password: &str,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let mut cmd = Command::new("qpdf");
    cmd.arg("--decrypt")
        .arg("--password=".to_owned() + owner_password)
        .arg(input_path)
        .arg(output_path);

    let result = cmd.output().map_err(|e| DxError::Config {
        message: format!("Failed to run qpdf: {}", e),
        source: None,
    })?;

    if !result.status.success() {
        return Err(DxError::Config {
            message: "Failed to remove restrictions".to_string(),
            source: None,
        });
    }

    Ok(ToolOutput::success_with_path(
        "PDF restrictions removed",
        output_path,
    ))
}

/// Batch encrypt multiple PDFs.
pub fn batch_encrypt<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    password: &str,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create directory: {}", e),
        source: None,
    })?;

    let mut encrypted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let output_path = output_dir.join(file_name);

        if encrypt(input_path, &output_path, password).is_ok() {
            encrypted.push(output_path);
        }
    }

    Ok(ToolOutput::success(format!("Encrypted {} PDFs", encrypted.len())).with_paths(encrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let none = PdfPermissions::none();
        assert!(!none.printing);
        assert!(!none.copy_contents);

        let all = PdfPermissions::all();
        assert!(all.printing);
        assert!(all.modify_contents);
    }

    #[test]
    fn test_encryption_strength() {
        assert!(!EncryptionStrength::Rc4_128.is_aes());
        assert!(EncryptionStrength::Aes256.is_aes());
    }
}
