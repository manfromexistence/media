//! Markdown converter.
//!
//! Convert markdown files to HTML and other formats.

use crate::error::{DxError, Result};
use crate::tools::ToolOutput;
use std::path::Path;

/// Markdown conversion options.
#[derive(Debug, Clone, Default)]
pub struct MarkdownOptions {
    /// Include CSS styling.
    pub include_css: bool,
    /// Custom CSS content.
    pub custom_css: Option<String>,
    /// Enable syntax highlighting.
    pub syntax_highlight: bool,
    /// Generate table of contents.
    pub table_of_contents: bool,
    /// Enable GFM (GitHub Flavored Markdown).
    pub gfm: bool,
}

impl MarkdownOptions {
    /// Styled HTML output.
    pub fn styled() -> Self {
        Self {
            include_css: true,
            custom_css: None,
            syntax_highlight: true,
            table_of_contents: true,
            gfm: true,
        }
    }

    /// Plain HTML without styling.
    pub fn plain() -> Self {
        Self::default()
    }
}

/// Default CSS for markdown HTML output.
const DEFAULT_CSS: &str = r"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    color: #333;
}
h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}
code {
    background: #f4f4f4;
    padding: 2px 5px;
    border-radius: 3px;
    font-family: 'Fira Code', Consolas, Monaco, monospace;
}
pre {
    background: #f4f4f4;
    padding: 15px;
    border-radius: 5px;
    overflow-x: auto;
}
pre code {
    background: none;
    padding: 0;
}
blockquote {
    border-left: 4px solid #ddd;
    margin: 0;
    padding-left: 20px;
    color: #666;
}
table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}
th, td {
    border: 1px solid #ddd;
    padding: 8px 12px;
    text-align: left;
}
th {
    background: #f4f4f4;
}
img {
    max-width: 100%;
}
a {
    color: #0366d6;
}
";

/// Convert markdown to HTML.
///
/// # Arguments
/// * `input` - Path to markdown file
/// * `output` - Path for HTML output
///
/// # Example
/// ```no_run
/// use dx_media::tools::document::markdown_to_html;
///
/// markdown_to_html("README.md", "readme.html").unwrap();
/// ```
pub fn markdown_to_html<P: AsRef<Path>>(input: P, output: P) -> Result<ToolOutput> {
    markdown_to_html_with_options(input, output, MarkdownOptions::styled())
}

/// Convert markdown to HTML with options.
pub fn markdown_to_html_with_options<P: AsRef<Path>>(
    input: P,
    output: P,
    options: MarkdownOptions,
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

    let markdown_content = std::fs::read_to_string(input_path).map_err(|e| DxError::FileIo {
        path: input_path.to_path_buf(),
        message: format!("Failed to read input file: {}", e),
        source: None,
    })?;

    // Simple markdown to HTML conversion
    let html_body = convert_markdown(&markdown_content);

    // Build full HTML document
    let css = if options.include_css {
        options.custom_css.as_deref().unwrap_or(DEFAULT_CSS)
    } else {
        ""
    };

    let title = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Document");

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>{}</style>
</head>
<body>
{}
</body>
</html>"#,
        title, css, html_body
    );

    std::fs::write(output_path, &html).map_err(|e| DxError::FileIo {
        path: output_path.to_path_buf(),
        message: format!("Failed to write output file: {}", e),
        source: None,
    })?;

    Ok(ToolOutput::success_with_path(
        "Converted markdown to HTML",
        output_path,
    ))
}

