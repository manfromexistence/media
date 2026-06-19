//! Random data generation utilities.
//!
//! Generate random strings, numbers, and data.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Character set for random string generation.
#[derive(Debug, Clone, Copy, Default)]
pub enum CharSet {
    /// Alphanumeric (a-z, A-Z, 0-9).
    #[default]
    Alphanumeric,
    /// Alphabetic (a-z, A-Z).
    Alphabetic,
    /// Lowercase (a-z).
    Lowercase,
    /// Uppercase (A-Z).
    Uppercase,
    /// Numeric (0-9).
    Numeric,
    /// Hexadecimal (0-9, a-f).
    Hex,
    /// All printable ASCII.
    Ascii,
}

/// Simple pseudo-random number generator state.
struct Rng {
    state: u64,
}

impl Rng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn with_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // PCG algorithm
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let xorshifted = (((self.state >> 18) ^ self.state) >> 27) as u32;
        let rot = (self.state >> 59) as u32;
        ((xorshifted >> rot) | (xorshifted << ((-(rot as i32)) & 31))) as u64
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min))
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

/// Generate a random string.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::random::{string, CharSet};
///
/// let s = string(16, CharSet::Alphanumeric).unwrap();
/// ```
pub fn string(length: usize, charset: CharSet) -> Result<ToolOutput> {
    let chars = match charset {
        CharSet::Alphanumeric => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        CharSet::Alphabetic => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        CharSet::Lowercase => "abcdefghijklmnopqrstuvwxyz",
        CharSet::Uppercase => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        CharSet::Numeric => "0123456789",
        CharSet::Hex => "0123456789abcdef",
        CharSet::Ascii => {
            "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
        }
    };

    let chars: Vec<char> = chars.chars().collect();
    let mut rng = Rng::new();

    let result: String = (0..length)
        .map(|_| {
            let idx = rng.next_range(0, chars.len() as u64) as usize;
            chars[idx]
        })
        .collect();

    Ok(ToolOutput::success(result.clone())
        .with_metadata("length", length.to_string())
        .with_metadata("charset", format!("{:?}", charset)))
}

/// Generate a random integer.
pub fn integer(min: i64, max: i64) -> Result<ToolOutput> {
    if min > max {
        return Err(DxError::Config {
            message: "min must be <= max".to_string(),
            source: None,
        });
    }

    let mut rng = Rng::new();
    let range = (max - min) as u64 + 1;
    let value = min + (rng.next_u64() % range) as i64;

    Ok(ToolOutput::success(value.to_string())
        .with_metadata("min", min.to_string())
        .with_metadata("max", max.to_string()))
}

/// Generate a random float.
pub fn float(min: f64, max: f64) -> Result<ToolOutput> {
    if min > max {
        return Err(DxError::Config {
            message: "min must be <= max".to_string(),
            source: None,
        });
    }

    let mut rng = Rng::new();
    let value = min + rng.next_f64() * (max - min);

    Ok(ToolOutput::success(format!("{:.6}", value))
        .with_metadata("min", min.to_string())
        .with_metadata("max", max.to_string()))
}

/// Generate random bytes.
pub fn bytes(count: usize) -> Result<ToolOutput> {
    let mut rng = Rng::new();
    let data: Vec<u8> = (0..count).map(|_| rng.next_u32() as u8).collect();

    // Return as hex string
    let hex: String = data.iter().map(|b| format!("{:02x}", b)).collect();

    Ok(ToolOutput::success(hex).with_metadata("byte_count", count.to_string()))
}

/// Generate random bytes and save to file.
pub fn bytes_to_file<P: AsRef<Path>>(count: usize, output: P) -> Result<ToolOutput> {
    let output_path = output.as_ref();
    let mut rng = Rng::new();
    let data: Vec<u8> = (0..count).map(|_| rng.next_u32() as u8).collect();

    std::fs::write(output_path, &data).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        format!("Generated {} random bytes", count),
        output_path,
    ))
}

/// Generate a random boolean.
pub fn boolean() -> Result<ToolOutput> {
    let mut rng = Rng::new();
    let value = rng.next_u32() % 2 == 0;

    Ok(ToolOutput::success(value.to_string()).with_metadata("value", value.to_string()))
}

/// Pick a random item from a list.
pub fn pick(items: &[&str]) -> Result<ToolOutput> {
    if items.is_empty() {
        return Err(DxError::Config {
            message: "Cannot pick from empty list".to_string(),
            source: None,
        });
    }

    let mut rng = Rng::new();
    let idx = rng.next_range(0, items.len() as u64) as usize;
    let item = items[idx];

    Ok(ToolOutput::success(item.to_string())
        .with_metadata("index", idx.to_string())
        .with_metadata("total_items", items.len().to_string()))
}

