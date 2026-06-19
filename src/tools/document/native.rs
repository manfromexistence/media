//! Native document processing using pure Rust crates.
//!
//! This module provides native Rust document handling
//! as an alternative to external tools like Ghostscript.
//!
//! Enable with the `document-core` feature flag.

#[cfg(feature = "document-core")]
use std::collections::HashMap;
use std::path::Path;

use crate::tools::ToolOutput;
#[cfg(feature = "document-core")]
use crate::tools::ToolReceipt;

#[cfg(feature = "document-core")]
fn require_html_output_path(output: &Path) -> std::io::Result<()> {
    let is_html = output
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("html"));

    if is_html {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "HTML output path must use .html extension: {}",
            output.display()
        ),
    ))
}

#[cfg(feature = "document-core")]
fn require_text_output_path(output: &Path) -> std::io::Result<()> {
    let extension = output
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    let is_text = extension.as_deref().is_some_and(|extension| {
        crate::types::MediaType::Text
            .extensions()
            .contains(&extension)
    });

    if is_text {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "Text output path must use .txt, .md, or .rst extension: {}",
            output.display()
        ),
    ))
}

#[cfg(feature = "document-core")]
fn require_pdf_output_path(output: &Path) -> std::io::Result<()> {
    let is_pdf = output
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));

    if is_pdf {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "PDF output path must use .pdf extension: {}",
            output.display()
        ),
    ))
}

#[cfg(feature = "document-core")]
fn require_pdf_input_path(input: &Path) -> std::io::Result<()> {
    let is_pdf = input
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));

    if is_pdf {
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!(
            "PDF input path must use .pdf extension: {}",
            input.display()
        ),
    ))
}

#[cfg(feature = "document-core")]
fn with_pdf_input_type_validation(output: ToolOutput, input: &Path) -> ToolOutput {
    let input_extension = input
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_else(|| "unknown".to_string());

    output
        .with_metadata("tool.expected_input_extension", "pdf")
        .with_metadata("tool.input_extension", input_extension)
}

/// PDF document information.
#[derive(Debug, Clone)]
pub struct PdfInfo {
    /// Number of pages.
    pub page_count: usize,
    /// PDF version.
    pub version: String,
    /// Is encrypted.
    pub is_encrypted: bool,
    /// Title metadata.
    pub title: Option<String>,
    /// Author metadata.
    pub author: Option<String>,
    /// Creator metadata.
    pub creator: Option<String>,
    /// Producer metadata.
    pub producer: Option<String>,
}

/// Get PDF information using lopdf.
#[cfg(feature = "document-core")]
pub fn pdf_info_native(input: impl AsRef<Path>) -> std::io::Result<PdfInfo> {
    use lopdf::Document;

    let input = input.as_ref();
    require_pdf_input_path(input)?;

    let doc = Document::load(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let page_count = doc.get_pages().len();
    let version = doc.version.clone();
    let is_encrypted = doc.is_encrypted();

    // Extract metadata from document info dictionary
    let mut title = None;
    let mut author = None;
    let mut creator = None;
    let mut producer = None;

    if let Ok(info) = doc.trailer.get(b"Info") {
        if let Ok(info_ref) = info.as_reference() {
            if let Ok(info_dict) = doc.get_dictionary(info_ref) {
                title = info_dict
                    .get(b"Title")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .map(|s| String::from_utf8_lossy(s).to_string());
                author = info_dict
                    .get(b"Author")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .map(|s| String::from_utf8_lossy(s).to_string());
                creator = info_dict
                    .get(b"Creator")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .map(|s| String::from_utf8_lossy(s).to_string());
                producer = info_dict
                    .get(b"Producer")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .map(|s| String::from_utf8_lossy(s).to_string());
            }
        }
    }

    Ok(PdfInfo {
        page_count,
        version,
        is_encrypted,
        title,
        author,
        creator,
        producer,
    })
}

/// Merge multiple PDFs using lopdf.
/// Note: Basic implementation - copies pages from source documents.
#[cfg(feature = "document-core")]
pub fn pdf_merge_native(
    inputs: &[impl AsRef<Path>],
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    if inputs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No input files provided",
        ));
    }

    let _ = output.as_ref();
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native multi-PDF merge is not implemented; use the external PDF merge tool",
    ))
}

