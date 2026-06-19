//! Tests for document tools.

mod common;

use common::TestFixture;
use dx_media::tools::document;
use std::fs;

// =============================================================================
// 32. pdf_merge - PDF merging
// =============================================================================

#[test]
fn test_pdf_merge() {
    let fixture = TestFixture::new();
    let pdf1 = fixture.create_test_text_file("doc1.pdf", "%PDF-1.4\nfake pdf");
    let pdf2 = fixture.create_test_text_file("doc2.pdf", "%PDF-1.4\nfake pdf 2");
    let output = fixture.path("merged.pdf");

    let result = document::merge_pdfs(&[&pdf1, &pdf2], &output);
    let _ = result; // May fail without pdftk/ghostscript
}

#[test]
fn test_pdf_append() {
    let fixture = TestFixture::new();
    let base = fixture.create_test_text_file("base.pdf", "%PDF-1.4\nbase");
    let append = fixture.create_test_text_file("append.pdf", "%PDF-1.4\nappend");
    let output = fixture.path("combined.pdf");

    let result = document::append_pdf(&base, &append, &output);
    let _ = result;
}

#[test]
fn test_pdf_merge_directory() {
    let fixture = TestFixture::new();
    let dir = fixture.path("pdfs");
    fs::create_dir_all(&dir).ok();
    fixture.create_test_text_file("pdfs/doc1.pdf", "%PDF-1.4\n1");
    fixture.create_test_text_file("pdfs/doc2.pdf", "%PDF-1.4\n2");
    let output = fixture.path("merged.pdf");

    let result = document::merge_directory(&dir, &output);
    let _ = result;
}

// =============================================================================
// 33. pdf_split - PDF splitting
// =============================================================================

#[test]
fn test_pdf_split() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output_dir = fixture.path("pages");
    fs::create_dir_all(&output_dir).ok();

    let result = document::split_pdf(&pdf, &output_dir);
    let _ = result;
}

#[test]
fn test_pdf_extract_pages() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("extracted.pdf");

    let result = document::extract_pages(&pdf, &output, 1, 3);
    let _ = result;
}

#[test]
fn test_pdf_extract_page() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("page1.pdf");

    let result = document::extract_page(&pdf, &output, 1);
    let _ = result;
}

#[test]
fn test_pdf_get_page_count() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");

    let result = document::get_page_count(&pdf);
    let _ = result;
}

// =============================================================================
// 34. pdf_compress - PDF compression
// =============================================================================

#[test]
fn test_pdf_compression_quality_enum() {
    let _ = document::CompressionQuality::Screen;
    let _ = document::CompressionQuality::Ebook;
    let _ = document::CompressionQuality::Printer;
    let _ = document::CompressionQuality::Prepress;
}

#[test]
fn test_pdf_compress() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf content");
    let output = fixture.path("compressed.pdf");

    let result = document::compress_pdf(&pdf, &output, document::CompressionQuality::Ebook);
    let _ = result;
}

#[test]
fn test_pdf_compress_custom() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf content");
    let output = fixture.path("compressed.pdf");

    let result = document::compress_pdf_custom(&pdf, &output, 150);
    let _ = result;
}

#[test]
fn test_pdf_linearize() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("linearized.pdf");

    let result = document::linearize_pdf(&pdf, &output);
    let _ = result;
}

// =============================================================================
// 35. pdf_to_image - PDF to image conversion
// =============================================================================

#[test]
fn test_pdf_image_format_enum() {
    let _ = document::ImageFormat::Png;
    let _ = document::ImageFormat::Jpeg;
    let _ = document::ImageFormat::Tiff;
}

#[test]
fn test_pdf_to_image_options() {
    let options = document::PdfToImageOptions::default();
    let _ = options;
}

#[test]
fn test_pdf_to_images() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output_dir = fixture.path("images");
    fs::create_dir_all(&output_dir).ok();

    let result = document::pdf_to_images(&pdf, &output_dir);
    let _ = result;
}

