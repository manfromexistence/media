//! Integration tests for document processing tools.
//!
//! These tests require various external tools (Ghostscript, pdftk, etc.)
//! and will be skipped if not available.

mod common;

use common::TestFixture;

fn skip_if_no_ghostscript() {
    if !dx_media::tools::document::check_ghostscript() {
        eprintln!("Skipping test: Ghostscript not available");
    }
}

#[test]
fn test_markdown_to_html() {
    use dx_media::tools::document::markdown_to_html;

    let fixture = TestFixture::new();

    let md_file = fixture.path("test.md");
    std::fs::write(&md_file, "# Hello\n\nThis is **markdown**.").unwrap();

    let html_file = fixture.path("test.html");

    let result = markdown_to_html(&md_file, &html_file);
    assert!(
        result.is_ok(),
        "Markdown to HTML should succeed: {:?}",
        result.err()
    );
    assert!(html_file.exists(), "HTML file should exist");
}

#[test]
fn test_text_extract() {
    use dx_media::tools::document::extract;

    let fixture = TestFixture::new();

    let text_file = fixture.path("test.txt");
    std::fs::write(&text_file, "Sample text content").unwrap();

    let result = extract(&text_file);
    assert!(
        result.is_ok(),
        "Text extraction should succeed: {:?}",
        result.err()
    );
}

#[test]
#[ignore] // Requires Ghostscript
fn test_pdf_compress() {
    skip_if_no_ghostscript();

    use dx_media::tools::document::{CompressionQuality, compress_pdf};

    let fixture = TestFixture::new();

    // This test requires a real PDF file
    let input_pdf = fixture.path("input.pdf");
    if !input_pdf.exists() {
        eprintln!("Skipping: No test PDF available");
        return;
    }

    let output_pdf = fixture.path("compressed.pdf");

    let result = compress_pdf(&input_pdf, &output_pdf, CompressionQuality::Ebook);
    assert!(
        result.is_ok(),
        "PDF compression should succeed: {:?}",
        result.err()
    );
    assert!(output_pdf.exists(), "Compressed PDF should exist");
}

#[test]
#[ignore] // Requires pdftk or similar
fn test_pdf_merge() {
    use dx_media::tools::document::merge_pdfs;

    let fixture = TestFixture::new();

    // This test requires real PDF files
    let pdf1 = fixture.path("doc1.pdf");
    let pdf2 = fixture.path("doc2.pdf");

    if !pdf1.exists() || !pdf2.exists() {
        eprintln!("Skipping: No test PDFs available");
        return;
    }

    let output = fixture.path("merged.pdf");

    let result = merge_pdfs(&[&pdf1, &pdf2], &output);
    assert!(
        result.is_ok(),
        "PDF merge should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "Merged PDF should exist");
}

#[test]
#[ignore] // Requires pdftk or similar
fn test_pdf_split() {
    use dx_media::tools::document::split_pdf;

    let fixture = TestFixture::new();

    let input_pdf = fixture.path("input.pdf");
    if !input_pdf.exists() {
        eprintln!("Skipping: No test PDF available");
        return;
    }

    let output_dir = fixture.path("split");

    let result = split_pdf(&input_pdf, &output_dir);
    assert!(
        result.is_ok(),
        "PDF split should succeed: {:?}",
        result.err()
    );
    assert!(output_dir.exists(), "Split output directory should exist");
}

#[test]
#[ignore] // Requires Ghostscript or similar
fn test_pdf_to_images() {
    use dx_media::tools::document::pdf_to_images;

    let fixture = TestFixture::new();

    let input_pdf = fixture.path("input.pdf");
    if !input_pdf.exists() {
        eprintln!("Skipping: No test PDF available");
        return;
    }

    let output_dir = fixture.path("images");

    let result = pdf_to_images(&input_pdf, &output_dir);
    assert!(
        result.is_ok(),
        "PDF to images should succeed: {:?}",
        result.err()
    );
    assert!(output_dir.exists(), "Images output directory should exist");
}

#[test]
#[ignore] // Requires wkhtmltopdf or similar
fn test_html_to_pdf() {
    use dx_media::tools::document::html_to_pdf;

    let fixture = TestFixture::new();

    let html_file = fixture.path("test.html");
    std::fs::write(&html_file, "<html><body><h1>Test</h1></body></html>").unwrap();

    let pdf_file = fixture.path("output.pdf");

    let result = html_to_pdf(&html_file, &pdf_file);
    assert!(
        result.is_ok(),
        "HTML to PDF should succeed: {:?}",
        result.err()
    );
    assert!(pdf_file.exists(), "PDF file should exist");
}

#[test]
#[ignore] // Requires pdftk or similar
fn test_pdf_watermark() {
    use dx_media::tools::document::text_watermark;

    let fixture = TestFixture::new();

    let input_pdf = fixture.path("input.pdf");
    if !input_pdf.exists() {
        eprintln!("Skipping: No test PDF available");
        return;
    }

    let output_pdf = fixture.path("watermarked.pdf");

    let result = text_watermark(&input_pdf, &output_pdf, "CONFIDENTIAL");
    assert!(
        result.is_ok(),
        "PDF watermark should succeed: {:?}",
        result.err()
    );
    assert!(output_pdf.exists(), "Watermarked PDF should exist");
}

#[test]
#[ignore] // Requires pdftk or similar
fn test_pdf_encrypt() {
    use dx_media::tools::document::encrypt;

    let fixture = TestFixture::new();

    let input_pdf = fixture.path("input.pdf");
    if !input_pdf.exists() {
        eprintln!("Skipping: No test PDF available");
        return;
    }

    let output_pdf = fixture.path("encrypted.pdf");

    let result = encrypt(&input_pdf, &output_pdf, "password123");
    assert!(
        result.is_ok(),
        "PDF encryption should succeed: {:?}",
        result.err()
    );
    assert!(output_pdf.exists(), "Encrypted PDF should exist");
}