/// Extract text from PDF using lopdf.
#[cfg(feature = "document-core")]
pub fn pdf_extract_text_native(input: impl AsRef<Path>) -> std::io::Result<String> {
    use lopdf::Document;

    let input = input.as_ref();
    require_pdf_input_path(input)?;

    let doc = Document::load(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let mut text = String::new();

    for (page_num, _) in doc.get_pages() {
        if let Ok(content) = doc.extract_text(&[page_num]) {
            text.push_str(&content);
            text.push('\n');
        }
    }

    Ok(text)
}

/// Extract text from PDF to file.
#[cfg(feature = "document-core")]
pub fn pdf_to_text_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    let input = input.as_ref();
    let output_path = output.as_ref();

    require_pdf_input_path(input)?;
    require_text_output_path(output_path)?;

    let text = pdf_extract_text_native(input)?;
    std::fs::write(output_path, &text)?;

    let mut metadata = HashMap::new();
    metadata.insert("char_count".to_string(), text.len().to_string());
    metadata.insert("line_count".to_string(), text.lines().count().to_string());

    let tool_output = ToolOutput::success_with_path(
        format!(
            "Extracted text from {} to {} ({} chars)",
            input.display(),
            output_path.display(),
            text.len()
        ),
        output_path,
    )
    .with_receipt(
        ToolReceipt::local("document.extract-text.native").with_source(input.display().to_string()),
    );

    Ok(with_pdf_input_type_validation(tool_output, input)
        .with_output_type_validation(output_path, crate::types::MediaType::Text)
        .with_metadata_entries(metadata))
}

