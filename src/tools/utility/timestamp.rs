//! Timestamp utilities.
//!
//! Convert and manipulate timestamps.

use crate::error::Result;
use crate::tools::ToolOutput;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Timestamp format.
#[derive(Debug, Clone, Copy, Default)]
pub enum TimestampFormat {
    /// Unix timestamp (seconds since epoch).
    #[default]
    Unix,
    /// Unix timestamp in milliseconds.
    UnixMillis,
    /// Unix timestamp in nanoseconds.
    UnixNanos,
    /// ISO 8601 format.
    Iso8601,
    /// RFC 2822 format.
    Rfc2822,
}

/// Get current timestamp.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::timestamp::{now, TimestampFormat};
///
/// let ts = now(TimestampFormat::Unix).unwrap();
/// ```
pub fn now(format: TimestampFormat) -> Result<ToolOutput> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let output = format_duration(now, format);

    Ok(ToolOutput::success(output.clone())
        .with_metadata("format", format!("{:?}", format))
        .with_metadata("timestamp", output))
}

/// Convert timestamp to different format.
pub fn convert(timestamp: &str, from: TimestampFormat, to: TimestampFormat) -> Result<ToolOutput> {
    let duration = parse_timestamp(timestamp, from)?;
    let output = format_duration(duration, to);

    Ok(ToolOutput::success(output.clone())
        .with_metadata("from", format!("{:?}", from))
        .with_metadata("to", format!("{:?}", to)))
}

/// Parse timestamp string.
fn parse_timestamp(s: &str, format: TimestampFormat) -> Result<Duration> {
    let s = s.trim();

    match format {
        TimestampFormat::Unix => {
            let secs: u64 = s.parse().map_err(|_| crate::error::DxError::Config {
                message: format!("Invalid unix timestamp: {}", s),
                source: None,
            })?;
            Ok(Duration::from_secs(secs))
        }
        TimestampFormat::UnixMillis => {
            let millis: u64 = s.parse().map_err(|_| crate::error::DxError::Config {
                message: format!("Invalid millisecond timestamp: {}", s),
                source: None,
            })?;
            Ok(Duration::from_millis(millis))
        }
        TimestampFormat::UnixNanos => {
            let nanos: u64 = s.parse().map_err(|_| crate::error::DxError::Config {
                message: format!("Invalid nanosecond timestamp: {}", s),
                source: None,
            })?;
            Ok(Duration::from_nanos(nanos))
        }
        TimestampFormat::Iso8601 => {
            // Parse ISO 8601: YYYY-MM-DDTHH:MM:SSZ
            parse_iso8601(s)
        }
        TimestampFormat::Rfc2822 => {
            // Basic RFC 2822 parsing
            parse_rfc2822(s)
        }
    }
}

/// Format duration as string.
fn format_duration(duration: Duration, format: TimestampFormat) -> String {
    match format {
        TimestampFormat::Unix => duration.as_secs().to_string(),
        TimestampFormat::UnixMillis => duration.as_millis().to_string(),
        TimestampFormat::UnixNanos => duration.as_nanos().to_string(),
        TimestampFormat::Iso8601 => format_iso8601(duration),
        TimestampFormat::Rfc2822 => format_rfc2822(duration),
    }
}

/// Parse ISO 8601 timestamp.
fn parse_iso8601(s: &str) -> Result<Duration> {
    // Format: YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS+00:00
    let s = s.trim().trim_end_matches('Z');

    // Handle timezone offset
    let (datetime, _offset) =
        if let Some(pos) = s.rfind('+').or_else(|| s.rfind('-').filter(|&p| p > 10)) {
            (&s[..pos], &s[pos..])
        } else {
            (s, "+00:00")
        };

    let parts: Vec<&str> = datetime.split('T').collect();
    if parts.len() != 2 {
        return Err(crate::error::DxError::Config {
            message: format!("Invalid ISO 8601 format: {}", s),
            source: None,
        });
    }

    let date_parts: Vec<u32> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();

    let time_str = parts[1].split('.').next().unwrap_or(parts[1]);
    let time_parts: Vec<u32> = time_str.split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return Err(crate::error::DxError::Config {
            message: format!("Invalid ISO 8601 format: {}", s),
            source: None,
        });
    }

    let year = date_parts[0];
    let month = date_parts[1];
    let day = date_parts[2];
    let hour = time_parts[0];
    let minute = time_parts[1];
    let second = *time_parts.get(2).unwrap_or(&0);

    let secs = datetime_to_unix(year, month, day, hour, minute, second);
    Ok(Duration::from_secs(secs))
}

/// Parse RFC 2822 timestamp (basic).
fn parse_rfc2822(s: &str) -> Result<Duration> {
    // Format: Day, DD Mon YYYY HH:MM:SS +0000
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.len() < 5 {
        return Err(crate::error::DxError::Config {
            message: format!("Invalid RFC 2822 format: {}", s),
            source: None,
        });
    }

    let day: u32 = parts[1].trim_matches(',').parse().unwrap_or(1);
    let month = month_from_abbrev(parts[2]).unwrap_or(1);
    let year: u32 = parts[3].parse().unwrap_or(1970);

    let time_parts: Vec<u32> = parts[4].split(':').filter_map(|p| p.parse().ok()).collect();

    let hour = *time_parts.first().unwrap_or(&0);
    let minute = *time_parts.get(1).unwrap_or(&0);
    let second = *time_parts.get(2).unwrap_or(&0);

    let secs = datetime_to_unix(year, month, day, hour, minute, second);
    Ok(Duration::from_secs(secs))
}

