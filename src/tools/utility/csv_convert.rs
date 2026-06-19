//! CSV conversion utilities.
//!
//! Convert CSV to other formats and vice versa.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

/// CSV parsing options.
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// Field delimiter.
    pub delimiter: char,
    /// Has header row.
    pub has_header: bool,
    /// Quote character.
    pub quote: char,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_header: true,
            quote: '"',
        }
    }
}

/// Convert CSV to JSON.
///
/// # Example
/// ```no_run
/// use dx_media::tools::utility::csv_convert;
///
/// csv_convert::csv_to_json("data.csv", "data.json").unwrap();
/// ```
pub fn csv_to_json<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    csv_to_json_with_options(input, output, CsvOptions::default())
}

/// Convert CSV to JSON with options.
pub fn csv_to_json_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: CsvOptions,
) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let rows = parse_csv(&content, &options)?;

    let json = if options.has_header && !rows.is_empty() {
        // Use first row as keys
        let headers = &rows[0];
        let data: Vec<String> = rows[1..]
            .iter()
            .map(|row| {
                let pairs: Vec<String> = headers
                    .iter()
                    .zip(row.iter())
                    .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            })
            .collect();
        format!("[{}]", data.join(","))
    } else {
        // Array of arrays
        let data: Vec<String> = rows
            .iter()
            .map(|row| {
                let values: Vec<String> = row
                    .iter()
                    .map(|v| format!("\"{}\"", escape_json(v)))
                    .collect();
                format!("[{}]", values.join(","))
            })
            .collect();
        format!("[{}]", data.join(","))
    };

    std::fs::write(output_path, &json).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        format!("Converted {} rows to JSON", rows.len()),
        output_path,
    )
    .with_metadata("row_count", rows.len().to_string()))
}

/// Convert JSON to CSV.
pub fn json_to_csv<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let csv = json_to_csv_string(&content)?;

    std::fs::write(output_path, &csv).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        "Converted JSON to CSV",
        output_path,
    ))
}

/// Convert CSV to Markdown table.
pub fn csv_to_markdown<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let rows = parse_csv(&content, &CsvOptions::default())?;

    if rows.is_empty() {
        return Err(DxError::Config {
            message: "Empty CSV file".to_string(),
            source: None,
        });
    }

    let mut markdown = String::new();

    // Header row
    markdown.push_str("| ");
    markdown.push_str(&rows[0].join(" | "));
    markdown.push_str(" |\n");

    // Separator
    markdown.push('|');
    for _ in &rows[0] {
        markdown.push_str(" --- |");
    }
    markdown.push('\n');

    // Data rows
    for row in &rows[1..] {
        markdown.push_str("| ");
        markdown.push_str(&row.join(" | "));
        markdown.push_str(" |\n");
    }

    std::fs::write(output_path, &markdown).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        format!("Converted {} rows to Markdown", rows.len()),
        output_path,
    ))
}

/// Convert CSV to HTML table.
pub fn csv_to_html<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    let input_path = input.as_ref();
    let output_path = output.as_ref();

    let content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let rows = parse_csv(&content, &CsvOptions::default())?;

    if rows.is_empty() {
        return Err(DxError::Config {
            message: "Empty CSV file".to_string(),
            source: None,
        });
    }

    let mut html = String::from("<table>\n");

    // Header
    html.push_str("  <thead>\n    <tr>\n");
    for cell in &rows[0] {
        html.push_str(&format!("      <th>{}</th>\n", escape_html(cell)));
    }
    html.push_str("    </tr>\n  </thead>\n");

    // Body
    html.push_str("  <tbody>\n");
    for row in &rows[1..] {
        html.push_str("    <tr>\n");
        for cell in row {
            html.push_str(&format!("      <td>{}</td>\n", escape_html(cell)));
        }
        html.push_str("    </tr>\n");
    }
    html.push_str("  </tbody>\n</table>");

    std::fs::write(output_path, &html).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        format!("Converted {} rows to HTML", rows.len()),
        output_path,
    ))
}

/// Parse CSV content.
fn parse_csv(content: &str, options: &CsvOptions) -> Result<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut prev_was_quote = false;

    for c in content.chars() {
        if prev_was_quote {
            prev_was_quote = false;
            if c == options.quote {
                // Escaped quote
                current_field.push(options.quote);
                continue;
            }
            in_quotes = false;
        }

        if c == options.quote {
            if in_quotes {
                prev_was_quote = true;
            } else if current_field.is_empty() {
                in_quotes = true;
            } else {
                current_field.push(c);
            }
        } else if c == options.delimiter && !in_quotes {
            current_row.push(current_field.trim().to_string());
            current_field = String::new();
        } else if c == '\n' && !in_quotes {
            current_row.push(current_field.trim().to_string());
            if !current_row.iter().all(|s| s.is_empty()) {
                rows.push(current_row);
            }
            current_row = Vec::new();
            current_field = String::new();
        } else if c != '\r' {
            current_field.push(c);
        }
    }

    // Handle last row
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field.trim().to_string());
        if !current_row.iter().all(|s| s.is_empty()) {
            rows.push(current_row);
        }
    }

    Ok(rows)
}

