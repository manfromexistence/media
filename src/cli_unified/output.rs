//! Output formatting utilities

use console::style;
use serde::Serialize;

use super::args::OutputFormat;

pub fn print_json<T: Serialize>(data: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

pub fn print_success(message: &str) {
    println!("{}", style(format!("✅ {}", message)).green());
}

pub fn print_error(message: &str) {
    eprintln!("{}", style(format!("❌ {}", message)).red());
}

pub fn print_info(message: &str) {
    println!("{}", style(message).cyan());
}

pub fn print_table_header(columns: &[&str]) {
    println!("{}", "─".repeat(80));
    let mut header = String::new();
    for (i, col) in columns.iter().enumerate() {
        if i > 0 {
            header.push_str(" ");
        }
        header.push_str(&format!("{:<20}", style(col).bold()));
    }
    println!("{}", header);
    println!("{}", "─".repeat(80));
}

pub fn print_table_row(values: &[String]) {
    let mut row = String::new();
    for (i, val) in values.iter().enumerate() {
        if i > 0 {
            row.push_str(" ");
        }
        row.push_str(&format!("{:<20}", truncate(val, 18)));
    }
    println!("{}", row);
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn format_output<T: Serialize>(
    data: &T,
    format: &OutputFormat,
    table_fn: impl FnOnce(),
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => print_json(data)?,
        OutputFormat::Table => table_fn(),
        OutputFormat::Simple => table_fn(),
    }
    Ok(())
}