/// Shuffle a list.
pub fn shuffle(items: &[&str]) -> Result<ToolOutput> {
    let mut rng = Rng::new();
    let mut result: Vec<&str> = items.to_vec();

    // Fisher-Yates shuffle
    for i in (1..result.len()).rev() {
        let j = rng.next_range(0, (i + 1) as u64) as usize;
        result.swap(i, j);
    }

    Ok(ToolOutput::success(result.join("\n")).with_metadata("item_count", items.len().to_string()))
}

/// Generate a random password.
pub fn password(length: usize, include_symbols: bool) -> Result<ToolOutput> {
    let base = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let symbols = "!@#$%^&*()_+-=[]{}|;:,.<>?";

    let chars: Vec<char> = if include_symbols {
        format!("{}{}", base, symbols).chars().collect()
    } else {
        base.chars().collect()
    };

    let mut rng = Rng::new();

    let result: String = (0..length)
        .map(|_| {
            let idx = rng.next_range(0, chars.len() as u64) as usize;
            chars[idx]
        })
        .collect();

    Ok(ToolOutput::success(result)
        .with_metadata("length", length.to_string())
        .with_metadata("includes_symbols", include_symbols.to_string()))
}

/// Generate multiple random values.
pub fn batch_integers(count: usize, min: i64, max: i64) -> Result<ToolOutput> {
    if min > max {
        return Err(DxError::Config {
            message: "min must be <= max".to_string(),
            source: None,
        });
    }

    let mut rng = Rng::new();
    let range = (max - min) as u64 + 1;

    let values: Vec<String> = (0..count)
        .map(|_| {
            let value = min + (rng.next_u64() % range) as i64;
            value.to_string()
        })
        .collect();

    Ok(ToolOutput::success(values.join("\n"))
        .with_metadata("count", count.to_string())
        .with_metadata("min", min.to_string())
        .with_metadata("max", max.to_string()))
}

/// Generate a random color.
pub fn color(format: &str) -> Result<ToolOutput> {
    let mut rng = Rng::new();
    let r = rng.next_u32() as u8;
    let g = rng.next_u32() as u8;
    let b = rng.next_u32() as u8;

    let output = match format.to_lowercase().as_str() {
        "hex" => format!("#{:02x}{:02x}{:02x}", r, g, b),
        "rgb" => format!("rgb({}, {}, {})", r, g, b),
        "hsl" => {
            // Convert to HSL
            let (h, s, l) = rgb_to_hsl(r, g, b);
            format!("hsl({}, {}%, {}%)", h, s, l)
        }
        _ => format!("#{:02x}{:02x}{:02x}", r, g, b),
    };

    Ok(ToolOutput::success(output)
        .with_metadata("r", r.to_string())
        .with_metadata("g", g.to_string())
        .with_metadata("b", b.to_string()))
}

/// Convert RGB to HSL.
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (u32, u32, u32) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = f64::midpoint(max, min);

    if max == min {
        return (0, 0, (l * 100.0) as u32);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    ((h * 360.0) as u32, (s * 100.0) as u32, (l * 100.0) as u32)
}

/// Generate random data with seed for reproducibility.
pub fn string_with_seed(length: usize, charset: CharSet, seed: u64) -> Result<ToolOutput> {
    let chars = match charset {
        CharSet::Alphanumeric => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        CharSet::Alphabetic => "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        CharSet::Lowercase => "abcdefghijklmnopqrstuvwxyz",
        CharSet::Uppercase => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        CharSet::Numeric => "0123456789",
        CharSet::Hex => "0123456789abcdef",
        CharSet::Ascii => {
            "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
        }
    };

    let chars: Vec<char> = chars.chars().collect();
    let mut rng = Rng::with_seed(seed);

    let result: String = (0..length)
        .map(|_| {
            let idx = rng.next_range(0, chars.len() as u64) as usize;
            chars[idx]
        })
        .collect();

    Ok(ToolOutput::success(result)
        .with_metadata("length", length.to_string())
        .with_metadata("seed", seed.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let result = string(16, CharSet::Alphanumeric).unwrap();
        assert!(result.success);
        assert_eq!(result.message.len(), 16);
    }

    #[test]
    fn test_integer() {
        let result = integer(1, 10).unwrap();
        let value: i64 = result.message.parse().unwrap();
        assert!(value >= 1 && value <= 10);
    }

    #[test]
    fn test_seeded_reproducibility() {
        let r1 = string_with_seed(10, CharSet::Alphanumeric, 12345).unwrap();
        let r2 = string_with_seed(10, CharSet::Alphanumeric, 12345).unwrap();
        assert_eq!(r1.message, r2.message);
    }
}