/// Convert a JSON array of objects or arrays into CSV text.
fn json_to_csv_string(json: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json).map_err(|e| DxError::Config {
        message: format!("Invalid JSON: {}", e),
        source: None,
    })?;

    let rows = value.as_array().ok_or_else(|| DxError::Config {
        message: "Expected JSON array".to_string(),
        source: None,
    })?;

    if rows.is_empty() {
        return Ok(String::new());
    }

    if rows.iter().all(Value::is_object) {
        return json_object_rows_to_csv(rows);
    }

    if rows.iter().all(Value::is_array) {
        return json_array_rows_to_csv(rows);
    }

    Err(DxError::Config {
        message: "Expected JSON array of objects or arrays".to_string(),
        source: None,
    })
}

fn json_object_rows_to_csv(rows: &[Value]) -> Result<String> {
    let headers: Vec<String> = rows
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|object| object.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    if headers.is_empty() {
        return Ok(String::new());
    }

    let mut csv = String::new();
    push_csv_row(&mut csv, headers.iter().map(String::as_str));

    for row in rows {
        let object = row.as_object().ok_or_else(|| DxError::Config {
            message: "Expected JSON object row".to_string(),
            source: None,
        })?;
        let fields = headers
            .iter()
            .map(|header| json_cell_to_string(object.get(header)))
            .collect::<Result<Vec<_>>>()?;
        push_csv_row(&mut csv, fields.iter().map(String::as_str));
    }

    Ok(csv)
}

fn json_array_rows_to_csv(rows: &[Value]) -> Result<String> {
    let mut csv = String::new();

    for row in rows {
        let values = row.as_array().ok_or_else(|| DxError::Config {
            message: "Expected JSON array row".to_string(),
            source: None,
        })?;
        let fields = values
            .iter()
            .map(|value| json_cell_to_string(Some(value)))
            .collect::<Result<Vec<_>>>()?;
        push_csv_row(&mut csv, fields.iter().map(String::as_str));
    }

    Ok(csv)
}

fn json_cell_to_string(value: Option<&Value>) -> Result<String> {
    match value {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(value)) => Ok(value.clone()),
        Some(Value::Bool(value)) => Ok(value.to_string()),
        Some(Value::Number(value)) => Ok(value.to_string()),
        Some(value @ (Value::Array(_) | Value::Object(_))) => {
            serde_json::to_string(value).map_err(|e| DxError::Config {
                message: format!("Failed to encode nested JSON cell: {}", e),
                source: None,
            })
        }
    }
}

fn push_csv_row<'a>(csv: &mut String, fields: impl IntoIterator<Item = &'a str>) {
    let mut first = true;
    for field in fields {
        if !first {
            csv.push(',');
        }
        first = false;
        csv.push_str(&escape_csv_field(field));
    }
    csv.push('\n');
}

fn escape_csv_field(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Escape JSON string.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape HTML string.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Get CSV statistics.
pub fn csv_stats<P: AsRef<Path>>(input: P) -> Result<ToolOutput> {
    let content = std::fs::read_to_string(input.as_ref()).map_err(|e| DxError::FileIo {
        path: input.as_ref().to_path_buf(),
        message: format!("Failed to read file: {}", e),
        source: None,
    })?;

    let rows = parse_csv(&content, &CsvOptions::default())?;
    let column_count = rows.first().map_or(0, |r| r.len());

    Ok(
        ToolOutput::success(format!("Rows: {}\nColumns: {}", rows.len(), column_count))
            .with_metadata("row_count", rows.len().to_string())
            .with_metadata("column_count", column_count.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let csv = "a,b,c\n1,2,3\n4,5,6";
        let rows = parse_csv(csv, &CsvOptions::default()).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["a", "b", "c"]);
    }

    #[test]
    fn test_quoted_csv() {
        let csv = r#"name,value
"hello, world",123"#;
        let rows = parse_csv(csv, &CsvOptions::default()).unwrap();
        assert_eq!(rows[1][0], "hello, world");
    }

    #[test]
    fn json_object_rows_escape_csv_cells() {
        let csv = json_to_csv_string(r#"[{"name":"Alice, A.","note":"uses \"quotes\""}]"#).unwrap();
        assert_eq!(csv, "name,note\n\"Alice, A.\",\"uses \"\"quotes\"\"\"\n");
    }
}