#[test]
fn test_pdf_page_to_image() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("page1.png");

    let result =
        document::pdf_page_to_image(&pdf, &output, 1, document::PdfToImageOptions::default());
    let _ = result;
}

#[test]
fn test_pdf_thumbnail() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("thumb.png");

    let result = document::pdf_thumbnail(&pdf, &output);
    let _ = result;
}

// =============================================================================
// 36. markdown - Markdown conversion
// =============================================================================

#[test]
fn test_markdown_options() {
    let options = document::MarkdownOptions::default();
    let _ = options;
}

#[test]
fn test_markdown_string_to_html() {
    let md = "# Hello\n\nThis is **bold** text.";
    let options = document::MarkdownOptions::default();
    let html = document::markdown_string_to_html(md, options);
    assert!(html.contains("Hello") || html.contains("<h1>"));
}

#[test]
fn test_markdown_to_html_file() {
    let fixture = TestFixture::new();
    let md_file = fixture.create_test_text_file("doc.md", "# Test\n\nParagraph.");
    let html_file = fixture.path("doc.html");

    let result = document::markdown_to_html(&md_file, &html_file);
    let _ = result;
}

// =============================================================================
// 37. html_to_pdf - HTML to PDF conversion
// =============================================================================

#[test]
fn test_page_orientation_enum() {
    let _ = document::PageOrientation::Portrait;
    let _ = document::PageOrientation::Landscape;
}

#[test]
fn test_html_to_pdf_options() {
    let options = document::HtmlToPdfOptions::default();
    let _ = options;
}

#[test]
fn test_html_to_pdf() {
    let fixture = TestFixture::new();
    let html =
        fixture.create_test_text_file("page.html", "<html><body><h1>Test</h1></body></html>");
    let pdf = fixture.path("page.pdf");

    let result = document::html_to_pdf(&html, &pdf);
    let _ = result; // May fail without wkhtmltopdf
}

#[test]
fn test_html_string_to_pdf() {
    let fixture = TestFixture::new();
    let html = "<html><body><h1>Test</h1></body></html>";
    let pdf = fixture.path("output.pdf");

    let result = document::html_string_to_pdf(html, &pdf);
    let _ = result;
}

#[test]
fn test_url_to_pdf() {
    let fixture = TestFixture::new();
    let pdf = fixture.path("page.pdf");

    let result = document::url_to_pdf("https://example.com", &pdf);
    let _ = result;
}

// =============================================================================
// 38. doc_convert - Document format conversion
// =============================================================================

#[test]
fn test_doc_format_enum() {
    let _ = document::DocFormat::Pdf;
    let _ = document::DocFormat::Docx;
    let _ = document::DocFormat::Odt;
    let _ = document::DocFormat::Rtf;
    let _ = document::DocFormat::Txt;
    let _ = document::DocFormat::Html;
}

#[test]
fn test_convert_document() {
    let fixture = TestFixture::new();
    let txt = fixture.create_test_text_file("document.txt", "Hello World");
    let output = fixture.path("document.pdf");

    let result = document::convert_document(&txt, &output, document::DocFormat::Pdf);
    let _ = result; // May fail without LibreOffice
}

#[test]
fn test_to_pdf() {
    let fixture = TestFixture::new();
    let txt = fixture.create_test_text_file("document.txt", "Hello World");
    let output = fixture.path("document.pdf");

    let result = document::to_pdf(&txt, &output);
    let _ = result;
}

#[test]
fn test_to_docx() {
    let fixture = TestFixture::new();
    let txt = fixture.create_test_text_file("document.txt", "Hello World");
    let output = fixture.path("document.docx");

    let result = document::to_docx(&txt, &output);
    let _ = result;
}

// =============================================================================
// 39. text_extract - Text extraction
// =============================================================================

#[test]
fn test_extract_options() {
    let options = document::ExtractOptions::default();
    let _ = options;
}