/// Simple markdown to HTML converter.
fn convert_markdown(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in markdown.lines() {
        // Code blocks
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                let code_lang = line[3..].trim();
                html.push_str(&format!("<pre><code class=\"language-{}\">", code_lang));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        // Close list if empty line
        if line.trim().is_empty() {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str("<br>\n");
            continue;
        }

        // Headers
        if line.starts_with("######") {
            html.push_str(&format!("<h6>{}</h6>\n", process_inline(line[6..].trim())));
        } else if line.starts_with("#####") {
            html.push_str(&format!("<h5>{}</h5>\n", process_inline(line[5..].trim())));
        } else if line.starts_with("####") {
            html.push_str(&format!("<h4>{}</h4>\n", process_inline(line[4..].trim())));
        } else if line.starts_with("###") {
            html.push_str(&format!("<h3>{}</h3>\n", process_inline(line[3..].trim())));
        } else if line.starts_with("##") {
            html.push_str(&format!("<h2>{}</h2>\n", process_inline(line[2..].trim())));
        } else if line.starts_with('#') {
            html.push_str(&format!("<h1>{}</h1>\n", process_inline(line[1..].trim())));
        }
        // Horizontal rule
        else if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            html.push_str("<hr>\n");
        }
        // Blockquote
        else if line.starts_with('>') {
            html.push_str(&format!(
                "<blockquote>{}</blockquote>\n",
                process_inline(line[1..].trim())
            ));
        }
        // Unordered list
        else if line.trim().starts_with("- ") || line.trim().starts_with("* ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>\n", process_inline(&line.trim()[2..])));
        }
        // Ordered list
        else if line
            .trim()
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
            && line.contains(". ")
        {
            if let Some(pos) = line.find(". ") {
                html.push_str(&format!("<li>{}</li>\n", process_inline(&line[pos + 2..])));
            }
        }
        // Paragraph
        else {
            html.push_str(&format!("<p>{}</p>\n", process_inline(line)));
        }
    }

    if in_list {
        html.push_str("</ul>\n");
    }
    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

/// Process inline markdown (bold, italic, links, code).
fn process_inline(text: &str) -> String {
    let mut result = escape_html(text);

    // Bold
    let bold_re = regex::Regex::new(r"\*\*(.+?)\*\*").unwrap();
    result = bold_re
        .replace_all(&result, "<strong>$1</strong>")
        .to_string();

    // Italic
    let italic_re = regex::Regex::new(r"\*(.+?)\*").unwrap();
    result = italic_re.replace_all(&result, "<em>$1</em>").to_string();

    // Inline code
    let code_re = regex::Regex::new(r"`(.+?)`").unwrap();
    result = code_re.replace_all(&result, "<code>$1</code>").to_string();

    // Links
    let link_re = regex::Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap();
    result = link_re
        .replace_all(&result, "<a href=\"$2\">$1</a>")
        .to_string();

    // Images
    let img_re = regex::Regex::new(r"!\[(.+?)\]\((.+?)\)").unwrap();
    result = img_re
        .replace_all(&result, "<img src=\"$2\" alt=\"$1\">")
        .to_string();

    result
}

/// Escape HTML special characters.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert markdown string to HTML string.
pub fn markdown_string_to_html(markdown: &str, options: MarkdownOptions) -> String {
    let html_body = convert_markdown(markdown);

    if options.include_css {
        let css = options.custom_css.as_deref().unwrap_or(DEFAULT_CSS);
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
    <style>{}</style>
</head>
<body>
{}
</body>
</html>"#,
            css, html_body
        )
    } else {
        html_body
    }
}

/// Batch convert multiple markdown files.
pub fn batch_markdown_to_html<P: AsRef<Path>>(
    inputs: &[P],
    output_dir: P,
    options: MarkdownOptions,
) -> Result<ToolOutput> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|e| DxError::FileIo {
        path: output_dir.to_path_buf(),
        message: format!("Failed to create output directory: {}", e),
        source: None,
    })?;

    let mut converted = Vec::new();

    for input in inputs {
        let input_path = input.as_ref();
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        let output_path = output_dir.join(format!("{}.html", file_stem));

        if markdown_to_html_with_options(input_path, &output_path, options.clone()).is_ok() {
            converted.push(output_path);
        }
    }

    Ok(
        ToolOutput::success(format!("Converted {} markdown files", converted.len()))
            .with_paths(converted),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_markdown() {
        let md = "# Hello\n\nThis is **bold** and *italic*.";
        let html = convert_markdown(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
        assert!(html.contains("<em>"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
    }
}