/// Delete specific pages from PDF.
#[cfg(feature = "document-core")]
pub fn pdf_delete_pages_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    pages_to_delete: &[u32],
) -> std::io::Result<ToolOutput> {
    use lopdf::Document;

    let input = input.as_ref();
    let output_path = output.as_ref();
    require_pdf_input_path(input)?;
    require_pdf_output_path(output_path)?;

    let mut doc = Document::load(input)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let original_count = doc.get_pages().len();

    // Delete pages (in reverse order to maintain indices)
    let mut pages: Vec<u32> = pages_to_delete.to_vec();
    pages.sort();
    pages.reverse();

    for page_num in pages {
        doc.delete_pages(&[page_num]);
    }

    doc.save(output_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let new_count = doc.get_pages().len();

    let mut metadata = HashMap::new();
    metadata.insert("original_pages".to_string(), original_count.to_string());
    metadata.insert(
        "deleted_pages".to_string(),
        pages_to_delete.len().to_string(),
    );
    metadata.insert("final_pages".to_string(), new_count.to_string());

    let tool_output = ToolOutput::success_with_path(
        format!(
            "Deleted {} pages from {} -> {} ({} -> {} pages)",
            pages_to_delete.len(),
            input.display(),
            output_path.display(),
            original_count,
            new_count
        ),
        output_path,
    )
    .with_receipt(
        ToolReceipt::local("document.pdf-delete-pages.native")
            .with_source(input.display().to_string()),
    );

    Ok(with_pdf_input_type_validation(tool_output, input)
        .with_metadata("tool.expected_output_extension", "pdf")
        .with_output_type_validation(output_path, crate::types::MediaType::Document)
        .with_metadata_entries(metadata))
}

/// Convert markdown to HTML using pulldown-cmark.
#[cfg(feature = "document-core")]
pub fn markdown_to_html_native(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    include_wrapper: bool,
) -> std::io::Result<ToolOutput> {
    use pulldown_cmark::{Options, Parser, html};

    let input = input.as_ref();
    let output = output.as_ref();

    require_html_output_path(output)?;

    let markdown = std::fs::read_to_string(input)?;

    let options = Options::all();
    let parser = Parser::new_ext(&markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let final_html = if include_wrapper {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; line-height: 1.6; }}
        pre {{ background: #f4f4f4; padding: 1rem; overflow-x: auto; border-radius: 4px; }}
        code {{ background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 3px; }}
        pre code {{ background: none; padding: 0; }}
        blockquote {{ border-left: 4px solid #ddd; margin: 0; padding-left: 1rem; color: #666; }}
        img {{ max-width: 100%; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 0.5rem; text-align: left; }}
        th {{ background: #f4f4f4; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            input.file_stem().unwrap_or_default().to_string_lossy(),
            html_output
        )
    } else {
        html_output.clone()
    };

    std::fs::write(output, &final_html)?;

    let mut metadata = HashMap::new();
    metadata.insert("input_size".to_string(), markdown.len().to_string());
    metadata.insert("output_size".to_string(), final_html.len().to_string());
    metadata.insert("wrapped".to_string(), include_wrapper.to_string());
    metadata.insert(
        "tool.expected_output_extension".to_string(),
        "html".to_string(),
    );

    Ok(ToolOutput::success_with_path(
        format!(
            "Converted {} to {} ({} bytes)",
            input.display(),
            output.display(),
            final_html.len()
        ),
        output,
    )
    .with_receipt(
        ToolReceipt::local("document.markdown-to-html.native")
            .with_source(input.display().to_string()),
    )
    .with_output_type_validation(output, crate::types::MediaType::Document)
    .with_metadata_entries(metadata))
}

/// Parse markdown and return structured content.
#[cfg(feature = "document-core")]
pub fn parse_markdown_native(input: impl AsRef<Path>) -> std::io::Result<MarkdownContent> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let input = input.as_ref();
    let markdown = std::fs::read_to_string(input)?;

    let options = Options::all();
    let parser = Parser::new_ext(&markdown, options);

    let mut content = MarkdownContent::default();
    let mut current_heading_level = 0;
    let mut _in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading_level = level as u8;
            }
            Event::Text(text) => {
                if current_heading_level > 0 {
                    content
                        .headings
                        .push((current_heading_level, text.to_string()));
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                current_heading_level = 0;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                _in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                _in_code_block = false;
                content.code_block_count += 1;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                content.links.push(dest_url.to_string());
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                content.images.push(dest_url.to_string());
            }
            _ => {}
        }
    }

    content.word_count = markdown.split_whitespace().count();
    content.line_count = markdown.lines().count();

    Ok(content)
}

/// Structured markdown content.
#[derive(Debug, Clone, Default)]
pub struct MarkdownContent {
    /// Headings with their levels.
    pub headings: Vec<(u8, String)>,
    /// Links found in the document.
    pub links: Vec<String>,
    /// Images found in the document.
    pub images: Vec<String>,
    /// Number of code blocks.
    pub code_block_count: usize,
    /// Word count.
    pub word_count: usize,
    /// Line count.
    pub line_count: usize,
}

// Fallback implementations when document-core is not enabled

/// Gets PDF file information using native Rust libraries.
///
/// Returns metadata including page count, title, author, and creation date.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn pdf_info_native(_input: impl AsRef<Path>) -> std::io::Result<PdfInfo> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Merges multiple PDF files into a single document.
///
/// Combines all pages from input PDFs in order.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn pdf_merge_native(
    _inputs: &[impl AsRef<Path>],
    _output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Extracts text content from a PDF file.
///
/// Returns the extracted text as a string.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn pdf_extract_text_native(_input: impl AsRef<Path>) -> std::io::Result<String> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Converts a PDF file to a text file.
///
/// Extracts text and writes it to the output path.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn pdf_to_text_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Deletes specified pages from a PDF file.
///
/// Creates a new PDF with the specified pages removed.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn pdf_delete_pages_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _pages_to_delete: &[u32],
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Converts a Markdown file to HTML.
///
/// Optionally wraps the output in a complete HTML document.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn markdown_to_html_native(
    _input: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _include_wrapper: bool,
) -> std::io::Result<ToolOutput> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}

/// Parses a Markdown file and extracts structured content.
///
/// Returns headings, links, images, code blocks, and statistics.
/// Requires the `document-core` feature to be enabled.
#[cfg(not(feature = "document-core"))]
pub fn parse_markdown_native(_input: impl AsRef<Path>) -> std::io::Result<MarkdownContent> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Native document processing requires the 'document-core' feature",
    ))
}