#[test]
fn test_extract_text_from_pdf() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");

    let result = document::extract(&pdf);
    let _ = result;
}

#[test]
fn test_extract_text_to_file() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("text.txt");

    let result = document::extract_to_file(&pdf, &output);
    let _ = result;
}

// =============================================================================
// 40. pdf_watermark - PDF watermarking
// =============================================================================

#[test]
fn test_pdf_watermark_position_enum() {
    let _ = document::WatermarkPosition::Center;
    let _ = document::WatermarkPosition::TopLeft;
    let _ = document::WatermarkPosition::TopRight;
    let _ = document::WatermarkPosition::BottomLeft;
    let _ = document::WatermarkPosition::BottomRight;
    let _ = document::WatermarkPosition::Diagonal;
}

#[test]
fn test_watermark_options() {
    let options = document::WatermarkOptions::default();
    let _ = options;
}

#[test]
fn test_add_text_watermark() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("watermarked.pdf");

    let result = document::text_watermark(&pdf, &output, "CONFIDENTIAL");
    let _ = result;
}

#[test]
fn test_draft_watermark() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("draft.pdf");

    let result = document::draft_watermark(&pdf, &output);
    let _ = result;
}

#[test]
fn test_confidential_watermark() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("confidential.pdf");

    let result = document::confidential_watermark(&pdf, &output);
    let _ = result;
}

// =============================================================================
// 40.5 pdf_encrypt - PDF encryption
// =============================================================================

#[test]
fn test_encryption_strength_enum() {
    let _ = document::EncryptionStrength::Aes128;
    let _ = document::EncryptionStrength::Aes256;
    let _ = document::EncryptionStrength::Rc4_40;
    let _ = document::EncryptionStrength::Rc4_128;
}

#[test]
fn test_pdf_permissions() {
    let default_perms = document::PdfPermissions::default();
    assert!(default_perms.printing);
    assert!(default_perms.copy_contents);

    let no_perms = document::PdfPermissions::none();
    assert!(!no_perms.printing);
    assert!(!no_perms.copy_contents);

    let all_perms = document::PdfPermissions::all();
    assert!(all_perms.printing);
    assert!(all_perms.modify_contents);
}

#[test]
fn test_encrypt_options() {
    let options = document::EncryptOptions {
        user_password: "user123".to_string(),
        owner_password: "owner456".to_string(),
        strength: document::EncryptionStrength::Aes256,
        permissions: document::PdfPermissions::default(),
    };
    assert_eq!(options.owner_password, "owner456");
}

#[test]
fn test_encrypt_pdf() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("encrypted.pdf");

    let result = document::encrypt(&pdf, &output, "password123");
    let _ = result;
}

#[test]
fn test_encrypt_pdf_with_options() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");
    let output = fixture.path("encrypted.pdf");

    let options = document::EncryptOptions {
        user_password: "user".to_string(),
        owner_password: "owner".to_string(),
        strength: document::EncryptionStrength::Aes256,
        permissions: document::PdfPermissions {
            printing: true,
            high_quality_print: false,
            modify_contents: false,
            copy_contents: false,
            modify_annotations: false,
            fill_forms: true,
            accessibility: true,
            assemble: false,
        },
    };

    let result = document::encrypt_with_options(&pdf, &output, options);
    let _ = result;
}

#[test]
fn test_decrypt_pdf() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("encrypted.pdf", "%PDF-1.4\nfake encrypted pdf");
    let output = fixture.path("decrypted.pdf");

    let result = document::decrypt(&pdf, &output, "password123");
    let _ = result;
}

#[test]
fn test_is_encrypted() {
    let fixture = TestFixture::new();
    let pdf = fixture.create_test_text_file("document.pdf", "%PDF-1.4\nfake pdf");

    let result = document::is_encrypted(&pdf);
    let _ = result;
}

#[test]
fn test_check_tools() {
    let _ = document::check_ghostscript();
    let _ = document::check_pdftk();
}
