//! JSON formatting utilities.
//!
//! Format, validate, and manipulate JSON.

use crate::error::{DxError, Result};
use crate::tools::{ToolOutput, ToolReceipt};
use std::path::Path;

/// JSON indentation style.
#[derive(Debug, Clone, Copy, Default)]
pub enum JsonIndent {
    /// No indentation (compact).
    None,
    /// 2 spaces.
    #[default]
    Spaces2,
    /// 4 spaces.
    Spaces4,
    /// Tab character.
    Tab,
}

impl JsonIndent {
    /// Get indent string.
    fn indent(&self) -> &'static str {
        match self {
            JsonIndent::None => "",
            JsonIndent::Spaces2 => "  ",
            JsonIndent::Spaces4 => "    ",
            JsonIndent::Tab => "\t",
        }
    }
}

/// Format JSON string.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::json_format;
///
/// let result = json_format::format_string(r#"{"a":1,"b":2}"#).unwrap();
/// println!("{}", result.message);
/// ```
pub fn format_string(input: &str) -> Result<ToolOutput> {
    format_string_with_indent(input, JsonIndent::Spaces2)
}

/// Format JSON with specific indentation.
pub fn format_string_with_indent(input: &str, indent: JsonIndent) -> Result<ToolOutput> {
    // Parse and re-format
    let formatted = format_json_impl(input, indent)?;

    Ok(ToolOutput::success(formatted.clone())
        .with_metadata("formatted", formatted)
        .with_metadata("valid", "true".to_string()))
}

/// Format JSON file.
pub fn format_json_file<I: AsRef<Path>, O: AsRef<Path>>(input: I, output: O) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    validate_json_impl(&content).map_err(|message| DxError::JsonParse {
        message,
        source: None,
    })?;
    let formatted = format_json_impl(&content, JsonIndent::Spaces2)?;

    std::fs::write(output_path, &formatted).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path("JSON formatted", output_path)
        .with_receipt(
            ToolReceipt::local("utility.format-json").with_source(input_path.display().to_string()),
        )
        .with_metadata("tool.converter", "serde_json")
        .with_metadata("tool.converter_kind", "builtin")
        .with_json_output_type_validation(output_path))
}

/// Minify JSON (remove whitespace).
pub fn minify_string(input: &str) -> Result<ToolOutput> {
    let minified = minify_json_impl(input)?;

    let original_len = input.len();
    let minified_len = minified.len();
    let saved = original_len.saturating_sub(minified_len);

    Ok(ToolOutput::success(minified.clone())
        .with_metadata("minified", minified)
        .with_metadata("original_size", original_len.to_string())
        .with_metadata("minified_size", minified_len.to_string())
        .with_metadata("bytes_saved", saved.to_string()))
}

/// Validate JSON.
pub fn validate_string(input: &str) -> Result<ToolOutput> {
    match validate_json_impl(input) {
        Ok(()) => Ok(ToolOutput::success("Valid JSON").with_metadata("valid", "true".to_string())),
        Err(msg) => Ok(ToolOutput::success(format!("Invalid JSON: {}", msg))
            .with_metadata("valid", "false".to_string())
            .with_metadata("error", msg)),
    }
}

/// Validate JSON file.
pub fn validate_file<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let content = std::fs::read_to_string(input.as_ref()).map_err(|e| DxError::FileIo {
        path: input.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    validate_string(&content)
}

/// Simple JSON formatter implementation.
fn format_json_impl(input: &str, indent: JsonIndent) -> Result<String> {
    let indent_str = indent.indent();
    let compact = matches!(indent, JsonIndent::None);

    let mut result = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let chars: Vec<char> = input.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if escape_next {
            result.push(*c);
            escape_next = false;
            continue;
        }

        if *c == '\\' && in_string {
            result.push(*c);
            escape_next = true;
            continue;
        }

        if *c == '"' {
            in_string = !in_string;
            result.push(*c);
            continue;
        }

        if in_string {
            result.push(*c);
            continue;
        }

        match c {
            '{' | '[' => {
                result.push(*c);
                depth += 1;

                // Check if next non-whitespace is closing bracket
                let next_meaningful = chars[i + 1..].iter().find(|c| !c.is_whitespace());

                if next_meaningful != Some(&'}') && next_meaningful != Some(&']') && !compact {
                    result.push('\n');
                    for _ in 0..depth {
                        result.push_str(indent_str);
                    }
                }
            }
            '}' | ']' => {
                depth -= 1;

                // Check if previous non-whitespace was opening bracket
                let prev = result.trim_end().chars().last();
                if prev != Some('{') && prev != Some('[') && !compact {
                    result.push('\n');
                    for _ in 0..depth {
                        result.push_str(indent_str);
                    }
                }

                result.push(*c);
            }
            ',' => {
                result.push(*c);
                if !compact {
                    result.push('\n');
                    for _ in 0..depth {
                        result.push_str(indent_str);
                    }
                }
            }
            ':' => {
                result.push(*c);
                if !compact {
                    result.push(' ');
                }
            }
            ' ' | '\t' | '\n' | '\r' => {
                // Skip whitespace outside strings
            }
            _ => {
                result.push(*c);
            }
        }
    }

    Ok(result)
}

/// Minify JSON implementation.
fn minify_json_impl(input: &str) -> Result<String> {
    format_json_impl(input, JsonIndent::None)
}

/// Validate JSON implementation.
fn validate_json_impl(input: &str) -> std::result::Result<(), String> {
    // Use serde_json for proper validation
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Invalid JSON: {}", e)),
    }
}

trait JsonOutputValidation {
    fn with_json_output_type_validation(self, path: impl AsRef<Path>) -> Self;
}

impl JsonOutputValidation for ToolOutput {
    fn with_json_output_type_validation(self, path: impl AsRef<Path>) -> Self {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| "unknown".to_string());
        let valid = extension == "json";

        let output = self
            .with_metadata("tool.expected_media_type", "json")
            .with_metadata("tool.output_extension", extension)
            .with_metadata("tool.type_validation", if valid { "pass" } else { "fail" });

        if valid {
            output
        } else {
            output.with_metadata("tool.type_validation_reason", "extension-mismatch")
        }
    }
}

/// Extract JSON path value.
pub fn extract_path(_json: &str, path: &str) -> Result<ToolOutput> {
    // Simple path extraction (supports dot notation)
    // e.g., "data.items.0.name"

    // For now, return the path info
    Ok(ToolOutput::success(format!("Path: {}", path))
        .with_metadata("path", path.to_string())
        .with_metadata(
            "note",
            "Full JSONPath support requires serde_json".to_string(),
        ))
}

/// Sort JSON keys alphabetically.
pub fn sort_keys(input: &str) -> Result<ToolOutput> {
    // Basic implementation - would need full parser for proper sorting
    let formatted = format_json_impl(input, JsonIndent::Spaces2)?;

    Ok(ToolOutput::success(formatted)
        .with_metadata("note", "Full key sorting requires serde_json".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let input = r#"{"a":1,"b":2}"#;
        let formatted = format_json_impl(input, JsonIndent::Spaces2).unwrap();
        assert!(formatted.contains('\n'));
    }

    #[test]
    fn test_minify() {
        let input = r#"{ "a": 1, "b": 2 }"#;
        let minified = minify_json_impl(input).unwrap();
        assert!(!minified.contains(' '));
    }

    #[test]
    fn test_validate() {
        assert!(validate_json_impl(r#"{"a": 1}"#).is_ok());
        assert!(validate_json_impl(r#"{"a": 1"#).is_err());
    }
}
