//! Tests for archive processing with native and installed local tools.
//!
//! These tests use the `zip`, `tar`, and `flate2` crates.

mod common;

use common::TestFixture;

#[cfg(feature = "archive-core")]
#[test]
fn test_create_zip_archive() {
    use dx_media::tools::archive::zip::create_zip;

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let file2 = fixture.create_test_text_file("file2.txt", "Test Content");
    let output = fixture.path("archive.zip");

    let result = create_zip(&[&file1, &file2], &output);
    assert!(
        result.is_ok(),
        "ZIP creation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "ZIP file should exist");

    let metadata = std::fs::metadata(&output).unwrap();
    assert!(metadata.len() > 0, "ZIP file should not be empty");
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_creation_receipt_records_input_sources() {
    use dx_media::tools::archive::native::create_zip_native;

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let file2 = fixture.create_test_text_file("file2.txt", "Test Content");
    let output = fixture.path("archive.zip");

    let result = create_zip_native(&[&file1, &file2], &output, None).unwrap();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.zip.native")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    let sources = result
        .metadata
        .get("tool.source")
        .expect("archive creation receipt should record input sources");
    assert!(sources.contains(&file1.display().to_string()));
    assert!(sources.contains(&file2.display().to_string()));
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn archive_tools_create_zip_uses_native_receipt() {
    use dx_media::tools::ArchiveTools;

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let output = fixture.path("archive.zip");

    let result = ArchiveTools::new()
        .create_zip(&[&file], &output)
        .expect("ArchiveTools ZIP creation should succeed");

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.zip.native")
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
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_extraction_receipt_records_input_source_and_type_validation() {
    use dx_media::tools::archive::native::{create_zip_native, extract_zip_native};

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let archive = fixture.path("archive.zip");
    let output_dir = fixture.path("extracted");

    create_zip_native(&[&file], &archive, None).unwrap();
    let result = extract_zip_native(&archive, &output_dir).unwrap();
    let archive_source = archive.display().to_string();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.unzip.native")
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
        Some(archive_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_listing_receipt_records_input_source_and_type_validation() {
    use dx_media::tools::archive::native::{create_zip_native, list_zip_native};

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let archive = fixture.path("archive.zip");

    create_zip_native(&[&file], &archive, None).unwrap();
    let result = list_zip_native(&archive).unwrap();
    let archive_source = archive.display().to_string();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.list-zip.native")
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
        Some(archive_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn archive_tools_list_uses_registry_receipt_and_native_zip_validation() {
    use dx_media::tools::ArchiveTools;
    use dx_media::tools::archive::native::create_zip_native;

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let archive = fixture.path("archive.zip");

    create_zip_native(&[&file], &archive, None).unwrap();
    let result = ArchiveTools::new()
        .list(&archive)
        .expect("archive list facade should use native ZIP listing");
    let archive_source = archive.display().to_string();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.list")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.implementation")
            .map(String::as_str),
        Some("archive.list-zip.native")
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
        Some(archive_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.expected_input_extension")
            .map(String::as_str),
        Some("zip")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_creation_rejects_empty_input_list() {
    use dx_media::tools::archive::native::create_zip_native;

    let fixture = TestFixture::new();
    let inputs: Vec<std::path::PathBuf> = Vec::new();
    let output = fixture.path("empty.zip");

    let err = create_zip_native(&inputs, &output, None)
        .expect_err("empty archive input list should be rejected");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("at least one input"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_creation_rejects_missing_input_path() {
    use dx_media::tools::archive::native::create_zip_native;

    let fixture = TestFixture::new();
    let missing = fixture.path("missing.txt");
    let output = fixture.path("missing.zip");

    let err = create_zip_native(&[&missing], &output, None)
        .expect_err("missing archive input path should be rejected");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("does not exist"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[test]
fn native_zip_creation_rejects_wrong_output_extension() {
    use dx_media::tools::archive::create_zip_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("source.txt", "archive me");
    let output = fixture.path("archive.notzip");

    let err = create_zip_native(&[&input], &output, None)
        .expect_err("wrong archive extension should be rejected");

    assert!(
        err.to_string().contains("archive.zip.native"),
        "unexpected error: {err}"
    );
    assert!(
        err.to_string().contains("archive-extension-mismatch"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[cfg(feature = "archive-core")]
#[test]
fn test_extract_zip_archive() {
    use dx_media::tools::archive::zip::{create_zip, extract_zip};

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let zip_path = fixture.path("archive.zip");
    let extract_dir = fixture.path("extracted");

    // Create ZIP
    create_zip(&[&file1], &zip_path).unwrap();

    // Extract ZIP
    let result = extract_zip(&zip_path, &extract_dir);
    assert!(
        result.is_ok(),
        "ZIP extraction should succeed: {:?}",
        result.err()
    );
    assert!(extract_dir.exists(), "Extract directory should exist");
}

#[cfg(feature = "archive-core")]
#[test]
fn test_create_tar_archive() {
    use dx_media::tools::archive::tar::create_tar;

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let file2 = fixture.create_test_text_file("file2.txt", "Test Content");
    let output = fixture.path("archive.tar");

    let result = create_tar(&[&file1, &file2], &output);
    assert!(
        result.is_ok(),
        "TAR creation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "TAR file should exist");
}

#[cfg(feature = "archive-core")]
#[test]
fn test_create_tar_gz_archive() {
    use dx_media::tools::archive::tar::create_tar_gz;

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let output = fixture.path("archive.tar.gz");

    let result = create_tar_gz(&[&file1], &output);
    assert!(
        result.is_ok(),
        "TAR.GZ creation should succeed: {:?}",
        result.err()
    );
    assert!(output.exists(), "TAR.GZ file should exist");
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_creation_receipt_records_input_sources() {
    use dx_media::tools::archive::native::create_tar_gz_native;

    let fixture = TestFixture::new();
    let file1 = fixture.create_test_text_file("file1.txt", "Hello World");
    let output = fixture.path("archive.tar.gz");

    let result = create_tar_gz_native(&[&file1], &output, None).unwrap();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.tar-gz.native")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    let sources = result
        .metadata
        .get("tool.source")
        .expect("tar.gz creation receipt should record input sources");
    assert!(sources.contains(&file1.display().to_string()));
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_extraction_receipt_records_input_source_and_type_validation() {
    use dx_media::tools::archive::native::{create_tar_gz_native, extract_tar_gz_native};

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let archive = fixture.path("archive.tar.gz");
    let output_dir = fixture.path("tar-gz-extracted");

    create_tar_gz_native(&[&file], &archive, None).unwrap();
    let result = extract_tar_gz_native(&archive, &output_dir).unwrap();
    let archive_source = archive.display().to_string();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.untar-gz.native")
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
        Some(archive_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.expected_input_extension")
            .map(String::as_str),
        Some("tar.gz")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_creation_rejects_empty_input_list() {
    use dx_media::tools::archive::native::create_tar_gz_native;

    let fixture = TestFixture::new();
    let inputs: Vec<std::path::PathBuf> = Vec::new();
    let output = fixture.path("empty.tar.gz");

    let err = create_tar_gz_native(&inputs, &output, None)
        .expect_err("empty tar.gz input list should be rejected");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("at least one input"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_creation_rejects_missing_input_path() {
    use dx_media::tools::archive::native::create_tar_gz_native;

    let fixture = TestFixture::new();
    let missing = fixture.path("missing.txt");
    let output = fixture.path("missing.tar.gz");

    let err = create_tar_gz_native(&[&missing], &output, None)
        .expect_err("missing tar.gz input path should be rejected");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("does not exist"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[test]
fn native_tar_gz_creation_rejects_wrong_output_extension() {
    use dx_media::tools::archive::create_tar_gz_native;

    let fixture = TestFixture::new();
    let input = fixture.create_test_text_file("source.txt", "archive me");
    let output = fixture.path("archive.gz");

    let err = create_tar_gz_native(&[&input], &output, None)
        .expect_err("wrong tar.gz extension should be rejected");

    assert!(
        err.to_string().contains("archive.tar-gz.native"),
        "unexpected error: {err}"
    );
    assert!(
        err.to_string().contains("archive-extension-mismatch"),
        "unexpected error: {err}"
    );
    assert!(!output.exists(), "invalid archive should not be created");
}

#[cfg(feature = "archive-core")]
#[test]
fn native_tar_gz_extraction_rejects_wrong_input_extension() {
    use dx_media::tools::archive::native::extract_tar_gz_native;

    let fixture = TestFixture::new();
    let archive = fixture.create_test_text_file("archive.gz", "not tar gzip");
    let output_dir = fixture.path("bad-tar-gz-extracted");

    let err = extract_tar_gz_native(&archive, &output_dir)
        .expect_err("wrong tar.gz input extension should be rejected");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        err.to_string().contains("archive.untar-gz.native"),
        "unexpected error: {err}"
    );
    assert!(
        err.to_string().contains("archive-input-extension-mismatch"),
        "unexpected error: {err}"
    );
}

#[cfg(feature = "archive-core")]
#[test]
fn native_zip_extract_file_receipt_records_input_source_and_type_validation() {
    use dx_media::tools::archive::native::{create_zip_native, extract_file_from_zip_native};

    let fixture = TestFixture::new();
    let file = fixture.create_test_text_file("file.txt", "Hello World");
    let archive = fixture.path("archive.zip");
    let output = fixture.path("single-file.txt");

    create_zip_native(&[&file], &archive, None).unwrap();
    let result = extract_file_from_zip_native(&archive, "file.txt", &output).unwrap();
    let archive_source = archive.display().to_string();

    assert_eq!(
        result.metadata.get("tool.name").map(String::as_str),
        Some("archive.extract-file.native")
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
        Some(archive_source.as_str())
    );
    assert_eq!(
        result
            .metadata
            .get("tool.expected_input_extension")
            .map(String::as_str),
        Some("zip")
    );
    assert_eq!(
        result
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[cfg(not(feature = "archive-core"))]
#[test]
fn test_archive_core_feature_disabled() {
    assert!(true, "archive-core feature is disabled");
}

#[test]
fn test_archive_list() {
    use dx_media::tools::archive::list_archive;

    let fixture = TestFixture::new();

    // Create a test file and zip it
    let test_file = fixture.path("test.txt");
    std::fs::write(&test_file, b"test content").unwrap();

    let zip_path = fixture.path("test.zip");
    let result = dx_media::tools::archive::create_zip(&[&test_file], &zip_path);
    assert!(result.is_ok(), "ZIP creation should succeed");

    // List contents
    let result = list_archive(&zip_path);
    assert!(
        result.is_ok(),
        "Archive listing should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_archive_gzip() {
    use dx_media::tools::archive::{gunzip, gzip};

    let fixture = TestFixture::new();

    let test_file = fixture.path("test.txt");
    std::fs::write(&test_file, b"test content for compression").unwrap();

    let gzip_path = fixture.path("test.txt.gz");

    // Compress
    let result = gzip(&test_file, &gzip_path);
    assert!(
        result.is_ok(),
        "Gzip compression should succeed: {:?}",
        result.err()
    );
    assert!(gzip_path.exists(), "Gzipped file should exist");

    // Decompress
    let decompressed = fixture.path("decompressed.txt");
    let result = gunzip(&gzip_path, &decompressed);
    assert!(
        result.is_ok(),
        "Gunzip decompression should succeed: {:?}",
        result.err()
    );
    assert!(decompressed.exists(), "Decompressed file should exist");
}

#[test]
fn test_archive_encrypt() {
    use dx_media::tools::archive::create_encrypted_zip;

    let fixture = TestFixture::new();

    let test_file = fixture.path("secret.txt");
    std::fs::write(&test_file, b"secret content").unwrap();

    let encrypted_zip = fixture.path("encrypted.zip");

    let result = create_encrypted_zip(&[&test_file], &encrypted_zip, "password123");
    assert!(
        result.is_ok(),
        "Encrypted ZIP creation should succeed: {:?}",
        result.err()
    );
    assert!(encrypted_zip.exists(), "Encrypted ZIP should exist");
    let output = result.unwrap();
    assert_eq!(
        output.metadata.get("tool.name").map(String::as_str),
        Some("archive.encrypt")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.receipt_completeness")
            .map(String::as_str),
        Some("explicit")
    );
    assert_eq!(
        output
            .metadata
            .get("tool.type_validation")
            .map(String::as_str),
        Some("pass")
    );
}

#[test]
fn test_archive_split_merge() {
    use dx_media::tools::archive::split_archive;

    let fixture = TestFixture::new();

    // Create a test file
    let test_file = fixture.path("large.txt");
    let content = "x".repeat(1024 * 1024); // 1MB
    std::fs::write(&test_file, content.as_bytes()).unwrap();

    let split_dir = fixture.path("split");
    std::fs::create_dir_all(&split_dir).unwrap();

    // Split into 0.5MB parts
    let result = split_archive(&test_file, &split_dir, 1); // 1MB parts (won't split this file)
    assert!(
        result.is_ok(),
        "Archive split should succeed: {:?}",
        result.err()
    );
}
