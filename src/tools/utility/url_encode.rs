//! URL encoding and decoding.
//!
//! Encode and decode URLs and query strings.

use crate::error::Result;
use crate::tools::ToolOutput;

/// URL encode a string.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::url_encode;
///
/// let result = url_encode::encode("Hello World!").unwrap();
/// println!("{}", result.message); // Hello%20World%21
/// ```
pub fn encode(input: &str) -> Result<ToolOutput> {
    let encoded = url_encode_impl(input);
    Ok(ToolOutput::success(encoded.clone())
        .with_metadata("encoded", encoded)
        .with_metadata("original_length", input.len().to_string()))
}

/// URL decode a string.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::url_encode;
///
/// let result = url_encode::decode("Hello%20World%21").unwrap();
/// println!("{}", result.message); // Hello World!
/// ```
pub fn decode(input: &str) -> Result<ToolOutput> {
    let decoded = url_decode_impl(input);
    Ok(ToolOutput::success(decoded.clone()).with_metadata("decoded", decoded))
}

/// URL encode implementation.
fn url_encode_impl(input: &str) -> String {
    let mut result = String::new();

    for byte in input.bytes() {
        match byte {
            // Unreserved characters (RFC 3986)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            // Space -> +
            b' ' => {
                result.push('+');
            }
            // Everything else -> %XX
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }

    result
}

/// URL decode implementation.
fn url_decode_impl(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                // Parse hex value
                let hex = &input[i + 1..i + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    result.push(byte);
                    i += 3;
                } else {
                    result.push(bytes[i]);
                    i += 1;
                }
            }
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            byte => {
                result.push(byte);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Encode path component (stricter encoding).
pub fn encode_path(input: &str) -> Result<ToolOutput> {
    let mut result = String::new();

    for byte in input.bytes() {
        match byte {
            // Path-safe characters
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }

    Ok(ToolOutput::success(result.clone())
        .with_metadata("encoded", result)
        .with_metadata("type", "path".to_string()))
}

/// Encode query string value.
pub fn encode_query_value(input: &str) -> Result<ToolOutput> {
    encode(input).map(|mut o| {
        o.metadata
            .insert("type".to_string(), "query-value".to_string());
        o
    })
}

/// Parse query string into key-value pairs.
pub fn parse_query_string(input: &str) -> Result<ToolOutput> {
    let input = input.trim_start_matches('?');
    let mut pairs = Vec::new();

    for part in input.split('&') {
        if part.is_empty() {
            continue;
        }

        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let value = kv.next().unwrap_or("");

        let decoded_key = url_decode_impl(key);
        let decoded_value = url_decode_impl(value);

        pairs.push(format!("{} = {}", decoded_key, decoded_value));
    }

    Ok(ToolOutput::success(pairs.join("\n")).with_metadata("param_count", pairs.len().to_string()))
}

/// Build query string from key-value pairs.
pub fn build_query_string(params: &[(&str, &str)]) -> Result<ToolOutput> {
    let encoded: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode_impl(k), url_encode_impl(v)))
        .collect();

    let query = encoded.join("&");

    Ok(ToolOutput::success(query.clone())
        .with_metadata("query", query)
        .with_metadata("param_count", params.len().to_string()))
}

/// Encode component (stricter than full URL encoding).
pub fn encode_component(input: &str) -> Result<ToolOutput> {
    let mut result = String::new();

    for byte in input.bytes() {
        match byte {
            // Only alphanumeric and a few safe chars
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }

    Ok(ToolOutput::success(result.clone())
        .with_metadata("encoded", result)
        .with_metadata("type", "component".to_string()))
}

/// Decode and re-encode (normalize URL encoding).
pub fn normalize(input: &str) -> Result<ToolOutput> {
    let decoded = url_decode_impl(input);
    let encoded = url_encode_impl(&decoded);

    Ok(ToolOutput::success(encoded.clone()).with_metadata("normalized", encoded))
}

/// Check if string needs encoding.
pub fn needs_encoding(input: &str) -> Result<ToolOutput> {
    let needs = input.bytes().any(|b| {
        !matches!(
            b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
        )
    });

    Ok(ToolOutput::success(
        if needs {
            "String needs URL encoding"
        } else {
            "String is URL-safe"
        }
        .to_string(),
    )
    .with_metadata("needs_encoding", needs.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let input = "Hello World!";
        let encoded = url_encode_impl(input);
        assert_eq!(encoded, "Hello+World%21");

        let decoded = url_decode_impl(&encoded);
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_special_chars() {
        let input = "a=1&b=2";
        let encoded = url_encode_impl(input);
        assert!(encoded.contains("%3D"));
        assert!(encoded.contains("%26"));
    }
}