/// Convert datetime to Unix timestamp.
fn datetime_to_unix(year: u32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> u64 {
    // Days in each month (non-leap year)
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let is_leap = |y: u32| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);

    let mut days: u64 = 0;

    // Years since 1970
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }

    // Months
    for m in 1..month {
        days += days_in_month[m as usize] as u64;
        if m == 2 && is_leap(year) {
            days += 1;
        }
    }

    // Days
    days += (day - 1) as u64;

    // Convert to seconds
    days * 86400 + (hour as u64) * 3600 + (minute as u64) * 60 + second as u64
}

/// Format as ISO 8601.
fn format_iso8601(duration: Duration) -> String {
    let secs = duration.as_secs();
    let (year, month, day, hour, minute, second) = unix_to_datetime(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Format as RFC 2822.
fn format_rfc2822(duration: Duration) -> String {
    let secs = duration.as_secs();
    let (year, month, day, hour, minute, second) = unix_to_datetime(secs);

    let day_of_week = ((secs / 86400 + 4) % 7) as usize; // Jan 1, 1970 was Thursday (4)
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    format!(
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} +0000",
        days[day_of_week], day, months[month as usize], year, hour, minute, second
    )
}

/// Convert Unix timestamp to datetime.
fn unix_to_datetime(secs: u64) -> (u32, u32, u32, u32, u32, u32) {
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = |y: u32| y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);

    let mut remaining = secs;
    let second = (remaining % 60) as u32;
    remaining /= 60;
    let minute = (remaining % 60) as u32;
    remaining /= 60;
    let hour = (remaining % 24) as u32;
    let mut days = (remaining / 24) as u32;

    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let mut month = 1u32;
    loop {
        let dim = if month == 2 && is_leap(year) {
            29
        } else {
            days_in_month[month as usize]
        };
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }

    let day = days + 1;

    (year, month, day, hour, minute, second)
}

/// Convert month abbreviation to number.
fn month_from_abbrev(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

/// Add duration to timestamp.
pub fn add(timestamp: &str, seconds: i64, format: TimestampFormat) -> Result<ToolOutput> {
    let duration = parse_timestamp(timestamp, format)?;

    let new_duration = if seconds >= 0 {
        duration + Duration::from_secs(seconds as u64)
    } else {
        duration.saturating_sub(Duration::from_secs((-seconds) as u64))
    };

    let output = format_duration(new_duration, format);

    Ok(ToolOutput::success(output.clone())
        .with_metadata("original", timestamp.to_string())
        .with_metadata("added_seconds", seconds.to_string()))
}

/// Get difference between two timestamps.
pub fn diff(ts1: &str, ts2: &str, format: TimestampFormat) -> Result<ToolOutput> {
    let d1 = parse_timestamp(ts1, format)?;
    let d2 = parse_timestamp(ts2, format)?;

    let diff_secs = if d1 > d2 {
        (d1 - d2).as_secs()
    } else {
        (d2 - d1).as_secs()
    };

    let days = diff_secs / 86400;
    let hours = (diff_secs % 86400) / 3600;
    let minutes = (diff_secs % 3600) / 60;
    let seconds = diff_secs % 60;

    Ok(ToolOutput::success(format!(
        "{}d {}h {}m {}s ({} seconds total)",
        days, hours, minutes, seconds, diff_secs
    ))
    .with_metadata("days", days.to_string())
    .with_metadata("hours", hours.to_string())
    .with_metadata("minutes", minutes.to_string())
    .with_metadata("seconds", seconds.to_string())
    .with_metadata("total_seconds", diff_secs.to_string()))
}

/// Get human-readable relative time.
pub fn relative(timestamp: &str, format: TimestampFormat) -> Result<ToolOutput> {
    let then = parse_timestamp(timestamp, format)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let (diff_secs, direction) = if now > then {
        ((now - then).as_secs(), "ago")
    } else {
        ((then - now).as_secs(), "from now")
    };

    let output = if diff_secs < 60 {
        format!("{} seconds {}", diff_secs, direction)
    } else if diff_secs < 3600 {
        format!("{} minutes {}", diff_secs / 60, direction)
    } else if diff_secs < 86400 {
        format!("{} hours {}", diff_secs / 3600, direction)
    } else if diff_secs < 2592000 {
        format!("{} days {}", diff_secs / 86400, direction)
    } else if diff_secs < 31536000 {
        format!("{} months {}", diff_secs / 2592000, direction)
    } else {
        format!("{} years {}", diff_secs / 31536000, direction)
    };

    Ok(ToolOutput::success(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let result = now(TimestampFormat::Unix).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_convert() {
        let result = convert("1000", TimestampFormat::Unix, TimestampFormat::UnixMillis).unwrap();
        assert_eq!(result.message, "1000000");
    }

    #[test]
    fn test_iso8601() {
        let result = convert("0", TimestampFormat::Unix, TimestampFormat::Iso8601).unwrap();
        assert_eq!(result.message, "1970-01-01T00:00:00Z");
    }
}
