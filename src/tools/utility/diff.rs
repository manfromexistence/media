//! File diff utilities.
//!
//! Compare files and generate diffs.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Diff output format.
#[derive(Debug, Clone, Copy, Default)]
pub enum DiffFormat {
    /// Unified diff format.
    #[default]
    Unified,
    /// Side by side.
    SideBySide,
    /// Context format.
    Context,
}

/// Compare two files.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::diff;
///
/// let result = diff::diff_files("file1.txt", "file2.txt").unwrap();
/// ```
pub fn diff_files<P: AsRef<Path>>(file1: P, file2: P) -> Result<ToolOutput> {
    diff_files_with_format(file1, file2, DiffFormat::default())
}

/// Compare two files with format.
pub fn diff_files_with_format<P: AsRef<Path>>(
    file1: P,
    file2: P,
    format: DiffFormat,
) -> Result<ToolOutput> {
    let file1_path = file1.as_ref();
    let file2_path = file2.as_ref();

    let content1 = std::fs::read_to_string(file1_path).map_err(|e| DxError::FileIo {
        path: file1_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let content2 = std::fs::read_to_string(file2_path).map_err(|e| DxError::FileIo {
        path: file2_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    diff_strings_with_format(&content1, &content2, format)
}

/// Compare two strings.
pub fn diff_strings(s1: &str, s2: &str) -> Result<ToolOutput> {
    diff_strings_with_format(s1, s2, DiffFormat::default())
}

/// Compare two strings with format.
pub fn diff_strings_with_format(s1: &str, s2: &str, format: DiffFormat) -> Result<ToolOutput> {
    let lines1: Vec<&str> = s1.lines().collect();
    let lines2: Vec<&str> = s2.lines().collect();

    let diff_result = compute_diff(&lines1, &lines2);

    let output = match format {
        DiffFormat::Unified => format_unified(&diff_result, &lines1, &lines2),
        DiffFormat::SideBySide => format_side_by_side(&diff_result, &lines1, &lines2),
        DiffFormat::Context => format_context(&diff_result, &lines1, &lines2),
    };

    let changes = diff_result
        .iter()
        .filter(|d| !matches!(d, DiffOp::Equal(_)))
        .count();

    Ok(ToolOutput::success(output)
        .with_metadata("changes", changes.to_string())
        .with_metadata("lines1", lines1.len().to_string())
        .with_metadata("lines2", lines2.len().to_string()))
}

/// Check if two files are identical.
pub fn files_identical<P: AsRef<Path>>(file1: P, file2: P) -> Result<ToolOutput> {
    let content1 = std::fs::read(file1.as_ref()).map_err(|e| DxError::FileIo {
        path: file1.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let content2 = std::fs::read(file2.as_ref()).map_err(|e| DxError::FileIo {
        path: file2.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let identical = content1 == content2;

    Ok(ToolOutput::success(
        if identical {
            "Files are identical"
        } else {
            "Files differ"
        }
        .to_string(),
    )
    .with_metadata("identical", identical.to_string()))
}

/// Save diff to file.
pub fn save_diff<P: AsRef<Path>>(file1: P, file2: P, output: P) -> Result<ToolOutput> {
    let diff = diff_files(&file1, &file2)?;

    std::fs::write(output.as_ref(), &diff.message).map_err(|e| DxError::FileIo {
        path: output.as_ref().to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        "Diff saved to file",
        output.as_ref(),
    ))
}

/// Diff operation.
#[derive(Debug, Clone)]
enum DiffOp {
    Equal(usize),
    Insert(usize),
    Delete(usize),
}

/// Compute diff using simple LCS algorithm.
fn compute_diff(lines1: &[&str], lines2: &[&str]) -> Vec<DiffOp> {
    // Simple diff algorithm
    let m = lines1.len();
    let n = lines2.len();

    // Build LCS table
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if lines1[i - 1] == lines2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = std::cmp::max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    // Backtrack to get diff
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines1[i - 1] == lines2[j - 1] {
            result.push(DiffOp::Equal(i - 1));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(DiffOp::Insert(j - 1));
            j -= 1;
        } else if i > 0 {
            result.push(DiffOp::Delete(i - 1));
            i -= 1;
        }
    }

    result.reverse();
    result
}

/// Format as unified diff.
fn format_unified(ops: &[DiffOp], lines1: &[&str], lines2: &[&str]) -> String {
    let mut output = String::new();

    for op in ops {
        match op {
            DiffOp::Equal(i) => {
                output.push_str(&format!(" {}\n", lines1[*i]));
            }
            DiffOp::Delete(i) => {
                output.push_str(&format!("-{}\n", lines1[*i]));
            }
            DiffOp::Insert(i) => {
                output.push_str(&format!("+{}\n", lines2[*i]));
            }
        }
    }

    output
}

/// Format as side by side.
fn format_side_by_side(ops: &[DiffOp], lines1: &[&str], lines2: &[&str]) -> String {
    let mut output = String::new();
    let width = 40;

    let mut i1 = 0;
    let mut i2 = 0;

    for op in ops {
        match op {
            DiffOp::Equal(_) => {
                let left = lines1.get(i1).unwrap_or(&"");
                let right = lines2.get(i2).unwrap_or(&"");
                output.push_str(&format!("{:<width$} | {}\n", truncate(left, width), right));
                i1 += 1;
                i2 += 1;
            }
            DiffOp::Delete(_) => {
                let left = lines1.get(i1).unwrap_or(&"");
                output.push_str(&format!("{:<width$} < \n", truncate(left, width)));
                i1 += 1;
            }
            DiffOp::Insert(_) => {
                output.push_str(&format!(
                    "{:<width$} > {}\n",
                    "",
                    lines2.get(i2).unwrap_or(&"")
                ));
                i2 += 1;
            }
        }
    }

    output
}

/// Format as context diff.
fn format_context(ops: &[DiffOp], lines1: &[&str], lines2: &[&str]) -> String {
    let mut output = String::new();

    output.push_str("*** Original\n");
    for op in ops {
        match op {
            DiffOp::Equal(i) | DiffOp::Delete(i) => {
                let prefix = if matches!(op, DiffOp::Delete(_)) {
                    "- "
                } else {
                    "  "
                };
                output.push_str(&format!("{}{}\n", prefix, lines1[*i]));
            }
            _ => {}
        }
    }

    output.push_str("--- Modified\n");
    for op in ops {
        match op {
            DiffOp::Equal(i) => {
                output.push_str(&format!("  {}\n", lines1[*i]));
            }
            DiffOp::Insert(i) => {
                output.push_str(&format!("+ {}\n", lines2[*i]));
            }
            _ => {}
        }
    }

    output
}

/// Truncate string to width.
fn truncate(s: &str, width: usize) -> String {
    if s.len() <= width {
        s.to_string()
    } else {
        format!("{}...", &s[..width.saturating_sub(3)])
    }
}

/// Get diff stats.
pub fn diff_stats<P: AsRef<Path>>(file1: P, file2: P) -> Result<ToolOutput> {
    let content1 = std::fs::read_to_string(file1.as_ref()).map_err(|e| DxError::FileIo {
        path: file1.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let content2 = std::fs::read_to_string(file2.as_ref()).map_err(|e| DxError::FileIo {
        path: file2.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();

    let diff = compute_diff(&lines1, &lines2);

    let additions = diff
        .iter()
        .filter(|d| matches!(d, DiffOp::Insert(_)))
        .count();
    let deletions = diff
        .iter()
        .filter(|d| matches!(d, DiffOp::Delete(_)))
        .count();
    let unchanged = diff
        .iter()
        .filter(|d| matches!(d, DiffOp::Equal(_)))
        .count();

    Ok(ToolOutput::success(format!(
        "Additions: {}\nDeletions: {}\nUnchanged: {}",
        additions, deletions, unchanged
    ))
    .with_metadata("additions", additions.to_string())
    .with_metadata("deletions", deletions.to_string())
    .with_metadata("unchanged", unchanged.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_strings() {
        let s1 = "line1\nline2\nline3";
        let s2 = "line1\nmodified\nline3";

        let result = diff_strings(s1, s2).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_identical() {
        let s = "same\ncontent";
        let result = diff_strings(s, s).unwrap();
        assert_eq!(result.metadata.get("changes").unwrap(), "0");
    }
}
