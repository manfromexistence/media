//! UUID generation utilities.
//!
//! Generate various types of UUIDs.

use crate::error::Result;
use crate::tools::ToolOutput;
use std::time::{SystemTime, UNIX_EPOCH};

/// UUID version.
#[derive(Debug, Clone, Copy, Default)]
pub enum UuidVersion {
    /// Version 4 (random).
    #[default]
    V4,
    /// Version 1 (time-based, simulated).
    V1,
    /// Nil UUID (all zeros).
    Nil,
}

/// Generate a UUID.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::uuid::{generate, UuidVersion};
///
/// let uuid = generate(UuidVersion::V4).unwrap();
/// ```
pub fn generate(version: UuidVersion) -> Result<ToolOutput> {
    let uuid = match version {
        UuidVersion::V4 => generate_v4(),
        UuidVersion::V1 => generate_v1(),
        UuidVersion::Nil => "00000000-0000-0000-0000-000000000000".to_string(),
    };

    Ok(ToolOutput::success(uuid.clone())
        .with_metadata("version", format!("{:?}", version))
        .with_metadata("uuid", uuid))
}

/// Generate a random UUID (v4).
pub fn generate_v4() -> String {
    let mut bytes = [0u8; 16];

    // Use simple random generation based on time and counter
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let seed = now.as_nanos() as u64;
    let mut state = seed;

    for byte in &mut bytes {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *byte = (state >> 33) as u8;
    }

    // Set version 4 and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

    format_uuid(&bytes)
}

/// Generate a time-based UUID (v1-like).
fn generate_v1() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let timestamp = now.as_nanos() as u64;
    let clock_seq = (now.subsec_nanos() & 0x3fff) as u16 | 0x8000;

    let time_low = (timestamp & 0xffffffff) as u32;
    let time_mid = ((timestamp >> 32) & 0xffff) as u16;
    let time_hi_version = ((timestamp >> 48) & 0x0fff) as u16 | 0x1000; // Version 1

    // Generate pseudo-random node
    let node: [u8; 6] = [
        (timestamp & 0xff) as u8,
        ((timestamp >> 8) & 0xff) as u8,
        ((timestamp >> 16) & 0xff) as u8,
        ((timestamp >> 24) & 0xff) as u8,
        ((timestamp >> 32) & 0xff) as u8,
        ((timestamp >> 40) & 0xff) as u8 | 0x01, // Multicast bit
    ];

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        time_low,
        time_mid,
        time_hi_version,
        clock_seq,
        node[0],
        node[1],
        node[2],
        node[3],
        node[4],
        node[5]
    )
}

/// Format bytes as UUID string.
fn format_uuid(bytes: &[u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

/// Generate multiple UUIDs.
pub fn generate_batch(count: usize, version: UuidVersion) -> Result<ToolOutput> {
    let uuids: Vec<String> = (0..count)
        .map(|_| match version {
            UuidVersion::V4 => generate_v4(),
            UuidVersion::V1 => generate_v1(),
            UuidVersion::Nil => "00000000-0000-0000-0000-000000000000".to_string(),
        })
        .collect();

    let output = uuids.join("\n");

    Ok(ToolOutput::success(output)
        .with_metadata("count", count.to_string())
        .with_metadata("version", format!("{:?}", version)))
}

/// Validate a UUID string.
pub fn validate(uuid: &str) -> Result<ToolOutput> {
    let uuid = uuid.trim();

    // Check format: 8-4-4-4-12
    let parts: Vec<&str> = uuid.split('-').collect();

    if parts.len() != 5 {
        return Ok(ToolOutput::failure("Invalid UUID format: expected 5 parts"));
    }

    let expected_lengths = [8, 4, 4, 4, 12];
    for (part, &expected) in parts.iter().zip(expected_lengths.iter()) {
        if part.len() != expected {
            return Ok(ToolOutput::failure(format!(
                "Invalid UUID format: part has {} chars, expected {}",
                part.len(),
                expected
            )));
        }

        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(ToolOutput::failure(
                "Invalid UUID format: contains non-hex characters",
            ));
        }
    }

    // Extract version
    let version_char = parts[2].chars().next().unwrap_or('0');
    let version = match version_char {
        '1' => "1 (time-based)",
        '2' => "2 (DCE security)",
        '3' => "3 (MD5 hash)",
        '4' => "4 (random)",
        '5' => "5 (SHA-1 hash)",
        _ => "unknown",
    };

    Ok(
        ToolOutput::success(format!("Valid UUID (version {})", version))
            .with_metadata("valid", "true")
            .with_metadata("version", version.to_string()),
    )
}

/// Parse UUID to bytes.
pub fn parse(uuid: &str) -> Result<ToolOutput> {
    let uuid = uuid.trim().replace('-', "");

    if uuid.len() != 32 {
        return Ok(ToolOutput::failure("Invalid UUID length"));
    }

    let bytes: Vec<String> = (0..16)
        .map(|i| uuid[i * 2..i * 2 + 2].to_string())
        .collect();

    Ok(
        ToolOutput::success(format!("Bytes: [{}]", bytes.join(", ")))
            .with_metadata("hex", bytes.join("")),
    )
}

/// Convert UUID to different formats.
pub fn format_as(uuid: &str, format: &str) -> Result<ToolOutput> {
    let uuid = uuid.trim().replace('-', "").to_lowercase();

    if uuid.len() != 32 {
        return Ok(ToolOutput::failure("Invalid UUID"));
    }

    let output = match format {
        "standard" | "canonical" => format!(
            "{}-{}-{}-{}-{}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        ),
        "uppercase" => format!(
            "{}-{}-{}-{}-{}",
            &uuid[0..8].to_uppercase(),
            &uuid[8..12].to_uppercase(),
            &uuid[12..16].to_uppercase(),
            &uuid[16..20].to_uppercase(),
            &uuid[20..32].to_uppercase()
        ),
        "urn" => format!(
            "urn:uuid:{}-{}-{}-{}-{}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        ),
        "compact" => uuid.clone(),
        "braces" => format!(
            "{{{}-{}-{}-{}-{}}}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        ),
        _ => return Ok(ToolOutput::failure(format!("Unknown format: {}", format))),
    };

    Ok(ToolOutput::success(output))
}

/// Generate namespace UUID (simulated v5).
pub fn generate_namespace(namespace: &str, name: &str) -> Result<ToolOutput> {
    // Simple hash-based generation (not cryptographically secure)
    let input = format!("{}{}", namespace, name);
    let hash = simple_hash(&input);

    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = ((hash >> (i * 4)) & 0xff) as u8;
    }

    // Set version 5 and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x50; // Version 5
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

    let uuid = format_uuid(&bytes);

    Ok(ToolOutput::success(uuid.clone())
        .with_metadata("namespace", namespace.to_string())
        .with_metadata("name", name.to_string()))
}

/// Simple non-cryptographic hash.
fn simple_hash(s: &str) -> u128 {
    let mut hash: u128 = 0;
    for (i, byte) in s.bytes().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u128);
        hash = hash.rotate_left((i % 7) as u32 + 1);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_v4() {
        let uuid = generate_v4();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(14).unwrap(), '4'); // Version 4
    }

    #[test]
    fn test_validate() {
        let result = validate("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_invalid_uuid() {
        let result = validate("not-a-uuid").unwrap();
        assert!(!result.success);
    }
}
