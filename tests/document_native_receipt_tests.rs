//! Tests for native document receipt and output-extension honesty.

mod common;

use common::TestFixture;

#[cfg(feature = "document-core")]
fn create_text_pdf(path: &std::path::Path) {
    use lopdf::{Document, Object, Stream, dictionary};

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let page_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });
    let content_id = doc.add_object(Stream::new(
        dictionary! {},
        b"BT /F1 12 Tf 100 700 Td (Receipt proof) Tj ET".to_vec(),
    ));

    doc.objects.insert(
        page_id,
        Object::Dictionary(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => resources_id,
        }),
    );
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set("Root", catalog_id);
    doc.save(path).expect("test PDF should be writable");
}

#[cfg(feature = "document-core")]
#[test]
fn document_tools_markdown_to_html_uses_native_receipt() {
    use dx_media::tools::DocumentTools;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("doc.md", "# Title\n\nBody.");
    let output = fixture.path("doc.html");

    let result = DocumentTools::new()
        .markdown_to_html(&input, &output)
        .expect("markdown facade should use native document-core converter");

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("document.markdown-to-html.native")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert!(output.exists(), "HTML output should be written");
}

#[test]
fn document_tools_extract_text_records_source_receipt_and_input_type_validation() {
    use dx_media::tools::DocumentTools;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("notes.txt", "Receipt proof\nSecond line");

    let result = DocumentTools::new()
        .extract_text(&input)
        .expect("text extraction facade should support local text fixtures");
    let input_source = input.display().to_string();

    assert!(result.message.contains("Receipt proof"));
    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("document.extract-text")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    assert_eq!(
        result.metadata.get("tool.source").map(String::as_str),
        Some(input_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.expected_output_media_type")
            .map(String::as_str),
        Some("text")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.input_extension")
            .map(String::as_str),
        Some("txt")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[test]
fn text_extract_to_file_rejects_non_text_output_before_writing() {
    use dx_media::tools::document::text_extract::extract_to_file;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("notes.txt", "Receipt proof\nSecond line");
    let output = fixture.path("notes.pdf");

    let error = extract_to_file(&input, &output)
        .expect_err("text extraction should reject non-text output paths before writing");

    assert!(
        error
            .to_string()
            .contains("Text output path must use .txt, .md, or .rst"),
        "error should explain the output extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected extracted text output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn markdown_to_html_native_rejects_non_html_output_extension() {
    use dx_media::tools::document::native::markdown_to_html_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("doc.md", "# Title\n\nBody.");
    let output = fixture.path("doc.pdf");

    let error = markdown_to_html_native(&input, &output, true)
        .expect_err("HTML converter should reject non-HTML output paths");

    assert!(
        error
            .to_string()
            .contains("HTML output path must use .html"),
        "error should explain the extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected HTML output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn pdf_to_text_native_rejects_non_text_output_before_writing() {
    use dx_media::tools::document::native::pdf_to_text_native;

    let fixture = TestFixture::new();
    let input = fixture.path("doc.pdf");
    let output = fixture.path("text.pdf");
    create_text_pdf(&input);

    let error = pdf_to_text_native(&input, &output)
        .expect_err("PDF text extraction should reject non-text output paths");

    assert!(
        error
            .to_string()
            .contains("Text output path must use .txt, .md, or .rst"),
        "error should explain the extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected text output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn pdf_delete_pages_native_rejects_non_pdf_output_before_writing() {
    use dx_media::tools::document::native::pdf_delete_pages_native;

    let fixture = TestFixture::new();
    let input = fixture.path("doc.pdf");
    let output = fixture.path("edited.docx");
    create_text_pdf(&input);

    let error = pdf_delete_pages_native(&input, &output, &[1])
        .expect_err("PDF page deletion should reject non-PDF output paths");

    assert!(
        error.to_string().contains("PDF output path must use .pdf"),
        "error should explain the extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected PDF output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn pdf_to_text_native_rejects_non_pdf_input_before_writing() {
    use dx_media::tools::document::native::pdf_to_text_native;

    let fixture = TestFixture::new();
    let input = fixture.path("renamed.txt");
    let output = fixture.path("text.txt");
    create_text_pdf(&input);

    let error = pdf_to_text_native(&input, &output)
        .expect_err("PDF text extraction should reject non-PDF input paths");

    assert!(
        error.to_string().contains("PDF input path must use .pdf"),
        "error should explain the input extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected text output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn pdf_delete_pages_native_rejects_non_pdf_input_before_writing() {
    use dx_media::tools::document::native::pdf_delete_pages_native;

    let fixture = TestFixture::new();
    let input = fixture.path("renamed.txt");
    let output = fixture.path("edited.pdf");
    create_text_pdf(&input);

    let error = pdf_delete_pages_native(&input, &output, &[1])
        .expect_err("PDF page deletion should reject non-PDF input paths");

    assert!(
        error.to_string().contains("PDF input path must use .pdf"),
        "error should explain the input extension contract: {error}"
    );
    assert!(
        !output.exists(),
        "rejected PDF output should not be written"
    );
}

#[cfg(feature = "document-core")]
#[test]
fn pdf_to_text_native_records_pdf_input_extension_metadata() {
    use dx_media::tools::document::native::pdf_to_text_native;

    let fixture = TestFixture::new();
    let input = fixture.path("doc.pdf");
    let output = fixture.path("text.txt");
    create_text_pdf(&input);

    let result = pdf_to_text_native(&input, &output)
        .expect("PDF text extraction should accept .pdf input and text output");

    assert_eq!(
        result
            .metadata
            .get("tool.expected_input_extension")
            .map(String::as_str),
        Some("pdf")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.input_extension")
            .map(String::as_str),
        Some("pdf")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
    assert!(output.exists(), "accepted text output should be written");
}
